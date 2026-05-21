use common::error::AppError;
use common::tenant_db::get_effective_db;
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
        let db = get_effective_db(&self.db).await?;

        let items: Vec<dept::Model> = dept::Entity::find()
            .order_by_asc(dept::Column::DeptSort)
            .all(&db)
            .await?;

        let vos: Vec<DeptVo> = items.into_iter().map(DeptVo::from).collect();
        Ok(vos)
    }

    pub async fn list_tree(&self) -> Result<Vec<DeptVo>, AppError> {
        let db = get_effective_db(&self.db).await?;

        let items: Vec<dept::Model> = dept::Entity::find()
            .order_by_asc(dept::Column::DeptSort)
            .all(&db)
            .await?;

        let vos: Vec<DeptVo> = items.into_iter().map(DeptVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    fn build_tree(depts: Vec<DeptVo>) -> Vec<DeptVo> {
        let mut result = Vec::new();
        let mut map: std::collections::HashMap<String, Vec<DeptVo>> = std::collections::HashMap::new();

        for dept in depts {
            map.entry(dept.parent_id.clone()).or_default().push(dept);
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
        map: &mut std::collections::HashMap<String, Vec<DeptVo>>,
    ) -> Option<Vec<DeptVo>> {
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

    pub async fn get_by_id(&self, id: String) -> Result<DeptVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: dept::Model = dept::Entity::find()
            .filter(dept::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("部门不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateDeptDto) -> Result<DeptVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let active_model = dto.into_active_model();
        let model = active_model.insert(&db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateDeptDto) -> Result<DeptVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let active_model = dto.into_active_model();
        let model = active_model.update(&db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: dept::Model = dept::Entity::find()
            .filter(dept::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("部门不存在".to_string()))?;

        let children: Vec<dept::Model> = dept::Entity::find()
            .filter(dept::Column::ParentId.eq(&id))
            .all(&db)
            .await?;

        for child in children {
            let mut am: dept::ActiveModel = child.into();
            am.delete_flag = Set(1);
            am.update(&db).await?;
        }

        let mut am: dept::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&db).await?;
        Ok(())
    }
}