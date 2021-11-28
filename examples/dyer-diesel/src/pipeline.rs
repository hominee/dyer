use crate::entity::*;
use diesel::mysql::MysqlConnection as Conn_mysql;
use diesel::pg::PgConnection as Conn_pg;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection as Conn_sqlite;
use dotenv::dotenv;
use dyer::*;
use std::env;
use std::sync::Once;

type Conn = (Conn_sqlite, Conn_pg, Conn_mysql);

#[dyer::pipeline]
pub async fn establish_connection(_app: &mut App<Entities>) -> Option<&'static Conn> {
    static INIT: Once = Once::new();
    static mut VAL: Option<Conn> = None;

    unsafe {
        INIT.call_once(|| {
            dotenv().ok();

            let database_url_mysql =
                env::var("DATABASE_URL_MYSQL").expect("DATABASE_URL_MYSQL URL cannot be null");
            let database_url_pg =
                env::var("DATABASE_URL_PG").expect("DATABASE_URL_PG URL cannot be null");
            let database_url_sqlite =
                env::var("DATABASE_URL_SQLITE").expect("DATABASE_URL_SQLITE URL cannot be null");
            VAL = Some((
                Conn_sqlite::establish(&database_url_sqlite)
                    .expect(&format!("error connectin to {}", database_url_sqlite)),
                Conn_pg::establish(&database_url_pg)
                    .expect(&format!("error connectin to {}", database_url_pg)),
                Conn_mysql::establish(&database_url_mysql)
                    .expect(&format!("error connectin to {}", database_url_mysql)),
            ));
        });
        VAL.as_ref()
    }
}

#[dyer::pipeline]
pub async fn store_quote(mut items: Vec<Entities>, _app: &mut App<Entities>) {
    use crate::schema::quotes::dsl::*;

    let conn = establish_connection(_app).await.unwrap();
    while let Some(Entities::Quote(item)) = items.pop() {
        // sqlite
        diesel::insert_into(quotes)
            .values(&item)
            .execute(&conn.0)
            .map_or_else(
                |e| println!("Inserting into sqlite database: {:?}", e.to_string()),
                |_| {},
            );
        // postgres
        diesel::insert_into(quotes)
            .values(&item)
            .get_result::<Quote>(&conn.1)
            .map_or_else(
                |e| println!("Inserting into Postgresql database: {:?}", e.to_string()),
                |_| {},
            );
        // mysql
        diesel::insert_into(quotes)
            .values(&item)
            .execute(&conn.2)
            .map_or_else(
                |e| println!("Inserting into Mysql database: {:?}", e.to_string()),
                |_| {},
            );
    }
}
