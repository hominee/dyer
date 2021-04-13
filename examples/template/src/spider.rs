// Set up initial condition when stepping into Spider and work to do when closing spider

use crate::entity::{Entities, Parg, Targ};
use crate::parser::*;
use dyer::{plug, App, ParseResult, ProfileInfo, Request, Response, Spider, Task};

type Stem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;
type Btem<E, T, P> = dyn Fn(Response<T, P>) -> ParseResult<E, T, P>;

pub struct MySpider {}

impl Spider<Entities, Targ, Parg> for MySpider {
    // preparation before opening spider
    fn open_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}

    // `Task` to be executed when starting `dyer`. Note that this function must reproduce a
    // non-empty vector, if not, the whole program will be left at blank.
    fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
        Ok(vec![])
    }

    // the generator of `Profile`
    // `dyer` consume the returned `Request`, generate a `Response` fed to the closure
    // to generate a `Profile`
    fn entry_profile<'a>(&self) -> ProfileInfo<'a, Targ, Parg> {
        ProfileInfo {
            req: Request::<Targ, Parg>::default(),
            parser: None,
        }
    }

    // set up parser that extracts `Entities` from the `Response`
    // by the name of Task.parser return the parser function
    //parser is indexed by a `String` name, like:
    //task.parser = "parse_quote".to_string();
    fn get_parser<'a>(&self, ind: String) -> Option<&'a Btem<Entities, Targ, Parg>> {
        plug!(get_parser(ind; parse_index))
    }

    // preparation before closing spider
    fn close_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}
}