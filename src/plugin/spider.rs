use crate::component::{ParseResult, Profile, Request, ResError, Response, Task};
use crate::engine::App;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

type Sitem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;

/// the core of `Dyer`, that drives almost the events of data flow, including dispathing parser to
/// parse `Response`, generating `Profile`,
/// generating `Task`, preparation before opening spider, affairs before closing spider.  
//#[async_trait]
pub trait Spider<U, T, P>: Send + Sync
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    U: Serialize + std::fmt::Debug + Clone,
{
    /// method to generate `Profile` throughout the whole program
    fn entry_profile(
        &self,
    ) -> (
        Request<T, P>,
        Option<
            &(dyn Fn(&mut Response<T, P>) -> BoxFuture<'_, Result<Profile<P>, ResError>>
                  + Send
                  + Sync),
        >,
    );

    /// method to generate `Task` when open `Spider`
    fn entry_task(&self) -> Sitem<Vec<Task<T>>>;

    /// preparation before enter `Spider`
    fn open_spider(&self, app: &mut App<U, T, P>);

    /// preparation before close `Spider`
    fn close_spider(&self, app: &mut App<U, T, P>);

    /// obtain parse throght ind
    fn get_parser<'a>(
        &self,
        ind: String,
    ) -> Option<&'a (dyn Fn(Response<T, P>) -> ParseResult<U, T, P>)>;
}
