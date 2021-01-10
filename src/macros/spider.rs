use crate::component::Task;
use crate::engine::{App, Elements};
use serde::{Deserialize, Serialize};

type Sitem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;

pub enum MethodIndex {
    GenProfile,
    RequestInit,
    GenRequest,
    String(String),
}

pub trait Spider<U, T, P>: Send + Sync
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    U: Serialize + std::fmt::Debug + Clone,
{
    fn entry_profile(&self) -> Sitem<&str>;
    fn entry_task(&self) -> Sitem<Vec<Task<T>>>;
    fn open_spider(&self, app: &mut App<U, T, P>);
    fn close_spider(&self, app: &mut App<U, T, P>);
    fn get_parser<'a>(
        &self,
        ind: MethodIndex,
    ) -> Option<&'a (dyn Fn(Elements<'a, U, T, P>) -> Sitem<Elements<'a, U, T, P>> + Send + Sync)>;
}
