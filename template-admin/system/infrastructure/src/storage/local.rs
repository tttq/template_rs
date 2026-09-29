use std::path::PathBuf;

use common::error::AppError;

use super::{Storage, StorageObject, StorageSettings};

pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(settings: &StorageSettings) -> Self {
        Self {
            root: PathBuf::from(&settings.local_path),
        }
    }
}

#[async_trait::async_trait]
impl Storage for LocalStorage {
    async fn put(&self, key: &str, data: Vec<u8>, _mime: &str) -> Result<(), AppError> {
        let full = self.root.join(key);
        if let Some(parent) = full.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Internal(format!("@storage_mkdir_failed:{}", e)))?;
        }
        tokio::fs::write(&full, &data)
            .await
            .map_err(|e| AppError::Internal(format!("@storage_write_failed:{}", e)))
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError> {
        tokio::fs::read(self.root.join(key))
            .await
            .map_err(|_| AppError::NotFound("@file_not_found".to_string()))
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        match tokio::fs::remove_file(self.root.join(key)).await {
            Ok(()) => Ok(()),
            // 与 S3 DELETE 语义对齐：文件本就不存在视为删除成功（幂等、可重试）
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AppError::Internal(format!("@storage_delete_failed:{}|{}", key, e))),
        }
    }

    async fn presign_url(&self, _key: &str) -> Option<String> {
        None
    }

    /// 按前缀列举本地文件（key 使用 `/` 分隔，与对象存储保持一致的键空间）
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<StorageObject>, AppError> {
        let root = self.root.clone();
        let prefix = prefix.trim_start_matches('/').to_string();
        tokio::task::spawn_blocking(move || {
            let mut objects = Vec::new();
            collect_objects(&root, &root.join(&prefix), &mut objects)?;
            Ok::<_, AppError>(objects)
        })
        .await
        .map_err(|e| AppError::Internal(format!("@storage_list_failed:{}", e)))?
    }
}

/// 递归收集目录下的文件（[`LocalStorage::list_prefix`] 的实现）
fn collect_objects(
    root: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<StorageObject>,
) -> Result<(), AppError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // 目录不存在 = 该前缀下没有对象（与 S3 空结果一致）
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(AppError::Internal(format!("@storage_dir_list_failed:{}|{}", dir.display(), e)));
        }
    };
    for entry in entries {
        let entry = entry.map_err(|e| AppError::Internal(format!("@storage_read_dir_failed:{}", e)))?;
        let path = entry.path();
        if path.is_dir() {
            collect_objects(root, &path, out)?;
            continue;
        }
        let meta = entry.metadata().ok();
        let key = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        out.push(StorageObject {
            key,
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            last_modified: meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(chrono::DateTime::<chrono::Utc>::from),
        });
    }
    Ok(())
}