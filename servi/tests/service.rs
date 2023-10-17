use futures::StreamExt;
use rsys_abi::{
    reservation_service_client::ReservationServiceClient, QueryRequest, ReserveRequest,
};
use rsys_servi::{config::Config, server_start};
use std::time::Duration;
use tokio::time;
use tonic::transport::Channel;

macro_rules! test_server_start {
    ($port:expr) => {
        test_server_start(Some($port))
    };
    () => {
        test_server_start(None)
    };
}

async fn test_server_start(port: Option<u16>) -> ReservationServiceClient<Channel> {
    let mut config = Config::load("../config.yml").await.unwrap();
    if port.is_some() {
        config.server.port = port.unwrap();
    }
    println!("{:?}", config);
    let url = format!("http://localhost:{}", config.server.port);

    tokio::spawn(async move {
        let _ = server_start(&config).await;
        println!("server stop");
    });

    time::sleep(Duration::from_millis(1000)).await;
    println!("Client connect to {:?}", url);
    ReservationServiceClient::connect(url).await.unwrap()
}

#[tokio::test]
async fn server_should_work() {
    let mut client = test_server_start!(50000).await;

    let data = rsys::generate_random_reservation();
    println!("{:?}", data);
    let req = tonic::Request::new(ReserveRequest {
        reservation: Some(data),
    });
    let resp = client.reserve(req).await;
    let data = resp.unwrap().into_inner();
    println!("{:?}", data);
}

#[tokio::test]
async fn load_query_steam() {
    let mut client = test_server_start!(50001).await;

    let uid = "rm_query_manyx";
    let req = tonic::Request::new(QueryRequest {
        uid: uid.to_string(),
    });
    let resp = client.query(req).await;
    let mut datas = resp.unwrap().into_inner();

    while let Some(Ok(item)) = datas.next().await {
        println!("{:?}", item);
    }
}
