use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use error_stack::{IntoReport, ResultExt};

use crate::{models::user_old_name::NewUserOldName, schema::users_old_names};

pub fn create(
    db_conn: &PgConnection,
    user_id: i32,
    old_name: String,
    first_time_with_new_name: DateTime<Utc>,
) -> error_stack::Result<usize, diesel::result::Error> {
    diesel::insert_into(users_old_names::table)
        .values(NewUserOldName {
            user_id,
            username: old_name.clone(),
            first_time_with_new_name,
        })
        .execute(db_conn)
        .into_report()
        .attach_printable_lazy(|| format!("values: user_id: {}, username: {}", user_id, old_name))
}
