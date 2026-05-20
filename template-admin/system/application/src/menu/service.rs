use common::error::AppError;
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
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
        let items: Vec<menu::Model> = menu::Entity::find()
            .order_by_asc(menu::Column::SortOrder)
            .all(&self.db)
            .await?;

        let vos: Vec<MenuVo> = items.into_iter().map(MenuVo::from).collect();
        Ok(vos)
    }

    pub async fn list_tree(&self) -> Result<Vec<MenuVo>, AppError> {
        let items: Vec<menu::Model> = menu::Entity::find()
            .order_by_asc(menu::Column::SortOrder)
            .all(&self.db)
            .await?;

        let vos: Vec<MenuVo> = items.into_iter().map(MenuVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    fn build_tree(menus: Vec<MenuVo>) -> Vec<MenuVo> {
        let mut result = Vec::new();
        let mut map: std::collections::HashMap<i64, Vec<MenuVo>> = std::collections::HashMap::new();

        for menu in menus {
            map.entry(menu.parent_id).or_default().push(menu);
        }

        if let Some(roots) = map.remove(&0) {
            for mut root in roots {
                root.children = Self::build_children(root.id, &mut map);
                result.push(root);
            }
        }

        result
    }

    fn build_children(
        parent_id: i64,
        map: &mut std::collections::HashMap<i64, Vec<MenuVo>>,
    ) -> Option<Vec<MenuVo>> {
        if let Some(children) = map.remove(&parent_id) {
            let mut result = Vec::new();
            for mut child in children {
                child.children = Self::build_children(child.id, map);
                result.push(child);
            }
            Some(result)
        } else {
            None
        }
    }

    pub async fn get_by_id(&self, id: i64) -> Result<MenuVo, AppError> {
        let model: menu::Model = menu::Entity::find()
            .filter(menu::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("菜单不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateMenuDto) -> Result<MenuVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateMenuDto) -> Result<MenuVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model: menu::Model = menu::Entity::find()
            .filter(menu::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("菜单不存在".to_string()))?;

        let children: Vec<menu::Model> = menu::Entity::find()
            .filter(menu::Column::ParentId.eq(id))
            .all(&self.db)
            .await?;

        for child in children {
            let mut am: menu::ActiveModel = child.into();
            am.delete_flag = Set(1);
            am.update(&self.db).await?;
        }

        let mut am: menu::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}
