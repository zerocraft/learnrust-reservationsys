use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Post::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Post::Title).string().not_null())
                    .col(ColumnDef::new(Post::Text).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Reservations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Reservations::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(ColumnDef::new(Reservations::UserId).string())
                    .col(ColumnDef::new(Reservations::ResourceId).string())
                    .col(ColumnDef::new(Reservations::RStatus).integer())
                    .col(ColumnDef::new(Reservations::StartTime).timestamp_with_time_zone())
                    .col(ColumnDef::new(Reservations::EndTime).timestamp_with_time_zone())
                    .col(ColumnDef::new(Reservations::Note).text())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ReservationChanges::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReservationChanges::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ReservationChanges::ReservationId).uuid())
                    .col(ColumnDef::new(ReservationChanges::Op).integer())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Reservations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ReservationChanges::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    Text,
}

#[derive(DeriveIden)]
enum Reservations {
    Table,
    Id,
    UserId,
    RStatus,
    ResourceId,
    StartTime,
    EndTime,
    Note,
}

#[derive(DeriveIden)]
enum ReservationChanges {
    Table,
    Id,
    ReservationId,
    Op,
}
