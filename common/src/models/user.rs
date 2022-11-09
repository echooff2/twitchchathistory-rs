use crate::schema::users;
use uuid::Uuid;

#[derive(Queryable, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub uuid: Uuid,
    pub username: String,
    pub twitch_user_id: String,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub twitch_user_id: String,
}
