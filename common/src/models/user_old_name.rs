use chrono::{DateTime, Utc};

use crate::schema::users_old_names;

#[derive(Queryable, Debug, Clone)]
pub struct UserOldName {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub first_time_with_new_name: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "users_old_names"]
pub struct NewUserOldName {
    pub user_id: i32,
    pub username: String,
    pub first_time_with_new_name: DateTime<Utc>,
}
