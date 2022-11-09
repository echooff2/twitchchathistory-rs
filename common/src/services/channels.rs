use diesel::{prelude::*, PgConnection};
use error_stack::{IntoReport, Result, ResultExt};

use crate::{
    models::channel::{Channel, NewChannel},
    schema::channels,
};

pub fn get_channel_by_twitch_id(
    db_conn: &PgConnection,
    twitch_channel_id: &str,
) -> Result<Option<Channel>, diesel::result::Error> {
    log::trace!("getting channel by twitch id: {:?}", twitch_channel_id);

    channels::table
        .filter(channels::twitch_channel_id.eq(twitch_channel_id))
        .first(db_conn)
        .optional()
        .into_report()
        .attach_printable_lazy(|| {
            format!(
                "database error: couldn't get channel with id {}",
                twitch_channel_id
            )
        })
}

pub fn create_channel_if_not_exists(
    db_conn: &PgConnection,
    twitch_channel_id: String,
    channel_name: String,
) -> Result<Channel, diesel::result::Error> {
    log::trace!(
        "creating or getting channel depending if it exists, twitch_channel_id: {:?}",
        twitch_channel_id
    );

    let channel = get_channel_by_twitch_id(db_conn, &twitch_channel_id)?;

    match channel {
        Some(channel) => Ok(channel),
        None => Ok(create_channel(db_conn, twitch_channel_id, channel_name)?),
    }
}

pub fn create_channel(
    db_conn: &PgConnection,
    twitch_channel_id: String,
    channel_name: String,
) -> Result<Channel, diesel::result::Error> {
    log::trace!("inserting channel into db: channel_name: {channel_name}");

    diesel::insert_into(channels::table)
        .values(NewChannel {
            twitch_channel_id,
            channel_name: channel_name.clone(),
        })
        .get_result(db_conn)
        .into_report()
        .attach_printable_lazy(|| {
            format!("database error: couldn't create channel: {channel_name}")
        })
}
