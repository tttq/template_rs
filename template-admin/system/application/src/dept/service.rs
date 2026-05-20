use common::error::AppError;
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use system_entity::dept;
use sea_orm::{QueryFilter, ColumnTrait, QueryOrder, ActiveValue::Set};
use sea_orm::prelude::*;

use super::dto::{CreateDeptDto, UpdateDeptDto, DeptVo};

#[derive(Clone, Service)]
pub struct DeptAppService {
    #[inject(component)]
    db: DbConn,
}

impl DeptAppService {
    pub async fn list(&self) -> Result<Vec<DeptVo>, AppError> {
        let items: Vec<dept::Model> = dept::Entity::find()
            .order_by_asc(dept::Column::DeptSort)
            .all(&self.db)
            .await?;

        let vos: Vec<DeptVo> = items.into_iter().map(DeptVo::from).collect();
        Ok(vos)
    }

    pub async fn list_tree(&self) -> Result<Vec<DeptVo>, AppError> {
        let items: Vec<dept::Model> = dept::Entity::find()
            .order_by_asc(dept::Column::DeptSort)
            .all(&self.db)
            .await?;

        let vos: Vec<DeptVo> = items.into_iter().map(DeptVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    fn build_tree(depts: Vec<DeptVo>) -> Vec<DeptVo> {
        let mut result = Vec::new();
        let mut map: std::collections::HashMap<i64, Vec<DeptVo>> = std::collections::HashMap::new();

        for dept in depts {
            map.entry(dept.parent_id).or_default().push(dept);
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
        map: &mut std::collections::HashMap<i64, Vec<DeptVo>>,
    ) -> Option<Vec<DeptVo>> {
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

    pub async fn get_by_id(&self, id: i64) -> Result<DeptVo, AppError> {
        let model: dept::Model = dept::Entity::find()
            .filter(dept::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("部门不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateDeptDto) -> Result<DeptVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateDeptDto) -> Result<DeptVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model: dept::Model = dept::Entity::find()
            .filter(dept::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("部门不存在".to_string()))?;

        let children: Vec<dept::Model> = dept::Entity::find()
            .filter(dept::Column::ParentId.eq(id))
            .all(&self.db)
            .await?;

        for child in children {
            let mut am: dept::ActiveModel = child.into();
            am.delete_flag = Set(1);
            am.update(&self.db).await?;
        }

        let mut am: dept::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}
