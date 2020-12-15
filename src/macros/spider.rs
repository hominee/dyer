use crate::component::ParseResult;
use crate::component::{ Response, Task,}; 
use crate::engine::App;

type Sitem<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub trait Spider<T>: Send + Sync {
    fn entry_profile(&self) -> Sitem<&str>;
    fn entry_task(&self) -> Sitem<Vec<Task>>;
    fn open_spider( &self, app: &mut App<T>); 
    fn close_spider(&self, app: &mut App<T>);
    fn get_parser<'a>(&self, ind: &str) -> Option<&'a dyn Fn(&Response) -> Sitem<ParseResult<T>>>;
}
