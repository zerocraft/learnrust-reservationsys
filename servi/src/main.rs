use std::env;

use anyhow::Result;
use rsys_servi::config::Config;
use rsys_servi::server_start;

#[tokio::main]
async fn main() -> Result<()> {
    let path = env::current_dir();
    println!("{:?}", path.unwrap().as_mut_os_string());
    let config = Config::load("./config.yml").await?;
    println!("{:?}", config);
    server_start(&config).await
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
