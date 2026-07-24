use bcrypt::{BcryptError, hash, verify, DEFAULT_COST};

/// 使用 bcrypt 哈希密码（默认 cost = 12）
pub fn hash_password(plain: &str) -> Result<String, BcryptError> {
    hash(plain, DEFAULT_COST)
}

/// 校验明文密码是否匹配 bcrypt 哈希值
pub fn verify_password(plain: &str, hashed: &str) -> Result<bool, BcryptError> {
    verify(plain, hashed)
}
