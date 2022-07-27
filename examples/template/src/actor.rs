pub mod affix;
pub mod entity;
pub mod middleware;
pub mod parser;
pub mod pipeline;

use affix::*;
use dyer::*;
use entity::*;
use parser::*;

// attribute #[dyer::actor] mark the struct and use it as a type implemented trait `Actor`
#[dyer::actor]
pub struct MyActor {
    pub start_uri: String,
}

#[dyer::async_trait]
impl Actor<Entities, Aff> for MyActor {
    // create an instance
    async fn new() -> Self {
        MyActor {
            start_uri: "https://example.com/some/path/to/site".into(),
        }
    }

    // preparation before opening actor
    async fn open_actor(&mut self, _app: &mut App<Entities>) {}

    /*
     * `Task` to be executed when starting `dyer`. Note that this function must reproduce a
     * non-empty vector, if not, the whole program will be left at blank.
     */
    async fn entry_task(&mut self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let task = Task::get(&self.start_uri)
            .parser(parse_func)
            // here the marker `MyActor` is the same as
            // the type implemented trait `Acotr`
            // change it if you like as long as it is unique
            .body(Body::empty(), "MyActor")
            .unwrap();
        Ok(vec![task])
    }

    /* the generator of `Affix`
     * `dyer` consume the returned `Request`, generate a `Response` fed to the closure
     * to generate a `Affix`
     */
    async fn entry_affix(&mut self) -> Option<Aff> {
        None
    }

    // preparation before closing actor
    async fn close_actor(&mut self, _app: &mut App<Entities>) {}
}
