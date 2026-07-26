use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::{role, role_menu};
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, QueryOrder};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;

use super::dto::{CreateRoleDto, UpdateRoleDto, RoleVo};

#[derive(Clone, Service)]
pub struct RoleAppService {
    #[inject(component)]
    db: DbConn,
}

impl RoleAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<RoleVo>, AppError> {

        let paginator = role::Entity::find()
            .paginate(&self.db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<role::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<RoleVo> = items.into_iter().map(RoleVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn list_tree(&self) -> Result<Vec<RoleVo>, AppError> {

        let items: Vec<role::Model> = role::Entity::find()
            .order_by_asc(role::Column::RoleSort)
            .all(&self.db)
            .await?;

        let vos: Vec<RoleVo> = items.into_iter().map(RoleVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    pub async fn list_all(&self) -> Result<Vec<RoleVo>, AppError> {

        let items: Vec<role::Model> = role::Entity::find()
            .order_by_asc(role::Column::RoleSort)
            .all(&self.db)
            .await?;

        let vos: Vec<RoleVo> = items.into_iter().map(RoleVo::from).collect();
        Ok(vos)
    }

    fn build_tree(roles: Vec<RoleVo>) -> Vec<RoleVo> {
        let mut map: std::collections::HashMap<String, Vec<RoleVo>> = std::collections::HashMap::new();

        for role in &roles {
            map.entry(role.parent_id.clone()).or_default();
        }

        for role in roles {
            map.entry(role.parent_id.clone()).or_default().push(role);
        }

        let mut result = Vec::new();
        if let Some(roots) = map.remove("0") {
            for mut root in roots {
                root.children = Self::build_children(root.id.clone(), &mut map);
                result.push(root);
            }
        }

        result
    }

    fn build_children(
        parent_id: String,
        map: &mut std::collections::HashMap<String, Vec<RoleVo>>,
    ) -> Option<Vec<RoleVo>> {
        if let Some(children) = map.remove(&parent_id) {
            let mut result = Vec::new();
            for mut child in children {
                child.children = Self::build_children(child.id.clone(), map);
                result.push(child);
            }
            Some(result)
        } else {
            None
        }
    }

    pub async fn get_by_id(&self, id: String) -> Result<RoleVo, AppError> {

        let model: role::Model = role::Entity::find()
            .filter(role::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?;

        let menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.eq(&id))
            .all(&self.db)
            .await?;

        let menu_ids: Vec<String> = menus.into_iter().map(|m| m.menu_id).collect();

        let mut vo: RoleVo = model.into();
        vo.menu_ids = Some(menu_ids);
        Ok(vo)
    }

    pub async fn create(&self, dto: CreateRoleDto) -> Result<RoleVo, AppError> {

        let existing: Option<role::Model> = role::Entity::find()
            .filter(role::Column::RoleCode.eq(&dto.role_code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("角色编码已存在".to_string()));
        }

        let menu_ids = dto.menu_ids.clone();
        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;

        if let Some(ids) = menu_ids {
            let models: Vec<role_menu::ActiveModel> = ids.into_iter().map(|menu_id| role_menu::ActiveModel {
                role_id: Set(model.id.clone()),
                menu_id: Set(menu_id),
                ..Default::default()
            }).collect();
            role_menu::Entity::insert_many_with_fill(models, &self.db).await?;
        }

        let mut vo: RoleVo = model.into();
        vo.menu_ids = None;
        Ok(vo)
    }

    pub async fn update(&self, dto: UpdateRoleDto) -> Result<RoleVo, AppError> {

        let menu_ids = dto.menu_ids.clone();
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;

        if let Some(ids) = menu_ids {
            role_menu::Entity::delete_many()
                .filter(role_menu::Column::RoleId.eq(&model.id))
                .exec(&self.db)
                .await?;

            let models: Vec<role_menu::ActiveModel> = ids.into_iter().map(|menu_id| role_menu::ActiveModel {
                role_id: Set(model.id.clone()),
                menu_id: Set(menu_id),
                ..Default::default()
            }).collect();
            role_menu::Entity::insert_many_with_fill(models, &self.db).await?;
        }

        let mut vo: RoleVo = model.into();
        vo.menu_ids = None;
        Ok(vo)
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: role::Model = role::Entity::find()
            .filter(role::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?;

        let children: Vec<role::Model> = role::Entity::find()
            .filter(role::Column::ParentId.eq(&id))
            .all(&self.db)
            .await?;

        if !children.is_empty() {
            return Err(AppError::BadRequest("该角色下存在子角色，无法删除".to_string()));
        }

        let mut am: role::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;

        role_menu::Entity::delete_many()
            .filter(role_menu::Column::RoleId.eq(&id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn assign_menus(&self, role_id: String, menu_ids: Vec<String>) -> Result<(), AppError> {

        let _model: role::Model = role::Entity::find()
            .filter(role::Column::Id.eq(&role_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?;

        role_menu::Entity::delete_many()
            .filter(role_menu::Column::RoleId.eq(&role_id))
            .exec(&self.db)
            .await?;

        let models: Vec<role_menu::ActiveModel> = menu_ids.into_iter().map(|menu_id| role_menu::ActiveModel {
            role_id: Set(role_id.clone()),
            menu_id: Set(menu_id),
            ..Default::default()
        }).collect();
        role_menu::Entity::insert_many_with_fill(models, &self.db).await?;

        Ok(())
    }

    pub async fn get_role_menu_ids(&self, role_id: String) -> Result<Vec<String>, AppError> {

        let menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.eq(&role_id))
            .all(&self.db)
            .await?;

        Ok(menus.into_iter().map(|m| m.menu_id).collect())
    }
}