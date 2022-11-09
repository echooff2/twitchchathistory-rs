use crate::{
    models::user::{NewUser, User},
    schema::users,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use error_stack::{IntoReport, Result, ResultExt};

use super::users_old_names;

pub fn get_user_by_user_id(
    user_id: &str,
    db_conn: &PgConnection,
) -> Result<Option<User>, diesel::result::Error> {
    users::table
        .filter(users::twitch_user_id.eq(user_id))
        .first(db_conn)
        .optional()
        .into_report()
        .attach_printable_lazy(|| {
            format!(
                "database error: couldn't get user with user id: {}",
                user_id
            )
        })
}

pub fn create_user(
    new_user: NewUser,
    db_conn: &PgConnection,
) -> Result<User, diesel::result::Error> {
    diesel::insert_into(users::table)
        .values(new_user.clone())
        .get_result(db_conn)
        .into_report()
        .attach_printable_lazy(|| {
            format!(
                "couldn't insert user {}, with id: {} into db",
                new_user.username, new_user.twitch_user_id
            )
        })
}

pub fn check_and_fix_username(db_conn: &PgConnection, user: User, user_name: &str, timestamp: DateTime<Utc>) -> error_stack::Result<(), diesel::result::Error> {
    if user.username == user_name {
        return Ok(());
    }

    users_old_names::create(db_conn, user.id, user.username, timestamp)?;

    diesel::update(users::table)
        .filter(users::id.eq(user.id))
        .set(users::username.eq(user_name))
        .execute(db_conn)
        .into_report()?;

    Ok(())
}