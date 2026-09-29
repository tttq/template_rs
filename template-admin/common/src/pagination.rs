//! 分页查询参数与分页结果
//!
//! 边界防护：`PageQuery` 反序列化时对 `page` / `page_size` 做钳制——
//! 客户端传入超大 `page_size` 会导致单次查询拉取海量行（内存暴涨 / DB 压力，
//! DoS 向量）；`page = 0` 则会被 OFFSET 计算放大成全表扫描。上限与默认值
//! 在此统一收敛，30+ 个列表接口无需各自校验。

use serde::{Deserialize, Serialize};
use serde_json;

/// 单页行数上限（超过按此值截断）
const MAX_PAGE_SIZE: u64 = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    #[serde(default = "default_page", deserialize_with = "clamp_min_one")]
    pub page: u64,
    #[serde(default = "default_page_size", deserialize_with = "clamp_max_page_size")]
    pub page_size: u64,
    pub sort_field: Option<String>,
    pub sort_order: Option<String>,
}

pub fn default_page() -> u64 {
    1
}

pub fn default_page_size() -> u64 {
    10
}

/// 页码钳制（>= 1），供自带 `page` / `page_size` 字段（未复用 [`PageQuery`]）的查询 DTO 使用：
/// `#[serde(default = "common::pagination::default_page", deserialize_with = "common::pagination::de_page")]`
///
/// 与 `PageQuery` 同一套边界防护，但实现不依赖 `serde_json`，URL query 与 JSON body 均可用。
pub fn de_page<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<u64>::deserialize(deserializer)?.unwrap_or_else(default_page);
    Ok(value.max(1))
}

/// 页大小钳制（1..=200），用途同 [`de_page`]。
pub fn de_page_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<u64>::deserialize(deserializer)?.unwrap_or_else(default_page_size);
    Ok(value.clamp(1, MAX_PAGE_SIZE))
}

/// 页码下界钳制：page >= 1（支持字符串或数字输入）
fn clamp_min_one<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    let v = match v {
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| D::Error::custom("页码必须为正整数"))?,
        serde_json::Value::String(s) => s.parse::<u64>().map_err(|_| D::Error::custom("页码必须为正整数"))?,
        _ => return Err(D::Error::custom("页码必须为数字或字符串")),
    };
    Ok(v.max(1))
}

/// 页大小上界钳制：1 <= page_size <= MAX_PAGE_SIZE（支持字符串或数字输入）
fn clamp_max_page_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    let v = match v {
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| D::Error::custom("页大小必须为正整数"))?,
        serde_json::Value::String(s) => s.parse::<u64>().map_err(|_| D::Error::custom("页大小必须为正整数"))?,
        _ => return Err(D::Error::custom("页大小必须为数字或字符串")),
    };
    Ok(v.clamp(1, MAX_PAGE_SIZE))
}

impl Default for PageQuery {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
            sort_field: None,
            sort_order: Some("desc".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResult<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T: Serialize> PageResult<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        let total_pages = if page_size > 0 {
            total.div_ceil(page_size)
        } else {
            0
        };
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            page: 0,
            page_size: 0,
            total_pages: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let q: PageQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.page, 1);
        assert_eq!(q.page_size, 10);
    }

    #[test]
    fn test_page_size_clamped_to_upper_bound() {
        // 超大 page_size 被钳制，防止单查询拉取海量行
        let q: PageQuery = serde_json::from_str(r#"{"page":1,"pageSize":999999999}"#).unwrap();
        assert_eq!(q.page_size, MAX_PAGE_SIZE);
    }

    #[test]
    fn test_zero_values_normalized() {
        let q: PageQuery = serde_json::from_str(r#"{"page":0,"pageSize":0}"#).unwrap();
        assert_eq!(q.page, 1);
        assert_eq!(q.page_size, 1);
    }

    #[test]
    fn test_normal_values_pass_through() {
        let q: PageQuery = serde_json::from_str(r#"{"page":3,"pageSize":50}"#).unwrap();
        assert_eq!(q.page, 3);
        assert_eq!(q.page_size, 50);
    }

    #[test]
    fn test_total_pages_ceiling_division() {
        let r = PageResult::<i32>::new(Vec::new(), 101, 1, 10);
        assert_eq!(r.total_pages, 11);
        let r = PageResult::<i32>::new(Vec::new(), 0, 1, 10);
        assert_eq!(r.total_pages, 0);
    }
}
