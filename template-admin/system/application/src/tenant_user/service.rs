use common::error::AppError;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use sea_orm::{ActiveValue::Set, QueryFilter, ColumnTrait};
use sea_orm::prelude::*;
use system_entity::{tenant_user, user};
// user entity has DeriveSoftDelete, so model.delete() triggers the macro
// which returns Err("Record was soft-deleted"). Use manual soft delete instead.

#[derive(Clone, Service)]
pub struct TenantUserService {
    #[inject(component)]
    master_db: DbConn,
}

impl TenantUserService {
    pub async fn create_user(
        &self,
        tenant_db: &DatabaseConnection,
        tenant_id: &str,
        tenant_code: &str,
        user_name: &str,
        email: Option<&String>,
        phone: Option<&String>,
    ) -> Result<user::Model, AppError> {
        let user_model = user::ActiveModel {
            user_name: Set(user_name.to_string()),
            pass_word: Set(common::hash_password("123456")?),
            nick_name: Set(None),
            email: Set(email.cloned()),
            phone: Set(phone.cloned()),
            identity_type: Set(Some("username".to_string())),
            identity_value: Set(email.or(phone).cloned()),
            status: Set(1),
            admin_flag: Set(0),
            dept_id: Set(None),
            ..Default::default()
        };
        let result = user_model.insert(tenant_db).await?;

        self.sync_to_master_index(
            tenant_id,
            tenant_code,
            &result.id,
            &result.user_name,
            "username",
            result.email.as_deref(),
        ).await?;

        Ok(result)
    }

    pub async fn delete_user(
        &self,
        tenant_db: &DatabaseConnection,
        user_name: &str,
    ) -> Result<(), AppError> {
        let user = user::Entity::find()
            .filter(user::Column::UserName.eq(user_name))
            .one(tenant_db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let mut am: user::ActiveModel = user.into();
        am.delete_flag = Set(1);
        am.update(tenant_db).await?;

        tenant_user::Entity::delete_many()
            .filter(tenant_user::Column::UserName.eq(user_name))
            .exec(&self.master_db)
            .await?;

        Ok(())
    }

    pub async fn sync_to_master_index(
        &self,
        tenant_id: &str,
        tenant_code: &str,
        user_id: &str,
        user_name: &str,
        identity_type: &str,
        identity_value: Option<&str>,
    ) -> Result<(), AppError> {
        let existing = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserName.eq(user_name))
            .one(&self.master_db)
            .await?;

        if let Some(existing) = existing {
            let mut am: tenant_user::ActiveModel = existing.into();
            am.tenant_id = Set(tenant_id.to_string());
            am.tenant_code = Set(tenant_code.to_string());
            am.user_id = Set(user_id.to_string());
            am.identity_type = Set(identity_type.to_string());
            am.identity_value = Set(identity_value.map(|s| s.to_string()));
            am.update(&self.master_db).await?;
        } else {
            let am = tenant_user::ActiveModel {
                user_name: Set(user_name.to_string()),
                tenant_id: Set(tenant_id.to_string()),
                tenant_code: Set(tenant_code.to_string()),
                user_id: Set(user_id.to_string()),
                identity_type: Set(identity_type.to_string()),
                identity_value: Set(identity_value.map(|s| s.to_string())),
                status: Set(1),
                ..Default::default()
            };
            am.insert(&self.master_db).await?;
        }

        Ok(())
    }

    pub async fn get_user_index_by_name(&self, user_name: &str) -> Result<Option<tenant_user::Model>, AppError> {
        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserName.eq(user_name))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.master_db)
            .await?;
        Ok(index)
    }

    pub async fn get_user_index_by_identity(&self, identity_type: &str, identity_value: &str) -> Result<Option<tenant_user::Model>, AppError> {
        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::IdentityType.eq(identity_type))
            .filter(tenant_user::Column::IdentityValue.eq(identity_value))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.master_db)
            .await?;
        Ok(index)
    }
}
