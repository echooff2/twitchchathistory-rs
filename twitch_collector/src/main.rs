#![feature(proc_macro_hygiene, decl_macro)]

use std::process::exit;

use diesel::r2d2::{ConnectionManager, Pool};

use common::config;
use twitch_api2::HelixClient;

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

mod twitch_watcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    if let Err(err) = config::load().await {
        println!("{}", err);
        exit(1);
    }

    let config = get_config_async!().await;

    // TODO log to file
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(config.log_level.into())
        .init();

    debug!("debug works");
    info!("info works");
    warn!("warn works");

    let twitch_api_client: HelixClient<reqwest::Client> = HelixClient::default();

    // let db_conn = PgConnection::establish(&get_config_async!().await.database.url.clone())?;
    let conn_manager = ConnectionManager::new(common::construct_db_url_async().await);
    let pool = Pool::new(conn_manager).expect("error while creating db pool");

    // run only returns in case of an error
    let run = twitch_watcher::run(pool.clone(), twitch_api_client).await.unwrap_err();

    println!("{}", run);
    error!("{:?}", run);

    info!("exiting");
    Ok(())
}
