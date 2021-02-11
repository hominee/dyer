extern crate typed_builder;

use crate::component::{Profile, Request, Response, Task};
use crate::engine::AppArg;
use futures::future::{FutureExt, LocalBoxFuture};
use std::sync::{Arc, Mutex};
use typed_builder::TypedBuilder;

/// default method for process `Profile` in `MiddleWare`
pub async fn hprofile<P>(_files: &mut Vec<Profile<P>>, _arg: Arc<Mutex<AppArg>>)
where
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
}

/// default method for process `Task` in `MiddleWare`
pub async fn htask<T>(_tasks: &mut Vec<Task<T>>, _arg: Arc<Mutex<AppArg>>)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
{
}

/// default method for process `Request` in `MiddleWare`
pub async fn hreq<T, P>(
    _reqs: &mut Vec<Request<T, P>>,
    _arg: Arc<Mutex<AppArg>>,
) -> (Vec<Task<T>>, Vec<Profile<P>>)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
    (vec![], vec![])
}

/// default method for process `Response` in `MiddleWare`
pub async fn hres<T, P>(_res: &mut Vec<Response<T, P>>, _arg: Arc<Mutex<AppArg>>)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
}

/// default method for process `Item` in `MiddleWare`
pub async fn hitem<U>(_items: &mut Vec<U>, _arg: Arc<Mutex<AppArg>>)
where
    U: Send,
{
}

/// default method for process failed `Response` in `MiddleWare`
pub async fn herr<T, P>(
    _res: &mut Vec<Response<T, P>>,
    _arg: Arc<Mutex<AppArg>>,
) -> (
    Vec<Task<T>>,
    Vec<Profile<P>>,
    Vec<Request<T, P>>,
    Vec<String>,
    bool,
)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
    let mut tasks = Vec::new();
    let mut profiles = Vec::new();
    let reqs = Vec::new();
    let mut yerrs = Vec::new();
    let redirect = false;
    while let Some(mut res) = _res.pop() {
        log::error!("response error: {}, uri: {}", res.status, res.task.uri);
        if res.task.trys >= 1 {
            let yield_err = format!(
                "status: {}\turi: {}\tcontent: {}",
                &res.status,
                &res.task.uri,
                res.content.as_ref().unwrap_or(&"".to_string())
            );
            log::error!("this task fails 3+ times. drop it.");
            profiles.push(res.profile);
            yerrs.push(yield_err);
        } else {
            log::error!("{} times failure, reuse this task.", res.task.trys);
            res.task.trys += 1;
            tasks.push(res.task);
            profiles.push(res.profile);
        }
    }
    (tasks, profiles, reqs, yerrs, redirect)
}

/// default method for failing parsing `Response` in `MiddleWare`
pub async fn hyerr<T, P>(_res: &mut Vec<Response<T, P>>, _arg: Arc<Mutex<AppArg>>)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone,
{
}

/// plugin that process data flow in and out of `Spider` between component, each member has a
/// default method corresponding to the most common cases. Customization is easy
/// ```
/// async fn hand_item<U>(items: &mut Vec<U>, _arg: Arc<Mutex<AppArg>>) where U: Send {
///     println!("process {} items", items.len());
/// }
/// let middleware = MiddleWare::builder().hand_item(&|items: &mut Vec<U>, arg: Arc<Mutex<AppArg>>| hand_item(items, arg).boxed_local() ).build().unwrap();
/// ```
/// the member that has been specified is assigned to the default method.
#[derive(TypedBuilder)]
pub struct MiddleWare<'md, U, T, P>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send + 'md,
    P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + 'md,
    U: Send + 'md,
{
    #[builder(
        default_code = r#" &|profiles: &mut Vec<Profile<P>>, arg: Arc<Mutex<AppArg>>| hprofile(profiles, arg).boxed_local() "#
    )]
    pub hand_profile:
        &'md dyn for<'a> Fn(&'a mut Vec<Profile<P>>, Arc<Mutex<AppArg>>) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#"&| tasks: &mut Vec<Task<T>>, arg: Arc<Mutex<AppArg>>| htask(tasks, arg).boxed_local() "#
    )]
    pub hand_task:
        &'md dyn for<'a> Fn(&'a mut Vec<Task<T>>, Arc<Mutex<AppArg>>) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#" &| req: &mut Vec<Request<T, P>>, arg: Arc<Mutex<AppArg>>| hreq(req, arg).boxed_local() "#
    )]
    pub hand_req: &'md dyn for<'a> Fn(
        &'a mut Vec<Request<T, P>>,
        Arc<Mutex<AppArg>>,
    ) -> LocalBoxFuture<'a, (Vec<Task<T>>, Vec<Profile<P>>)>,

    #[builder(
        default_code = r#"&|responses: &mut Vec<Response<T, P>>, arg: Arc<Mutex<AppArg>>| hres(responses, arg).boxed_local() "#
    )]
    pub hand_res: &'md dyn for<'a> Fn(
        &'a mut Vec<Response<T, P>>,
        Arc<Mutex<AppArg>>,
    ) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#"&|items: &mut Vec<U>, arg: Arc<Mutex<AppArg>>| hitem(items, arg).boxed_local() "#
    )]
    pub hand_item:
        &'md dyn for<'a> Fn(&'a mut Vec<U>, Arc<Mutex<AppArg>>) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#"&|yerrs: &mut Vec<Response<T, P>>, arg: Arc<Mutex<AppArg>>| hyerr(yerrs, arg).boxed_local() "#
    )]
    pub hand_yerr: &'md dyn for<'a> Fn(
        &'a mut Vec<Response<T, P>>,
        Arc<Mutex<AppArg>>,
    ) -> LocalBoxFuture<'a, ()>,

    #[builder(
        default_code = r#" &| mut res: &mut Vec<Response<T, P>>, arg: Arc<Mutex<AppArg>>| herr(res, arg).boxed_local() "#
    )]
    pub hand_err: &'md dyn for<'a> Fn(
        &'a mut Vec<Response<T, P>>,
        Arc<Mutex<AppArg>>,
    ) -> LocalBoxFuture<
        'a,
        (
            Vec<Task<T>>,
            Vec<Profile<P>>,
            Vec<Request<T, P>>,
            Vec<String>,
            bool,
        ),
    >,
}
