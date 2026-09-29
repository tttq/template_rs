//! RustFS 对象存储驱动（S3 兼容 API）
//!
//! RustFS 是高性能分布式对象存储系统，兼容 S3 API。
//! 通过 `rust-s3` crate 对接，配置项以 `rustfs_*` 为前缀，
//! 通过 `attachment.storage.driver = "rustfs"` 启用。
//!
//! # 与 MinIO 的关系
//! RustFS 与 MinIO 均为 S3 兼容对象存储，此驱动同样适用于 MinIO、AWS S3 等
//! 任何 S3 兼容服务。后续可扩展为专门的云存储驱动（如 `aliyun`、`tencent` 等）。

use common::error::AppError;
use s3::creds::Credentials;
use s3::{Bucket, Region};

use super::{Storage, StorageObject, StorageSettings};

pub struct RustfsStorage {
    bucket: Box<Bucket>,
}

impl RustfsStorage {
    pub fn new(settings: &StorageSettings) -> Self {
        let region = Region::Custom {
            region: settings.rustfs_region.clone(),
            endpoint: settings.rustfs_endpoint.clone(),
        };
        let creds = Credentials::new(
            Some(&settings.rustfs_access_key),
            Some(&settings.rustfs_secret_key),
            None,
            None,
            None,
        )
        .expect("RustFS 凭证构造失败");
        let mut bucket = Bucket::new(&settings.rustfs_bucket, region, creds).expect("Bucket 构造失败");
        if !settings.rustfs_secure {
            bucket = bucket.with_path_style();
        }
        Self { bucket }
    }
}

#[async_trait::async_trait]
impl Storage for RustfsStorage {
    async fn put(&self, key: &str, data: Vec<u8>, mime: &str) -> Result<(), AppError> {
        self.bucket
            .put_object_with_content_type(key, &data, mime)
            .await
            .map_err(|e| AppError::Internal(format!("@rustfs_upload_failed:{}", e)))?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError> {
        let resp = self
            .bucket
            .get_object(key)
            .await
            .map_err(|e| AppError::NotFound(format!("@rustfs_read_failed:{}", e)))?;
        Ok(resp.bytes().to_vec())
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        // 删除失败必须向上抛出：附件记录一旦软删除，对象删除失败就变成永久残留的孤儿文件，
        // 而 S3 对「键不存在」的 DELETE 返回 204（成功），因此这里报错一定是真实故障
        // （桶名 / 端点 / 密钥 / 权限等），绝不能静默吞掉。
        self.bucket
            .delete_object(key)
            .await
            .map_err(|e| AppError::Internal(format!("@rustfs_delete_failed:{}|{}", key, e)))?;
        Ok(())
    }

    async fn presign_url(&self, key: &str) -> Option<String> {
        self.bucket.presign_get(key, 300, None).await.ok()
    }

    /// 按前缀列举对象（rust-s3 的 `list` 内部已跟随 continuation-token 分页）。
    ///
    /// `LastModified` 是 RFC3339 字符串，解析失败时保留 None（调用方据此保守处理，不误删）。
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<StorageObject>, AppError> {
        let pages = self
            .bucket
            .list(prefix.to_string(), None)
            .await
            .map_err(|e| AppError::Internal(format!("@rustfs_list_failed:{}|{}", prefix, e)))?;
        let mut objects = Vec::new();
        for page in pages {
            for obj in page.contents {
                objects.push(StorageObject {
                    key: obj.key,
                    size: obj.size,
                    last_modified: chrono::DateTime::parse_from_rfc3339(&obj.last_modified)
                        .ok()
                        .map(|t| t.with_timezone(&chrono::Utc)),
                });
            }
        }
        Ok(objects)
    }
}