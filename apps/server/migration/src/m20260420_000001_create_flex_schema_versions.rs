use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("flex_schemas", "version").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("flex_schemas"))
                        .add_column(
                            ColumnDef::new(Alias::new("version"))
                                .integer()
                                .not_null()
                                .default(1),
                        )
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("flex_schema_versions"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("schema_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("version")).integer().not_null())
                    .col(
                        ColumnDef::new(Alias::new("fields_config"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("settings"))
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(Alias::new("schema_id"))
                            .col(Alias::new("version")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("flex_schema_versions"), Alias::new("schema_id"))
                            .to(Alias::new("flex_schemas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("flex_schema_versions")).to_owned())
            .await?;

        if manager.has_column("flex_schemas", "version").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("flex_schemas"))
                        .drop_column(Alias::new("version"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
