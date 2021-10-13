extern crate crawler;
extern crate diesel;
extern crate dotenv;
extern crate dyer;
extern crate select;
extern crate serde;
extern crate simple_logger;
extern crate tokio;

use diesel::mysql::MysqlConnection as Conn_mysql;
use diesel::pg::PgConnection as Conn_pg;
use diesel::sqlite::SqliteConnection as Conn_sqlite;

use crawler::entity::{Entities, Parg, Targ};
use crawler::middleware::handle_entities;
use crawler::pipeline::{establish_connection, store_quote};
use crawler::MySpider;
use dyer::*;
use std::sync::{Arc, Mutex};

type Conn = (Conn_sqlite, Conn_pg, Conn_mysql);

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let middleware = plug!(
        MiddleWare < Entities,
        Targ,
        Parg > { handle_entity: handle_entities }
    );
    let pipeline = plug!( PipeLine<Entities, Conn> {
        open_pipeline: establish_connection,
        process_entity: store_quote
    } );
    let spider = MySpider::new();
    let mut app = dyer::App::<Entities, Targ, Parg>::new();
    app.run(&spider, &middleware, pipeline).await.unwrap();
}
