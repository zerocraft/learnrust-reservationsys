use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::Post;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let alert = sea_query::Table::alter()
            .table(Post::Table)
            .add_column_if_not_exists(ColumnDef::new(Post::Tags).string())
            .to_owned();
        manager.alter_table(alert).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                sea_query::Table::alter()
                    .table(Post::Table)
                    .drop_column(Post::Tags)
                    .to_owned(),
            )
            .await
    }
}
