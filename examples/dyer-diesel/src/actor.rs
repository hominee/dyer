#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod affix;
pub mod entity;
pub mod middleware;
pub mod parser;
pub mod pipeline;
pub mod schema;

use affix::*;
use dyer::{Actor, *};
use entity::*;
use parser::*;
use pipeline::*;

#[dyer::actor]
pub struct MyActor {
    start_url: String,
}

#[dyer::async_trait]
impl Actor<Entities, Aff> for MyActor {
    async fn new() -> Self {
        MyActor {
            start_url: "https://quotes.toscrape.com".into(),
        }
    }

    async fn open_actor(&mut self, _app: &mut App<Entities>) {}

    async fn entry_task(&mut self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let task = Task::get(&self.start_url)
            .parser(parse_quote)
            .body(Body::empty(), "quote")
            .unwrap();
        Ok(vec![task])
    }

    async fn entry_affix(&mut self) -> Option<Aff> {
        None
    }

    async fn close_actor(&mut self, _app: &mut App<Entities>) {
        use crate::entity::Quote;
        use crate::schema::quotes::dsl::*;
        use diesel::prelude::*;

        let conn = establish_connection(_app).await;
        quotes
            .filter(author.eq("Mark Twain"))
            .limit(7)
            .load::<Quote>(&conn.as_ref().unwrap().0)
            .expect("Error loading quotes")
            .iter()
            .for_each(|r| {
                println!(
                    "{}\t{}\t{:?}\t{:?}",
                    r.id,
                    r.author,
                    r.role,
                    r.tags.as_ref().unwrap().0
                );
                println!("{}", r.text);
                println!("===========Load-From-Sqlite=============");
            });
        quotes
            .filter(author.eq("Mark Twain"))
            .limit(7)
            .load::<Quote>(&conn.as_ref().unwrap().1)
            .expect("Error loading quotes")
            .iter()
            .for_each(|r| {
                println!(
                    "{}\t{}\t{:?}\t{:?}",
                    r.id,
                    r.author,
                    r.role,
                    r.tags.as_ref().unwrap().0
                );
                println!("{}", r.text);
                println!("===========Load-From-PostgreSQL=============");
            });
        quotes
            .filter(author.eq("Mark Twain"))
            .limit(7)
            .load::<Quote>(&conn.as_ref().unwrap().2)
            .expect("Error loading quotes")
            .iter()
            .for_each(|r| {
                println!(
                    "{}\t{}\t{:?}\t{:?}",
                    r.id,
                    r.author,
                    r.role,
                    r.tags.as_ref().unwrap().0
                );
                println!("{}", r.text);
                println!("===========Load-From-Mysql=============");
            });
    }
}
