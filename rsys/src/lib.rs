pub mod entities;

pub mod error;
mod manager;

use async_trait::async_trait;
use error::RsysError;
use rsys_abi::{
    CancelRequest, ConfirmRequest, DateTimeOffset, GetRequest, ListenRequest, QueryRequest,
    Reservation, UpdateRequest,
};
use sea_orm::DatabaseConnection;
use sqlx::{postgres::PgRow, FromRow, Row};

#[async_trait]
pub trait Rsvp {
    async fn create(&self, rsvp: Reservation) -> Result<Reservation, RsysError>;

    async fn change_status(&self, change: ConfirmRequest) -> Result<Reservation, RsysError>;

    async fn update_note(&self, update: UpdateRequest) -> Result<Reservation, RsysError>;

    async fn delete(&self, cancel: CancelRequest) -> Result<usize, RsysError>;

    async fn get(&self, cancel: GetRequest) -> Result<Reservation, RsysError>;

    async fn query(&self, query: QueryRequest) -> Result<Vec<Reservation>, RsysError>;

    async fn listen(&self, listen: ListenRequest) -> Result<Vec<Reservation>, RsysError>;
}

#[derive(Debug)]
pub struct ReservationManager {
    pub constr: String,
    db: DatabaseConnection,
}

impl From<entities::reservations::Model> for Reservation {
    fn from(val: entities::reservations::Model) -> Self {
        Reservation {
            id: val.id.to_string(),
            uid: val.user_id.unwrap_or_default(),
            resource_id: val.resource_id.unwrap_or_default(),
            note: val.note.unwrap_or_default(),
            start: Some(DateTimeOffset(val.start_time.unwrap_or_default()).into()),
            end: Some(DateTimeOffset(val.end_time.unwrap_or_default()).into()),
            rstatus: val.r_status.unwrap_or_default(),
        }
    }
}

impl FromRow<'_, PgRow> for entities::reservations::Model {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get(0),
            user_id: row.get(1),
            resource_id: row.get(2),
            r_status: row.get(3),
            start_time: row.get(4),
            end_time: row.get(5),
            note: row.get(6),
        })
    }
}

pub fn env_con_str() -> String {
    dotenvy::var("DATABASE_URL").unwrap()
}

#[cfg(test)]
mod tests {
    use crate::entities::prelude::*;
    use crate::entities::reservations;
    use crate::env_con_str;
    use sea_orm::ActiveModelTrait;
    use sea_orm::Database;
    use sea_orm::EntityTrait;
    use sea_orm::ModelTrait;
    use sea_orm::Set;

    #[tokio::test]
    async fn orm_test_query_all() {
        let db = Database::connect(env_con_str()).await.unwrap();

        let result: Vec<reservations::Model> = Reservations::find().all(&db).await.unwrap();

        let mut i = 1;
        for r in result {
            println!("{} {:?} {:?} {:?} ", i, r.id, r.user_id, r.resource_id);
            i += 1;
        }
    }

    #[tokio::test]
    async fn orm_test_insert_rundom() {
        let db = Database::connect(env_con_str()).await.unwrap();
        let n = reservations::ActiveModel {
            ..Default::default()
        };
        let r = n.insert(&db).await;
        println!("{:?}", r.unwrap());
    }

    #[tokio::test]
    async fn orm_test_update_rundom() {
        let db = Database::connect(env_con_str()).await.unwrap();
        let r: Option<reservations::Model> = Reservations::find().one(&db).await.unwrap();

        let mut r = Into::<reservations::ActiveModel>::into(r.unwrap());
        println!("{:?} {:?}", &r.id, &r.note);

        if let Some(Some(oldvalue)) = r.note.take() {
            println!("oldvalue:{}", oldvalue);
            r.note = Set(Some(format!("{} {}", oldvalue, "edit").to_owned()));
        }
        println!("{:?}", r.is_changed());
        let r = r.update(&db).await.unwrap();
        println!("{:?} {:?}", r.id, r.note);
    }

    #[tokio::test]
    async fn orm_test_delete() {
        let db = &Database::connect(env_con_str()).await.unwrap();
        let r: Option<reservations::Model> = Reservations::find().one(db).await.unwrap();
        if let Some(r) = r {
            let _r = r.delete(db).await;
            println!("{:?}", _r.unwrap());
        }
    }
}
