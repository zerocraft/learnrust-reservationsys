pub mod entities;

mod error;
mod manager;

use async_trait::async_trait;
use error::RsysError;
use rsys_abi::{QueryRequest, Reservation};
use sea_orm::DatabaseConnection;

#[async_trait]
pub trait Rsvp {
    async fn create(&self, rsvp: Reservation) -> Result<Reservation, RsysError>;

    async fn change_status(&self, rsvp: Reservation) -> Result<Reservation, RsysError>;

    async fn update_note(&self, id: String, note: String) -> Result<Reservation, RsysError>;

    async fn delete(&self, id: String) -> Result<(), RsysError>;

    async fn query(&self, query: QueryRequest) -> Result<Vec<Reservation>, RsysError>;
}

#[derive(Debug)]
pub struct ReservationManager {
    db: DatabaseConnection,
}

#[cfg(test)]
mod tests {
    use crate::entities::prelude::*;
    use crate::entities::reservations;
    use sea_orm::Database;
    use sea_orm::EntityTrait;

    const DATABASE_URL: &str = "postgresql://postgres:post@localhost:5432/rsys";

    #[tokio::test]
    async fn orm_test() {
        let db = Database::connect(DATABASE_URL).await.unwrap();

        let result: Vec<reservations::Model> = Reservations::find().all(&db).await.unwrap();

        let mut i = 1;
        for r in result {
            println!("{} {:?} {:?} {:?} ", i, r.id, r.user_id, r.resource_id);
            i += 1;
        }
    }
}
