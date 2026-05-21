use sea_orm_ext::{FieldFillHandler, FieldFillOperation};
use sea_query::Value;

pub struct AuditFieldFillHandler {
    pub default_user: String,
}

impl AuditFieldFillHandler {
    pub fn new(default_user: String) -> Self {
        Self { default_user }
    }
}

impl FieldFillHandler for AuditFieldFillHandler {
    fn fill(
        &self,
        _entity_name: &str,
        field_name: &str,
        operation: FieldFillOperation,
    ) -> Option<Value> {
        let user_id = crate::user::get_current_user_id()
            .unwrap_or_else(|| self.default_user.clone());
        let user_name = crate::user::get_current_user_name()
            .unwrap_or_else(|| self.default_user.clone());

        match (field_name, operation) {
            ("create_time", FieldFillOperation::Insert)
            | ("update_time", FieldFillOperation::Update)
            | ("update_time", FieldFillOperation::Insert) => {
                Some(chrono::Utc::now().into())
            }
            ("create_by", FieldFillOperation::Insert)
            | ("update_by", FieldFillOperation::Update)
            | ("update_by", FieldFillOperation::Insert) => {
                Some(user_name.into())
            }
            ("create_id", FieldFillOperation::Insert)
            | ("update_id", FieldFillOperation::Update)
            | ("update_id", FieldFillOperation::Insert) => {
                Some(user_id.into())
            }
            ("version", FieldFillOperation::Insert) => Some(1i32.into()),
            _ => None,
        }
    }
}
