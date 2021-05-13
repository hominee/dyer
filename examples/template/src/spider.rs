use crate::entity::{Entities, Parg, Targ};
use crate::parser::*;
use dyer::*;
use dyer::dyer_macros::spider;

type Stem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;
type Btem<E, T, P> = dyn Fn(Response<T, P>) -> ParseResult<E, T, P>;

// attribute #[spider] mark the struct and use it as a type implemented trait `Spider`
#[spider]
pub struct MySpider {
    pub start_uri: String,
}

impl Spider<Entities, Targ, Parg> for MySpider {
    // create an instance 
    fn new() -> Self {
        MySpider{
            start_uri: "https://example.com/some/path/to/site".into()
        }
    }

    // preparation before opening spider
    fn open_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}

    /* 
     * `Task` to be executed when starting `dyer`. Note that this function must reproduce a
     * non-empty vector, if not, the whole program will be left at blank.
     */
    fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
        let mut task = Task::new();
        task.uri = self.start_uri.clone();
        Ok(vec![task])
    }

    /* the generator of `Profile`
     * `dyer` consume the returned `Request`, generate a `Response` fed to the closure
     * to generate a `Profile`
     */
    fn entry_profile<'a>(&self) -> ProfileInfo<'a, Targ, Parg> {
        ProfileInfo {
            req: Some( Request::<Targ, Parg>::new() ),
            parser: None,
        }
    }

    /* set up parser that extracts `Entities` from the `Response`
     * by the name of Task.parser return the parser function
     * parser is indexed by a `String` name, like:
     * task.parser = "parse_quote".to_string();
     */
    fn get_parser<'a>(&self, ind: &str) -> Option<&'a Btem<Entities, Targ, Parg>> {
        plug!(get_parser(ind; parse_func))
    }

    // preparation before closing spider
    fn close_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}
}