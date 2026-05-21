use common::error::AppError;
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
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
                content: generate_service_code(&entity_name_pascal, &entity_name_snake, module_path),
                language: "rust".to_string(),
            });
            files.push(CodeFileVo {
                file_path: format!("system/interface/src/handlers/{}_handler.rs", entity_name_snake),
                file_name: format!("{}_handler.rs", entity_name_snake),
                content: generate_handler_code(&entity_name_pascal, &entity_name_snake, module_path),
                language: "rust".to_string(),
            });
        }

        if config.generate_frontend {
            files.push(CodeFileVo {
                file_path: format!("src/api/{}.ts", entity_name_snake),
                file_name: format!("{}.ts", entity_name_snake),
                content: generate_frontend_api_code(&entity_name_snake, module_path),
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

fn generate_entity_code(_name: &str, _snake: &str, table: &str, columns: &[ColumnVo]) -> String {
    let mut fields = String::new();
    for col in columns {
        let rt = rust_type(&col.column_type);
        let opt = is_optional(&col.column_type, &col.is_nullable, col.is_primary_key);
        let field_name = col.column_name.replace("_", "_");
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
use summer_sea_orm::DbConn;
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
        {snake}::Entity::delete_by_id(id)
            .exec(&self.db)
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

    for col in columns {
        let rt = rust_type(&col.column_type);
        let opt = is_optional(&col.column_type, &col.is_nullable, col.is_primary_key);
        let field_name = col.column_name.clone();

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
            id: m.id,
        }}
    }}
}}
"#)
}

fn generate_service_code(name: &str, snake: &str, _module: &str) -> String {
    format!(r#"use common::error::AppError;
use common::pagination::{{PageQuery, PageResult}};
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use system_entity::{snake};
use sea_orm::{{EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, PaginatorTrait}};
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
        Ok(PageResult::new(items.into_iter().map({name}Vo::from).collect(), total))
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
            .ok_or_else(|| AppError::NotFound("数据不存在".to_string()))?;
        Ok({name}Vo::from(model))
    }}

    pub async fn create(&self, dto: Create{name}Dto) -> Result<{name}Vo, AppError> {{
        let model = {snake}::ActiveModel {{
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
            .ok_or_else(|| AppError::NotFound("数据不存在".to_string()))?;
        let mut model: {snake}::ActiveModel = existing.into();
        model.id = Set(id);
        model.version = Set(dto.version.unwrap_or(0));
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
}}
"#)
}

fn generate_handler_code(name: &str, snake: &str, module: &str) -> String {
    format!(r#"use summer_web::{{get, post, put, delete, Router}};
use summer_web::extractor::{{Component, Json, Path, Query}};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::{module}::service::{name}AppService;
use system_application::{module}::dto::*;

pub fn routes() -> Router {{
    Router::new()
        .typed_route(list_{snake})
        .typed_route(get_{snake})
        .typed_route(create_{snake})
        .typed_route(update_{snake})
        .typed_route(delete_{snake})
}}

#[get("/")]
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

#[get("/:id")]
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

#[post("/")]
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

#[put("/:id")]
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

#[delete("/:id")]
#[sa_check_permission("{snake}:delete")]
async fn delete_{snake}(
    Component(service): Component<{name}AppService>,
    Path(id): Path<String>,
) -> impl IntoResponse {{
    match service.delete(id).await {{
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }}
}}
"#)
}

fn generate_frontend_api_code(snake: &str, module: &str) -> String {
    format!(r#"import request from './request'

export interface {module}Vo {{
  id: string
}}

export const {snake}Api = {{
  list: (params: any) => request.get('/{module}/{snake}s', {{ params }}),
  getById: (id: string) => request.get(`/{module}/{snake}s/${{id}}`),
  create: (data: any) => request.post(`/{module}/{snake}s`, data),
  update: (id: string, data: any) => request.put(`/{module}/{snake}s/${{id}}`, data),
  delete: (id: string) => request.delete(`/{module}/{snake}s/${{id}}`),
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
        <a-button type="primary" @click="showCreateModal">
          <template #icon><PlusOutlined /></template>
          新增
        </a-button>
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
  </div>
</template>

<script setup lang="ts">
import {{ ref, reactive, onMounted }} from 'vue'
import {{ PlusOutlined }} from '@ant-design/icons-vue'
import {{ message }} from 'ant-design-vue'
import {{ {snake}Api }} from '@/api/{snake}'

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
    tableData.value = res.records
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

onMounted(() => fetchData())
</script>
"#,
        search_fields = search_fields.iter().map(|f| format!("        <a-col :span=\"4\">\n          <a-input v-model:value=\"searchForm.{}\" placeholder=\"{}\" allow-clear @pressEnter=\"fetchData\" />\n        </a-col>", f, f)).collect::<Vec<_>>().join("\n"),
        table_cols = table_columns.join(",\n")
    )
}
