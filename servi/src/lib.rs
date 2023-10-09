pub mod config;
mod error;
mod service;

use anyhow::{Ok, Result};
use config::Config;
use rsys::ReservationManager;
use rsys_abi::reservation_service_server::ReservationServiceServer;
use std::ops::Deref;

use tonic::transport::Server;

struct RServic {
    pub manager: ReservationManager,
}

impl RServic {
    pub async fn load_from_config(config: &Config) -> anyhow::Result<RServic> {
        anyhow::Ok(RServic {
            manager: ReservationManager::new(config.db.url.clone()).await?,
        })
    }
}

impl Deref for RServic {
    type Target = ReservationManager;
    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

pub async fn server_start(config: &Config) -> Result<()> {
    let svc = RServic::load_from_config(config).await?;
    let svc = ReservationServiceServer::new(svc);
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
