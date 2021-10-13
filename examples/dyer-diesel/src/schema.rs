pub mod sql_types {

    #[derive(SqlType)]
    #[postgres(type_name = "Roles")]
    #[mysql_type = "String"]
    #[sqlite_type = "Text"]
    pub struct Roles;

    #[derive(SqlType)]
    #[postgres(type_name = "Tags")]
    #[mysql_type = "String"]
    #[sqlite_type = "Text"]
    pub struct Tags;
}

table! {
    use crate::schema::sql_types::*;
    use diesel::sql_types::*;

    quotes (id) {
        id -> Bigint,
        role -> Roles,
        text -> Text,
        author -> Text,
        tags -> Nullable<Tags>,
    }
}
