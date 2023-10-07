pub mod config;
mod error;

use crate::error::ServError;
use futures::Stream;
use rsys::{ReservationManager, Rsvp};
use rsys_abi::*;
use std::{ops::Deref, pin::Pin};
use tonic::{async_trait, Request, Response, Status};

pub struct RServic {
    pub manager: ReservationManager,
}

impl Deref for RServic {
    type Target = ReservationManager;
    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

#[async_trait]
impl rsys_abi::reservation_service_server::ReservationService for RServic {
    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> Result<Response<Reservation>, Status> {
        let r = request.into_inner();
        if let Some(reservation) = r.reservation {
            let r = self.manager.create(reservation).await;
            if r.is_err() {
                return Err(ServError(r.err().unwrap()).into());
            }
            return Ok(Response::new(r.unwrap()));
        }
        return Err(Status::invalid_argument("no reservation"));
    }

    async fn confirm(
        &self,
        request: Request<ConfirmRequest>,
    ) -> Result<Response<Reservation>, Status> {
        let r = request.into_inner();
        let r = self.manager.change_status(r).await;
        if r.is_err() {
            return Err(ServError(r.err().unwrap()).into());
        }
        return Ok(Response::new(r.unwrap()));
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<Reservation>, Status> {
        let r = request.into_inner();
        let r = self.manager.update_note(r).await;
        if r.is_err() {
            return Err(ServError(r.err().unwrap()).into());
        }
        return Ok(Response::new(r.unwrap()));
    }

    async fn cancel(
        &self,
        request: Request<CancelRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        let r = request.into_inner();
        let r = self.manager.delete(r).await;
        if r.is_err() {
            return Err(ServError(r.err().unwrap()).into());
        }
        return Ok(Response::new(ActionResponse { done: true }));
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<Reservation>, Status> {
        let r = request.into_inner();
        let r = self.manager.get(r).await;
        if r.is_err() {
            return Err(ServError(r.err().unwrap()).into());
        }
        return Ok(Response::new(r.unwrap()));
    }

    type queryStream = ReservationStream;

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::queryStream>, Status> {
        let r = request.into_inner();
        let r = self.manager.query(r).await;
        if r.is_err() {
            return Err(ServError(r.err().unwrap()).into());
        }
        todo!()
        //return Ok(Response::new(r.unwrap()));
    }

    type listenStream = ReservationStream;

    async fn listen(
        &self,
        request: Request<ListenRequest>,
    ) -> Result<Response<Self::listenStream>, Status> {
        let r = request.into_inner();
        let r = self.manager.listen(r).await;
        if r.is_err() {
            return Err(ServError(r.err().unwrap()).into());
        }
        todo!()
    }
}
