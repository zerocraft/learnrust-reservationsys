use async_trait::async_trait;
use rand::distributions::{Alphanumeric, Distribution};
use rsys_abi::{convert_to_datetime, QueryRequest, Reservation};
use sea_orm::{ActiveValue, EntityTrait};

use crate::{
    entities::prelude::Reservations, entities::reservations, error::RsysError, ReservationManager,
    Rsvp,
};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn create(&self, mut _rsvp: Reservation) -> Result<Reservation, RsysError> {
        let mut r = reservations::ActiveModel {
            r_status: ActiveValue::set(Some(_rsvp.rstatus)),
            ..Default::default()
        };

        if _rsvp.uid != String::default() {
            r.user_id = ActiveValue::set(Some(_rsvp.uid.clone()));
        }
        if _rsvp.resource_id != String::default() {
            r.resource_id = ActiveValue::set(Some(_rsvp.resource_id.clone()));
        }
        if _rsvp.note != String::default() {
            r.note = ActiveValue::set(Some(_rsvp.note.clone()));
        }
        if let Some(start) = _rsvp.start.clone() {
            r.start_time = ActiveValue::set(Some(convert_to_datetime(start)));
        }
        if let Some(end) = _rsvp.end.clone() {
            r.end_time = ActiveValue::set(Some(convert_to_datetime(end)));
        }

        let res = Reservations::insert(r).exec(&self.db).await?;

        _rsvp.id = res.last_insert_id.to_string();

        Ok(_rsvp)
    }

    async fn change_status(&self, _rsvp: Reservation) -> Result<Reservation, RsysError> {
        todo!()
    }

    async fn update_note(&self, _id: String, _note: String) -> Result<Reservation, RsysError> {
        todo!()
    }

    async fn delete(&self, _id: String) -> Result<(), RsysError> {
        todo!()
    }

    async fn query(&self, _query: QueryRequest) -> Result<Vec<Reservation>, RsysError> {
        todo!()
    }
}

#[allow(dead_code)]
pub fn generate_random_string(length: usize) -> String {
    Alphanumeric
        .sample_iter(&mut rand::thread_rng())
        .take(length)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::generate_random_string;
    use crate::ReservationManager;
    use crate::Rsvp;
    use chrono::Duration;
    use chrono::Utc;
    use rand::prelude::*;
    use rsys_abi::convert_to_timestamp;
    use rsys_abi::Reservation;
    use sea_orm::Database;

    #[test]
    fn random_string() {
        for i in 1..11 {
            println!("{} {:?}", i, generate_random_string(7));
        }
    }

    #[tokio::test]
    async fn rm_create() {
        const DATABASE_URL: &str = "postgresql://postgres:post@localhost:5432/rsys";
        let rm: ReservationManager = ReservationManager {
            db: Database::connect(DATABASE_URL).await.unwrap(),
        };

        for _ in 0..10 {
            let _ = rm
                .create(Reservation {
                    uid: generate_random_string(7),
                    resource_id: generate_random_string(8),
                    start: Some(convert_to_timestamp(
                        Utc::now()
                            .checked_add_signed(Duration::hours(
                                rand::thread_rng().gen_range(-100..-1),
                            ))
                            .unwrap(),
                    )),
                    end: Some(convert_to_timestamp(
                        Utc::now()
                            .checked_add_signed(Duration::hours(
                                rand::thread_rng().gen_range(0..100),
                            ))
                            .unwrap(),
                    )),
                    rstatus: 0,
                    note: generate_random_string(10),
                    ..Default::default()
                })
                .await;
        }
    }
}
