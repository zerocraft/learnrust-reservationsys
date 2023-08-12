use thiserror::Error;

#[derive(Error, Debug)]
pub enum RsysError {
    #[error("unknown error")]
    Unknown,
    #[error("db error")]
    DbError(#[from] sea_orm::DbErr),
}
