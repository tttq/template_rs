//! 图形验证码：随机码生成、点阵字体绘制 PNG（纯函数，无 Redis 依赖）
//!
//! 说明：
//! - 使用 `font8x8` 内置点阵字体，不依赖系统字体文件，跨平台可运行
//! - 字符集剔除易混淆字符（I/L/O/0/1），避免人工识别困难
//! - 叠加干扰点/干扰线 + 逐字符随机色/倾斜/抖动，提高机器识别成本
//! - PNG 字节经 base64 后以 data URI 供 Web 直接展示；小程序走独立图片端点

use std::io::Cursor;

use base64::Engine;
use common::error::AppError;
use font8x8::unicode::UnicodeFonts;
use image::{ImageFormat, Rgba, RgbaImage};
use rand::Rng;

/// 图形码字符集：大写字母（除 I/L/O）+ 数字（除 0/1）
const CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
/// 图形码长度
const CODE_LEN: usize = 4;
/// 图片宽度 / 高度
const W: u32 = 136;
const H: u32 = 48;
/// 字符放大倍数（8x8 点阵 → 24px）
const SCALE: i32 = 3;

/// 生成 4 位随机图形验证码
pub fn random_captcha_code() -> String {
    (0..CODE_LEN)
        .map(|_| {
            let idx = rand::random_range(0..CHARS.len() as u8) as usize;
            CHARS[idx] as char
        })
        .collect()
}

/// 生成随机图形码 ID（32 位 hex，用作 Redis key）
pub fn random_captcha_id() -> String {
    let a = rand::random_range(0..u64::MAX);
    let b = rand::random_range(0..u64::MAX);
    format!("{a:016x}{b:016x}")
}

/// Redis key：图形码存储（TTL 内有效，一次性消费）
pub fn captcha_id_key(id: &str) -> String {
    format!("captcha:img:{id}")
}

/// Redis key：单 IP 每分钟获取上限
pub fn minute_limit_key(ip: &str) -> String {
    format!("captcha:min:{ip}")
}

/// Redis key：单 IP 每日获取上限
pub fn day_limit_key(ip: &str) -> String {
    format!("captcha:day:{ip}")
}

/// 按图形码内容绘制 PNG 字节
pub fn render_png(code: &str) -> Result<Vec<u8>, AppError> {
    let mut img = RgbaImage::from_pixel(W, H, Rgba([246, 248, 252, 255]));
    let mut rng = rand::rng();

    // 干扰点（浅灰随机散布）
    for _ in 0..160 {
        let x = rng.random_range(0..W);
        let y = rng.random_range(0..H);
        let g = rng.random_range(150..225u8);
        img.put_pixel(x, y, Rgba([g, g, g, 255]));
    }

    // 干扰线（浅色短折线，与字符叠置但弱于前景）
    for _ in 0..5 {
        let color = Rgba([
            rng.random_range(150..220u8),
            rng.random_range(150..220u8),
            rng.random_range(160..230u8),
            255,
        ]);
        let (mut x, mut y) = (rng.random_range(0..W), rng.random_range(0..H));
        let steps = rng.random_range(10..22i32);
        for _ in 0..steps {
            img.put_pixel(x, y, color);
            let nx = x as i32 + rng.random_range(-1..=1i32);
            let ny = y as i32 + rng.random_range(-1..=1i32);
            if nx < 0 || ny < 0 {
                break;
            }
            x = nx as u32;
            y = ny as u32;
            if x >= W || y >= H {
                break;
            }
        }
    }

    // 前景字符色板（深色，与干扰线区分）
    const PALETTE: [[u8; 3]; 6] = [
        [60, 88, 160],
        [38, 110, 88],
        [170, 84, 46],
        [118, 64, 150],
        [40, 92, 150],
        [156, 98, 32],
    ];

    let base_y = (H as i32 - 8 * SCALE) / 2; // ≈12
    let mut cursor_x = 6i32;
    for (ci, ch) in code.chars().enumerate() {
        // 每个字符独立倾斜（模拟斜体）与纵向抖动
        let slant = rng.random_range(-1..=1i32);
        let y_jitter = rng.random_range(-3..=3i32);
        let row_start = base_y + y_jitter;
        let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or([0u8; 8]);
        let color = PALETTE[rng.random_range(0..PALETTE.len() as u8) as usize];

        for (gy, row) in glyph.iter().enumerate() {
            let shear = (gy as i32 - 4) * slant;
            let y0 = row_start + (gy as i32) * SCALE;
            for gx in 0..8u32 {
                if row & (1 << gx) == 0 {
                    continue;
                }
                let x0 = cursor_x + shear + (gx as i32) * SCALE;
                for py in 0..SCALE {
                    for px in 0..SCALE {
                        let (xx, yy) = (x0 + px, y0 + py);
                        if xx >= 0 && yy >= 0 && (xx as u32) < W && (yy as u32) < H {
                            img.put_pixel(
                                xx as u32,
                                yy as u32,
                                Rgba([color[0], color[1], color[2], 255]),
                            );
                        }
                    }
                }
            }
        }

        // 字符间距 6px 基础 + 0~3px 随机
        cursor_x += 8 * SCALE + 6 + rng.random_range(0..=3i32);
        if ci + 1 < code.len() && cursor_x + 8 * SCALE > W as i32 {
            break; // 防御：避免越界（正常情况不会触发）
        }
    }

    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| AppError::Internal(format!("@captcha_generate_failed:{e}")))?;
    Ok(buf)
}

/// PNG 字节 → `data:image/png;base64,...`（Web 端直接用于 <img>）
pub fn png_data_uri(png: &[u8]) -> String {
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    )
}
