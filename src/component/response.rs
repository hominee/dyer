extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::component::{ParseError, Profile, Request, Task};
use crate::engine::{App, AppArg};
use crate::plugin::{MiddleWare, Spider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

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
    pub profile: Profile<P>,
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
    /// specifically, dispose a `Response`, handle failed or corrupt `Response`, and return `ParseResult` or `ParseError`.
    pub async fn parse<'b, E>(
        res: Response<T, P>,
        spd: &'static dyn Spider<E, T, P>,
        mware: &'t MiddleWare<'b, E, T, P>,
        arg: Arc<Mutex<AppArg>>,
    ) -> Result<ParseResult<E, T, P>, ParseError>
    where
        E: Serialize + std::fmt::Debug + Clone + Send,
    {
        //dispath handlers dependent on their status code
        let status = res.status;
        if status <= 299 && status >= 200 {
            log::debug!(
                "recycle profile, successful Response: uri: {}",
                &res.task.uri[0..77]
            );
            // status code between 200 - 299
            let content = res.content.clone().unwrap();
            let uri = res.task.uri.clone();
            let pfile = res.profile.clone();
            let ind = (&res.task.parser).to_string();
            let parser = spd
                .get_parser(ind)
                .expect(&format!("parser {} not found.", &res.task.parser));
            let data = (parser)(res);
            match data {
                Ok(prs) => Ok(prs),
                Err(_) => {
                    // no entities comes in.
                    // leave None as default.
                    log::error!("cannot parse Response");
                    let mut r = ParseResult {
                        req: vec![],
                        task: vec![],
                        profile: vec![pfile],
                        entities: vec![],
                        yield_err: vec![],
                    };
                    let s = format!("{}\n{}", uri, content);
                    r.yield_err.push(s);
                    Ok(r)
                }
            }
        } else {
            log::error!("failed Response: {:?}", res);
            let r = (mware.hand_err)(&mut vec![res], arg).await;
            Ok(ParseResult {
                task: r.0,
                profile: r.1,
                req: r.2,
                entities: vec![],
                yield_err: r.3,
            })
        }
    }

    /// parse multiple `Response` in `App`, then drive all `ParseResult` into `MiddleWare`
    pub async fn parse_all<'b, E>(
        app: &'t mut App<E, T, P>,
        round: usize,
        spd: &'static dyn Spider<E, T, P>,
        mware: &'t MiddleWare<'b, E, T, P>,
    ) where
        E: Serialize + std::fmt::Debug + Clone + Send,
    {
        let mut v = Vec::new();
        let len = app.res.lock().unwrap().len();
        vec![0; len.min(round)].iter().for_each(|_| {
            let t = app.res.lock().unwrap().remove(0);
            v.push(t);
        });
        (mware.hand_res)(&mut v, app.rt_args.clone()).await;
        while let Some(res) = v.pop() {
            let fut = Response::parse(res, spd, &mware, app.rt_args.clone());
            match fut.await {
                Ok(mut prs) => {
                    if !prs.req.is_empty() {
                        let (task, pfile) =
                            (mware.hand_req)(&mut prs.req, app.rt_args.clone()).await;
                        prs.profile.extend(pfile);
                        prs.task.extend(task);
                        app.req.lock().unwrap().extend(prs.req);
                    }
                    if !prs.profile.is_empty() {
                        (mware.hand_profile)(&mut prs.profile, app.rt_args.clone()).await;
                        app.profile.lock().unwrap().extend(prs.profile);
                    }
                    if !prs.task.is_empty() {
                        (mware.hand_task)(&mut prs.task, app.rt_args.clone()).await;
                        app.task_tmp.lock().unwrap().extend(prs.task);
                    }
                    if !prs.entities.is_empty() {
                        (mware.hand_item)(&mut prs.entities, app.rt_args.clone()).await;
                        app.result.lock().unwrap().extend(prs.entities);
                    }
                    if !prs.yield_err.is_empty() {
                        app.yield_err.lock().unwrap().extend(prs.yield_err);
                    };
                    let mut rate = &mut app.rt_args.lock().unwrap().rate;
                    if rate.active == true && rate.uptime >= 0.5 * rate.interval {
                        rate.active = false;
                        rate.uptime = 0.0;
                    }
                }
                Err(_) => {
                    // res has err code (non-200) and cannot handled by error handle
                    // discard the response that without task or profile.
                    log::error!("parse response failed.");
                }
            }
        }
    }
}

impl<T, P> Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + Debug + Clone,
{
    pub fn default(req: Option<&Request<T, P>>) -> Response<T, P> {
        let r = match req {
            Some(r) => r.clone(),
            None => Request::default(),
        };
        Response {
            task: r.task,
            profile: r.profile,
            msg: None,
            content: None,
            headers: std::collections::HashMap::new(),
            status: 0,
        }
    }
}
