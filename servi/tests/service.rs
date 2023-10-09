use rsys_abi::{reservation_service_client::ReservationServiceClient, ReserveRequest};
use rsys_servi::{config::Config, server_start};
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn server_should_word() {
    let config = Config::load("../config.yml").await.unwrap();
    println!("{:?}", config);
    let url = format!("http://localhost:{}", config.server.port);
    tokio::spawn(async move {
        let _ = server_start(&config).await;
    });
    time::sleep(Duration::from_millis(1000)).await;
    println!("Client connect to {:?}", url);
    let mut client = ReservationServiceClient::connect(url).await.unwrap();
    let data = rsys::generate_random_reservation();
    println!("{:?}", data);
    let req = tonic::Request::new(ReserveRequest {
        reservation: Some(data),
    });
    let resp = client.reserve(req).await;
    let data = resp.unwrap().into_inner();
    println!("{:?}", data);
}
