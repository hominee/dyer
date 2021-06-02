extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::component::{Profile,utils, Request, Task};
use crate::engine::{App, Spider};
use crate::plugin::MiddleWare;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// the parsed result returned by `parser`.
pub struct ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
    /// a vector of `Request`
    pub req: Vec<Request<T, P>>,
    /// a vector of `Task`
    pub task: Vec<Task<T>>,
    /// a vector of `Profile`
    pub profile: Vec<Profile<P>>,
    /// a vector of customized `Entity`
    pub entities: Vec<E>,
    /// a vector of record for failed `Response`, for the use of debug.
    pub yield_err: Vec<String>,
}

impl<E, T, P> ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
    pub fn new() -> Self {
        ParseResult {
            task: Vec::new(),
            profile: Vec::new(),
            req: Vec::new(),
            entities: Vec::new(),
            yield_err: Vec::new(),
        }
    }
}
unsafe impl<E, T, P> Sync for ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
}
unsafe impl<E, T, P> Send for ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
}

/// The result of a `Resquest`, returned by `Client`, contains `Task` and `Profile` which consist
/// of,
#[derive(Clone, Debug)]
pub struct Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
    /// `Task` that make this `Response`
    pub task: Task<T>,
    /// `Profile` that make this `Response`
    pub profile: Option<Profile<P>>,
    /// status code returned by the server
    pub status: usize,
    /// the content of this `Request`
    pub content: Option<String>,
    /// the headers of this `Response`, note that `HeaderName` such as `set-cookie` appears
    /// multiple times, is joined with `::`
    pub headers: HashMap<String, String>,
    /// error message returned by server or `dyner` if `Request` goes wrong
    pub msg: Option<String>,
}
unsafe impl<T, P> Sync for Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
}
unsafe impl<T, P> Send for Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
}

impl<'t, T, P> Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone + Send,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    /// create an instance from `Response` or `None`
    pub fn new(req: Option<&Request<T, P>>) -> Response<T, P> {
        let ( task, profile ) = match req {
            Some(r) => {
                ( r.task.clone(),  r.profile.clone() )
            }
            None => {
                 ( Task::new(), None, )
            }
        };
        Response {
            task: task,
            profile: profile,
            msg: None,
            content: None,
            headers: std::collections::HashMap::new(),
            status: 0,
        }
    }

    /// specifically, dispose a `Response`, handle failed or corrupt `Response`, and return `ParseResult` or `ParseError`.
    pub async fn parse<E>(res: Response<T, P>, spd: &'t dyn Spider<E, T, P>) -> ( ParseResult<E, T, P>, u64 )
    where
        E: Serialize + std::fmt::Debug + Clone + Send,
    {
        log::info!("successful requested: {}", &res.task.uri);
        let salt = [&res.task.uri, &res.task.method, &res.task.able.to_string(), &res.task.trys.to_string()];
        let hash = utils::hash(salt.iter());
        let ind = &res.task.parser;
        let parser = spd
            .get_parser(ind)
            .expect(&format!("parser {} not found.", &res.task.parser));
        ( (parser)(res), hash )
    }

    /// parse multiple `Response` in `App`, then drive all `ParseResult` into `MiddleWare`
    pub async fn parse_all<'b, E>(
        app: &'t mut App<E, T, P>,
        round: usize,
        spd: &'t dyn Spider<E, T, P>,
        mware: &'t MiddleWare<'b, E, T, P>,
    ) where
        E: Serialize + std::fmt::Debug + Clone + Send,
    {
        let mut v = Vec::new();
        let mut tsks = Vec::new();
        let mut pfiles = Vec::new();
        let mut reqs = Vec::new();
        let mut yerr = Vec::new();
        let mut ens = Vec::new();
        let mut errs = Vec::new();
        let mut hashes = Vec::new();

        let len = app.res.lock().unwrap().len();
        vec![0; len.min(round)].iter().for_each(|_| {
            let t = app.res.lock().unwrap().remove(0);
            if t.status >= 200 && t.status <= 299 {
                v.push(t);
            } else {
                errs.push(t);
            }
        });
        if errs.len() > 0 {
            (mware.handle_err)(&mut errs, app).await;
        }
        if v.len() > 0 {
            (mware.handle_res)(&mut v, app).await;
        }
        while let Some(res) = v.pop() {
            let ( prs, hash ) = Response::parse(res, spd).await;
            hashes.push(hash);
            tsks.extend(prs.task);
            pfiles.extend(prs.profile);
            reqs.extend(prs.req);
            yerr.extend(prs.yield_err);
            ens.extend(prs.entities);
        }
        if !hashes.is_empty() {
            app.fut_res.direct_join(hashes).await;
        }
        if !reqs.is_empty() {
            let (task, pfile) = (mware.handle_req)(&mut reqs, app).await;
            app.profile.lock().unwrap().extend(pfile);
            app.task.lock().unwrap().extend(task);
            app.req.lock().unwrap().extend(reqs);
        }
        if !pfiles.is_empty() {
            (mware.handle_profile)(&mut pfiles, app).await;
            app.profile.lock().unwrap().extend(pfiles);
        }
        if !tsks.is_empty() {
            (mware.handle_task)(&mut tsks, app).await;
            app.task_tmp.lock().unwrap().extend(tsks);
        }
        if !ens.is_empty() {
            (mware.handle_entity)(&mut ens, app).await;
            app.entities.lock().unwrap().extend(ens);
        }
        if !yerr.is_empty() {
            app.yield_err.lock().unwrap().extend(yerr);
        }
    }
}
