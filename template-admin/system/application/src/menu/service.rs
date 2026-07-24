use common::error::AppError;
use common::tenant_db::get_effective_db;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::menu;
use sea_orm::{QueryFilter, ColumnTrait, QueryOrder, ActiveValue::Set};
use sea_orm::prelude::*;

use super::dto::{CreateMenuDto, UpdateMenuDto, MenuVo};

#[derive(Clone, Service)]
pub struct MenuAppService {
    #[inject(component)]
    db: DbConn,
}

impl MenuAppService {
    pub async fn list(&self) -> Result<Vec<MenuVo>, AppError> {
        let db = get_effective_db(&self.db).await?;

        let items: Vec<menu::Model> = menu::Entity::find()
            .order_by_asc(menu::Column::SortOrder)
            .all(&db)
            .await?;

        let vos: Vec<MenuVo> = items.into_iter().map(MenuVo::from).collect();
        Ok(vos)
    }

    pub async fn list_tree(&self) -> Result<Vec<MenuVo>, AppError> {
        let db = get_effective_db(&self.db).await?;

        let items: Vec<menu::Model> = menu::Entity::find()
            .order_by_asc(menu::Column::SortOrder)
            .all(&db)
            .await?;

        let vos: Vec<MenuVo> = items.into_iter().map(MenuVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    fn build_tree(menus: Vec<MenuVo>) -> Vec<MenuVo> {
        let mut result = Vec::new();
        let mut map: std::collections::HashMap<String, Vec<MenuVo>> = std::collections::HashMap::new();

        for menu in menus {
            map.entry(menu.parent_id.clone()).or_default().push(menu);
        }

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
        map: &mut std::collections::HashMap<String, Vec<MenuVo>>,
    ) -> Option<Vec<MenuVo>> {
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

    pub async fn get_by_id(&self, id: String) -> Result<MenuVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: menu::Model = menu::Entity::find()
            .filter(menu::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("菜单不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateMenuDto) -> Result<MenuVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let active_model = dto.into_active_model();
        let model = active_model.insert(&db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateMenuDto) -> Result<MenuVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let active_model = dto.into_active_model();
        let model = active_model.update(&db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: menu::Model = menu::Entity::find()
            .filter(menu::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("菜单不存在".to_string()))?;

        let children: Vec<menu::Model> = menu::Entity::find()
            .filter(menu::Column::ParentId.eq(&id))
            .all(&db)
            .await?;

        // 批量软删除子菜单（单条 SQL）
        if !children.is_empty() {
            let child_models: Vec<menu::ActiveModel> = children.into_iter().map(Into::into).collect();
            menu::Entity::delete_many_soft(child_models, &db).await?;
        }

        // 软删除父菜单：保留 update 审计字段填充（who/when deleted）
        let mut am: menu::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&db).await?;
        Ok(())
    }
}