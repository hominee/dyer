use crate::component::{ParseResult, Profile, Request, ResError, Response, Task};
use crate::engine::App;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

type Sitem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;
type Parse<'life, U, T, P> = &'life dyn Fn(Response<T, P>) -> ParseResult<U, T, P>;
type Bitem<'a, P> = BoxFuture<'a, Result<Profile<P>, ResError>>;

/// infomation returned by `entry_profile` to consume `Request` and get a `Profile` via parser
pub struct ProfileInfo<'b, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
    pub req: Request<T, P>,
    pub parser: Option<&'b (dyn Fn(Response<T, P>) -> Bitem<'b, P> + Send + Sync)>,
}

/// the core of `Dyer`, that drives almost the events of data flow, including dispathing parser to
/// parse `Response`, generating `Profile`,
/// generating `Task`, preparation before opening spider, affairs before closing spider.  
pub trait Spider<U, T, P>: Send + Sync
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    U: Serialize + std::fmt::Debug + Clone,
{
    /// method to generate `Profile` throughout the whole program
    fn entry_profile<'a>(&self) -> ProfileInfo<'a, T, P>;

    /// method to generate `Task` when open `Spider`
    fn entry_task(&self) -> Sitem<Vec<Task<T>>>;

    /// preparation before enter `Spider`
    fn open_spider(&self, app: &mut App<U, T, P>);

    /// preparation before close `Spider`
    fn close_spider(&self, app: &mut App<U, T, P>);

    /// obtain parse throght ind
    fn get_parser<'a>(&self, ind: String) -> Option<Parse<'a, U, T, P>>;
}
