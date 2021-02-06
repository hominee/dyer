use crate::component::{Profile, Request, Response, Task};
//use crate::engine::App;
use async_trait::async_trait;

#[async_trait]
pub trait MiddleWare<U, T, P, C>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
    C: Send,
    U: Send,
{
    ///handle status code non-[200, 299]
    ///if the error is not remendable then recycle the the Request
    ///into Task and Profile
    async fn hand_err(
        &self,
        res: Response<T, P>,
    ) -> Option<(
        Option<Vec<Task<T>>>,
        Option<Profile<P>>,
        Option<Request<T, P>>,
        Option<String>,
        bool,
    )>;

    ///handle extracted item from parser
    async fn hand_item(&self, items: &mut Vec<U>);

    ///handle task from parser
    async fn hand_task(&self, tasks: &mut Vec<Task<T>>);

    ///handle task from parser
    async fn hand_profile(&self, pfiles: &mut Profile<P>);

    ///handle constructed Request if necessary
    async fn hand_req(
        &self,
        req: Request<T, P>,
    ) -> (Option<Request<T, P>>, Option<Profile<P>>, Option<Task<T>>);

    ///handle downloader Response if necessary
    async fn hand_res(&self, res: &mut Response<T, P>);
}

///impl Default for object that implementes MiddleWare
///if user not manually impl MiddleWare, then this actively used
///basically, just do nothing except print out
pub struct MiddleWareDefault<U, T, P, C>
where
    C: Send,
{
    _u: std::marker::PhantomData<U>,
    _t: std::marker::PhantomData<T>,
    _p: std::marker::PhantomData<P>,
    _c: std::marker::PhantomData<C>,
}

impl<U, T, P, C> MiddleWareDefault<U, T, P, C>
where
    C: Send,
{
    pub fn new() -> Self {
        MiddleWareDefault {
            _u: std::marker::PhantomData::<U>,
            _t: std::marker::PhantomData::<T>,
            _p: std::marker::PhantomData::<P>,
            _c: std::marker::PhantomData::<C>,
        }
    }
}
unsafe impl<U, T, P, C> Send for MiddleWareDefault<U, T, P, C> where C: Send {}
unsafe impl<U, T, P, C> Sync for MiddleWareDefault<U, T, P, C> where C: Send {}

#[async_trait]
impl<U, T, P, C> MiddleWare<U, T, P, C> for MiddleWareDefault<U, T, P, C>
where
    T: std::fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send,
    P: std::fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
    U: Send,
    C: Send,
{
    async fn hand_err(
        &self,
        mut res: Response<T, P>,
    ) -> Option<(
        Option<Vec<Task<T>>>,
        Option<Profile<P>>,
        Option<Request<T, P>>,
        Option<String>,
        bool,
    )> {
        log::error!("response error: {}, uri: {}", res.status, res.task.uri);
        let redirect = false;
        if res.task.trys >= 1 {
            let yield_err = format!(
                "status: {}\turi: {}\tcontent: {}",
                &res.status,
                &res.task.uri,
                res.content.as_ref().unwrap_or(&"".to_string())
            );
            log::error!("this task fails 3+ times. drop it.");
            Some((None, Some(res.profile), None, Some(yield_err), redirect))
        } else {
            log::error!("{} times failure, reuse this task.", res.task.trys);
            res.task.trys += 1;
            Some((
                Some(vec![res.task]),
                Some(res.profile),
                None,
                None,
                redirect,
            ))
        }
    }

    async fn hand_item(&self, _items: &mut Vec<U>) {}

    async fn hand_task(&self, _tasks: &mut Vec<Task<T>>) {}

    async fn hand_profile(&self, _pfiles: &mut Profile<P>) {}

    async fn hand_req(
        &self,
        _req: Request<T, P>,
    ) -> (Option<Request<T, P>>, Option<Profile<P>>, Option<Task<T>>) {
        (Some(_req), None, None)
    }

    async fn hand_res(&self, _res: &mut Response<T, P>) {}
}
