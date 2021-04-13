extern crate hyper_timeout;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
extern crate tokio;

use crate::component::{Client, Profile, Request, Response, Task};
use crate::plugin::Spider;
use crate::plugin::{MiddleWare, PipeLine};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use signal_hook::flag as signal_flag;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;

/// Arguments that control the `App` at runtime, including using history or not,  
/// `Task` `Profile` `Request` `Response` `Entity` consuming and generating
/// There shall be an introduction to every member(maybe coming soon).
pub struct AppArg {
    /// time tap added to created Tasks or Profiles
    pub gap: u64,
    /// gap to forcefully join the spawned task
    pub join_gap: u64,
    /// gap to forcefully join the spawned task if none of items meeting join_gap
    pub join_gap_emer: f64,
    /// number that once for a concurrent future poll
    pub round_req: usize,
    /// cache request minimal length
    pub round_req_min: usize,
    /// cache request maximal length
    pub round_req_max: usize,
    /// buffer length for the created task.
    pub buf_task_tmp: usize,
    /// maximal spawned task that cached
    pub spawn_task_max: usize,
    /// construct req from task one time
    pub round_task: usize,
    /// minimal task(profile) consumed per round
    pub round_task_min: usize,
    /// consume response once upon a time
    pub round_res: usize,
    /// minimal profile number
    pub profile_min: usize,
    /// maximal profile number
    pub profile_max: usize,
    ///consume yield_err once upon a time
    pub round_yield_err: usize,
    ///consume Entity once upon a time
    pub round_result: usize,
    pub skip_history: bool,
    /// control the task speed runtime
    pub rate: Rate,
}

impl AppArg {
    fn new() -> Self {
        AppArg {
            gap: 20,
            join_gap: 7,
            join_gap_emer: 0.1,
            round_req: 10,
            round_req_min: 7,
            round_req_max: 70,
            buf_task_tmp: 10000,
            spawn_task_max: 100,
            round_task: 100,
            round_task_min: 70,
            round_res: 100,
            profile_min: 1000,
            profile_max: 5000,
            round_yield_err: 100,
            round_result: 100,
            skip_history: true,
            rate: Rate::new(),
        }
    }
}

/// some infomation about `dyer` at rumtime where speed and error-handler based on
#[derive(std::fmt::Debug)]
pub struct Rate {
    /// all time the app runs
    pub uptime: f64,
    /// the time that a cycle lasts, backup application history once running out
    pub cycle: f64,
    /// time the app runs in each cycle
    pub cycle_usage: f64,
    /// consist of many intervals,
    pub period: f64,
    /// time used in each period
    pub period_usage: f64,
    /// between 0-1, the rate that low mode lasts in each period
    pub period_threshold: f64,
    /// a time gap when updating some infomation
    pub interval: f64,
    /// if false, the app spawns task that multiplied by low_rate in each interval
    pub active: bool,
    /// normally the speed that the app spawns tasks in the whole interval
    pub load: f64,
    /// failed tasks in each interval
    pub err: u64,
    /// remaining jobs to do in each cycle in each interval
    pub remains: u64,
    /// the rate applied to limit the requests to be spawned in low mode
    pub low_rate: f64,
    /// time anchor at which update some infomation
    pub anchor: f64,
    /// vector of gap each request takes to receive response header in each interval  
    pub stamps: Vec<f64>,
}

impl Rate {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Rate {
            uptime: 0.0,
            cycle: 600.0,
            cycle_usage: 0.0,
            active: true,
            load: 99.0,
            remains: 110,
            low_rate: 0.333,
            err: 0,
            anchor: now + 30.0,
            interval: 30.0,
            period: 200.0,
            period_usage: 0.0,
            period_threshold: 0.168,
            stamps: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        if now > self.anchor {
            self.cycle_usage += self.interval;
            self.period_usage += self.interval;
            self.anchor += self.interval;
            self.uptime += self.interval;
            if self.period_usage.rem_euclid(self.period) <= self.period_threshold * self.period {
                log::debug!("inactive period");
                self.active = false;
            } else {
                log::debug!("active period");
                self.active = true;
                self.period_usage = self.period_usage.rem_euclid(self.period);
                self.stamps.clear();
                self.remains = self.load as u64;
            }
        }
    }

    /// backup the `Task` `Profile` `Request` for some time in case of interupt
    pub fn backup(&mut self) -> bool {
        if self.cycle_usage >= self.cycle {
            self.cycle_usage = self.cycle_usage.rem_euclid(self.cycle);
            return true;
        }
        false
    }

    /// decide the length of `Task` to be spawned
    pub fn get_len(&mut self, tm: Option<f64>) -> usize {
        let now = match tm {
            Some(now) => now,
            None => std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };
        let delta = self.load * (self.anchor - now) / self.interval;
        let len = if self.remains as f64 >= delta + 0.5 && delta >= 0.0 {
            self.remains as f64 - delta
        } else if (self.remains as f64) < delta + 0.5 && delta >= 0.0 {
            self.remains = delta as u64;
            0.0
        } else {
            self.remains as f64
        };
        log::debug!("remains:{}, delta: {}, len: {}", self.remains, delta, len);
        self.remains = self.remains - (len as u64) + 1;
        if len > 0.0 {
            log::debug!("only {} tasks are valid by rate control.", len);
        }
        if self.active {
            len.ceil() as usize
        } else {
            (self.low_rate * len).ceil() as usize
        }
    }
}

/// An abstraction and collection of data flow of `Dyer`,  
pub struct App<Entity, T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
{
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
    T: 'static + Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone + Sync + Send,
    P: 'static + Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone + Sync + Send,
    Entity: Serialize + std::fmt::Debug + Clone + Send + Sync,
{
    pub fn new() -> Self {
        App {
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
            rt_args: Arc::new(Mutex::new(AppArg::new())),
        }
    }

    /// get the overview of `App`
    pub fn info(&mut self) {
        let mut vs = Vec::new();
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
        log::info!("{}", vs.join("    "));
    }

    /// to see whether to generate `Profile`
    pub fn enough_profile(&self) -> bool {
        let rd1 = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            * 3000.0)
            % 1.0;
        let profile_len = self.profile.lock().unwrap().len()
            + self.fut_profile.lock().unwrap().len()
            + self.req.lock().unwrap().len()
            + self.req_tmp.lock().unwrap().len();
        let less = profile_len <= self.rt_args.lock().unwrap().profile_min;
        let profile_max = self.rt_args.lock().unwrap().profile_max;
        let exceed = !less && profile_len <= profile_max && rd1 <= 0.333;
        let fut_exceed = profile_len < profile_max;
        let mut emer = false;
        let rd2 = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            * 3000.0)
            % 1.0;
        if profile_len < self.task.lock().unwrap().len() && rd2 <= 0.01 {
            emer = true;
        }
        (less || exceed) && fut_exceed || emer
    }

    /// drive and consume extracted Entity into `PipeLine`
    pub async fn plineout<C>(&mut self, pipeline: &PipeLine<'a, Entity, C>)
    where
        C: Send + 'a,
    {
        if self.yield_err.lock().unwrap().len() > self.rt_args.lock().unwrap().round_yield_err {
            log::debug!("pipeline put out yield_parse_err");
            (pipeline.process_yerr)(&mut self.yield_err).await;
        }
        if self.result.lock().unwrap().len() > self.rt_args.lock().unwrap().round_result {
            self.info();
            log::debug!("pipeline put out Entity");
            (pipeline.process_item)(&mut self.result).await;
        }
        if self.task_tmp.lock().unwrap().len() >= self.rt_args.lock().unwrap().buf_task_tmp {
            log::debug!("pipeline out buffered task.");
            let vfiles = self.buf_task("data/tasks/");
            let file_name = format!("data/tasks/{}", 1 + vfiles.last().unwrap_or(&0));
            Task::stored(&file_name, &mut self.task_tmp);
            self.task_tmp.lock().unwrap().clear();
        }
    }

    /// load and balance `Request`
    pub async fn update_req(&mut self, middleware: &MiddleWare<'a, Entity, T, P>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let len_req_tmp = self.req_tmp.lock().unwrap().len();
        if len_req_tmp <= self.rt_args.lock().unwrap().round_req_min {
            log::debug!("req_tmp does not contains enough Reqeust, take them from self.req");
            // cached request is not enough
            let len_req = self.req.lock().unwrap().len();
            let mut buf_req = Vec::new();
            let mut requests = Vec::new();

            //  limit len_req and reqs that is availible by now
            for _ in 0..len_req {
                let request = self.req.lock().unwrap().remove(0);
                if request.able <= now {
                    requests.push(request);
                } else {
                    self.req.lock().unwrap().insert(0, request);
                    break;
                }
            }
            let (buf_task, buf_pfile) = (middleware.hand_req)(&mut requests, self).await;
            buf_req.extend(requests);
            self.req_tmp.lock().unwrap().extend(buf_req);
            self.task.lock().unwrap().extend(buf_task);
            self.profile.lock().unwrap().extend(buf_pfile);
        }
    }

    /// spawn polling `Request` as `tokio::task` and executing asynchronously,
    pub async fn spawn_task(&mut self) {
        if self.fut_res.lock().unwrap().len() > self.rt_args.lock().unwrap().spawn_task_max {
            log::warn!("enough Future Response, spawn no task.");
        } else {
            log::trace!("take request out to be executed.");
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let mut futs = Vec::new();
            let mut req_tmp = self.req_tmp.lock().unwrap();
            let mut args = self.rt_args.lock().unwrap();
            let len = args.round_req.min(req_tmp.len());
            let len_load = args.rate.get_len(None).min(len);
            if len_load > 0 {
                log::info!(
                    "spawn {} tokio task to execute Request concurrently",
                    len_load
                );
            }
            if len_load > 0 {
                vec![0; len_load].iter().for_each(|_| {
                    let req = req_tmp.pop().unwrap();
                    futs.push(req);
                });
                let tbase_res = self.res.clone();
                let arg = self.rt_args.clone();
                let john = task::spawn(async move {
                    Client::exec_all(futs, tbase_res, arg).await;
                });
                self.fut_res.lock().unwrap().push((now, john));
            }
        }
    }

    /// load cached `Task` from caced directory
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

    /// preparation before closing `Dyer`
    pub async fn close<'b, C>(
        &'a mut self,
        spd: &'a dyn Spider<Entity, T, P>,
        middleware: &'a MiddleWare<'b, Entity, T, P>,
        pipeline: &'a PipeLine<'b, Entity, C>,
    ) where
        C: Send + 'b,
    {
        Response::parse_all(self, usize::MAX, spd, middleware).await;
        log::info!("sending all of them into Pipeline");
        (pipeline.process_yerr)(&mut self.yield_err).await;
        (pipeline.process_item)(&mut self.result).await;
        (pipeline.close_pipeline)().await;
        log::info!("All work is Done.");
    }

    /// drive `Dyer` into running.
    pub async fn run<'b, C>(
        &'a mut self,
        spd: &'a dyn Spider<Entity, T, P>,
        middleware: &'a MiddleWare<'b, Entity, T, P>,
        pipeline: PipeLine<'b, Entity, C>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        C: Send + 'a,
    {
        // signal handling initial
        let term = Arc::new(AtomicUsize::new(0));
        const SIGINT: usize = signal_hook::SIGINT as usize;
        signal_flag::register_usize(signal_hook::SIGINT, Arc::clone(&term), SIGINT).unwrap();

        spd.open_spider(self);
        //skip the history and start new fields to staart with, some Profile required
        if self.rt_args.lock().unwrap().skip_history {
            log::warn!("skipped the history.");
            Profile::exec_all::<Entity, T>(self.profile.clone(), 3usize, spd.entry_profile()).await;
            let tasks = spd.entry_task().unwrap();
            self.task.lock().unwrap().extend(tasks);
        } else {
            log::warn!("use the history file.");
            let reqs: Vec<Request<T, P>> = Request::load("data/request");
            let req_tmp: Vec<Request<T, P>> = Request::load("data/request_tmp");
            let profile: Vec<Profile<P>> = Profile::load("data/profile");
            let vfiles = self.buf_task("data/tasks/");
            if !vfiles.is_empty() {
                let file = format!("data/tasks/{}", vfiles[0]);
                let task: Vec<Task<T>> = Task::load(&file);
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
                    log::info!("receive Ctrl+c signal, preparing ...");

                    //finish remaining futures
                    let mut futs = Vec::new();
                    while let Some(res) = self.fut_res.lock().unwrap().pop() {
                        futs.push(res.1);
                    }
                    while !futs.is_empty() {
                        let mut v = Vec::new();
                        for _ in 0..5 {
                            if let Some(itm) = futs.pop() {
                                v.push(itm)
                            }
                        }
                        join_all(v).await;
                        log::debug!("join 7 future response ");
                    }
                    log::trace!("join all future response to be executed");

                    // dispath them
                    self.close(spd, middleware, &pipeline).await;
                    log::info!("executing close_spider...");
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
                        log::info!("all work is done.");
                        self.close(spd, middleware, &pipeline).await;
                        log::info!("executing close_spider...");
                        spd.close_spider(self);
                        break;
                    }

                    // consume valid request in cbase_reqs_tmp
                    // if not enough take them from self.req
                    self.update_req(middleware).await;

                    //take req out to finish
                    self.spawn_task().await;

                    // before we construct request check profile first
                    if self.enough_profile() {
                        log::debug!(
                            "profile length too few or not exceeding max, generate Profile"
                        );
                        let pfile = self.profile.clone();
                        log::info!("spawn {} tokio task to generate Profile concurrently", 3);
                        let f = spd.entry_profile();
                        let johp = task::spawn(async move {
                            Profile::exec_all::<Entity, T>(pfile, 3usize, f).await;
                        });
                        self.fut_profile.lock().unwrap().push((now, johp));
                    }

                    // count for profiles length if not more than round_task_min
                    let len_p = self.fut_profile.lock().unwrap().len();
                    if self.rt_args.lock().unwrap().round_task_min > len_p && len_p != 0 {
                        // not enough profile to construct request
                        // await the spawned task done
                        log::debug!("count for profiles length if not more than round_task_min");
                        let (_, jh) = self.fut_profile.lock().unwrap().pop().unwrap();
                        jh.await.unwrap();
                    }

                    // parse response
                    //extract the parseResult
                    let round_res = self.rt_args.lock().unwrap().round_res;
                    Response::parse_all(self, round_res, spd, middleware).await;

                    //pipeline put out yield_parse_err and Entity
                    self.plineout(&pipeline).await;

                    // if task is running out, load them from nex buf_task
                    if self.task.lock().unwrap().is_empty() {
                        let vfiles = self.buf_task("data/tasks/");
                        if !vfiles.is_empty() {
                            let file = format!("data/tasks/{}", vfiles[0]);
                            log::warn!("remove used task in {}", file);
                            std::fs::remove_file(&file).unwrap();
                        }
                        if vfiles.len() <= 1 {
                            log::debug!("no task buffer file found. use task_tmp");
                            let mut task_tmp = Vec::new();
                            let mut tmp = self.task_tmp.lock().unwrap();
                            for _ in 0..tmp.len() {
                                let tsk = tmp.pop().unwrap();
                                task_tmp.push(tsk);
                            }
                            drop(tmp);
                            self.task.lock().unwrap().extend(task_tmp);
                        } else if vfiles.len() >= 2 {
                            let file_new = format!("data/tasks/{}", vfiles[1]);
                            log::info!("load new task in {}", file_new);
                            let tsks = Task::load(&file_new);
                            self.task.lock().unwrap().extend(tsks);
                        }
                    }

                    // construct request
                    let len_t = self.task.lock().unwrap().len();
                    let len_p = self.profile.lock().unwrap().len();
                    if len_t != 0 && len_p != 0 {
                        //let gen_request = spd.get_parser(MethodIndex::GenRequest);
                        let round_task = self.rt_args.lock().unwrap().round_task;
                        let reqs =
                            Request::gen(self.profile.clone(), self.task.clone(), round_task);
                        self.req.lock().unwrap().extend(reqs);
                    }

                    //join the older tokio-task
                    let join_gap = self.rt_args.lock().unwrap().join_gap;
                    let join_gap_emer = self.rt_args.lock().unwrap().join_gap_emer;
                    Client::watch(
                        self.fut_res.clone(),
                        self.fut_profile.clone(),
                        join_gap,
                        join_gap_emer,
                    )
                    .await;
                    self.rt_args.lock().unwrap().rate.update();
                    if self.rt_args.lock().unwrap().rate.backup() {
                        self.close(spd, middleware, &pipeline).await;

                        log::info!("backup history...");
                        Profile::stored("data/profile", &mut self.profile);
                        Task::stored("data/task", &mut self.task);
                        Task::stored("data/task_tmp", &mut self.task_tmp);
                        Request::stored("data/request", &mut self.req);
                        Request::stored("data/request_tmp", &mut self.req_tmp);
                    }
                    //self.info();
                    //std::thread::sleep(std::time::Duration::from_millis(150));
                }

                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
