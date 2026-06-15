use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "flex_schema_versions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub schema_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub version: i32,
    pub fields_config: Json,
    pub settings: Json,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::flex_schemas::Entity",
        from = "Column::SchemaId",
        to = "super::flex_schemas::Column::Id"
    )]
    FlexSchemas,
}

impl Related<super::flex_schemas::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FlexSchemas.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
