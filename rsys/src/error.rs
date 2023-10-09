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
    #[error("config error: {0}")]
    ConfigError(String),
    #[error("server error: {0}")]
    ServerError(String),
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

// impl From<sea_orm::DbErr> for RsysError {
//     fn from(value: sea_orm::DbErr) -> Self {
//         RsysError::DbError(value)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::error::RsysError;

    #[test]
    fn debug_errors() {
        let err = RsysError::Unknown;
        println!("{:?}\n{}", err, err);
        // to_string() 只读时使用，不会分配新的内存
        // to_owned() 后续需要修改时使用，分配新的内存来存储新字符串
        let err = RsysError::ConfigError("config".to_string());
        println!("{:?}\n{}", err, err);
    }
}
