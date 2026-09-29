//! 统一附件存储抽象（RustFS 对象存储 + 本地磁盘）
//!
//! - 定义 [`Storage`] trait（put/get/delete/presign_url/list_prefix）
//! - [`StorageService`] 作为可注入组件，按配置 `attachment.storage.driver` 选择驱动
//! - 内部使用 `static OnceLock<StorageDriver>` 池化：驱动仅初始化一次，全应用线程共享
//! - 对象键约定：`{category}/{biz_type}/{yyyyMM}/{file_id}.{ext}`
//!
//! # 驱动选择
//! | `driver` 值    | 存储后端     | 说明                     |
//! |----------------|-------------|--------------------------|
//! | `"local"`      | 本地磁盘      | 开发/单机场景（`local_path`） |
//! | `"rustfs"`     | RustFS 对象存储（S3 兼容 API） | 生产/分布式场景 |
//!
//! 只认这两个值：写错会**启动即 panic**，不会静默退回本地磁盘。
//!
//! # 配置读取（重要，曾踩坑）
//! summer 的 `ConfigRegistry::get_by_prefix` 是**按单个键**在 TOML 顶层查找（不拆分 `.`），
//! 因此 `[attachment.storage]` 这种嵌套表必须以顶层键 `attachment` 为前缀整体反序列化
//! （见 [`AttachmentSettings`]）。若把 `config_prefix()` 直接写成 `"attachment.storage"`，
//! 查找必然落空 → 配合 `#[serde(default)]` 会拿到全默认值（`driver = "local"`），
//! 表现为"配置写了 rustfs，文件却写进本地 `./uploads/attachment`"且毫无报错。
//!
//! # 扩展指南
//! 如需接入新的云存储（如阿里云 OSS、腾讯云 COS 等）：
//! 1. 在 `storage/` 下新建模块（如 `aliyun.rs`），实现 [`Storage`] trait
//! 2. 在 [`StorageDriver`] 枚举中新增变体
//! 3. 在 [`StorageService::get_driver`] 中匹配新 driver 值
//! 4. 在配置文件中添加对应字段（建议遵循 `{provider}_*` 命名）

pub mod local;
pub mod rustfs;

use std::sync::OnceLock;

use common::error::AppError;
use summer::config::Configurable;
use summer::plugin::service::Service;

/// 对象存储中的一个对象（[`Storage::list_prefix`] 返回，用于保留期清理与孤儿对象对账）
#[derive(Debug, Clone)]
pub struct StorageObject {
    pub key: String,
    pub size: u64,
    /// 最后修改时间；解析失败或实现拿不到时为 None，调用方按「不确定」保守处理
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, data: Vec<u8>, mime: &str) -> Result<(), AppError>;
    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError>;
    async fn delete(&self, key: &str) -> Result<(), AppError>;
    async fn presign_url(&self, key: &str) -> Option<String>;
    /// 按前缀列举对象（实现内部处理分页）
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<StorageObject>, AppError>;
}

/// 存储配置（通过 `attachment.storage` 前缀加载）
///
/// # 新驱动扩展
/// 新增云存储时，在此结构体添加 `{provider}_*` 字段，并实现 Default trait。
/// 保持 `driver` 字段不变，在 StorageDriver 枚举中新增变体即可。
#[derive(Clone, serde::Deserialize)]
#[serde(default)]
pub struct StorageSettings {
    /// 驱动类型：`"local"` | `"rustfs"`
    pub driver: String,

    // ===== 本地存储 =====
    /// 本地存储根目录（仅 driver=local 时生效）
    pub local_path: String,

    // ===== RustFS 对象存储（S3 兼容 API）=====
    /// RustFS Endpoint（如 `http://127.0.0.1:9000`）
    pub rustfs_endpoint: String,
    /// Bucket 名称
    pub rustfs_bucket: String,
    /// Access Key
    pub rustfs_access_key: String,
    /// Secret Key
    pub rustfs_secret_key: String,
    /// 区域（如 `us-east-1`）
    pub rustfs_region: String,
    /// 是否启用 HTTPS
    pub rustfs_secure: bool,
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            driver: "local".to_string(),
            local_path: "./uploads/attachment".to_string(),
            rustfs_endpoint: "http://127.0.0.1:9000".to_string(),
            rustfs_bucket: "attachment".to_string(),
            rustfs_access_key: String::new(),
            rustfs_secret_key: String::new(),
            rustfs_region: "us-east-1".to_string(),
            rustfs_secure: false,
        }
    }
}

/// 顶层 `[attachment]` 配置容器。
///
/// 之所以不直接在 [`StorageSettings`] 上声明 `"attachment.storage"` 前缀：
/// summer 的 `get_by_prefix` 只按单个键取表，嵌套表需要先用顶层键反序列化，
/// 否则取到空表 → `#[serde(default)]` 让 `driver` 变成 `local`（静默写本地磁盘）。
#[derive(Clone, serde::Deserialize, Default)]
#[serde(default)]
pub struct AttachmentSettings {
    /// `[attachment.storage]`
    pub storage: StorageSettings,
}

impl Configurable for AttachmentSettings {
    fn config_prefix() -> &'static str {
        "attachment"
    }
}

enum StorageDriver {
    Rustfs(rustfs::RustfsStorage),
    Local(local::LocalStorage),
}

impl StorageDriver {
    async fn put(&self, key: &str, data: Vec<u8>, mime: &str) -> Result<(), AppError> {
        match self {
            StorageDriver::Rustfs(d) => d.put(key, data, mime).await,
            StorageDriver::Local(d) => d.put(key, data, mime).await,
        }
    }
    async fn get(&self, key: &str) -> Result<Vec<u8>, AppError> {
        match self {
            StorageDriver::Rustfs(d) => d.get(key).await,
            StorageDriver::Local(d) => d.get(key).await,
        }
    }
    async fn delete(&self, key: &str) -> Result<(), AppError> {
        match self {
            StorageDriver::Rustfs(d) => d.delete(key).await,
            StorageDriver::Local(d) => d.delete(key).await,
        }
    }
    async fn presign_url(&self, key: &str) -> Option<String> {
        match self {
            StorageDriver::Rustfs(d) => d.presign_url(key).await,
            StorageDriver::Local(d) => d.presign_url(key).await,
        }
    }
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<StorageObject>, AppError> {
        match self {
            StorageDriver::Rustfs(d) => d.list_prefix(prefix).await,
            StorageDriver::Local(d) => d.list_prefix(prefix).await,
        }
    }
}

static DRIVER: OnceLock<StorageDriver> = OnceLock::new();

#[derive(Clone, Service)]
pub struct StorageService {
    /// 顶层 `[attachment]` 配置（内层为 `[attachment.storage]`）
    #[inject(config)]
    settings: AttachmentSettings,
}

impl StorageService {
    fn get_driver(&self) -> &StorageDriver {
        DRIVER.get_or_init(|| match self.storage().driver.as_str() {
            // 未知驱动直接 panic：宁可启动失败，也不能把附件静默写到本地磁盘
            // （容器/多实例场景下本地磁盘是临时盘，会导致文件"上传成功但读不到"）
            "rustfs" => StorageDriver::Rustfs(rustfs::RustfsStorage::new(self.storage())),
            "local" => StorageDriver::Local(local::LocalStorage::new(self.storage())),
            other => panic!(
                "不支持的 attachment.storage.driver = {other:?}（仅支持 \"local\" / \"rustfs\"）"
            ),
        })
    }

    /// 存储配置（内层 `[attachment.storage]`）
    pub fn storage(&self) -> &StorageSettings {
        &self.settings.storage
    }

    /// 驱动名（启动自检/日志用）
    pub fn driver_name(&self) -> &'static str {
        if self.is_local() {
            "local"
        } else {
            "rustfs"
        }
    }

    /// 供启动日志输出的一行描述
    pub fn describe(&self) -> String {
        let s = self.storage();
        if self.is_local() {
            format!("local（root={}）", s.local_path)
        } else {
            format!(
                "rustfs（endpoint={} bucket={}）",
                s.rustfs_endpoint, s.rustfs_bucket
            )
        }
    }

    pub fn is_local(&self) -> bool {
        self.settings.storage.driver != "rustfs"
    }

    pub async fn put(&self, key: &str, data: Vec<u8>, mime: &str) -> Result<(), AppError> {
        self.get_driver().put(key, data, mime).await
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, AppError> {
        self.get_driver().get(key).await
    }

    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.get_driver().delete(key).await
    }

    pub async fn presign_url(&self, key: &str) -> Option<String> {
        self.get_driver().presign_url(key).await
    }

    /// 按前缀列举对象（保留期清理 / 孤儿对象对账用）
    pub async fn list_prefix(&self, prefix: &str) -> Result<Vec<StorageObject>, AppError> {
        self.get_driver().list_prefix(prefix).await
    }
}