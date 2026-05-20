use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    pub page: u64,
    pub page_size: u64,
    pub sort_field: Option<String>,
    pub sort_order: Option<String>,
}

impl Default for PageQuery {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 10,
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
            (total + page_size - 1) / page_size
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