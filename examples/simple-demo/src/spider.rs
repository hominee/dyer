pub mod entity;
pub mod middleware;
pub mod parser;
pub mod pipeline;

use dyer::dyer_macros::spider;
use dyer::{plug, App, ParseResult, ProfileInfo, Request, Response, Spider, Task};
use entity::{Entities, Parg, Targ};
use parser::*;

type Stem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;
type Btem<E, T, P> = dyn Fn(Response<T, P>) -> ParseResult<E, T, P>;

#[spider]
pub struct MySpider {}

impl Spider<Entities, Targ, Parg> for MySpider {
    // create an instance
    fn new() -> Self {
        MySpider {}
    }

    // preparation before opening spider
    fn open_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}

    // `Task` to be executed when starting `dyer`. Note that this function must reproduce a
    // non-empty vector, if not, the whole program will be left at blank.
    fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
        let mut task = Task::new();

        // all infomation needed is uri and parser
        task.uri = "https://quotes.toscrape.com".to_string();
        // parser is indexed by a `String` name, you can check that in the function `get_parser`.
        task.parser = "parse_quote".to_string();
        Ok(vec![task])
    }

    // the generator of `Profile`
    // `dyer` consume the returned `Request`, generate a `Response` fed to the closure
    // to generate a `Profile`
    fn entry_profile<'a>(&self) -> ProfileInfo<'a, Targ, Parg> {
        let mut req = Request::<Targ, Parg>::new();
        req.task.uri = "https://quotes.toscrape.com".to_string();
        ProfileInfo {
            req: Some(req),
            parser: None,
        }
    }

    // set up parser that extracts `Entities` from the `Response`
    // by the name of Task.parser return the parser function
    //parser is indexed by a `String` name, like:
    //task.parser = "parse_quote".to_string();
    fn get_parser<'a>(&self, ind: &str) -> Option<&'a Btem<Entities, Targ, Parg>> {
        // specify the parser here, like:
        // plug!(get_parser(<+input-string-as-index+>; <+parse_func_0+>, <parse_func_1>, ...))
        plug!(get_parser(ind; parse_quote))
    }

    // preparation before closing spider
    fn close_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}
}
