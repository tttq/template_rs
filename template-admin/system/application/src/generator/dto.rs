use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableVo {
    pub table_name: String,
    pub table_comment: Option<String>,
    pub engine: Option<String>,
    pub create_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnVo {
    pub column_name: String,
    pub data_type: String,
    pub column_type: String,
    pub column_comment: Option<String>,
    pub is_nullable: String,
    pub is_primary_key: bool,
    pub column_default: Option<String>,
    pub ordinal_position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorConfig {
    pub table_name: String,
    pub module_name: String,
    pub business_name: String,
    pub entity_name: String,
    pub generate_frontend: bool,
    pub generate_backend: bool,
    pub parent_menu_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeFileVo {
    pub file_path: String,
    pub file_name: String,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewVo {
    pub files: Vec<CodeFileVo>,
}
