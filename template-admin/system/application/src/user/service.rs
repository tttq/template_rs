use common::error::AppError;
use common::pagination::PageResult;
use common::user::get_current_tenant_mode;
use summer::plugin::service::Service;
use summer_redis::Redis;
use sea_orm_ext::DbConn;
use system_entity::{user, user_role, tenant_user, tenant};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait, TransactionTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use sea_orm_ext::{ignore_tenant, TenantIgnoreGuard};
use chrono::Utc;
use fast_excel::ExportTaskContext;
use summer::App;
use summer::plugin::ComponentRegistry;

use super::dto::{CreateUserDto, UpdateUserDto, UserVo, UserQuery, UserExportQuery};
use crate::auth::permission_sync;

/// 去重 id 列表（保留首次出现顺序），避免重复角色绑定触发唯一约束错误
fn dedup_ids(ids: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    ids.into_iter().filter(|id| seen.insert(id.clone())).collect()
}

#[derive(Clone, Service)]
pub struct UserAppService {
    #[inject(component)]
    db: DbConn,
    #[inject(component)]
    redis: Redis,
}

impl UserAppService {
    pub async fn list(&self, query: UserQuery) -> Result<PageResult<UserVo>, AppError> {
        let mut select = user::Entity::find();

        if let Some(v) = &query.user_name {
            if !v.is_empty() {
                select = select.filter(user::Column::UserName.contains(v));
            }
        }
        if let Some(v) = &query.email {
            if !v.is_empty() {
                select = select.filter(user::Column::Email.contains(v));
            }
        }
        if let Some(v) = &query.phone {
            if !v.is_empty() {
                select = select.filter(user::Column::Phone.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(user::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(user::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(user::Column::CreateBy.contains(v));
            }
        }

        let paginator = select.paginate(&self.db, query.page_query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<user::Model> = paginator.fetch_page(query.page_query.page - 1).await?;

        let vos: Vec<UserVo> = items.into_iter().map(UserVo::from).collect();
        Ok(PageResult::new(vos, total, query.page_query.page, query.page_query.page_size))
    }

    /// 导出用户列表（支持按ID列表导出）
    pub async fn export(&self, query: UserExportQuery) -> Result<Vec<UserVo>, AppError> {
        let items: Vec<user::Model> = Self::export_select(&query).all(&self.db).await?;
        Ok(items.into_iter().map(UserVo::from).collect())
    }

    /// 可导出行数（同步/异步分流判定；与 `export` 同口径，只 COUNT 不拉数据）
    pub async fn export_count(&self, query: &UserExportQuery) -> Result<u64, AppError> {
        Self::export_select(query).count(&self.db).await.map_err(AppError::from)
    }

    /// 导出筛选条件（列表导出与行数统计共用同一口径）
    fn export_select(query: &UserExportQuery) -> sea_orm::Select<user::Entity> {
        let mut select = user::Entity::find();

        if let Some(v) = &query.user_name {
            if !v.is_empty() {
                select = select.filter(user::Column::UserName.contains(v));
            }
        }
        if let Some(v) = &query.email {
            if !v.is_empty() {
                select = select.filter(user::Column::Email.contains(v));
            }
        }
        if let Some(v) = &query.phone {
            if !v.is_empty() {
                select = select.filter(user::Column::Phone.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(user::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(user::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(user::Column::CreateBy.contains(v));
            }
        }
        // 如果指定了IDs，按ID过滤
        if let Some(ids_str) = &query.ids {
            if !ids_str.is_empty() {
                let ids: Vec<String> = ids_str.split(',').map(|s| s.trim().to_string()).collect();
                select = select.filter(user::Column::Id.is_in(ids));
            }
        }

        select
    }

    pub async fn get_by_id(&self, id: String) -> Result<UserVo, AppError> {

        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        let roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(&id))
            .all(&self.db)
            .await?;

        let role_ids: Vec<String> = roles.into_iter().map(|r| r.role_id).collect();

        let mut vo: UserVo = model.into();
        vo.role_ids = Some(role_ids);
        Ok(vo)
    }

    pub async fn create(&self, dto: CreateUserDto) -> Result<UserVo, AppError> {

        let existing: Option<user::Model> = user::Entity::find()
            .filter(user::Column::UserName.eq(&dto.user_name))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("@username_taken".to_string()));
        }

        let role_ids = dto.role_ids.clone();
        let active_model = dto.into_active_model()?;
        // 用户与其角色绑定同一事务：避免用户已创建、绑定失败留下"无角色用户"
        let tx = self.db.inner().begin().await?;
        let model = active_model.insert(&tx).await?;

        if let Some(ids) = role_ids {
            let models: Vec<user_role::ActiveModel> = dedup_ids(ids).into_iter().map(|role_id| user_role::ActiveModel {
                user_id: Set(model.id.clone()),
                role_id: Set(role_id),
                ..Default::default()
            }).collect();
            user_role::Entity::insert_many_with_fill(models, &tx).await?;
        }
        tx.commit().await?;

        self.sync_tenant_user_index(&model).await?;

        let mut vo: UserVo = model.into();
        vo.role_ids = None;
        Ok(vo)
    }

    pub async fn update(&self, dto: UpdateUserDto) -> Result<UserVo, AppError> {

        let role_ids = dto.role_ids.clone();
        let roles_changed = role_ids.is_some();
        let active_model = dto.into_active_model();
        // 用户字段 + 角色绑定同一事务：角色绑定是"先清空再插入"，
        // 插入失败时若清空已提交，该用户会失去全部角色（权限被清空）
        let tx = self.db.inner().begin().await?;
        let model = active_model.update(&tx).await?;

        if let Some(ids) = role_ids {
            user_role::Entity::delete_many()
                .filter(user_role::Column::UserId.eq(&model.id))
                .exec(&tx)
                .await?;

            let models: Vec<user_role::ActiveModel> = dedup_ids(ids).into_iter().map(|role_id| user_role::ActiveModel {
                user_id: Set(model.id.clone()),
                role_id: Set(role_id),
                ..Default::default()
            }).collect();
            user_role::Entity::insert_many_with_fill(models, &tx).await?;
        }
        tx.commit().await?;

        if roles_changed {
            // 提交后再刷新权限快照（含 redis 通知，不能纳入事务）
            permission_sync::sync_and_notify(&self.db, &self.redis, std::slice::from_ref(&model.id)).await;
        }

        self.sync_tenant_user_index(&model).await?;

        let mut vo: UserVo = model.into();
        vo.role_ids = None;
        Ok(vo)
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        // 用户软删与其角色绑定清理同一事务，避免半删状态留下孤儿绑定
        let tx = self.db.inner().begin().await?;
        let mut am: user::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&tx).await?;

        user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(&id))
            .exec(&tx)
            .await?;
        tx.commit().await?;

        self.remove_tenant_user_index(&id).await?;

        Ok(())
    }

    pub async fn update_status(&self, id: String, status: i32) -> Result<(), AppError> {

        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        // 先读取租户用户索引，再把「主库用户状态 + 索引状态」放进同一事务，
        // 避免开关用户后索引状态与用户状态不一致（database 租户模式下会串号）
        let index_row = self.find_tenant_user_index(&id).await?;
        let tx = self.db.inner().begin().await?;
        let mut active: user::ActiveModel = model.into();
        active.status = Set(status);
        active.update(&tx).await?;

        if let Some(idx) = index_row {
            let mut idx_am: tenant_user::ActiveModel = idx.into();
            idx_am.status = Set(status);
            let _guard = TenantIgnoreGuard::new();
            idx_am.update(&tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    pub async fn assign_roles(&self, user_id: String, role_ids: Vec<String>) -> Result<(), AppError> {

        let _model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        // 先清空再插入必须同一事务，否则插入失败会让用户失去全部角色
        let tx = self.db.inner().begin().await?;
        user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(&user_id))
            .exec(&tx)
            .await?;

        let models: Vec<user_role::ActiveModel> = dedup_ids(role_ids).into_iter().map(|role_id| user_role::ActiveModel {
            user_id: Set(user_id.clone()),
            role_id: Set(role_id),
            ..Default::default()
        }).collect();
        user_role::Entity::insert_many_with_fill(models, &tx).await?;
        tx.commit().await?;

        permission_sync::sync_and_notify(&self.db, &self.redis, &[user_id]).await;

        Ok(())
    }

    pub async fn get_role_ids(&self, user_id: String) -> Result<Vec<String>, AppError> {

        let roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(&user_id))
            .all(&self.db)
            .await?;

        Ok(roles.into_iter().map(|r| r.role_id).collect())
    }

    #[ignore_tenant]
    async fn sync_tenant_user_index(&self, user_model: &user::Model) -> Result<(), AppError> {
        let tenant_mode = get_current_tenant_mode();
        if tenant_mode.as_deref() != Some("database") {
            return Ok(());
        }

        let tenant_id = match common::user::get_current_tenant_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&tenant_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Internal("@tenant_not_found".to_string()))?;

        let existing = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserId.eq(&user_model.id))
            .filter(tenant_user::Column::TenantId.eq(&tenant_id))
            .one(&self.db)
            .await?;

        if let Some(idx) = existing {
            let mut am: tenant_user::ActiveModel = idx.into();
            am.user_name = Set(user_model.user_name.clone());
            am.identity_type = Set(user_model.identity_type.clone().unwrap_or_else(|| "username".to_string()));
            am.identity_value = Set(user_model.email.clone().or(user_model.phone.clone()));
            am.status = Set(user_model.status);
            am.update(&self.db).await?;
        } else {
            tenant_user::ActiveModel {
                user_name: Set(user_model.user_name.clone()),
                tenant_id: Set(tenant_id.clone()),
                tenant_code: Set(tenant_model.tenant_code.clone()),
                user_id: Set(user_model.id.clone()),
                identity_type: Set(user_model.identity_type.clone().unwrap_or_else(|| "username".to_string())),
                identity_value: Set(user_model.email.clone().or(user_model.phone.clone())),
                status: Set(user_model.status),
                ..Default::default()
            }.insert(&self.db).await?;
        }

        Ok(())
    }

    #[ignore_tenant]
    async fn remove_tenant_user_index(&self, user_id: &str) -> Result<(), AppError> {
        let tenant_mode = get_current_tenant_mode();
        if tenant_mode.as_deref() != Some("database") {
            return Ok(());
        }

        let tenant_id = match common::user::get_current_tenant_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        if let Some(idx) = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserId.eq(user_id))
            .filter(tenant_user::Column::TenantId.eq(&tenant_id))
            .one(&self.db)
            .await?
        {
            let mut am: tenant_user::ActiveModel = idx.into();
            am.status = Set(0);
            am.update(&self.db).await?;
        }

        Ok(())
    }

    #[ignore_tenant]
    async fn find_tenant_user_index(&self, user_id: &str) -> Result<Option<tenant_user::Model>, AppError> {
        Ok(tenant_user::Entity::find()
            .filter(tenant_user::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?)
    }
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 用户导出表头
pub const USER_HEADERS: &[&str] = &[
    "用户名", "昵称", "邮箱", "手机号", "状态", "创建时间", "创建人", "创建人ID", "修改时间",
    "修改人", "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 用户列表 → 导出行
pub fn user_rows(vos: &[UserVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            vec![
                v.user_name.clone(),
                v.nick_name.clone().unwrap_or_default(),
                v.email.clone().unwrap_or_default(),
                v.phone.clone().unwrap_or_default(),
                if v.status == 1 { "启用" } else { "禁用" }.to_string(),
                ts(v.create_time),
                v.create_by.clone().unwrap_or_default(),
                v.create_id.clone().unwrap_or_default(),
                ts(v.update_time),
                v.update_by.clone().unwrap_or_default(),
                v.update_id.clone().unwrap_or_default(),
            ]
        })
        .collect()
}

/// 异步导出（> 10 万条）：导出中心按 `task_type = "user"` 直接调用。
async fn user_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<UserAppService>()
        .map_err(|e| AppError::Internal(format!("用户服务组件未就绪：{e}")))?;
    let query: UserExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(user_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "user",
    sheet_name = "用户数据",
    headers = USER_HEADERS,
    rows = user_export_rows,
}
