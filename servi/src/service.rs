use crate::{error::ServError, RServic};
use futures::Stream;
use rsys::{error::RsysError, Rsvp};
use rsys_abi::*;
use std::{pin::Pin, task::Poll};
use tokio::sync::mpsc::Receiver;
use tonic::{async_trait, Request, Response, Status};

pub struct RStream<T> {
    inner: Receiver<Result<T, RsysError>>,
}

impl<T> RStream<T> {
    pub fn new(inner: Receiver<Result<T, RsysError>>) -> Self {
        RStream { inner }
    }
}

impl<T> Stream for RStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(Ok(i))) => Poll::Ready(Some(Ok(i))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(ServError(e).into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
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
        return Ok(Response::new(Box::pin(RStream::new(r))));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use rsys_abi::reservation_service_server::ReservationService;

    #[tokio::test]
    async fn test() {
        let config = Config::load("../config.yml").await.unwrap();
        let svc = RServic::load_from_config(&config).await.unwrap();
        let req = tonic::Request::new(ReserveRequest {
            reservation: Some(rsys::generate_random_reservation()),
        });
        println!("req:{:?}", req);
        let resp = svc.reserve(req).await.unwrap();
        println!("resp:{:?}", resp);
        println!("resp-inner:{:?}", resp.into_inner());
    }
}
