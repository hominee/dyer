use crate::schema::*;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    mysql::Mysql as DB_mysql, // Mysql
    pg::Pg as DB_pg,          // Postgresql
    serialize::{self, IsNull, Output, ToSql},
    sqlite::Sqlite as DB_sqlite, // Sqlite
    Insertable,
    Queryable,
};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[dyer::entity]
#[derive(Serialize, Debug, Clone)]
pub enum Entities {
    Quote(Quote),
}

#[derive(Deserialize, Serialize, Insertable, Queryable, Debug, Clone)]
#[table_name = "quotes"]
pub struct Quote {
    pub id: i64,
    pub role: Roles,
    pub text: String,
    pub author: String,
    pub tags: Option<Tags>,
}

#[derive(Deserialize, AsExpression, FromSqlRow, Serialize, Debug, Clone)]
#[sql_type = "crate::schema::sql_types::Tags"]
pub struct Tags(pub Vec<String>);

impl ToSql<crate::schema::sql_types::Tags, DB_sqlite> for Tags {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB_sqlite>) -> serialize::Result {
        let r = serde_json::to_string(self);
        if let Ok(byte) = r {
            out.write_all(byte.as_bytes())?;
            Ok(IsNull::No)
        } else {
            out.write_all(b"")?;
            Ok(IsNull::Yes)
        }
    }
}
impl FromSql<crate::schema::sql_types::Tags, DB_sqlite> for Tags {
    fn from_sql(bytes: Option<&<DB_sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let Some(byte) = bytes {
            let r = serde_json::from_slice::<Tags>(byte.read_blob());
            if let Ok(item) = r {
                return Ok(item);
            }
        }
        Err("Invalid string and unable to convert into Tags".into())
    }
}

impl ToSql<crate::schema::sql_types::Tags, DB_pg> for Tags {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB_pg>) -> serialize::Result {
        let r = serde_json::to_string(self);
        if let Ok(byte) = r {
            out.write_all(byte.as_bytes())?;
            Ok(IsNull::No)
        } else {
            out.write_all(b"")?;
            Ok(IsNull::Yes)
        }
    }
}
impl FromSql<crate::schema::sql_types::Tags, DB_pg> for Tags {
    fn from_sql(bytes: Option<&<DB_pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let Some(byte) = bytes {
            let r = serde_json::from_slice::<Tags>(byte);
            if let Ok(item) = r {
                return Ok(item);
            }
        }
        Err("Invalid string and unable to convert into Tags".into())
    }
}

impl ToSql<crate::schema::sql_types::Tags, DB_mysql> for Tags {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB_mysql>) -> serialize::Result {
        let r = serde_json::to_string(self);
        if let Ok(byte) = r {
            out.write_all(byte.as_bytes())?;
            Ok(IsNull::No)
        } else {
            out.write_all(b"")?;
            Ok(IsNull::Yes)
        }
    }
}
impl FromSql<crate::schema::sql_types::Tags, DB_mysql> for Tags {
    fn from_sql(bytes: Option<&<DB_mysql as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let Some(byte) = bytes {
            let r = serde_json::from_slice::<Tags>(byte);
            if let Ok(item) = r {
                return Ok(item);
            }
        }
        Err("Invalid string and unable to convert into Tags".into())
    }
}

#[derive(Deserialize, AsExpression, FromSqlRow, Serialize, Debug, Clone)]
#[sql_type = "crate::schema::sql_types::Roles"]
pub enum Roles {
    Long,
    Short,
}

impl ToSql<crate::schema::sql_types::Roles, DB_sqlite> for Roles {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB_sqlite>) -> serialize::Result {
        match *self {
            Roles::Long => out.write_all(b"Long")?,
            Roles::Short => out.write_all(b"Short")?,
        }
        Ok(IsNull::No)
    }
}
impl FromSql<crate::schema::sql_types::Roles, DB_sqlite> for Roles {
    fn from_sql(bytes: Option<&<DB_sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        if let Some(byte) = bytes {
            match byte.read_blob() {
                b"long" => Ok(Roles::Long),
                b"short" => Ok(Roles::Short),
                b"Long" => Ok(Roles::Long),
                b"Short" => Ok(Roles::Short),
                _ => Err("Unexpected enum variant".into()),
            }
        } else {
            Err("Unexpected enum variant".into())
        }
    }
}

impl ToSql<crate::schema::sql_types::Roles, DB_pg> for Roles {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB_pg>) -> serialize::Result {
        match *self {
            Roles::Long => out.write_all(b"Long")?,
            Roles::Short => out.write_all(b"Short")?,
        }
        Ok(IsNull::No)
    }
}
impl FromSql<crate::schema::sql_types::Roles, DB_pg> for Roles {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match bytes {
            Some(b"long") => Ok(Roles::Long),
            Some(b"short") => Ok(Roles::Short),
            Some(b"Long") => Ok(Roles::Long),
            Some(b"Short") => Ok(Roles::Short),
            Some(_) | None => Err("Unexpected enum variant".into()),
        }
    }
}

impl ToSql<crate::schema::sql_types::Roles, DB_mysql> for Roles {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB_mysql>) -> serialize::Result {
        match *self {
            Roles::Long => out.write_all(b"Long")?,
            Roles::Short => out.write_all(b"Short")?,
        }
        Ok(IsNull::No)
    }
}
impl FromSql<crate::schema::sql_types::Roles, DB_mysql> for Roles {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match bytes {
            Some(b"long") => Ok(Roles::Long),
            Some(b"short") => Ok(Roles::Short),
            Some(b"Long") => Ok(Roles::Long),
            Some(b"Short") => Ok(Roles::Short),
            Some(_) | None => Err("Unexpected enum variant".into()),
        }
    }
}
