use std::env;

use anyhow::{Ok, Result};
use rsys::ReservationManager;
use rsys_abi::reservation_service_server::ReservationServiceServer;
use rsys_servi::{config::Config, RServic};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let path = env::current_dir();
    println!("{:?}", path.unwrap().as_mut_os_string());
    let config = Config::load("./config.yml").await?;
    println!("{:?}", config);
    let svc = RServic {
        manager: ReservationManager::new(config.db.url).await?,
    };
    let svc = ReservationServiceServer::new(svc);
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    #[test]
    fn current_path() {
        let path = env::current_dir();
        println!("{:?}", path.unwrap().as_mut_os_string());
    }
}
