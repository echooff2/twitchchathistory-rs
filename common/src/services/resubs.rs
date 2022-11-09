use diesel::{prelude::*, PgConnection};
use error_stack::IntoReport;

use crate::{models::resub::{Resub, NewResub}, schema::resubs};

pub fn create_resub_return(
    db_conn: &PgConnection,
    new_resub: NewResub
) -> error_stack::Result<Resub, diesel::result::Error> {
    diesel::insert_into(resubs::table)
        .values(new_resub)
        .get_result(db_conn)
        .into_report()
}
