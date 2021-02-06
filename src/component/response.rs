extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;

use crate::component::{ParseError, Profile, Request, Task};
use crate::engine::App;
use crate::macros::{MiddleWare, MiddleWareDefault, Spider};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

pub struct ParseResult<E, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
    pub req: Option<Request<T, P>>,
    pub task: Option<Vec<Task<T>>>,
    pub profile: Option<Vec<Profile<P>>>,
    pub entities: Option<Vec<E>>,
    pub yield_err: Option<String>,
    pub low_mode: bool,
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

#[derive(Clone, Debug)]
pub struct Response<T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
    pub task: Task<T>,
    pub profile: Profile<P>,
    pub status: usize,
    pub trys: u8,
    pub content: Option<String>,
    pub headers: HashMap<String, String>,
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
    pub async fn parse<E, C>(
        mut res: Response<T, P>,
        spd: &'static dyn Spider<E, T, P, C>,
        mware: &'t Option<Box<dyn MiddleWare<E, T, P, C>>>,
        default_mw: Option<&'t MiddleWareDefault<E, T, P, C>>,
    ) -> Result<ParseResult<E, T, P>, ParseError>
    where
        E: Serialize + std::fmt::Debug + Clone + Send,
        C: Send,
    {
        //dispath handlers dependent on their status code
        let status = res.status;
        if status <= 299 && status >= 200 {
            debug!("successful Response: uri: {}", &res.task.uri[0..77]);
            // status code between 200 - 299
            if mware.is_some() {
                mware.as_ref().unwrap().hand_res(&mut res).await;
            } else {
                default_mw.unwrap().hand_res(&mut res).await;
            }
            if mware.is_some() {
                mware.as_ref().unwrap().hand_profile(&mut res.profile).await;
            } else {
                default_mw.unwrap().hand_profile(&mut res.profile).await;
            }
            info!("recycle profile");
            let mut r = ParseResult {
                req: None,
                task: None,
                profile: Some(vec![res.profile.clone()]),
                entities: None,
                yield_err: None,
                low_mode: false,
            };
            let content = res.content.clone().unwrap();
            let uri = res.task.uri.clone();
            let ind = (&res.task.parser).to_string();
            let parser = spd
                .get_parser(ind)
                .expect(&format!("parser {} not found.", &res.task.parser));
            let data = (parser)(res);
            match data {
                Ok(prs) => {
                    if let Some(mut items) = prs.entities {
                        if mware.is_some() {
                            mware.as_ref().unwrap().hand_item(&mut items).await;
                        } else {
                            default_mw.unwrap().hand_item(&mut items).await;
                        }
                        r.entities = Some(items);
                    }
                    if let Some(mut tasks) = prs.task {
                        if mware.is_some() {
                            mware.as_ref().unwrap().hand_task(&mut tasks).await;
                        } else {
                            default_mw.unwrap().hand_task(&mut tasks).await;
                        }
                        r.task = Some(tasks);
                    }
                    if let Some(reqs) = prs.req {
                        let (req, pfile, task) = if mware.is_some() {
                            mware.as_ref().unwrap().hand_req(reqs).await
                        } else {
                            default_mw.unwrap().hand_req(reqs).await
                        };
                        r.req = req;
                        if let Some(profile) = pfile {
                            let mut pfiles = r.profile.unwrap_or(vec![]);
                            pfiles.push(profile);
                            r.profile = Some(pfiles);
                        }
                        if let Some(tsk) = task {
                            let mut tasks = r.task.unwrap_or(vec![]);
                            tasks.push(tsk);
                            r.task = Some(tasks);
                        }
                    }
                    r.yield_err = prs.yield_err;
                }
                Err(_) => {
                    // no entities comes in.
                    // leave None as default.
                    error!("cannot parse Response");
                    let s = format!("{}\n{}", uri, content);
                    r.yield_err = Some(s);
                }
            }
            return Ok(r);
        } else {
            log::error!("failed Response: {:?}", res);
            let r = if mware.is_some() {
                mware.as_ref().unwrap().hand_err(res).await
            } else {
                default_mw.unwrap().hand_err(res).await
            };
            match r {
                Some(r) => {
                    let file = if let Some(file) = r.1 {
                        Some(vec![file])
                    } else {
                        None
                    };

                    Ok(ParseResult {
                        task: r.0,
                        profile: file,
                        req: r.2,
                        entities: None,
                        yield_err: r.3,
                        low_mode: r.4,
                    })
                }
                None => Err(ParseError {
                    desc: "Non-200s status code , not good".to_owned(),
                }),
            }
        }
    }

    pub async fn parse_all<E, C>(
        apk: &'t mut App<E, T, P, C>,
        round: usize,
        spd: &'static dyn Spider<E, T, P, C>,
    ) where
        E: Serialize + std::fmt::Debug + Clone + Send,
        C: Send,
    {
        let mut v = Vec::new();
        let len = apk.res.lock().unwrap().len();
        vec![0; len.min(round)].iter().for_each(|_| {
            let t = apk.res.lock().unwrap().pop().unwrap();
            v.push(t);
        });
        while let Some(res) = v.pop() {
            let fut = if apk.middleware.is_some() {
                Response::parse(res, spd, &apk.middleware, None)
            } else {
                Response::parse(res, spd, &None, Some(&apk.default_mw))
            };
            match fut.await {
                Ok(d) => {
                    if let Some(da) = d.profile {
                        apk.profile.lock().unwrap().extend(da);
                    }
                    if let Some(ta) = d.task {
                        apk.task_tmp.lock().unwrap().extend(ta);
                    }
                    if let Some(re) = d.req {
                        apk.req.lock().unwrap().push(re);
                    }
                    if let Some(err) = d.yield_err {
                        apk.yield_err.lock().unwrap().push(err);
                    }
                    if let Some(en) = d.entities {
                        // pipeline out put the entities
                        apk.result.lock().unwrap().extend(en.into_iter());
                    }
                    let mut rate = &mut apk.rt_args.lock().unwrap().rate;
                    if rate.active == true
                        && d.low_mode == true
                        && rate.uptime >= 0.5 * rate.interval
                    {
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
            trys: 0,
        }
    }
}
