use crate::component::ParseResult;
use crate::component::{Profile, Request, ResError, Response, Task, UserAgent};
use crate::engine::App;
use hyper::{Body, Request as hRequest};

type Sitem<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub trait Spider<T> {
    fn entry_profile(&self) -> Sitem<&str>;
    fn entry_task(&self) -> Sitem<Vec<Task>>;
    fn open_spider(
        &self,
    ) -> (
        Option<Vec<UserAgent>>,
        Option<Vec<Request>>,
        Option<Vec<Task>>,
        Option<Vec<Profile>>,
    );
    fn close_spider(&self, app: &mut App<T>);
    fn gen_profile(&self, res: hRequest<Body>) -> Result<Profile, ResError>;
    fn get_parser<'a>(&self, ind: &str) -> Option<&'a dyn Fn(&Response) -> Sitem<ParseResult<T>>>;
}

/*
 *#[derive(std::fmt::Debug)]
 *pub struct Mate { }
 *
 *pub trait MSpider {
 *    fn meta() -> &'static (Vec<&'static str>, Vec<&'static str>);
 *    fn methods<T>() -> &'static Vec<&'static Item<T>> ;
 *    fn get_parser<T>(ind: &str) -> Option<&'static dyn Fn(&T, &Response) -> Result<ParseResult, Box<dyn std::error::Error + Send + Sync>>>;
 *    fn map<T>() -> std::collections::HashMap<&'static str, &'static Item<T>>;
 *    fn fmap<T>(f: &&Item<T>) -> String where T: 'static ;
 *}
 */
