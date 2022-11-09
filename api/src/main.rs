#![feature(proc_macro_hygiene, decl_macro)]

use rocket_contrib::database;

use common::config;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

// #[macro_use]
// mod config;
// mod models;
// mod schema;
// mod twitch_watcher;

#[database("chat")]
struct ChatDbConn(diesel::PgConnection);

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    config::load_blocking()?;

    // TODO pass info from my config to rocket config

    rocket::ignite()
        .attach(ChatDbConn::fairing())
        .mount("/", routes![])
        .launch();

    trace!("exiting");
    Ok(())
}
