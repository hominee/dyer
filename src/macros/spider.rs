use crate::component::{ParseResult, Response, Task};
use crate::engine::App;
use serde::{Deserialize, Serialize};

type Sitem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;

pub trait Spider<U, T, P, C>: Send + Sync
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    U: Serialize + std::fmt::Debug + Clone,
    C: Send,
{
    fn entry_profile(&self) -> Sitem<&str>;
    fn entry_task(&self) -> Sitem<Vec<Task<T>>>;
    fn open_spider(&self, app: &mut App<U, T, P, C>);
    fn close_spider(&self, app: &mut App<U, T, P, C>);
    fn get_parser<'a>(
        &self,
        ind: String,
    ) -> Option<&'a (dyn Fn(Response<T, P>) -> Sitem<ParseResult<U, T, P>> + Send + Sync)>;
}
