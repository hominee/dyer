extern crate hyper_timeout;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
extern crate tokio;

use crate::component::{Client, ParseResult, Profile, Request, Response, Task, UserAgent};
use crate::macros::{MethodIndex, Spider};
use crate::macros::{MiddleWare, MiddleWareDefault, Pipeline, PipelineDefault};
use futures::future::join_all;
use log::info;
use rand::prelude::Rng;
use serde::{Deserialize, Serialize};
use signal_hook::flag as signal_flag;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::task;

pub enum Elements<'a, Entity, T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    Entity: Serialize + std::fmt::Debug + Clone,
{
    Pfile(Profile<P>),
    Tsk(Task<T>),
    Res(Response<T, P>),
    Req(Request<T, P>),
    Rst(Vec<Entity>),
    PrsRst(ParseResult<Entity, T, P>),

    RefPfile(&'a Profile<P>),
    RefTsk(&'a Task<T>),
    RefRes(&'a Response<T, P>),
    RefReq(&'a Request<T, P>),
    RefRst(&'a Vec<Entity>),
    RefPrsRst(&'a ParseResult<Entity, T, P>),

    RefmPfile(&'a mut Profile<P>),
    RefmTsk(&'a mut Task<T>),
    RefmRes(&'a mut Response<T, P>),
    RefmReq(&'a mut Request<T, P>),
    RefmRst(&'a mut Vec<Entity>),
    RefmPrsRst(&'a mut ParseResult<Entity, T, P>),

    Array(Vec<Elements<'a, Entity, T, P>>),
}
unsafe impl<'a, E, T, P> Send for Elements<'a, E, T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
}
unsafe impl<'a, E, T, P> Sync for Elements<'a, E, T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
}

impl<'a, E, T, P> std::fmt::Debug for Elements<'a, E, T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    E: Serialize + std::fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Elements::Req(d) => "Request:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::RefReq(d) => "Request:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::RefmReq(d) => "Request:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::Tsk(d) => "Tsk:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::RefTsk(d) => "Tsk:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::RefmTsk(d) => "Tsk:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::Pfile(d) => "Profile:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::RefPfile(d) => "Profile:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::RefmPfile(d) => "Profile:".to_string() + &serde_json::to_string(&d).unwrap(),
            Elements::Res(_d) => "Response".to_string(),
            Elements::RefRes(_d) => "reference to Response".to_string(),
            Elements::RefmRes(_d) => "mutable reference Response".to_string(),
            Elements::Rst(_d) => "Result".to_string(),
            Elements::RefRst(_d) => "reference to Result".to_string(),
            Elements::RefmRst(_d) => "mutable reference Result".to_string(),
            Elements::Array(_d) => "array of Elements".to_string(),

            _ => "other type".to_string(),
        };
        f.write_fmt(format_args!("Elements:({})", s))
    }
}

/// number that once for a concurrent future poll
pub struct AppArg {
    pub gap: u64,               // time tap added to created Tasks or Profiles
    pub join_gap: u64,          // gap to forcefully join the spawned task
    pub round_req: usize,       // consume req one time
    pub round_req_min: usize,   // cache request minimal length
    pub round_req_max: usize,   // cache request maximal length
    pub buf_task_tmp: usize,    // buffer length for the created task.
    pub round_task: usize,      // construct req from task one time
    pub round_task_min: usize,  // minimal task(profile) consumed per round
    pub round_res: usize,       // consume response once upon a time
    pub profile_min: usize,     // minimal profile number
    pub profile_max: usize,     // maximal profile number
    pub round_yield_err: usize, //consume yield_err once upon a time
    pub round_result: usize,    //consume Entity once upon a time
    pub skip_history: bool,
    pub rate: Rate, // control the task speed runtime
}

impl Default for AppArg {
    fn default() -> Self {
        AppArg {
            gap: 20,
            join_gap: 7,
            round_req: 10,
            round_req_min: 7,
            round_req_max: 70,
            buf_task_tmp: 10000,
            round_task: 100,
            round_task_min: 70,
            round_res: 100,
            profile_min: 1000,
            profile_max: 5000,
            round_yield_err: 100,
            round_result: 100,
            skip_history: false,
            rate: Rate::new(),
        }
    }
}

#[derive(std::fmt::Debug)]
pub struct Rate {
    pub alltime: f64,
    pub uptime: f64,
    pub active: bool,
    pub load: f64,
    pub low_load: f64,
    pub err: u64,
    pub remains: u64,
    pub low_remains: u64,
    pub anchor: f64,
    pub interval: f64,
    pub peroid: f64,
    pub stamps: Vec<f64>,
}

impl Rate {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            + 30.0;
        Rate {
            alltime: 0.0,
            uptime: 0.0,
            active: true,
            load: 90.0,
            low_load: 90.0,
            remains: 110,
            low_remains: 90,
            err: 0,
            anchor: now,
            interval: 30.0,
            peroid: 200.0,
            stamps: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        if now > self.anchor {
            self.uptime += self.interval;
            self.anchor += self.interval;
            self.alltime += self.interval;
            if self.uptime.rem_euclid(self.peroid) <= 0.168 * self.peroid {
                log::info!("inactive peroid");
                self.active = false;
                self.low_remains = self.low_load as u64;
            } else {
                log::info!("active peroid");
                self.active = true;
                self.uptime = self.uptime.rem_euclid(self.peroid);
                /*
                 *if self.remains >= 2 {
                 *    self.load -= 0.1;
                 *}
                 *if self.err >= 1 {
                 *    self.load -= 0.2;
                 *    return ();
                 *}
                 */
                if self.stamps.len() >= 3 && (self.err as f64) / (self.stamps.len() as f64) <= 0.1 {
                    let mean: f64 =
                        self.stamps.iter().map(|t| t).sum::<f64>() / (self.stamps.len() as f64);
                    let dev = (self.stamps.iter().map(|t| (t - mean).powi(2)).sum::<f64>()
                        / (self.stamps.len() as f64))
                        .sqrt();
                    self.stamps.clear();
                    if dev / mean <= 0.15 {
                        let load = (self.load * 0.7 + 0.3 * mean.recip() * self.interval)
                            .max(3.8 * self.low_load);
                        log::info!("lantency is stable, load from {} to {}.", self.load, load);
                        self.load = load;
                    } else {
                        let load = (self.load * 0.5 + 0.5 * mean.recip() * self.interval)
                            .max(3.0 * self.low_load);
                        log::info!(
                    "lantency is turbulent, increase the weight of mean, load from {} to {}.",
                    self.load, load ,
                );
                        self.load = load;
                    }
                } else if self.stamps.len() >= 150 {
                    self.stamps.clear();
                }
                self.remains = (self.low_load as u64 * 3).max(self.load as u64);
            }
        }
    }

    pub fn backup(&mut self) -> bool {
        if self.alltime >= 600.0 {
            self.alltime = 0.0;
            return true;
        }
        false
    }

    pub fn get_len(&mut self, tm: Option<u64>) -> usize {
        let now = match tm {
            Some(now) => now as f64,
            None => std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };
        if self.active {
            let delta = self.load * (self.anchor - now) / self.interval;
            let len = if self.remains as f64 >= delta + 0.5 {
                self.remains as f64 - delta
            } else {
                0.0
            };
            log::info!("remains:{}, delta: {}, len: {}", self.remains, delta, len);
            //let len = self.remains - (self.load * (self.anchor - now) / self.interval) as u64;
            self.remains = self.remains - (len as u64);
            log::info!("limit the engine to spawning {} tasks.", len);
            len.ceil() as usize
        } else {
            let delta = self.low_load * (self.anchor - now) / self.interval;
            let len = if self.low_remains as f64 >= delta + 0.5 {
                self.low_remains as f64 - delta
            } else {
                0.0
            };
            log::info!(
                "remains:{}, delta: {}, len: {}",
                self.low_remains,
                delta,
                len
            );
            self.low_remains = self.low_remains - (len as u64);
            log::info!("limit the engine to spawning {} tasks.", len);
            len.ceil() as usize
        }
    }
}

pub struct App<Entity, T, P>
where
    T: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
    P: Serialize + for<'a> Deserialize<'a> + std::fmt::Debug + Clone,
{
    pub uas: Arc<Vec<UserAgent>>,
    pub task: Arc<Mutex<Vec<Task<T>>>>,
    pub task_tmp: Arc<Mutex<Vec<Task<T>>>>,
    pub profile: Arc<Mutex<Vec<Profile<P>>>>,
    pub req: Arc<Mutex<Vec<Request<T, P>>>>,
    pub req_tmp: Arc<Mutex<Vec<Request<T, P>>>>,
    pub res: Arc<Mutex<Vec<Response<T, P>>>>,
    pub result: Arc<Mutex<Vec<Entity>>>,
    pub yield_err: Arc<Mutex<Vec<String>>>,
    pub fut_res: Arc<Mutex<Vec<(u64, task::JoinHandle<()>)>>>,
    pub fut_profile: Arc<Mutex<Vec<(u64, task::JoinHandle<()>)>>>,
    pub rt_args: Arc<Mutex<AppArg>>,
}

impl<'a, Entity, T, P> App<Entity, T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    Entity: Serialize + std::fmt::Debug + Clone + Send + Sync,
{
    pub fn new() -> Self {
        App {
            uas: Arc::new(Vec::new()),
            task: Arc::new(Mutex::new(Vec::new())),
            task_tmp: Arc::new(Mutex::new(Vec::new())),
            profile: Arc::new(Mutex::new(Vec::new())),
            req: Arc::new(Mutex::new(Vec::new())),
            req_tmp: Arc::new(Mutex::new(Vec::new())),
            res: Arc::new(Mutex::new(Vec::new())),
            result: Arc::new(Mutex::new(Vec::new())),
            yield_err: Arc::new(Mutex::new(Vec::new())),
            fut_res: Arc::new(Mutex::new(Vec::new())),
            fut_profile: Arc::new(Mutex::new(Vec::new())),
            rt_args: Arc::new(Mutex::new(AppArg::default())),
        }
    }

    pub fn info(&self) {
        let mut vs = Vec::new();
        let len_uas = self.uas.len();
        if len_uas != 0 {
            vs.push(format!("{} UserAgent", len_uas));
        }
        let len_task = self.task.lock().unwrap().len();
        if len_task != 0 {
            vs.push(format!("{} Task", len_task));
        }
        let len_task_tmp = self.task_tmp.lock().unwrap().len();
        if len_task_tmp != 0 {
            vs.push(format!("{} Task_tmp", len_task_tmp));
        }
        let len_profile = self.profile.lock().unwrap().len();
        if len_profile != 0 {
            vs.push(format!("{} Profile", len_profile));
        }
        let len_req = self.req.lock().unwrap().len();
        if len_req != 0 {
            vs.push(format!("{} Request", len_req));
        }
        let len_req_tmp = self.req_tmp.lock().unwrap().len();
        if len_req_tmp != 0 {
            vs.push(format!("{} Request_Tmp", len_req_tmp));
        }
        let len_res = self.res.lock().unwrap().len();
        if len_res != 0 {
            vs.push(format!("{} Response", len_res));
        }
        let len_result = self.result.lock().unwrap().len();
        if len_result != 0 {
            vs.push(format!("{} Result", len_result));
        }
        let len_yield_err = self.yield_err.lock().unwrap().len();
        if len_yield_err != 0 {
            vs.push(format!("{} Yield Error", len_yield_err));
        }
        let len_fut_res = self.fut_res.lock().unwrap().len();
        if len_fut_res != 0 {
            vs.push(format!("{} Future Response", len_fut_res));
        }
        let len_fut_profile = self.fut_profile.lock().unwrap().len();
        if len_fut_profile != 0 {
            vs.push(format!("{} Future Profile", len_fut_profile));
        }
        info!("{}", vs.join("\n"));
    }

    pub fn enough_profile(&self) -> bool {
        let mut rng = rand::thread_rng();
        let profile_len = self.profile.lock().unwrap().len()
            + self.fut_profile.lock().unwrap().len()
            + self.req.lock().unwrap().len()
            + self.req_tmp.lock().unwrap().len();
        let less = profile_len <= self.rt_args.lock().unwrap().profile_min;
        let profile_max = self.rt_args.lock().unwrap().profile_max;
        let exceed = !less && profile_len <= profile_max && rng.gen::<f64>() <= 0.333;
        let fut_exceed = profile_len < profile_max;
        let mut emer = false;
        if profile_len < self.task.lock().unwrap().len() && rng.gen::<f64>() <= 0.01 {
            emer = true;
        }
        (less || exceed) && fut_exceed || emer
    }

    pub async fn plineout<C>(
        &mut self,
        pline: Option<&'a dyn Pipeline<Entity, C>>,
        default_pl: &PipelineDefault<Entity>,
    ) {
        if self.yield_err.lock().unwrap().len() > self.rt_args.lock().unwrap().round_yield_err {
            info!("pipeline put out yield_parse_err");
            match pline {
                Some(pl) => {
                    pl.process_yielderr(&mut self.yield_err).await;
                }
                None => {
                    default_pl.process_yielderr(&mut self.yield_err).await;
                }
            }
        }
        if self.result.lock().unwrap().len() > self.rt_args.lock().unwrap().round_result {
            info!("pipeline put out Entity");
            match pline {
                Some(pl) => {
                    pl.process_item(&mut self.result).await;
                }
                None => {
                    default_pl.process_yielderr(&mut self.yield_err).await;
                }
            }
        }
        if self.task_tmp.lock().unwrap().len() >= self.rt_args.lock().unwrap().buf_task_tmp {
            log::info!("pipeline out buffered task.");
            let vfiles = self.buf_task("../data/tasks/");
            let file_name = format!("../data/tasks/{}", 1 + vfiles.last().unwrap_or(&0));
            Task::stored(&file_name, &mut self.task_tmp);
            self.task_tmp.lock().unwrap().clear();
        }
    }

    pub fn update_req(
        &mut self,
        mware: Option<&'a dyn MiddleWare<Entity, T, P>>,
        default_mw: &'a MiddleWareDefault<Entity>,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let len_req_tmp = self.req_tmp.lock().unwrap().len();
        if len_req_tmp <= self.rt_args.lock().unwrap().round_req_min {
            info!("req_tmp does not contains enough Reqeust, take them from self.req");
            // cached request is not enough
            let len_req = self.req.lock().unwrap().len();
            let mut buf_req = Vec::new();
            let mut buf_task = Vec::new();
            let mut buf_pfile = Vec::new();
            for _ in 0..len_req {
                let request = self.req.lock().unwrap().remove(0);
                let (req, pfile, tsk) = match mware {
                    Some(mw) => mw.hand_req(request),
                    None => default_mw.hand_req(request),
                };
                if let Some(profile) = pfile {
                    buf_pfile.push(profile);
                }
                if let Some(task) = tsk {
                    buf_task.push(task);
                }
                if let Some(request) = req {
                    if request.able <= now {
                        // put the request into cbase_req_tmp
                        buf_req.push(request);
                    } else {
                        log::debug!("reach the unavailible request, stop.");
                        self.req.lock().unwrap().insert(0, request);
                        break;
                    }
                }
                if len_req_tmp + buf_req.len() > self.rt_args.lock().unwrap().round_req_max {
                    log::debug!("take full of Request from self.req");
                    break;
                }
            }
            self.req_tmp.lock().unwrap().extend(buf_req);
            self.task.lock().unwrap().extend(buf_task);
            self.profile.lock().unwrap().extend(buf_pfile);
        }
    }

    pub async fn spawn_task(&'a mut self, spd: &'static dyn Spider<Entity, T, P>) {
        if self.fut_res.lock().unwrap().len() > 3000 {
            log::warn!("enough Future Response, spawn no task.");
        } else {
            log::debug!("take request out to be executed.");
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let mut futs = Vec::new();
            let mut req_tmp = self.req_tmp.lock().unwrap();
            let mut args = self.rt_args.lock().unwrap();
            let len = args.round_req.min(req_tmp.len());
            let len_load = args.rate.get_len(None).min(len);
            vec![0; len_load].iter().for_each(|_| {
                let req = req_tmp.pop().unwrap();
                futs.push(req);
            });
            let tbase_res = self.res.clone();
            let f = spd.get_parser(MethodIndex::RequestInit);
            let arg = self.rt_args.clone();
            info!(
                "spawn {} tokio task to execute Request concurrently",
                len_load
            );
            let john = task::spawn(async move {
                Client::exec_all(futs, tbase_res, arg, f).await;
            });
            self.fut_res.lock().unwrap().push((now, john));
        }
    }

    pub fn buf_task(&self, path: &str) -> Vec<usize> {
        let mut vfiles = std::fs::read_dir(path)
            .unwrap()
            .map(|name| {
                name.unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap()
            })
            .collect::<Vec<usize>>();
        vfiles.sort();
        vfiles
    }

    pub async fn close<C>(
        &'a mut self,
        spd: &'static dyn Spider<Entity, T, P>,
        mware: Option<&'a dyn MiddleWare<Entity, T, P>>,
        pline: Option<&'a dyn Pipeline<Entity, C>>,
        default_mw: &'a MiddleWareDefault<Entity>,
        default_pl: &'a PipelineDefault<Entity>,
    ) {
        let gap = self.rt_args.lock().unwrap().gap;
        match mware {
            Some(ware) => Response::parse_all(self, usize::MAX, spd, ware, gap),
            None => Response::parse_all(self, usize::MAX, spd, default_mw, gap),
        }
        info!("sending all of them into Pipeline");
        match pline {
            Some(pl) => {
                pl.process_yielderr(&mut self.yield_err).await;
                pl.process_item(&mut self.result).await;
                pl.close_pipeline().await;
            }
            None => {
                default_pl.process_item(&mut self.result).await;
                default_pl.process_yielderr(&mut self.yield_err).await;
                default_pl.close_pipeline().await;
            }
        }

        log::info!("All work is Done. exit gracefully");
    }

    pub async fn run<C>(
        &'a mut self,
        spd: &'static dyn Spider<Entity, T, P>,
        mware: Option<&'a dyn MiddleWare<Entity, T, P>>,
        pline: Option<&'a dyn Pipeline<Entity, C>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // signal handling initial
        let term = Arc::new(AtomicUsize::new(0));
        const SIGINT: usize = signal_hook::SIGINT as usize;
        signal_flag::register_usize(signal_hook::SIGINT, Arc::clone(&term), SIGINT).unwrap();

        let default_pl = PipelineDefault::new();
        let default_mw = MiddleWareDefault::new();
        spd.open_spider(self);
        //skip the history and start new fields to staart with, some Profile required
        if self.rt_args.lock().unwrap().skip_history {
            log::warn!("skipped the history.");
            let uri = spd.entry_profile().unwrap();
            let uas = self.uas.clone();
            let gap = self.rt_args.lock().unwrap().gap;
            Profile::exec_all::<Entity, T>(None, self.profile.clone(), uri, 3usize, gap, uas).await;
            let tasks = spd.entry_task().unwrap();
            self.task.lock().unwrap().extend(tasks);
        } else {
            log::warn!("use the history file.");
            let reqs: Vec<Request<T, P>> = Request::load("../data/request").unwrap();
            let req_tmp: Vec<Request<T, P>> = Request::load("../data/request_tmp").unwrap();
            let profile: Vec<Profile<P>> = Profile::load("../data/profile").unwrap();
            let vfiles = self.buf_task("../data/tasks/");
            if !vfiles.is_empty() {
                let file = format!("../data/tasks/{}", vfiles[0]);
                let task: Vec<Task<T>> = Task::load(&file).unwrap();
                log::info!("{} loaded {} Task.", file, task.len());
                self.task = Arc::new(Mutex::new(task));
            } else {
                log::error!("task buffers are not found.");
            }
            if reqs.is_empty() && req_tmp.is_empty() && vfiles.is_empty() {
                panic!("not any task or request imported externally.");
            }
            self.req = Arc::new(Mutex::new(reqs));
            self.req_tmp = Arc::new(Mutex::new(req_tmp));
            self.profile = Arc::new(Mutex::new(profile));
        }

        loop {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            match term.load(Ordering::Relaxed) {
                SIGINT => {
                    // receive the Ctrl+c signal
                    // by default  request  task profile and result yield err are going to stroed into
                    // file
                    info!("receive Ctrl+c signal, preparing ...");

                    //finish remaining futures
                    let mut futs = Vec::new();
                    while let Some(res) = self.fut_res.lock().unwrap().pop() {
                        futs.push(res.1);
                    }
                    while !futs.is_empty() {
                        let mut v = Vec::new();
                        for _ in 0..20 {
                            if let Some(itm) = futs.pop() {
                                v.push(itm)
                            }
                        }
                        join_all(v).await;
                    }
                    info!("join all future response to be executed");
                    self.info();

                    // dispath them
                    self.close(spd, mware, pline, &default_mw, &default_pl)
                        .await;
                    info!("executing close_spider...");
                    spd.close_spider(self);
                    break;
                }

                0 => {
                    // if all task request and other things are done the quit
                    if self.req.lock().unwrap().is_empty()
                        && self.req_tmp.lock().unwrap().is_empty()
                        && self.task.lock().unwrap().is_empty()
                        && self.task_tmp.lock().unwrap().is_empty()
                        && self.fut_res.lock().unwrap().is_empty()
                        && self.fut_profile.lock().unwrap().is_empty()
                        && self.res.lock().unwrap().is_empty()
                    {
                        info!("all work is done.");
                        self.close(spd, mware, pline, &default_mw, &default_pl)
                            .await;
                        info!("executing close_spider...");
                        spd.close_spider(self);
                        break;
                    }

                    // consume valid request in cbase_reqs_tmp
                    // if not enough take them from self.req
                    self.update_req(mware, &default_mw);

                    //take req out to finish
                    self.spawn_task(spd).await;

                    // before we construct request check profile first
                    if self.enough_profile() {
                        info!("profile length too few or not exceeding max, generate Profile");
                        let uas = self.uas.clone();
                        let uri = spd.entry_profile().unwrap();
                        let pfile = self.profile.clone();
                        let f = spd.get_parser(MethodIndex::GenProfile);
                        let gap = self.rt_args.lock().unwrap().gap;
                        info!("spawn {} tokio task to generate Profile concurrently", 3);
                        let johp = task::spawn(async move {
                            Profile::exec_all::<Entity, T>(f, pfile, uri, 3usize, gap, uas).await;
                        });
                        self.fut_profile.lock().unwrap().push((now, johp));
                    }

                    // count for profiles length if not more than round_task_min
                    let len_p = self.fut_profile.lock().unwrap().len();
                    if self.rt_args.lock().unwrap().round_task_min > len_p && len_p != 0 {
                        // not enough profile to construct request
                        // await the spawned task done
                        info!("count for profiles length if not more than round_task_min");
                        let (_, jh) = self.fut_profile.lock().unwrap().pop().unwrap();
                        jh.await.unwrap();
                    }

                    // parse response
                    //extract the parseResult
                    info!("parsing Response ...");
                    let gap = self.rt_args.lock().unwrap().gap;
                    let round_res = self.rt_args.lock().unwrap().round_res;
                    match mware {
                        Some(ware) => Response::parse_all(self, round_res, spd, ware, gap),
                        None => Response::parse_all(self, round_res, spd, &default_mw, gap),
                    }

                    //pipeline put out yield_parse_err and Entity
                    self.plineout(pline, &default_pl).await;

                    // if task is running out, load them from nex buf_task
                    if self.task.lock().unwrap().is_empty() {
                        let vfiles = self.buf_task("../data/tasks/");
                        let file = format!("../data/tasks/{}", vfiles[0]);
                        log::warn!("remove used task in {}", file);
                        std::fs::remove_file(&file).unwrap();
                        if vfiles.len() == 1 {
                            log::info!("no task buffer file found. use task_tmp");
                            let mut task_tmp = Vec::new();
                            for _ in 0..self.task_tmp.lock().unwrap().len() {
                                let tsk = self.task_tmp.lock().unwrap().pop().unwrap();
                                task_tmp.push(tsk);
                            }
                            self.task.lock().unwrap().extend(task_tmp);
                        } else if vfiles.len() >= 2 {
                            let file_new = format!("../data/tasks/{}", vfiles[1]);
                            log::info!("load new task in {}", file_new);
                            let tsks = Task::load(&file_new).unwrap_or(vec![]);
                            self.task.lock().unwrap().extend(tsks);
                        }
                    }

                    // construct request
                    let len_t = self.task.lock().unwrap().len();
                    let len_p = self.profile.lock().unwrap().len();
                    if len_t != 0 && len_p != 0 {
                        info!("generate Request");
                        let gen_request = spd.get_parser(MethodIndex::GenRequest);
                        let round_task = self.rt_args.lock().unwrap().round_task;
                        let reqs = Request::gen(
                            self.profile.clone(),
                            self.task.clone(),
                            round_task,
                            gen_request,
                        );
                        self.req.lock().unwrap().extend(reqs);
                    }

                    //join the older tokio-task
                    info!("join the older tokio task.");
                    let join_gap = self.rt_args.lock().unwrap().join_gap;
                    Client::watch(self.fut_res.clone(), self.fut_profile.clone(), join_gap).await;
                    self.rt_args.lock().unwrap().rate.update();
                    if self.rt_args.lock().unwrap().rate.backup() {
                        self.close(spd, mware, pline, &default_mw, &default_pl)
                            .await;

                        log::info!("backup history...");
                        Profile::stored("../data/profile", &self.profile);
                        Task::stored("../data/task", &mut self.task);
                        Task::stored("../data/task_tmp", &mut self.task_tmp);
                        Request::stored("../data/request", &mut self.req);
                        Request::stored("../data/request_tmp", &mut self.req_tmp);
                    }
                    self.info();
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }

                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
