extern crate typed_builder;

use crate::component::{Profile, Request, Response, Task};
use crate::engine::App;
use futures::future::{FutureExt, LocalBoxFuture};
use typed_builder::TypedBuilder;

/// default method for process `Profile` in `MiddleWare`
async fn hprofile<E, T, P>(_profiles: &mut Vec<Profile<P>>, _app: &mut App<E, T, P>)
where
    E: Send,
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
}

/// default method for process `Task` in `MiddleWare`
async fn htask<E, T, P>(_tasks: &mut Vec<Task<T>>, _app: &mut App<E, T, P>)
where
    E: Send,
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
}

/// default method for process `Request` in `MiddleWare`
async fn hreq<E, T, P>(
    _reqs: &mut Vec<Request<T, P>>,
    _app: &mut App<E, T, P>,
) -> (Vec<Task<T>>, Vec<Profile<P>>)
where
    E: Send,
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
    (vec![], vec![])
}

/// default method for process `Response` in `MiddleWare`
async fn hres<E, T, P>(_res: &mut Vec<Response<T, P>>, _app: &mut App<E, T, P>)
where
    E: Send,
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
}

/// default method for process `Item` in `MiddleWare`
async fn hitem<E, T, P>(_items: &mut Vec<E>, _app: &mut App<E, T, P>)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
    E: Send,
{
}

/// default method for process failed `Response` in `MiddleWare`
async fn herr<E, T, P>(_res: &mut Vec<Response<T, P>>, _app: &mut App<E, T, P>)
where
    E: Send,
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
    let mut tasks = Vec::new();
    let mut profiles = Vec::new();
    let reqs = Vec::new();
    let mut yerrs = Vec::new();
    while let Some(mut res) = _res.pop() {
        log::error!("response error: {}, uri: {}", res.status, res.task.uri);
        if res.task.trys >= 1 {
            let yield_err = format!(
                "status: {}\turi: {}\tcontent: {}\n",
                &res.status,
                &res.task.uri,
                res.content.as_ref().unwrap_or(&"".to_string())
            );
            log::error!("this task fails 3+ times. drop it.");
            if let Some(profile) = res.profile {
                profiles.push(profile);
            }
            yerrs.push(yield_err);
        } else {
            log::error!("{} times failure, reuse this task.", res.task.trys);
            res.task.trys += 1;
            tasks.push(res.task);
            if let Some(profile) = res.profile {
                profiles.push(profile);
            }
        }
    }
    _app.task.lock().unwrap().extend(tasks);
    _app.profile.lock().unwrap().extend(profiles);
    _app.req.lock().unwrap().extend(reqs);
    _app.yield_err.lock().unwrap().extend(yerrs);
}

/// default method for failing parsing `Response` in `MiddleWare`
async fn hyerr<E, T, P>(_res: &mut Vec<Response<T, P>>, _app: &mut App<E, T, P>)
where
    E: Send,
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
}

/// plugin that process data flow in and out of `Spider` between component, each member has a
/// default method corresponding to the most common cases. Customization is easy
/// ```compile_fail
/// use futures::future::FutureExt;
/// # use crate::dyer::component::{Profile, Request, Response, Task};
/// # use crate::dyer::{plug, MiddleWare};
/// # use crate::dyer::engine::App;
/// # use serde::{Deserialize, Serialize};
/// # #[derive(std::fmt::Debug)]
/// # pub struct E;
/// # unsafe impl Send for E {}
/// # unsafe impl Sync for E {}
/// # #[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
/// # pub struct T;
/// # unsafe impl Send for T {}
/// # #[derive(std::fmt::Debug, Serialize, Deserialize, Clone)]
/// # pub struct P;
/// # async fn handle_profile(_profiles: &mut Vec<Profile<P>>, _app: &mut App<E, T, P>) {}
/// # async fn handle_task(_tasks: &mut Vec<Task<T>>, _app: &mut App<E, T, P>) {}
/// # async fn handle_req(
/// #     _reqs: &mut Vec<Request<T, P>>,
/// #     _app: &mut App<E, T, P>,
/// # ) -> (Vec<Task<T>>, Vec<Profile<P>>) {
/// #     (vec![], vec![])
/// # }
/// # async fn handle_res(_res: &mut Vec<Response<T, P>>, _app: &mut App<E, T, P>) {}
/// # async fn handle_item(_items: &mut Vec<E>, _app: &mut App<E, T, P>) {}
/// # async fn handle_err(_res: &mut Vec<Response<T, P>>, _app: &mut App<E, T, P>) {}
/// # async fn handle_yerr(_res: &mut Vec<Response<T, P>>, _app: &mut App<E, T, P>) {}
///
/// // the recommanded way to initialize a `MiddleWare` by means of macro `plug!`
/// plug!(MiddleWare<E, T, P> {
///         handle_profile: handle_profile,
///         handle_task: handle_task,
///         handle_req: handle_req,
///         handle_res: handle_res,
///         handle_item: handle_item
///         handle_err: handle_err
///         //handle_yerr: handle_yerr
///     }
/// );
///
/// // the the second way is also at ease
/// let mut md = MiddleWare::<E, T, P>::builder().build();
/// md.handle_yerr = &|_yerrs: &mut Vec<Response<T, P>>, _app: &mut App<E, T, P>| {
///     handle_yerr(_yerrs, _app).boxed_local()
/// };
///
/// // the traditional way is supported as well
/// // but remeber that initializations of all members are required
/// MiddleWare::<E, T, P> {
///     handle_task: &|_tasks: &mut Vec<Task<T>>, _app: &mut App<E, T, P>| {
///         handle_task(_tasks, _app).boxed_local()
///     },
///     handle_item: &|_items: &mut Vec<E>, _app: &mut App<E, T, P>| {
///         handle_item(_items, _app).boxed_local()
///     },
///     ...
/// };
/// ```
/// the member that has not been specified is assigned to the default method.
#[derive(TypedBuilder)]
pub struct MiddleWare<'md, E, T, P>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send + 'md,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + 'md,
    E: Send + 'md,
{
    #[builder(
        default_code = r#" &|profiles: &mut Vec<Profile<P>>, app: &mut App<E, T, P>| hprofile(profiles, app).boxed_local() "#
    )]
    pub handle_profile: &'md dyn for<'a> Fn(
        &'a mut Vec<Profile<P>>,
        &'a mut App<E, T, P>,
    ) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#"&| tasks: &mut Vec<Task<T>>, app: &mut App<E, T, P> | htask(tasks, app).boxed_local() "#
    )]
    pub handle_task:
        &'md dyn for<'a> Fn(&'a mut Vec<Task<T>>, &'a mut App<E, T, P>) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#" &| req: &mut Vec<Request<T, P>>, app: &mut App<E, T, P>| hreq(req, app).boxed_local() "#
    )]
    pub handle_req: &'md dyn for<'a> Fn(
        &'a mut Vec<Request<T, P>>,
        &'a mut App<E, T, P>,
    ) -> LocalBoxFuture<'a, (Vec<Task<T>>, Vec<Profile<P>>)>,

    #[builder(
        default_code = r#"&|responses: &mut Vec<Response<T, P>>, app: &mut App<E, T, P>| hres(responses, app).boxed_local() "#
    )]
    pub handle_res: &'md dyn for<'a> Fn(
        &'a mut Vec<Response<T, P>>,
        &'a mut App<E, T, P>,
    ) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#"&|items: &mut Vec<E>, app: &mut App<E, T, P>| hitem(items, app).boxed_local() "#
    )]
    pub handle_entity:
        &'md dyn for<'a> Fn(&'a mut Vec<E>, &'a mut App<E, T, P>) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#"&|yerrs: &mut Vec<Response<T, P>>, app: &mut App<E, T, P>| hyerr(yerrs, app).boxed_local() "#
    )]
    pub handle_yerr: &'md dyn for<'a> Fn(
        &'a mut Vec<Response<T, P>>,
        &'a mut App<E, T, P>,
    ) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#" &| mut res: &mut Vec<Response<T, P>>, app: &mut App<E, T, P>| herr(res, app).boxed_local() "#
    )]
    pub handle_err: &'md dyn for<'a> Fn(
        &'a mut Vec<Response<T, P>>,
        &'a mut App<E, T, P>,
    ) -> LocalBoxFuture<'a, ()>,
}
