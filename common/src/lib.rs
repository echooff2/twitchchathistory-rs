#[macro_use]
extern crate diesel;

#[macro_use]
extern crate anyhow;

pub mod config;
pub mod models;
pub mod schema;
pub mod services;

// pub async fn establish_db_connection() -> diesel::ConnectionResult<PgConnection> {
//     PgConnection::establish(&construct_db_url())
// }

pub fn construct_db_url_blocking() -> String {
    let config = get_config_blocking!();

    format!(
        "postgres://{}:{}@{}/{}",
        config.database.username.clone(),
        config.database.password,
        config.database.url.clone(),
        config.database.db.clone(),
    )
}

pub async fn construct_db_url_async() -> String {
    let config = get_config_async!().await;

    format!(
        "postgres://{}:{}@{}/{}",
        config.database.username.clone(),
        config.database.password,
        config.database.url.clone(),
        config.database.db.clone(),
    )
}
