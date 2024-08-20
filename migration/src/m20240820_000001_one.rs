use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Caller::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Caller::Seq).small_integer().primary_key())
                    .col(ColumnDef::new(Caller::BlockId).big_unsigned())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Caller::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Caller {
    Table,
    Seq,
    BlockId,
}
