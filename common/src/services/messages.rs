use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use error_stack::{IntoReport, Result, ResultExt};

use crate::{
    models::{
        message::{MsgType, NewMessage},
        resub::NewResub,
        user::NewUser,
    },
    schema::messages,
};

use super::{
    resubs,
    users::{self, create_user},
};

pub fn create_message(
    db_conn: &PgConnection,
    msg: String,
    msg_type: MsgType,
    channel_id: i32,
    send_time: DateTime<Utc>,
    bits: Option<i64>,
    new_resub: Option<NewResub>,
    twitch_user_id: String,
    twitch_username: String,
    _twitch_displayname: String,
) -> Result<usize, diesel::result::Error> {
    let user = users::get_user_by_user_id(&twitch_user_id, db_conn)?
        .map_or_else(
            || {
                let new_user = NewUser {
                    username: twitch_username.clone(),
                    twitch_user_id,
                };

                create_user(new_user, db_conn)
            },
            |v| Ok(v),
        )
        .attach_printable_lazy(|| {
            format!("couldn't create message for user because of db error while getting user")
        })?;

    let user_id = user.id;

    users::check_and_fix_username(db_conn, user, &twitch_username, send_time)?;

    let new_resub = new_resub.map(|r| resubs::create_resub_return(db_conn, r));

    let resub_id = match new_resub {
        Some(r) => Some(r?),
        None => None,
    }.map(|r| r.id);

    // message.user_id = user.id;
    let message = NewMessage {
        msg,
        msg_type,
        user_id,
        channel_id,
        send_time,
        bits,
        resub_id,
    };

    diesel::insert_into(messages::table)
        .values(message)
        .execute(db_conn)
        .into_report()
        .attach_printable("database error: couldn't insert message")
}
