#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate select;

pub mod entity;
pub mod middleware;
pub mod parser;
pub mod pipeline;
pub mod schema;

use dyer::dyer_macros::spider;
use dyer::*;
use entity::{Entities, Parg, Targ};
use parser::*;

type Stem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;
type Btem<E, T, P> = dyn Fn(Response<T, P>) -> ParseResult<E, T, P>;

#[spider]
pub struct MySpider {
    start_url: String,
}

impl Spider<Entities, Targ, Parg> for MySpider {
    fn new() -> Self {
        MySpider {
            start_url: "https://quotes.toscrape.com".into(),
        }
    }

    fn open_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}

    fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
        let mut tsk = Task::new();
        tsk.uri = self.start_url.clone();
        tsk.parser = "parse_quote".to_string();
        Ok(vec![tsk])
    }

    fn entry_profile<'a>(&self) -> ProfileInfo<'a, Targ, Parg> {
        let mut req = Request::<Targ, Parg>::new();
        req.task.uri = "https://quotes.toscrape.com".to_string();
        ProfileInfo {
            req: Some(req),
            parser: None,
        }
    }

    fn get_parser<'a>(&self, ind: &str) -> Option<&'a Btem<Entities, Targ, Parg>> {
        plug!(get_parser(ind; parse_quote))
    }

    fn close_spider(&self, _app: &mut App<Entities, Targ, Parg>) {
        use crate::entity::Quote;
        use crate::schema::quotes::dsl::*;
        use diesel::prelude::*;

        let conn = dyer::Client::block_exec(crate::pipeline::establish_connection());
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
