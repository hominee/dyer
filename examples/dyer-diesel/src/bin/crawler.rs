extern crate crawler;
extern crate diesel;
extern crate dotenv;
extern crate dyer;
extern crate serde;
extern crate simple_logger;
extern crate tokio;

use diesel::mysql::MysqlConnection as Conn_mysql;
use diesel::pg::PgConnection as Conn_pg;
use diesel::sqlite::SqliteConnection as Conn_sqlite;

use crawler::entity::*;
use crawler::middleware::handle_entities;
use crawler::pipeline::{establish_connection, store_quote};
use crawler::MyActor;
use dyer::*;

pub type Conn = (Conn_sqlite, Conn_pg, Conn_mysql);

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let middleware = MiddleWare::<Entities>::builder()
        .entity(&handle_entities)
        .build("quote");
    let pipeline = PipeLine::<Entities, &'static Conn>::builder()
        .initializer(&establish_connection)
        .entity(&store_quote)
        .build("quote");
    let mut actor = MyActor::new().await;
    let mut app = dyer::App::<Entities>::new();
    app.run(&mut actor, &middleware, &pipeline).await.unwrap();
}
