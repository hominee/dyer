pub mod affix;
pub mod entity;
pub mod middleware;
pub mod parser;
pub mod pipeline;

use affix::*;
use dyer::*;

use entity::Entities;
use parser::*;

#[actor]
pub struct MyActor {}

#[dyer::async_trait]
impl Actor<Entities, Aff> for MyActor {
    // create an instance
    async fn new() -> Self {
        Self {}
    }

    // preparation before opening actor
    async fn open_actor(&mut self, _app: &mut App<Entities>) {}

    // `Task` to be executed when starting `dyer`. Note that this function must reproduce a
    // non-empty vector, if not, the whole program will be left at blank.
    async fn entry_task(&mut self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        // all infomation needed is uri and parser
        let task = Task::get("https://quotes.toscrape.com")
            .parser(parse_quote)
            //.proxy("http://127.0.0.1:1080") // require feature `proxy` enabled
            .extensions(3i32)
            .body(Body::empty(), "quote")
            .unwrap();
        println!("{:?}", task);
        Ok(vec![task])
    }

    // the generator of `Affix`
    // `dyer` consume the returned `Request`, generate a `Response` fed to the closure
    // to generate a `Affix`
    async fn entry_affix(&mut self) -> Option<Aff> {
        None
    }

    // preparation before closing actor
    async fn close_actor(&mut self, _app: &mut App<Entities>) {}
}
