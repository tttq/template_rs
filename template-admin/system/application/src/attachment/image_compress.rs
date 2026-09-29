//! 上传图片压缩：在保证清晰度的前提下收敛长边与体积，避免原图直存导致存储空间占用过大。
//!
//! - 触发条件：位图（jpeg / png / bmp）且原始字节数达到阈值；小图不动，避免无谓的画质损失与 CPU 开销；
//! - 保证清晰度：长边等比缩放至不超过 `MAX_EDGE`（Lanczos3 重采样），JPEG 按 `JPEG_QUALITY` 重编码；
//! - 始终按原格式重新编码，mime / 扩展名不变，调用方与库表无需额外处理；
//! - 解码失败（损坏文件、超大图）或压缩后反而更大时一律保留原图，不影响上传主流程。

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType as PngCompression, FilterType as PngFilter, PngEncoder};
use image::{DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageFormat, ImageReader};

/// 长边上限（像素）：等比缩放后长边不超过该值
const MAX_EDGE: u32 = 2560;
/// JPEG 重编码质量：85 在肉眼几乎无损的前提下显著减小体积
const JPEG_QUALITY: u8 = 85;
/// 触发压缩的最小字节数（512KB）：小图直接放行
const MIN_COMPRESS_BYTES: usize = 512 * 1024;
/// 解码像素上限（8000 万像素）：超过则不处理，避免解码时内存放大
const MAX_PIXELS: u64 = 80_000_000;

/// 可处理的位图格式；svg（矢量）、gif（可能为动图）、webp（无有损编码器）一律原样保存
fn decodable_format(mime_type: &str) -> Option<ImageFormat> {
    match mime_type {
        "image/jpeg" => Some(ImageFormat::Jpeg),
        "image/png" => Some(ImageFormat::Png),
        "image/bmp" => Some(ImageFormat::Bmp),
        _ => None,
    }
}

/// 压缩图片，返回（用于落盘的字节, 是否发生了压缩）。
/// 所有失败路径都返回原图，保证上传流程不受影响。
pub fn compress(mime_type: &str, data: Vec<u8>) -> (Vec<u8>, bool) {
    let Some(format) = decodable_format(mime_type) else {
        return (data, false);
    };
    if data.len() < MIN_COMPRESS_BYTES {
        return (data, false);
    }
    match reencode(&data, format) {
        // 压缩后反而更大（如已高度优化的 PNG）：保留原图
        Some(out) if out.len() < data.len() => (out, true),
        _ => (data, false),
    }
}

/// 解码 → 应用 EXIF 方向 → 超长边则等比缩放 → 按原格式重编码
fn reencode(data: &[u8], format: ImageFormat) -> Option<Vec<u8>> {
    let reader = ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let mut decoder = reader.into_decoder().ok()?;
    let (w, h) = decoder.dimensions();
    if w == 0 || h == 0 || w as u64 * h as u64 > MAX_PIXELS {
        return None;
    }
    // 手机拍摄的 JPEG 常带 EXIF 方向，重编码会丢掉该元数据，需先把方向作用到像素上
    let orientation = decoder.orientation().ok()?;
    let mut img = DynamicImage::from_decoder(decoder).ok()?;
    img.apply_orientation(orientation);

    let long = img.width().max(img.height());
    if long > MAX_EDGE {
        let scale = MAX_EDGE as f32 / long as f32;
        let nw = ((img.width() as f32 * scale).round() as u32).max(1);
        let nh = ((img.height() as f32 * scale).round() as u32).max(1);
        img = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    }

    let mut out = Vec::new();
    match format {
        ImageFormat::Jpeg => {
            // JPEG 无透明通道，转 RGB8 后按质量重编码
            let buf = img.to_rgb8();
            JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY)
                .write_image(buf.as_raw(), buf.width(), buf.height(), ExtendedColorType::Rgb8)
                .ok()?;
        }
        ImageFormat::Png => {
            // 保留透明通道；默认压缩级别偏低，这里用 Balanced + 自适应滤波换取更小体积
            let buf = img.to_rgba8();
            PngEncoder::new_with_quality(&mut out, PngCompression::Default, PngFilter::Adaptive)
                .write_image(buf.as_raw(), buf.width(), buf.height(), ExtendedColorType::Rgba8)
                .ok()?;
        }
        _ => {
            let buf = img.to_rgba8();
            image::codecs::bmp::BmpEncoder::new(&mut out)
                .write_image(buf.as_raw(), buf.width(), buf.height(), ExtendedColorType::Rgba8)
                .ok()?;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb, Rgba};

    /// 生成难以压缩的大图（伪随机噪点），确保字节数超过触发阈值
    fn noisy_pixels(x: u32, y: u32) -> [u8; 3] {
        let v = x.wrapping_mul(2_654_435_761).wrapping_add(y.wrapping_mul(40_503));
        [v as u8, (v >> 8) as u8, (v >> 16) as u8]
    }

    fn big_jpeg(w: u32, h: u32) -> Vec<u8> {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(w, h, |x, y| Rgb(noisy_pixels(x, y)));
        let mut out = Vec::new();
        JpegEncoder::new_with_quality(&mut out, 95)
            .write_image(img.as_raw(), w, h, ExtendedColorType::Rgb8)
            .unwrap();
        out
    }

    fn decode(data: &[u8]) -> DynamicImage {
        ImageReader::new(std::io::Cursor::new(data))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
    }

    #[test]
    fn compress_caps_long_edge() {
        let src = big_jpeg(4000, 3000);
        assert!(src.len() > MIN_COMPRESS_BYTES, "测试图需超过阈值，实际 {} 字节", src.len());

        let (out, compressed) = compress("image/jpeg", src.clone());
        assert!(compressed);
        assert!(out.len() < src.len(), "压缩后应更小：{} -> {}", src.len(), out.len());

        let img = decode(&out);
        assert_eq!(img.width().max(img.height()), MAX_EDGE);
        // 等比缩放：4000x3000 -> 2560x1920
        assert_eq!((img.width(), img.height()), (MAX_EDGE, 1920));
    }

    #[test]
    fn keep_small_image_untouched() {
        let src = big_jpeg(200, 150);
        assert!(src.len() < MIN_COMPRESS_BYTES);
        let (out, compressed) = compress("image/jpeg", src.clone());
        assert!(!compressed);
        assert_eq!(out, src);
    }

    #[test]
    fn skip_non_bitmap_and_broken_data() {
        let svg = vec![b'x'; MIN_COMPRESS_BYTES + 1];
        let (out, compressed) = compress("image/svg+xml", svg.clone());
        assert!(!compressed);
        assert_eq!(out, svg);

        // 损坏的 JPEG：解码失败必须回退原图，不能影响上传
        let broken = vec![0u8; MIN_COMPRESS_BYTES + 1];
        let (out, compressed) = compress("image/jpeg", broken.clone());
        assert!(!compressed);
        assert_eq!(out, broken);
    }

    #[test]
    fn keep_png_alpha() {
        let (w, h) = (3000u32, 3000u32);
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(w, h, |x, y| {
            let [r, g, b] = noisy_pixels(x, y);
            Rgba([r, g, b, if (x + y) % 2 == 0 { 255 } else { 128 }])
        });
        let mut src = Vec::new();
        // 用低压缩级别编码，模拟未优化的原图
        PngEncoder::new_with_quality(&mut src, PngCompression::Fast, PngFilter::NoFilter)
            .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
            .unwrap();
        assert!(src.len() > MIN_COMPRESS_BYTES);

        let (out, compressed) = compress("image/png", src.clone());
        assert!(compressed);
        assert!(out.len() < src.len());

        let decoded = decode(&out);
        assert_eq!(decoded.width().max(decoded.height()), MAX_EDGE);
        assert!(decoded.has_alpha());
    }
}
