use std::{path::Path, thread};

use sqlx::{migrate::Migrator, types::Uuid, Connection, Executor, PgConnection, PgPool};
use tokio::runtime::Runtime;

#[allow(dead_code)]
pub struct DbTester {
    server_url: String,
    dbname: String,
}

impl DbTester {
    pub fn new(server_url: impl Into<String>, migration_path: impl Into<String>) -> Self {
        let uuid = Uuid::new_v4();
        let dbname = format!("test_{}", uuid);
        let dbname_c = dbname.clone();
        let base_url = server_url.into().clone();
        let biz_url = format!("{}/{}", base_url, dbname);
        let biz_url_c = biz_url.clone();
        let migration_path: String = migration_path.into();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut conn = PgConnection::connect(&base_url).await.unwrap();
                conn.execute(format!(r#"CREATE DATABASE "{}""#, dbname_c).as_str())
                    .await
                    .unwrap();
                let mut conn = PgConnection::connect(&biz_url_c).await.unwrap();
                let mig = Migrator::new(Path::new(migration_path.as_str()))
                    .await
                    .unwrap();
                mig.run(&mut conn).await.unwrap();
            })
        })
        .join()
        .expect("Failed to create database");

        Self {
            server_url: biz_url,
            dbname,
        }
    }

    pub async fn get_pool(&self) -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(self.server_url.as_str())
            .await
            .unwrap()
    }
}

impl Drop for DbTester {
    fn drop(&mut self) {
        let server_url = self.server_url.clone();
        let db_name = self.dbname.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut conn = PgConnection::connect(server_url.as_str()).await.unwrap();
                sqlx::query(&format!(r#"select pg_terminate_backend(pid) from pg_stat_activity where pid <> pg_backend_pid() and datname = '{}'"#
                    ,db_name)).execute(&mut conn).await.expect("terminate all other connections");
                conn.execute(format!(r#"drop database "{}""#, db_name).as_str())
                    .await
                    .expect("error drop database");
            })
        })
        .join()
        .expect("Failed to drop database");
    }
}
