use crate::{
    entities::prelude::Reservations, entities::reservations, error::RsysError, ReservationManager,
    Rsvp,
};
use async_trait::async_trait;
use futures::StreamExt;
use rsys_abi::{
    convert_to_datetime, CancelRequest, ConfirmRequest, GetRequest, ListenRequest, QueryRequest,
    Reservation, UpdateRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, Database, EntityTrait, QueryFilter, Set,
};
use sqlx::{types::Uuid, PgPool, Row};
use tokio::sync::mpsc::{self, Receiver};

impl ReservationManager {
    pub async fn new(constr: String) -> Result<Self, RsysError> {
        let con = Database::connect(constr.clone()).await;
        if con.is_err() {
            return Err(RsysError::ConfigError(constr));
        }
        Ok(ReservationManager {
            constr,
            db: con.unwrap(),
        })
    }

    pub async fn create_sqlx(
        mut rsvp: Reservation,
        pool: PgPool,
    ) -> Result<Reservation, RsysError> {
        let id: Uuid = sqlx::query(
            "insert into rsvp.reservations (user_id,resource_id,r_status,start_time,end_time,note)
         values($1,$2,$3,$4,$5,$6) returning id",
        )
        .bind(rsvp.uid.clone())
        .bind(rsvp.resource_id.clone())
        .bind(rsvp.rstatus)
        .bind(convert_to_datetime(rsvp.start.clone().unwrap()))
        .bind(convert_to_datetime(rsvp.end.clone().unwrap()))
        .bind(rsvp.note.clone())
        .fetch_one(&pool)
        .await?
        .get(0);

        // let id = sqlx::query!(
        //     "insert into rsvp.reservations (user_id,resource_id,r_status,start_time,end_time,note) values($1,$2,$3,$4,$5,$6) returning id",
        //     rsvp.uid.clone(),
        //     rsvp.resource_id.clone(),
        //     rsvp.rstatus.clone(),
        //     convert_to_datetime(rsvp.start.clone().unwrap()),
        //     convert_to_datetime(rsvp.end.clone().unwrap()),
        //     rsvp.note.clone()
        // )
        // .fetch_one(&pool)
        // .await?
        // .id;

        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    pub async fn query_sqlx(id: &str, pool: PgPool) -> Result<reservations::Model, RsysError> {
        let uuid = Uuid::parse_str(id).map_err(|_err| RsysError::Unknown)?;
        let result: reservations::Model =
            sqlx::query_as("select * from rsvp.reservations where id = $1")
                .bind(uuid)
                .fetch_one(&pool)
                .await?;
        Ok(result)
    }

    pub async fn query_many_sqlx(
        uid: &str,
        pool: PgPool,
    ) -> Result<mpsc::Receiver<reservations::Model>, RsysError> {
        let uid = uid.to_string();
        let (tx, rx) = mpsc::channel::<reservations::Model>(128);
        tokio::spawn(async move {
            let mut rsvps = sqlx::query_as("select * from rsvp.reservations where user_id = $1")
                .bind(uid)
                .fetch_many(&pool);
            while let Some(Ok(ret)) = rsvps.next().await {
                match ret {
                    sqlx::Either::Left(_) => {}
                    sqlx::Either::Right(r) => {
                        if tx.send(r).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(rx)
    }
}

#[async_trait]
impl Rsvp for ReservationManager {
    async fn create(&self, mut _rsvp: Reservation) -> Result<Reservation, RsysError> {
        let mut r = reservations::ActiveModel {
            r_status: ActiveValue::set(Some(_rsvp.rstatus)),
            ..Default::default()
        };

        let mut cond_s = Condition::all();
        if _rsvp.uid != String::default() {
            r.user_id = ActiveValue::set(Some(_rsvp.uid.clone()));
        }
        if _rsvp.resource_id != String::default() {
            r.resource_id = ActiveValue::set(Some(_rsvp.resource_id.clone()));
            cond_s = cond_s.add(reservations::Column::ResourceId.eq(_rsvp.resource_id.clone()));
        }
        if _rsvp.note != String::default() {
            r.note = ActiveValue::set(Some(_rsvp.note.clone()));
        }
        let mut cond_t = Condition::any();
        if let Some(start) = _rsvp.start.clone() {
            let st = Some(convert_to_datetime(start));
            r.start_time = ActiveValue::set(st);
            cond_t = cond_t.add(
                Condition::all()
                    .add(reservations::Column::StartTime.lt(st))
                    .add(reservations::Column::EndTime.gt(st)),
            );
        }
        if let Some(end) = _rsvp.end.clone() {
            let et = Some(convert_to_datetime(end));
            r.end_time = ActiveValue::set(et);
            cond_t = cond_t.add(
                Condition::all()
                    .add(reservations::Column::StartTime.gt(et))
                    .add(reservations::Column::EndTime.lt(et)),
            );
        }

        if Reservations::find()
            .filter(cond_s.add(cond_t))
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(RsysError::AlreadyBooked);
        }

        let res = Reservations::insert(r).exec(&self.db).await?;

        _rsvp.id = res.last_insert_id.to_string();

        Ok(_rsvp)
    }

    async fn change_status(&self, change: ConfirmRequest) -> Result<Reservation, RsysError> {
        let mut result = Reservation {
            ..Default::default()
        };
        if let Ok(id) = Uuid::parse_str(change.id.as_str()) {
            let e = Reservations::find_by_id(id).one(&self.db).await?;
            let mut e: reservations::ActiveModel = e.unwrap().into();
            e.r_status = Set(Some(1));
            let r = e.update(&self.db).await?;
            result.id = id.to_string();
            result.rstatus = r.r_status.unwrap_or_default();
        } else {
            return Err(RsysError::NoReservation);
        }
        Ok(result)
    }

    async fn update_note(&self, update: UpdateRequest) -> Result<Reservation, RsysError> {
        if let Ok(id) = Uuid::parse_str(update.id.as_str()) {
            let e = Reservations::find_by_id(id).one(&self.db).await?;
            let mut e: reservations::ActiveModel = e.unwrap().into();
            e.note = Set(Some(update.note));
            let r = e.update(&self.db).await?;
            return Ok(r.into());
        } else {
            return Err(RsysError::NoReservation);
        }
    }

    async fn get(&self, get: GetRequest) -> Result<Reservation, RsysError> {
        if let Ok(id) = Uuid::parse_str(get.id.as_str()) {
            let e = Reservations::find_by_id(id).one(&self.db).await?;
            if let Some(r) = e {
                return Ok(r.into());
            }
        }
        return Err(RsysError::NoReservation);
    }

    async fn delete(&self, cancel: CancelRequest) -> Result<usize, RsysError> {
        if let Ok(id) = Uuid::parse_str(cancel.id.as_str()) {
            let result = Reservations::delete_by_id(id).exec(&self.db).await?;
            return Ok(result.rows_affected as usize);
        }
        Ok(0)
    }

    async fn query(&self, _query: QueryRequest) -> Receiver<Result<Reservation, RsysError>> {
        let uid = _query.uid;
        let result = Reservations::find()
            .filter(reservations::Column::UserId.eq(uid))
            .stream(&self.db)
            .await;
        let mut result = result.unwrap();
        let (tx, rx) = mpsc::channel::<Result<Reservation, RsysError>>(128);
        loop {
            let i = result.next().await;
            if i.is_none() {
                break;
            }
            match i.unwrap() {
                Ok(item) => {
                    if tx.send(Ok(item.into())).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(RsysError::DbError(err))).await;
                    break;
                }
            }
        }
        rx
    }

    async fn listen(&self, _listen: ListenRequest) -> Result<Vec<Reservation>, RsysError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::env_con_str;
    use crate::generate_random_reservation;
    use crate::generate_random_string;
    use crate::ReservationManager;
    use crate::Rsvp;
    use chrono::Duration;
    use chrono::Utc;
    use rand::prelude::*;
    use rsys_abi::convert_to_timestamp;
    use rsys_abi::QueryRequest;
    use rsys_abi::Reservation;
    use rsys_abi::UpdateRequest;
    use sqlx::postgres::PgPoolOptions;

    #[test]
    fn random_string() {
        for i in 1..11 {
            println!("{} {:?}", i, generate_random_string(7));
        }
    }

    #[tokio::test]
    async fn rm_createx() {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(env_con_str().as_str())
            .await
            .unwrap();
        let r = ReservationManager::create_sqlx(
            Reservation {
                start: Some(convert_to_timestamp(
                    Utc::now()
                        .checked_add_signed(Duration::hours(rand::thread_rng().gen_range(-100..-1)))
                        .unwrap(),
                )),
                end: Some(convert_to_timestamp(
                    Utc::now()
                        .checked_add_signed(Duration::hours(rand::thread_rng().gen_range(0..100)))
                        .unwrap(),
                )),
                ..Default::default()
            },
            pool,
        )
        .await;
        println!("{:?}", r);
    }

    #[tokio::test]
    async fn rm_queryx() {
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        let result = rm
            .create(Reservation::new_pending(
                generate_random_string(7),
                generate_random_string(8),
                generate_random_string(11),
                Utc::now()
                    .checked_add_signed(Duration::hours(rand::thread_rng().gen_range(1..101)))
                    .unwrap(),
                Utc::now()
                    .checked_add_signed(Duration::hours(rand::thread_rng().gen_range(-101..-1)))
                    .unwrap(),
            ))
            .await;
        let result = result.unwrap();
        println!("{:?}", result);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(env_con_str().as_str())
            .await
            .unwrap();
        let result = ReservationManager::query_sqlx(result.id.as_str(), pool).await;
        println!("{:?}", result.unwrap());
    }

    #[tokio::test]
    async fn rm_query_manyx() {
        let uid = "rm_query_manyx";
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        for _ in 0..5 {
            let mut r = generate_random_reservation();
            r.uid = uid.to_string();
            let _ = rm.create(r).await;
        }
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(env_con_str().as_str())
            .await
            .unwrap();
        let mut result = ReservationManager::query_many_sqlx(uid, pool)
            .await
            .unwrap();
        while let Some(r) = result.recv().await {
            println!("{:?}", r);
        }
    }

    #[tokio::test]
    async fn rm_query_many() {
        let uid = "rm_query_manyx";
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        let mut result = rm
            .query(QueryRequest {
                uid: uid.to_string(),
            })
            .await;
        while let Some(i) = result.recv().await {
            println!("{:?}", i);
        }
    }

    #[tokio::test]
    async fn rm_create() {
        let rm = ReservationManager::new(env_con_str()).await.unwrap();

        for i in 1..11 {
            let result = rm
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
            let result = result.unwrap();
            println!("{:?} {:?} {:?}", i, result.id, result.uid);
        }
    }

    #[tokio::test]
    async fn rm_create_single() {
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        let result = rm.create(generate_random_reservation()).await;
        let result = result.unwrap();
        println!("{:?} {:?}", result.id, result.uid);
    }

    #[tokio::test]
    async fn test_already_booked() {
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        let resource_id = "test_resource";
        let off = rand::thread_rng().gen_range(1..101);
        for _ in 0..2 {
            let result = rm
                .create(Reservation {
                    uid: generate_random_string(7),
                    resource_id: resource_id.to_string(),
                    start: Some(convert_to_timestamp(
                        Utc::now().checked_add_signed(Duration::hours(off)).unwrap(),
                    )),
                    end: Some(convert_to_timestamp(
                        Utc::now()
                            .checked_add_signed(Duration::hours(off + 1))
                            .unwrap(),
                    )),
                    rstatus: 0,
                    note: generate_random_string(10),
                    ..Default::default()
                })
                .await;
            println!("{:?}", result);
        }
    }

    #[tokio::test]
    async fn test_change_status() {
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        let off = rand::thread_rng().gen_range(1..101);
        let result = rm
            .create(Reservation {
                uid: generate_random_string(7),
                resource_id: generate_random_string(8),
                start: Some(convert_to_timestamp(
                    Utc::now().checked_add_signed(Duration::hours(off)).unwrap(),
                )),
                end: Some(convert_to_timestamp(
                    Utc::now()
                        .checked_add_signed(Duration::hours(off + 1))
                        .unwrap(),
                )),
                rstatus: 0,
                note: generate_random_string(10),
                ..Default::default()
            })
            .await;
        if let Ok(data) = result {
            println!("{:?}", data);
            let data = rm.change_status(data.id.into()).await;
            println!("{:?}", data);
            let data = rm.change_status("data.id.as_str()".into()).await;
            println!("{:?}", data);
        }
    }

    #[tokio::test]
    async fn test_change_note() {
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        let off = rand::thread_rng().gen_range(1..101);
        let result = rm
            .create(Reservation {
                uid: generate_random_string(7),
                resource_id: generate_random_string(8),
                start: Some(convert_to_timestamp(
                    Utc::now().checked_add_signed(Duration::hours(off)).unwrap(),
                )),
                end: Some(convert_to_timestamp(
                    Utc::now()
                        .checked_add_signed(Duration::hours(off + 1))
                        .unwrap(),
                )),
                rstatus: 0,
                note: generate_random_string(10),
                ..Default::default()
            })
            .await;
        if let Ok(data) = result {
            println!("{:?}", data);
            let data = rm
                .update_note(UpdateRequest {
                    id: data.id,
                    note: generate_random_string(20),
                })
                .await;
            println!("{:?}", data);
        }
    }

    #[tokio::test]
    async fn test_delete() {
        let rm = ReservationManager::new(env_con_str()).await.unwrap();
        let off = rand::thread_rng().gen_range(1..101);
        let result = rm
            .create(Reservation {
                uid: generate_random_string(7),
                resource_id: generate_random_string(8),
                start: Some(convert_to_timestamp(
                    Utc::now().checked_add_signed(Duration::hours(off)).unwrap(),
                )),
                end: Some(convert_to_timestamp(
                    Utc::now()
                        .checked_add_signed(Duration::hours(off + 1))
                        .unwrap(),
                )),
                rstatus: 0,
                note: generate_random_string(10),
                ..Default::default()
            })
            .await;
        if let Ok(data) = result {
            println!("{:?}", data);
            println!("{:?}", rm.delete(data.id.into()).await);
        }
    }
}
