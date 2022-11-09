use crate::schema::resubs;
use byteorder::{ReadBytesExt, NetworkEndian};
use diesel::{
    pg::Pg,
    sql_types::SmallInt,
    types::{FromSql, ToSql},
};
use strum_macros::FromRepr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow, FromRepr)]
#[repr(i16)]
#[sql_type = "SmallInt"]
pub enum Tier {
    Prime = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl ToSql<SmallInt, Pg> for Tier
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, Pg>,
    ) -> diesel::serialize::Result {
        <i16 as ToSql<SmallInt, Pg>>::to_sql(&(*self as i16), out)
    }
}

impl FromSql<SmallInt, Pg> for Tier {
    fn from_sql(
        bytes: Option<&<Pg as diesel::backend::Backend>::RawValue>,
    ) -> diesel::deserialize::Result<Self> {
        let mut bytes = bytes.ok_or(anyhow!("no bytes given"))?;

        // Postgres uses network endian
        let tier_num = bytes.read_i16::<NetworkEndian>()?;

        Tier::from_repr(tier_num).ok_or(anyhow!("tier outside allowed range, str: {tier_num}").into())
    }
}
impl TryFrom<&str> for Tier {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Prime" => Ok(Self::Prime),
            "1000" => Ok(Self::One),
            "2000" => Ok(Self::Two),
            "3000" => Ok(Self::Three),
            _ => Err(anyhow!("Couldn't convert from String to Tier"))
        }
    }
}

#[derive(Queryable, Debug, Clone)]
pub struct Resub {
    pub id: i32,
    pub uuid: Uuid,
    pub cumulative_month: i16,
    pub tier: Tier,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "resubs"]
pub struct NewResub {
    pub cumulative_month: i16,
    pub tier: Tier,
}
