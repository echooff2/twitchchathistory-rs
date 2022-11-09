use crate::schema::channels;
use uuid::Uuid;

#[derive(Queryable, Debug, Clone)]
pub struct Channel {
    pub id: i32,
    pub uuid: Uuid,
    pub twitch_channel_id: String,
    pub channel_name: String,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "channels"]
pub struct NewChannel {
    pub twitch_channel_id: String,
    pub channel_name: String,
}
