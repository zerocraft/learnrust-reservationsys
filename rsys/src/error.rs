use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RsysError {
    #[error("unknown error")]
    Unknown,
    #[error("db error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("sqlx error")]
    DbxError(sqlx::Error),
    #[error("{0} error")]
    ReservationError(String),
    #[error("already booked")]
    AlreadyBooked,
    #[error("no reservation")]
    NoReservation,
}

impl PartialEq for RsysError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DbxError(_), Self::DbxError(_)) => false,
            (v1, v2) => v1 == v2,
        }
    }
}

impl From<sqlx::Error> for RsysError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(err) => {
                let pgerr: &PgDatabaseError = err.downcast_ref();
                match (pgerr.schema(), pgerr.table()) {
                    (Some("rsvp"), Some("reservations")) => {
                        RsysError::ReservationError("reservationsX".to_owned())
                    }
                    _ => RsysError::DbxError(sqlx::Error::Database(err)),
                }
            }
            _ => RsysError::DbxError(err),
        }
    }
}
