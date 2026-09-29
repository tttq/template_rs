//! AI 中转平台加密工具（docs/ai-relay-platform-design.md §4）
//!
//! - 上游渠道密钥：AES-256-GCM 加密存储（`ai_channel.api_key_cipher`）
//! - 下游 API Key：`sk-` + 48 位随机串，仅存 SHA-256 哈希（`ai_api_key.key_hash`）
//! - 兑换码 / 邀请码生成
//!
//! 加密密钥来自环境变量 `AI_CHANNEL_KEY_SECRET`（32 字节 base64），
//! 未配置时使用开发默认密钥（生产环境必须配置）。

use std::sync::OnceLock;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::Engine;
use rand::distr::{Alphanumeric, SampleString};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// AES-GCM 密钥长度（字节）
const KEY_LEN: usize = 32;
/// AES-GCM nonce 长度（字节）
const NONCE_LEN: usize = 12;
/// 开发环境默认密钥（base64 of 32 bytes），生产环境务必配置 AI_CHANNEL_KEY_SECRET
const DEV_SECRET_BASE64: &str = "ZGV2LW9ubHktc2VjcmV0LWtleS0zMi1ieXRlcy1iYXNlNjQ=";

fn cipher() -> &'static Aes256Gcm {
    static CIPHER: OnceLock<Aes256Gcm> = OnceLock::new();
    CIPHER.get_or_init(|| {
        let secret = std::env::var("AI_CHANNEL_KEY_SECRET")
            .unwrap_or_else(|_| DEV_SECRET_BASE64.to_string());
        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(secret.as_bytes())
            .ok()
            .filter(|v| v.len() == KEY_LEN)
            .unwrap_or_else(|| {
                log::warn!("AI_CHANNEL_KEY_SECRET 未配置或格式非法（需 32 字节 base64），使用开发默认密钥");
                // 回退：对 secret 做 SHA-256 得到 32 字节
                let mut hasher = Sha256::new();
                hasher.update(secret.as_bytes());
                hasher.finalize().to_vec()
            });
        Aes256Gcm::new_from_slice(&key_bytes).expect("AES-256 密钥长度非法")
    })
}

/// AES-256-GCM 加密，输出 base64(nonce || ciphertext)
pub fn encrypt_string(plain: &str) -> Result<String, String> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher()
        .encrypt(nonce, plain.as_bytes())
        .map_err(|e| format!("AES-GCM 加密失败: {}", e))?;
    let mut buf = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    buf.extend_from_slice(&nonce_bytes);
    buf.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

/// AES-256-GCM 解密（输入为 encrypt_string 的输出）
pub fn decrypt_string(cipher_b64: &str) -> Result<String, String> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(cipher_b64.as_bytes())
        .map_err(|e| format!("密文 base64 解码失败: {}", e))?;
    if raw.len() <= NONCE_LEN {
        return Err("@cipher_text_invalid".to_string());
    }
    let (nonce_bytes, ciphertext) = raw.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher()
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("AES-GCM 解密失败: {}", e))?;
    String::from_utf8(plain).map_err(|e| format!("解密结果非 UTF-8: {}", e))
}

/// SHA-256 摘要的十六进制表示（API Key 哈希存储）
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        hex.push_str(&format!("{:02x}", b));
    }
    hex
}

fn random_alphanumeric(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), len)
}

/// 生成下游 API Key：`sk-` + 48 位随机串
pub fn generate_api_key() -> String {
    format!("sk-{}", random_alphanumeric(48))
}

/// 生成兑换码：`RC-` + 32 位随机串
pub fn generate_redeem_code() -> String {
    format!("RC-{}", random_alphanumeric(32))
}

/// 生成 8 位大写邀请码
pub fn generate_invite_code() -> String {
    random_alphanumeric(8).to_uppercase()
}

/// 生成客户端密钥：`cs-` + 32 位随机串（客户端管理页新增时自动生成）
pub fn generate_client_secret() -> String {
    format!("cs-{}", random_alphanumeric(32))
}

/// 生成 6 位数字邮箱验证码
pub fn generate_email_code() -> String {
    let v = rand::random_range(0..1_000_000u32);
    format!("{:06}", v)
}

/// 邮箱验证码 Redis key（relay 发送 / system 登录、门户注册与找回校验共享约定）
pub fn email_code_redis_key(email: &str, scene: &str) -> String {
    format!("mail:code:{}:{}", email, scene)
}

/// API Key 展示前缀：前 8 位 + `****`
pub fn api_key_prefix(key: &str) -> String {
    let head: String = key.chars().take(8).collect();
    format!("{}****", head)
}

/// 密钥脱敏：`sk-****` + 尾 4 位（渠道编辑回显用）
pub fn mask_secret_tail(secret: &str) -> String {
    let tail: String = secret.chars().rev().take(4).collect::<String>().chars().rev().collect();
    format!("****{}", tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plain = "sk-upstream-abc-123";
        let enc = encrypt_string(plain).unwrap();
        assert_ne!(enc, plain);
        let dec = decrypt_string(&enc).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn test_sha256_hex() {
        let h = sha256_hex("abc");
        assert_eq!(h.len(), 64);
        assert_eq!(h, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn test_generate_api_key() {
        let k = generate_api_key();
        assert!(k.starts_with("sk-"));
        assert_eq!(k.len(), 3 + 48);
    }

    #[test]
    fn test_mask() {
        assert_eq!(api_key_prefix("sk-abcdefgh12345678"), "sk-abcde****");
        assert_eq!(mask_secret_tail("sk-abcdef1234"), "****1234");
    }
}
