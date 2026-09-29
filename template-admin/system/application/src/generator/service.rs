use common::error::AppError;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use sea_orm::{ConnectionTrait, Statement, DatabaseBackend};
use std::io::Write;

use super::dto::*;

#[derive(Clone, Service)]
pub struct GeneratorAppService {
    #[inject(component)]
    db: DbConn,
}

impl GeneratorAppService {
    pub async fn list_tables(&self) -> Result<Vec<TableVo>, AppError> {
        let sql = r#"SELECT table_name, obj_description((quote_ident(table_schema) || '.' || quote_ident(table_name))::regclass) as table_comment FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE' ORDER BY table_name"#;
        let rows = self.db.query_all_raw(Statement::from_string(DatabaseBackend::Postgres, sql)).await.map_err(|e| AppError::Internal(e.to_string()))?;
        let mut tables = Vec::new();
        for row in rows {
            let table_name: String = row.try_get("", "table_name").unwrap_or_default();
            let table_comment: Option<String> = row.try_get("", "table_comment").ok().flatten();
            tables.push(TableVo {
                table_name,
                table_comment,
                engine: Some("PostgreSQL".to_string()),
                create_time: None,
            });
        }
        Ok(tables)
    }

    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnVo>, AppError> {
        let sql = format!(
            r#"SELECT c.column_name, c.data_type, c.udt_name as column_type,
               col_description((quote_ident(c.table_schema) || '.' || quote_ident(c.table_name))::regclass, c.ordinal_position) as column_comment,
               c.is_nullable, CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key,
               c.column_default, c.ordinal_position
               FROM information_schema.columns c
               LEFT JOIN (
                   SELECT ku.column_name, ku.table_name, ku.table_schema
                   FROM information_schema.table_constraints tc
                   JOIN information_schema.key_column_usage ku ON tc.constraint_name = ku.constraint_name
                   WHERE tc.constraint_type = 'PRIMARY KEY'
               ) pk ON c.column_name = pk.column_name AND c.table_name = pk.table_name AND c.table_schema = pk.table_schema
               WHERE c.table_schema = 'public' AND c.table_name = '{}'
               ORDER BY c.ordinal_position"#,
            table_name
        );
        let rows = self.db.query_all_raw(Statement::from_string(DatabaseBackend::Postgres, sql)).await.map_err(|e| AppError::Internal(e.to_string()))?;
        let mut columns = Vec::new();
        for row in rows {
            let column_name: String = row.try_get("", "column_name").unwrap_or_default();
            let data_type: String = row.try_get("", "data_type").unwrap_or_default();
            let column_type: String = row.try_get("", "column_type").unwrap_or_default();
            let column_comment: Option<String> = row.try_get("", "column_comment").ok().flatten();
            let is_nullable: String = row.try_get("", "is_nullable").unwrap_or_else(|_| "YES".to_string());
            let is_primary_key: bool = row.try_get("", "is_primary_key").unwrap_or(false);
            let column_default: Option<String> = row.try_get("", "column_default").ok().flatten();
            let ordinal_position: i32 = row.try_get("", "ordinal_position").unwrap_or(0);
            columns.push(ColumnVo {
                column_name,
                data_type,
                column_type,
                column_comment,
                is_nullable,
                is_primary_key,
                column_default,
                ordinal_position,
            });
        }
        Ok(columns)
    }

    pub async fn preview(&self, config: GeneratorConfig) -> Result<PreviewVo, AppError> {
        let columns = self.get_table_columns(&config.table_name).await?;
        let mut files = Vec::new();

        let entity_name_pascal = to_pascal_case(&config.entity_name);
        let entity_name_snake = to_snake_case(&config.entity_name);
        let module_path = &config.module_name;

        if config.generate_backend {
            files.push(CodeFileVo {
                file_path: format!("system/entity/src/{}.rs", entity_name_snake),
                file_name: format!("{}.rs", entity_name_snake),
                content: generate_entity_code(&entity_name_pascal, &entity_name_snake, &config.table_name, &columns),
                language: "rust".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("system/domain/src/{}_repository.rs", entity_name_snake),
                file_name: format!("{}_repository.rs", entity_name_snake),
                content: generate_repository_trait_code(&entity_name_pascal, &entity_name_snake),
                language: "rust".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("system/infrastructure/src/persistence/{}_repository_impl.rs", entity_name_snake),
                file_name: format!("{}_repository_impl.rs", entity_name_snake),
                content: generate_repository_impl_code(&entity_name_pascal, &entity_name_snake, module_path),
                language: "rust".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("system/application/src/{}/dto.rs", module_path),
                file_name: "dto.rs".to_string(),
                content: generate_dto_code(&entity_name_pascal, &entity_name_snake, &columns),
                language: "rust".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("system/application/src/{}/service.rs", module_path),
                file_name: "service.rs".to_string(),
                content: generate_service_code(&entity_name_pascal, &entity_name_snake, module_path, &columns),
                language: "rust".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("system/interface/src/handlers/{}_handler.rs", entity_name_snake),
                file_name: format!("{}_handler.rs", entity_name_snake),
                content: generate_handler_code(&entity_name_pascal, &entity_name_snake, module_path),
                language: "rust".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("system/application/src/{}/mod.rs", module_path),
                file_name: "mod.rs".to_string(),
                content: generate_module_mod_code(&entity_name_pascal),
                language: "rust".to_string(),
            });
        }

        if config.generate_frontend {
            files.push(CodeFileVo {
                file_path: format!("src/api/{}.ts", entity_name_snake),
                file_name: format!("{}.ts", entity_name_snake),
                content: generate_frontend_api_code(&entity_name_pascal, &entity_name_snake),
                language: "typescript".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("src/views/{}/index.vue", entity_name_snake),
                file_name: "index.vue".to_string(),
                content: generate_frontend_page_code(&entity_name_pascal, &entity_name_snake, module_path, &columns),
                language: "vue".to_string(),
            });
        }

        Ok(PreviewVo { files })
    }

    pub async fn download(&self, config: GeneratorConfig) -> Result<Vec<u8>, AppError> {
        let preview = self.preview(config).await?;
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for file in &preview.files {
                zip.start_file(&file.file_path, options).map_err(|e| AppError::Internal(e.to_string()))?;
                zip.write_all(file.content.as_bytes()).map_err(|e| AppError::Internal(e.to_string()))?;
            }
            zip.finish().map_err(|e| AppError::Internal(e.to_string()))?;
        }
        Ok(buf)
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_').map(|w| {
        let mut c = w.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }).collect()
}

fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").to_lowercase()
}

fn rust_type(pg_type: &str) -> String {
    match pg_type {
        "int8" | "bigint" => "i64",
        "int4" | "integer" | "int" => "i32",
        "int2" | "smallint" => "i16",
        "bool" | "boolean" => "bool",
        "float4" | "real" => "f32",
        "float8" | "double precision" => "f64",
        "numeric" | "decimal" => "f64",
        "timestamptz" | "timestamp with time zone" => "DateTimeUtc",
        "date" => "chrono::NaiveDate",
        "json" | "jsonb" => "serde_json::Value",
        _ => "String",
    }.to_string()
}

fn is_optional(pg_type: &str, is_nullable: &str, is_pk: bool) -> bool {
    !is_pk && is_nullable == "YES" && pg_type != "timestamptz" && pg_type != "timestamp with time zone"
}

/// 业务列（排除主键/审计/软删/租户/乐观锁等系统列），导入导出按此列序生成
fn business_columns(columns: &[ColumnVo]) -> Vec<&ColumnVo> {
    columns
        .iter()
        .filter(|c| {
            !c.is_primary_key
                && !matches!(
                    c.column_name.as_str(),
                    "create_time" | "update_time" | "create_by" | "create_id" | "update_by"
                        | "update_id" | "version" | "delete_flag" | "tenant_id"
                )
        })
        .collect()
}

/// 列在导入落库时的取值表达式（基于 fast_excel::RowData / CellValue 解析，产出 Rust 类型值）
fn import_value_expr(col: &ColumnVo) -> String {
    let rt = rust_type(&col.column_type);
    let optional = is_optional(&col.column_type, &col.is_nullable, false);
    let base = match rt.as_str() {
        "i64" => "c.to_i64()",
        "i32" => "c.to_i64().map(|v| v as i32)",
        "i16" => "c.to_i64().map(|v| v as i16)",
        "f64" | "f32" => "c.to_f64()",
        "bool" => "c.to_bool()",
        "chrono::NaiveDate" => "c.to_date()",
        "DateTimeUtc" => "c.to_datetime().map(|d| d.and_utc())",
        "serde_json::Value" => {
            "Some(serde_json::Value::String(c.trimmed())).filter(|v| !v.as_str().unwrap_or(\"\").is_empty())"
        }
        _ => "Some(c.trimmed()).filter(|s| !s.is_empty())",
    };
    if optional {
        base.to_string()
    } else if rt == "DateTimeUtc" {
        // chrono::DateTime 无 Default，非空时间列缺省取当前时间
        format!("{base}.unwrap_or_else(|| chrono::Utc::now())")
    } else {
        format!("{base}.unwrap_or_default()")
    }
}

fn generate_entity_code(_name: &str, _snake: &str, table: &str, columns: &[ColumnVo]) -> String {
    let mut fields = String::new();
    for col in columns {
        let rt = rust_type(&col.column_type);
        let opt = is_optional(&col.column_type, &col.is_nullable, col.is_primary_key);
        let field_name = col.column_name.clone();
        let type_str = if opt { format!("Option<{}>", rt) } else { rt.clone() };

        let mut attrs = String::new();
        if col.is_primary_key {
            attrs.push_str("    #[sea_orm(primary_key, auto_generate)]\n");
        }
        if col.column_name == "create_time" {
            attrs.push_str("    #[serde(with = \"datetime_format\")]\n");
            attrs.push_str("    #[sea_orm_ext(insert)]\n");
        } else if col.column_name == "update_time" {
            attrs.push_str("    #[serde(with = \"datetime_format\")]\n");
            attrs.push_str("    #[sea_orm_ext(update)]\n");
        } else if col.column_name == "create_by" || col.column_name == "create_id" {
            attrs.push_str("    #[sea_orm_ext(insert)]\n");
        } else if col.column_name == "update_by" || col.column_name == "update_id" {
            attrs.push_str("    #[sea_orm_ext(update)]\n");
        } else if col.column_name == "version" {
            attrs.push_str("    #[sea_orm(version)]\n");
            attrs.push_str("    #[sea_orm_ext(insert_update)]\n");
        } else if col.column_name == "delete_flag" {
            attrs.push_str("    #[soft_delete(default = 0, del = 1)]\n");
        } else if col.column_name == "tenant_id" {
            attrs.push_str("    #[sea_orm_ext(TENANT)]\n");
        }

        fields.push_str(&format!("{}    pub {}: {},\n", attrs, field_name, type_str));
    }

    format!(r#"use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "{}")]
#[serde(rename_all = "camelCase")]
pub struct Model {{
{}
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}
"#, table, fields)
}

fn generate_repository_trait_code(name: &str, snake: &str) -> String {
    format!(r#"use common::error::AppError;
use system_entity::{snake}::Model;

#[async_trait::async_trait]
pub trait {name}Repository: Send + Sync {{
    async fn find_by_id(&self, id: String) -> Result<Option<Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<Model>, AppError>;
    async fn create(&self, model: system_entity::{snake}::ActiveModel) -> Result<Model, AppError>;
    async fn update(&self, model: system_entity::{snake}::ActiveModel) -> Result<Model, AppError>;
    async fn delete_by_id(&self, id: String) -> Result<(), AppError>;
}}
"#)
}

fn generate_repository_impl_code(name: &str, snake: &str, module: &str) -> String {
    format!(r#"use common::error::AppError;
use sea_orm_ext::DbConn;
use summer::plugin::service::Service;
use system_entity::{snake};
use system_domain::{module}::{name}Repository;
use sea_orm::{{ActiveModelTrait, EntityTrait, Set, QueryFilter, ColumnTrait}};

#[derive(Clone, Service)]
pub struct {name}RepositoryImpl {{
    #[inject(component)]
    db: DbConn,
}}

#[async_trait::async_trait]
impl {name}Repository for {name}RepositoryImpl {{
    async fn find_by_id(&self, id: String) -> Result<Option<{snake}::Model>, AppError> {{
        {snake}::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }}

    async fn find_all(&self) -> Result<Vec<{snake}::Model>, AppError> {{
        {snake}::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }}

    async fn create(&self, model: {snake}::ActiveModel) -> Result<{snake}::Model, AppError> {{
        model.insert(&self.db).await.map_err(|e| AppError::Internal(e.to_string()))
    }}

    async fn update(&self, model: {snake}::ActiveModel) -> Result<{snake}::Model, AppError> {{
        model.update(&self.db).await.map_err(|e| AppError::Internal(e.to_string()))
    }}

    async fn delete_by_id(&self, id: String) -> Result<(), AppError> {{
        let model = {snake}::Entity::find_by_id(&id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("@record_not_found".to_string()))?;
        let mut am: {snake}::ActiveModel = model.into();
        am.delete_flag = sea_orm::Set(1);
        am.update(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }}
}}
"#)
}

fn generate_dto_code(name: &str, snake: &str, columns: &[ColumnVo]) -> String {
    let mut vo_fields = String::new();
    let mut create_fields = String::new();
    let mut update_fields = String::new();
    // VO ← 实体逐列映射（字段名/类型与实体一致；时间列两边都是 chrono::DateTime<Utc>）
    let mut from_fields = String::new();

    for col in columns {
        let rt = rust_type(&col.column_type);
        let opt = is_optional(&col.column_type, &col.is_nullable, col.is_primary_key);
        let field_name = col.column_name.clone();
        from_fields.push_str(&format!("            {}: m.{},\n", field_name, field_name));

        if col.is_primary_key || col.column_name == "create_time" || col.column_name == "update_time"
            || col.column_name == "create_by" || col.column_name == "update_by"
            || col.column_name == "create_id" || col.column_name == "update_id"
            || col.column_name == "version" || col.column_name == "delete_flag"
            || col.column_name == "tenant_id" {
            if col.is_primary_key {
                vo_fields.push_str(&format!("    pub {}: {},\n", field_name, rt));
            } else if col.column_name == "create_time" || col.column_name == "update_time" {
                vo_fields.push_str(&format!("    #[serde(with = \"datetime_format\")]\n    pub {}: DateTime<Utc>,\n", field_name));
            } else {
                let type_str = if opt { format!("Option<{}>", rt) } else { rt.clone() };
                vo_fields.push_str(&format!("    pub {}: {},\n", field_name, type_str));
            }
            continue;
        }

        let type_str = if opt { format!("Option<{}>", rt) } else { rt.clone() };
        vo_fields.push_str(&format!("    pub {}: {},\n", field_name, type_str));
        create_fields.push_str(&format!("    pub {}: {},\n", field_name, type_str));
        update_fields.push_str(&format!("    pub {}: Option<{}>,\n", field_name, rt));
    }

    format!(r#"use chrono::{{DateTime, Utc}};
use common::datetime_format;
use serde::{{Deserialize, Serialize}};
use system_entity::{snake};
use sea_orm::ActiveValue::Set;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Create{name}Dto {{
{create_fields}}}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Update{name}Dto {{
    #[serde(default)]
    pub id: Option<String>,
{update_fields}
    #[serde(default)]
    pub version: Option<i32>,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {name}Vo {{
{vo_fields}}}

impl From<{snake}::Model> for {name}Vo {{
    fn from(m: {snake}::Model) -> Self {{
        Self {{
{from_fields}        }}
    }}
}}
"#)
}

fn generate_service_code(name: &str, snake: &str, _module: &str, columns: &[ColumnVo]) -> String {
    let biz = business_columns(columns);
    // 导出列名清单（静态写入生成代码，供导入模板/导出共用）
    let export_cols = biz
        .iter()
        .map(|c| format!("\"{}\"", c.column_name))
        .collect::<Vec<_>>()
        .join(", ");
    // 导入落库时逐字段的 Set 赋值语句（get 为取文件单元格的闭包）
    let import_set = biz
        .iter()
        .map(|c| {
            let expr = import_value_expr(c);
            format!("            am.{field} = Set(get(\"{col}\").and_then(|c| {expr}));",
                field = c.column_name,
                col = c.column_name)
        })
        .collect::<Vec<_>>()
        .join("\n");
    // 导出时逐字段取文本值（IntoCell 统一转 CellValue，再取文本；Option 自动映射空值）
    let export_text_entries = biz
        .iter()
        .map(|c| format!("                ::fast_excel::IntoCell::into_cell(&m.{}).to_text(),", c.column_name))
        .collect::<Vec<_>>()
        .join("\n");
    // 新增时逐字段从 DTO 落库
    let create_set = biz
        .iter()
        .map(|c| format!("            {}: Set(dto.{}),", c.column_name, c.column_name))
        .collect::<Vec<_>>()
        .join("\n");
    // 编辑时仅覆盖传入字段（未传 = 不修改）
    let update_set = biz
        .iter()
        .map(|c| {
            format!(
                "        if let Some(v) = dto.{} {{ model.{} = Set(v); }}",
                c.column_name, c.column_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(r#"use common::error::AppError;
use common::pagination::{{PageQuery, PageResult}};
use summer::App;
use summer::plugin::ComponentRegistry;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::{snake};
use sea_orm::{{EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, PaginatorTrait, TransactionTrait}};
use super::dto::*;

#[derive(Clone, Service)]
pub struct {name}AppService {{
    #[inject(component)]
    db: DbConn,
}}

impl {name}AppService {{
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<{name}Vo>, AppError> {{
        let paginator = {snake}::Entity::find()
            .paginate(&self.db, query.page_size);
        let total = paginator.num_items().await.map_err(|e| AppError::Internal(e.to_string()))?;
        let items = paginator.fetch_page(query.page - 1).await.map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(PageResult::new(items.into_iter().map({name}Vo::from).collect(), total, query.page, query.page_size))
    }}

    pub async fn list_all(&self) -> Result<Vec<{name}Vo>, AppError> {{
        let items = {snake}::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(items.into_iter().map({name}Vo::from).collect())
    }}

    pub async fn get_by_id(&self, id: String) -> Result<{name}Vo, AppError> {{
        let model = {snake}::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("@record_not_found".to_string()))?;
        Ok({name}Vo::from(model))
    }}

    pub async fn create(&self, dto: Create{name}Dto) -> Result<{name}Vo, AppError> {{
        let model = {snake}::ActiveModel {{
{create_set}
            ..Default::default()
        }};
        let result = model.insert(&self.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
        Ok({name}Vo::from(result))
    }}

    pub async fn update(&self, id: String, dto: Update{name}Dto) -> Result<{name}Vo, AppError> {{
        let existing = {snake}::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("@record_not_found".to_string()))?;
        let mut model: {snake}::ActiveModel = existing.into();
        model.id = Set(id);
        model.version = Set(dto.version.unwrap_or(0));
{update_set}
        let result = model.update(&self.db).await.map_err(|e| AppError::Internal(e.to_string()))?;
        Ok({name}Vo::from(result))
    }}

    pub async fn delete(&self, id: String) -> Result<(), AppError> {{
        {snake}::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }}

    // ===================== 导入导出（fast-excel_rs + 导出中心分流） =====================
    /// 导出列（业务列；导入模板 / 导入校验 / 同步导出 / 导出中心异步任务共用同一列序）
    pub const EXPORT_COLS: &'static [&'static str] = &[{export_cols}];
    /// 同步导出行数上限（前端同名常量需保持一致：超过改走「导出中心」异步任务）
    pub const MAX_EXPORT_ROWS: u64 = common::xlsx::MAX_SYNC_ROWS;

    /// 导入预览（不落库）：流式解析前 200 行，返回每行「列名→文本值」+ 解析错误
    pub async fn import_preview(&self, bytes: Vec<u8>) -> Result<serde_json::Value, AppError> {{
        use fast_excel::{{DynamicRow, ExcelReader, ExcelRow, ReadOptions, SheetSelector}};
        let reader = ExcelReader::from_bytes(bytes).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let sheet = reader.select(&SheetSelector::First)
            .map_err(|e| AppError::BadRequest(e.to_string()))?
            .into_iter().next().cloned()
            .ok_or_else(|| AppError::BadRequest("Excel 文件无工作表".to_string()))?;
        let mut options = ReadOptions::new();
        options.max_rows = Some(200);
        let mut stream = reader.stream_sheet(&sheet, &DynamicRow::columns(), options)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        let mut rows = Vec::new();
        while let Some(row) = stream.next_row().map_err(|e| AppError::BadRequest(e.to_string()))? {{
            let mut values = std::collections::HashMap::new();
            for (h, v) in DynamicRow::from_row(&row)
                .map(|d| d.entries)
                .unwrap_or_default()
            {{
                values.insert(h, v.to_text());
            }}
            rows.push(serde_json::json!({{
                "row": row.row_index,
                "values": values,
            }}));
        }}
        Ok(serde_json::json!({{
            "totalRows": rows.len(),
            "rows": rows,
        }}))
    }}

    /// 导入提交：流式解析 DynamicRow → 按列名映射字段 → 每 500 行一个短事务写入
    pub async fn import_commit(&self, bytes: Vec<u8>) -> Result<serde_json::Value, AppError> {{
        use fast_excel::{{DynamicRow, ExcelReader, ExcelRow, ReadOptions, SheetSelector}};
        use sea_orm::ActiveValue::Set;
        let reader = ExcelReader::from_bytes(bytes).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let sheet = reader.select(&SheetSelector::First)
            .map_err(|e| AppError::BadRequest(e.to_string()))?
            .into_iter().next().cloned()
            .ok_or_else(|| AppError::BadRequest("Excel 文件无工作表".to_string()))?;
        let mut options = ReadOptions::new();
        let mut stream = reader.stream_sheet(&sheet, &DynamicRow::columns(), options)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        const BATCH_SIZE: usize = 500;
        let mut tx: Option<sea_orm::DatabaseTransaction> = None;
        let mut batch = 0usize;
        let mut success = 0u64;
        let mut errors = Vec::<serde_json::Value>::new();
        while let Some(row) = stream.next_row().map_err(|e| AppError::BadRequest(e.to_string()))? {{
            let entries = match DynamicRow::from_row(&row) {{
                Ok(d) => d.entries,
                Err(e) => {{
                    errors.push(serde_json::json!({{
                        "row": row.row_index,
                        "messages": e.iter().map(|x| x.message.clone()).collect::<Vec<_>>(),
                    }}));
                    continue;
                }}
            }};
            let mut am = {snake}::ActiveModel {{
                ..Default::default()
            }};
            let mut row_data = std::collections::HashMap::<String, fast_excel::CellValue>::new();
            for (h, v) in entries {{
                row_data.insert(h, v);
            }}
            let get = |h: &str| row_data.get(h);
            // 逐字段映射（按列名取文件单元格，类型按列转换）
{import_set}
            if tx.is_none() {{
                tx = Some(self.db.inner().begin().await.map_err(|e| AppError::Internal(e.to_string()))?);
            }}
            am.insert(tx.as_ref().unwrap()).await.map_err(|e| AppError::Internal(e.to_string()))?;
            success += 1;
            batch += 1;
            if batch >= BATCH_SIZE {{
                if let Some(t) = tx.take() {{
                    t.commit().await.map_err(|e| AppError::Internal(e.to_string()))?;
                }}
                batch = 0;
            }}
        }}
        if let Some(t) = tx.take() {{
            t.commit().await.map_err(|e| AppError::Internal(e.to_string()))?;
        }}
        Ok(serde_json::json!({{
            "successRows": success,
            "errorRows": errors.len(),
            "errors": errors,
        }}))
    }}

    /// 生成导入模板（xlsx：表头 + 示例行 + 填写说明）
    pub async fn generate_template(&self) -> Result<Vec<u8>, AppError> {{
        use fast_excel::{{build_template, TemplateSpec, CellValue}};
        let columns: Vec<fast_excel::ColumnDef> = Self::EXPORT_COLS.iter().map(|h| fast_excel::ColumnDef::new(*h)).collect();
        let sample: Vec<CellValue> = Self::EXPORT_COLS.iter().map(|_| CellValue::Empty).collect();
        let spec = TemplateSpec::new(columns)
            .sheet_name("{name}")
            .note("表头列为导入字段；示例行为空占位，正式导入前请替换或删除。")
            .sample_row(sample);
        build_template(&spec).map_err(|e| AppError::BadRequest(e.to_string()))
    }}

    /// 可导出行数（导出分流判定）
    pub async fn export_count(&self) -> Result<u64, AppError> {{
        let total = {snake}::Entity::find()
            .count(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(total)
    }}

    /// 导出数据行（同步导出与导出中心异步任务共用同一列序）
    pub async fn export_rows(&self) -> Result<Vec<Vec<String>>, AppError> {{
        let items: Vec<{snake}::Model> = {snake}::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(items
            .into_iter()
            .map(|m| vec![
{export_text_entries}
            ])
            .collect())
    }}

    /// 同步导出 xlsx（≤ [`Self::MAX_EXPORT_ROWS`] 条功能页内直接下载；超过走导出中心异步任务）
    pub async fn export_xlsx(&self) -> Result<Vec<u8>, AppError> {{
        let total = self.export_count().await?;
        if total > Self::MAX_EXPORT_ROWS {{
            return Err(AppError::BadRequest(common::xlsx::too_large_message(total)));
        }}
        common::xlsx::xlsx_bytes("{name}", Self::EXPORT_COLS, self.export_rows().await?)
    }}
}}

/// 异步导出（> 10 万条）：导出中心按 `task_type = "{snake}"` 直接调用。
/// 注册即完成，无需执行器 / install() / main.rs 聚合。
async fn {snake}_export_rows(_ctx: fast_excel::ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {{
    let service = App::global()
        .try_get_component::<{name}AppService>()
        .map_err(|e| AppError::Internal(format!("服务组件未就绪：{{e}}")))?;
    service.export_rows().await
}}

fast_excel::export_task! {{
    task_type = "{snake}",
    sheet_name = "{name}",
    headers = {name}AppService::EXPORT_COLS,
    rows = {snake}_export_rows,
}}
"#)
}

fn generate_handler_code(name: &str, snake: &str, module: &str) -> String {
    format!(r#"use summer_web::{{get, post, put, delete, nest}};
use summer_web::extractor::{{Component, Json, Path, Query, Multipart}};
use summer_web::axum::response::IntoResponse;
use summer_web::axum::body::Body;
use summer_web::axum::http::StatusCode;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::{module}::service::{name}AppService;
use system_application::{module}::dto::*;

#[nest("/system")]
mod controller {{
    use super::*;

    #[get("/{snake}s")]
    #[sa_check_permission("{snake}:list")]
    async fn list_{snake}(
        Component(service): Component<{name}AppService>,
        Query(query): Query<PageQuery>,
    ) -> impl IntoResponse {{
        match service.list(query).await {{
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        }}
    }}

    #[get("/{snake}s/{{id}}")]
    #[sa_check_permission("{snake}:list")]
    async fn get_{snake}(
        Component(service): Component<{name}AppService>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {{
        match service.get_by_id(id).await {{
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        }}
    }}

    #[post("/{snake}s")]
    #[sa_check_permission("{snake}:add")]
    async fn create_{snake}(
        Component(service): Component<{name}AppService>,
        Json(dto): Json<Create{name}Dto>,
    ) -> impl IntoResponse {{
        match service.create(dto).await {{
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        }}
    }}

    #[put("/{snake}s/{{id}}")]
    #[sa_check_permission("{snake}:edit")]
    async fn update_{snake}(
        Component(service): Component<{name}AppService>,
        Path(id): Path<String>,
        Json(dto): Json<Update{name}Dto>,
    ) -> impl IntoResponse {{
        match service.update(id, dto).await {{
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        }}
    }}

    #[delete("/{snake}s/{{id}}")]
    #[sa_check_permission("{snake}:delete")]
    async fn delete_{snake}(
        Component(service): Component<{name}AppService>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {{
        match service.delete(id).await {{
            Ok(()) => Json(ApiResponse::success("@deleted_ok")),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        }}
    }}

    // ---------------- 导入导出（fast-excel_rs） ----------------

    /// Excel 导入（multipart：file + mode=preview|commit）
    #[post("/{snake}s/import")]
    #[sa_check_permission("{snake}:import")]
    async fn import_{snake}(
        Component(service): Component<{name}AppService>,
        mut form: Multipart,
    ) -> Result<impl IntoResponse, WebError> {{
        let mut bytes: Option<Vec<u8>> = None;
        let mut mode = "preview".to_string();
        while let Some(field) = form.next_field().await.map_err(|e| {{
            WebError::from(summer_web::error::KnownWebError::bad_request(format!("解析上传内容失败: {{e}}")))
        }})? {{
            match field.name().unwrap_or_default() {{
                "file" => {{
                    let data = field.bytes().await.map_err(|e| {{
                        WebError::from(summer_web::error::KnownWebError::bad_request(format!("读取上传文件失败: {{e}}")))
                    }})?;
                    bytes = Some(data.to_vec());
                }}
                "mode" => {{
                    if field.text().await.unwrap_or_default() == "commit" {{
                        mode = "commit".to_string();
                    }}
                }}
                _ => {{}}
            }}
        }}
        let Some(bytes) = bytes else {{
            return Ok(Json(ApiResponse::<()>::error(500, "缺少必填的 Excel 文件（字段名 file）")));
        }};
        let result = if mode == "commit" {{
            service.import_commit(bytes).await
        }} else {{
            service.import_preview(bytes).await
        }};
        Ok(match result {{
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
        }})
    }}

    /// 下载导入模板（xlsx）
    #[get("/{snake}s/import/template")]
    #[sa_check_permission("{snake}:import")]
    async fn import_template_{snake}(
        Component(service): Component<{name}AppService>,
    ) -> impl IntoResponse {{
        match service.generate_template().await {{
            Ok(bytes) => (
                StatusCode::OK,
                [
                    ("content-type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                    ("content-disposition", "attachment; filename=\"{snake}-import-template.xlsx\""),
                ],
                Body::from(bytes),
            ).into_response(),
            Err(e) => Json(ApiResponse::<()>::error(500, &e.to_string())).into_response(),
        }}
    }}

    /// 可导出行数（导出分流判定）
    #[get("/{snake}s/export/count")]
    #[sa_check_permission("{snake}:export")]
    async fn export_count_{snake}(
        Component(service): Component<{name}AppService>,
    ) -> impl IntoResponse {{
        match service.export_count().await {{
            Ok(total) => Json(ApiResponse::success(serde_json::json!({{ "total": total }}))),
            Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
        }}
    }}

    /// 同步导出 xlsx（≤ MAX_EXPORT_ROWS 直接下载；超过请到「导出中心」异步导出）
    #[get("/{snake}s/export")]
    #[sa_check_permission("{snake}:export")]
    async fn export_{snake}(
        Component(service): Component<{name}AppService>,
    ) -> impl IntoResponse {{
        match service.export_xlsx().await {{
            Ok(bytes) => (
                StatusCode::OK,
                [
                    ("content-type", common::xlsx::XLSX_MIME),
                    ("content-disposition", "attachment; filename=\"{snake}.xlsx\""),
                ],
                Body::from(bytes),
            ).into_response(),
            Err(e) => Json(ApiResponse::<()>::error(500, &e.to_string())).into_response(),
        }}
    }}
}}
"#)
}

/// 生成 application 模块入口（只声明 dto/service；异步导出在 service.rs 内用宏注册）
fn generate_module_mod_code(name: &str) -> String {
    format!(r#"pub mod dto;
pub mod service;

pub use service::{name}AppService;
"#)
}

fn generate_frontend_api_code(name: &str, snake: &str) -> String {
    format!(r#"import request from './request'

export interface {name}Vo {{
  id: string
}}

const API_BASE = '/api'

export const {snake}Api = {{
  list: (params: any) => request.get('/system/{snake}s', {{ params }}),
  getById: (id: string) => request.get(`/system/{snake}s/${{id}}`),
  create: (data: any) => request.post('/system/{snake}s', data),
  update: (id: string, data: any) => request.put(`/system/{snake}s/${{id}}`, data),
  delete: (id: string) => request.delete(`/system/{snake}s/${{id}}`),

  /** Excel 导入（mode=preview 预览 / commit 落库） */
  importData: (file: File, mode?: 'preview' | 'commit') => {{
    const form = new FormData()
    form.append('file', file)
    form.append('mode', mode || 'preview')
    return request.post(`/system/{snake}s/import`, form, {{
      headers: {{ 'Content-Type': 'multipart/form-data' }},
      timeout: 120000,
    }})
  }},

  /** 下载导入模板（xlsx） */
  importTemplate: async () => {{
    const token = localStorage.getItem('token')
    const resp = await fetch(`${{API_BASE}}/system/{snake}s/import/template`, {{
      headers: {{ ...(token ? {{ Authorization: `Bearer ${{token}}` }} : {{}}) }},
    }})
    if (!resp.ok) throw new Error(`模板下载失败（${{resp.status}}）`)
    const blob = await resp.blob()
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = '{snake}-import-template.xlsx'
    link.click()
    URL.revokeObjectURL(link.href)
  }},

  /** 可导出行数（导出分流判定） */
  exportCount: async () => {{
    const resp = await request.get<{{ total: number }}>(`/system/{snake}s/export/count`)
    return resp.total
  }},

  /** 同步导出 xlsx（≤ MAX_EXPORT_ROWS 直接下载） */
  export: async () => {{
    const token = localStorage.getItem('token')
    const resp = await fetch(`${{API_BASE}}/system/{snake}s/export`, {{
      headers: {{ ...(token ? {{ Authorization: `Bearer ${{token}}` }} : {{}}) }},
    }})
    if (!resp.ok) throw new Error(`导出失败（${{resp.status}}）`)
    const blob = await resp.blob()
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = '{snake}.xlsx'
    link.click()
    URL.revokeObjectURL(link.href)
  }},
}}
"#)
}

fn generate_frontend_page_code(_name: &str, snake: &str, _module: &str, columns: &[ColumnVo]) -> String {
    let search_fields: Vec<&str> = columns.iter()
        .filter(|c| !c.is_primary_key && (c.data_type == "character varying" || c.data_type == "text"))
        .take(3)
        .map(|c| c.column_name.as_str())
        .collect();

    let table_columns: Vec<String> = columns.iter()
        .filter(|c| !matches!(c.column_name.as_str(), "delete_flag" | "version" | "tenant_id" | "pass_word"))
        .take(8)
        .map(|c| {
            let label = c.column_comment.as_deref().unwrap_or(&c.column_name);
            let data_index = c.column_name.clone();
            let width = if c.is_primary_key { "100" } else { "150" };
            format!("      {{ title: '{}', dataIndex: '{}', width: {} }}", label, data_index, width)
        })
        .collect();

    format!(r#"<template>
  <div>
    <a-card :bordered="false">
      <a-row :gutter="16" class="mb-4">
{search_fields}
        <a-col :span="4">
          <a-button type="primary" @click="fetchData">查询</a-button>
          <a-button class="ml-2" @click="resetSearch">重置</a-button>
        </a-col>
      </a-row>
      <div class="mb-4">
        <a-space>
          <a-button type="primary" @click="showCreateModal">
            <template #icon><PlusOutlined /></template>
            新增
          </a-button>
          <a-button @click="downloadTemplate">下载导入模板</a-button>
          <a-button @click="openImport">导入</a-button>
          <a-button :loading="exporting" @click="doExport">导出</a-button>
        </a-space>
      </div>
      <a-table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        :pagination="pagination"
        @change="handleTableChange"
        row-key="id"
      >
        <template #bodyCell="{{ column, record }}">
          <template v-if="column.dataIndex === 'action'">
            <a-button type="link" @click="showEditModal(record)">编辑</a-button>
            <a-popconfirm title="确定删除？" @confirm="handleDelete(record.id)">
              <a-button type="link" danger>删除</a-button>
            </a-popconfirm>
          </template>
        </template>
      </a-table>
    </a-card>

    <a-modal v-model:open="modalVisible" :title="isEdit ? '编辑' : '新增'" @ok="handleSubmit" :confirm-loading="submitLoading">
      <a-form :model="formData" layout="vertical">
      </a-form>
    </a-modal>

    <!-- 导入弹窗：上传 xlsx → 预览 → 确认提交 -->
    <a-modal v-model:open="importVisible" title="Excel 导入" :footer="null" width="720px">
      <div class="mb-4">
        <a-upload
          :before-upload="handleImportFile"
          :show-upload-list="false"
          accept=".xlsx"
        >
          <a-button>选择文件</a-button>
        </a-upload>
        <a-space class="ml-2">
          <a-button @click="downloadTemplate">下载模板</a-button>
        </a-space>
      </div>
      <template v-if="importPreview">
        <a-alert
          :type="importPreview.errorRows > 0 ? 'warning' : 'success'"
          :message="`共 ${{ importPreview.totalRows }} 行，可导入 ${{ importPreview.totalRows - importPreview.errorRows }} 行，错误 ${{ importPreview.errorRows }} 行`"
          class="mb-4"
        />
        <a-table
          size="small"
          :columns="importColumns"
          :data-source="importPreview.rows"
          :pagination="{{ pageSize: 5 }}"
          row-key="row"
        >
          <template #bodyCell="{{ column, record }}">
            <template v-if="column.dataIndex === 'values'">
              <pre class="import-values">{{ JSON.stringify(record.values, null, 0) }}</pre>
            </template>
          </template>
        </a-table>
        <div class="mt-4">
          <a-button type="primary" :loading="importing" @click="commitImport">确认导入</a-button>
        </div>
      </template>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import {{ ref, reactive, onMounted }} from 'vue'
import {{ PlusOutlined }} from '@ant-design/icons-vue'
import {{ message }} from 'ant-design-vue'
import {{ {snake}Api }} from '@/api/{snake}'
import {{ exportTaskApi }} from '@/api/system/exportTask'

const loading = ref(false)
const submitLoading = ref(false)
const modalVisible = ref(false)
const isEdit = ref(false)
const tableData = ref([])
const formData = reactive({{}})

const pagination = reactive({{
  current: 1,
  pageSize: 10,
  total: 0,
}})

const columns = [
{table_cols}
  {{ title: '操作', dataIndex: 'action', width: 150, fixed: 'right' }},
]

async function fetchData() {{
  loading.value = true
  try {{
    const res = await {snake}Api.list({{ page: pagination.current, pageSize: pagination.pageSize }})
    tableData.value = res.items
    pagination.total = res.total
  }} finally {{
    loading.value = false
  }}
}}

function handleTableChange(pag: any) {{
  pagination.current = pag.current
  pagination.pageSize = pag.pageSize
  fetchData()
}}

function resetSearch() {{
  pagination.current = 1
  fetchData()
}}

function showCreateModal() {{
  isEdit.value = false
  Object.keys(formData).forEach(k => delete (formData as any)[k])
  modalVisible.value = true
}}

function showEditModal(record: any) {{
  isEdit.value = true
  Object.assign(formData, record)
  modalVisible.value = true
}}

async function handleSubmit() {{
  submitLoading.value = true
  try {{
    if (isEdit.value) {{
      await {snake}Api.update(formData.id, formData)
      message.success('更新成功')
    }} else {{
      await {snake}Api.create(formData)
      message.success('创建成功')
    }}
    modalVisible.value = false
    fetchData()
  }} finally {{
    submitLoading.value = false
  }}
}}

async function handleDelete(id: string) {{
  await {snake}Api.delete(id)
  message.success('删除成功')
  fetchData()
}}

// ---------------- 导入导出 ----------------
const importing = ref(false)
const exporting = ref(false)
const importVisible = ref(false)
const importFile = ref<File | null>(null)
const importPreview = ref<any>(null)
const importColumns = [
  {{ title: '行号', dataIndex: 'row', width: 70 }},
  {{ title: '单元格值', dataIndex: 'values' }},
]

function openImport() {{
  importFile.value = null
  importPreview.value = null
  importVisible.value = true
}}

async function handleImportFile(file: File) {{
  importFile.value = file
  try {{
    importPreview.value = await {snake}Api.importData(file, 'preview')
    message.success('预览完成，请核对后确认导入')
  }} catch (e: any) {{
    message.error(e.message || '预览失败')
  }}
  return false
}}

async function commitImport() {{
  if (!importFile.value) {{
    message.warning('请先选择文件')
    return
  }}
  importing.value = true
  try {{
    const res = await {snake}Api.importData(importFile.value, 'commit')
    message.success(`导入完成：成功 ${{ res.successRows }} 行，失败 ${{ res.errorRows }} 行`)
    importVisible.value = false
    fetchData()
  }} catch (e: any) {{
    message.error(e.message || '导入失败')
  }} finally {{
    importing.value = false
  }}
}}

async function downloadTemplate() {{
  try {{
    await {snake}Api.importTemplate()
  }} catch (e: any) {{
    message.error(e.message || '模板下载失败')
  }}
}}

/** 导出的同步上限（与后端 `MAX_EXPORT_ROWS` / `common::xlsx::MAX_SYNC_ROWS` 一致：10 万条） */
const EXPORT_MAX = 100000

async function doExport() {{
  exporting.value = true
  try {{
    const total = await {snake}Api.exportCount()
    if (total > EXPORT_MAX) {{
      // 超过 10 万条：自动转「导出中心」异步导出，完成后到导出中心查看/下载
      await exportTaskApi.create('{snake}', {{}})
      message.success('导出成功，请自行到导出中心查看')
      return
    }}
    // ≤ 10 万条：功能页内直接下载（后端同时在导出中心登记一条记录）
    await {snake}Api.export()
    message.success('导出成功')
  }} catch (e: any) {{
    message.error(e.message || '导出失败')
  }} finally {{
    exporting.value = false
  }}
}}

onMounted(() => fetchData())
</script>

<style scoped>
.import-values {{
  margin: 0;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
}}
</style>
"#,
        search_fields = search_fields.iter().map(|f| format!("        <a-col :span=\"4\">\n          <a-input v-model:value=\"searchForm.{}\" placeholder=\"{}\" allow-clear @pressEnter=\"fetchData\" />\n        </a-col>", f, f)).collect::<Vec<_>>().join("\n"),
        table_cols = table_columns.join(",\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_columns() -> Vec<ColumnVo> {
        vec![
            ColumnVo {
                column_name: "id".into(),
                data_type: "character varying".into(),
                column_type: "varchar".into(),
                column_comment: Some("主键".into()),
                is_nullable: "NO".into(),
                is_primary_key: true,
                column_default: None,
                ordinal_position: 1,
            },
            ColumnVo {
                column_name: "name".into(),
                data_type: "character varying".into(),
                column_type: "varchar".into(),
                column_comment: Some("名称".into()),
                is_nullable: "YES".into(),
                is_primary_key: false,
                column_default: None,
                ordinal_position: 2,
            },
            ColumnVo {
                column_name: "amount".into(),
                data_type: "numeric".into(),
                column_type: "numeric".into(),
                column_comment: Some("金额".into()),
                is_nullable: "YES".into(),
                is_primary_key: false,
                column_default: None,
                ordinal_position: 3,
            },
            ColumnVo {
                column_name: "create_time".into(),
                data_type: "timestamp with time zone".into(),
                column_type: "timestamptz".into(),
                column_comment: Some("创建时间".into()),
                is_nullable: "NO".into(),
                is_primary_key: false,
                column_default: None,
                ordinal_position: 4,
            },
            ColumnVo {
                column_name: "version".into(),
                data_type: "integer".into(),
                column_type: "int4".into(),
                column_comment: Some("版本".into()),
                is_nullable: "NO".into(),
                is_primary_key: false,
                column_default: Some("0".into()),
                ordinal_position: 5,
            },
        ]
    }

    /// 生成的代码不应残留 format! 占位符（{{ }} 转义正确）或未解析的模板变量
    #[test]
    fn generated_import_export_code_has_no_placeholder() {
        let cols = mock_columns();
        let svc = generate_service_code("Demo", "demo", "demo", &cols);
        assert!(svc.contains("EXPORT_COLS"), "应生成导出列清单");
        assert!(svc.contains("import_preview"), "应生成导入预览");
        assert!(svc.contains("import_commit"), "应生成导入提交");
        assert!(svc.contains("generate_template"), "应生成模板方法");
        assert!(svc.contains("export_count"), "应生成可导出行数方法");
        assert!(svc.contains("export_rows"), "应生成导出行方法（同步/异步共用）");
        assert!(svc.contains("export_xlsx"), "应生成同步导出方法");
        assert!(svc.contains("common::xlsx::too_large_message"), "超阈值应提示走导出中心");
        assert!(svc.contains("\"name\""), "业务列 name 应进入导出列");
        assert!(svc.contains("\"amount\""), "业务列 amount 应进入导出列");
        // 系统列不应出现在导出列
        assert!(!svc.contains("\"create_time\""), "审计列不应进入导出列");
        assert!(!svc.contains("\"version\""), "版本列不应进入导出列");
        // 生成代码应可编译：新增落库 / 编辑按需覆盖 / 分页参数齐全
        assert!(svc.contains("name: Set(dto.name),"), "新增应按 DTO 落库");
        assert!(svc.contains("if let Some(v) = dto.name"), "编辑应仅覆盖传入字段");
        assert!(
            svc.contains("total, query.page, query.page_size"),
            "PageResult::new 需带页码与每页条数"
        );

        let dto = generate_dto_code("Demo", "demo", &cols);
        assert!(dto.contains("name: m.name,"), "VO 应从实体逐列映射");
        assert!(dto.contains("create_time: m.create_time,"), "审计列同样需要映射");
        // 生成的 service 里不应残留 `{snake}` 等未替换占位符
        for token in ["{snake}", "{name}", "{module}", "{import_set}", "{export_cols}", "{export_text_entries}"] {
            assert!(!svc.contains(token), "生成代码残留模板占位符: {}", token);
        }

        let handler = generate_handler_code("Demo", "demo", "demo");
        for route in [
            "/demos/import",
            "/demos/import/template",
            "/demos/export/count",
            "/demos/export",
            "/demos/{id}",
        ] {
            assert!(handler.contains(route), "handler 应包含路由 {}", route);
        }
        assert!(handler.contains("Multipart"), "导入 handler 需要 Multipart");
        assert!(handler.contains("common::xlsx::XLSX_MIME"), "导出应返回 xlsx");
        for token in ["{snake}", "{name}", "{module}"] {
            assert!(!handler.contains(token), "handler 残留模板占位符: {}", token);
        }

        let module = generate_module_mod_code("Demo");
        assert!(module.contains("pub mod service;"), "模块入口应声明服务模块");
        assert!(!module.contains("export_executor"), "异步导出不再需要独立执行器文件");
        for token in ["{snake}", "{name}"] {
            assert!(!module.contains(token), "模块入口残留模板占位符: {}", token);
        }

        assert!(svc.contains("fast_excel::export_task!"), "service 应直接用宏注册异步导出");
        assert!(svc.contains("task_type = \"demo\""), "task_type 应为模块名");
        assert!(svc.contains("demo_export_rows"), "异步导出应复用服务层导出行");

        let api = generate_frontend_api_code("Demo", "demo");
        for fn_name in ["importData", "importTemplate", "exportCount", "export:"] {
            assert!(api.contains(fn_name), "前端 api 应包含 {}", fn_name);
        }
        assert!(api.contains("/system/demos/export/count"), "导出计数应指向功能页接口");
        for token in ["{snake}", "{name}", "{module}"] {
            assert!(!api.contains(token), "api 残留模板占位符: {}", token);
        }

        let page = generate_frontend_page_code("Demo", "demo", "demo", &cols);
        for fn_name in ["doExport", "commitImport", "downloadTemplate", "handleImportFile"] {
            assert!(page.contains(fn_name), "前端页面应包含 {}", fn_name);
        }
        assert!(page.contains("exportTaskApi.create('demo'"), "大数据导出应提交导出中心任务");
        assert!(
            page.contains("message.success('导出成功，请自行到导出中心查看')"),
            "大数据导出应提示用户到「导出中心」查看"
        );
        assert!(page.contains("const EXPORT_MAX = 100000"), "同步上限应为 10 万条");
        for token in ["{snake}", "{name}", "{module}", "{table_cols}", "{search_fields}"] {
            assert!(!page.contains(token), "页面残留模板占位符: {}", token);
        }
    }

}
