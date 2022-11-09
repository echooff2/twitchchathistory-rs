use chrono::{DateTime, Utc};
use diesel::{
    deserialize::FromSql,
    pg::Pg,
    serialize::ToSql,
    types::{IsNull, VarChar},
};
use uuid::Uuid;

use crate::schema::messages;

#[derive(Debug, AsExpression, FromSqlRow, Clone, Copy)]
#[sql_type = "VarChar"]
pub enum MsgType {
    /// normal message
    Message,
    /// action message - /me
    Action,
    /// when message is bits (msg.bits.is_some() == true)
    Bits,
    /// sub message, resub only
    Sub,
}

impl ToSql<VarChar, Pg> for MsgType
where
    String: ToSql<VarChar, Pg>,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            Self::Message => {
                <String as ToSql<VarChar, Pg>>::to_sql(&"message".to_owned(), out)?;
            }
            Self::Action => {
                <String as ToSql<VarChar, Pg>>::to_sql(&"action".to_owned(), out)?;
            }
            Self::Bits => {
                <String as ToSql<VarChar, Pg>>::to_sql(&"bits".to_owned(), out)?;
            }
            Self::Sub => {
                <String as ToSql<VarChar, Pg>>::to_sql(&"sub".to_owned(), out)?;
            }
        };

        Ok(IsNull::No)
    }
}

impl FromSql<VarChar, Pg> for MsgType {
    fn from_sql(
        bytes: Option<&<Pg as diesel::backend::Backend>::RawValue>,
    ) -> diesel::deserialize::Result<Self> {
        let bytes = bytes.ok_or(anyhow!("no bytes given"))?;

        match bytes {
            b"message" => Ok(MsgType::Message),
            b"action" => Ok(MsgType::Action),
            b"bits" => Ok(MsgType::Bits),
            b"sub" => Ok(MsgType::Sub),
            _ => Err(anyhow!("Bytes given doesn't match MsgType type"))?,
        }
    }
}

#[derive(Queryable, Debug, Clone)]
pub struct Message {
    pub id: i64,
    pub uuid: Uuid,
    pub msg: String,
    pub msg_type: MsgType,
    pub user_id: i32,
    pub channel_id: i32,
    pub resub_id: Option<i32>,
    pub send_time: DateTime<Utc>,
    pub bits: Option<i64>,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "messages"]
pub struct NewMessage {
    pub msg: String,
    pub msg_type: MsgType,
    pub user_id: i32,
    pub channel_id: i32,
    pub send_time: DateTime<Utc>,
    pub bits: Option<i64>,
    pub resub_id: Option<i32>,
}

impl From<Message> for NewMessage {
    fn from(message: Message) -> Self {
        Self {
            msg: message.msg,
            msg_type: message.msg_type,
            user_id: message.user_id,
            channel_id: message.channel_id,
            send_time: message.send_time,
            bits: message.bits,
            resub_id: message.resub_id,
        }
    }
}
