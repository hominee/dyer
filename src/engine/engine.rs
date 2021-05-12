extern crate hyper_timeout;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
extern crate tokio;

use crate::component::{Client, Profile, Request, Response, Task};
use crate::engine::{arg::ArgProfile, ArgApp};
use crate::plugin::Spider;
use crate::plugin::{MiddleWare, PipeLine};
use serde::{Deserialize, Serialize};
use signal_hook::flag as signal_flag;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::task;
use crate::utils;

// H hash of T serve as index for T
// and linkedlist provodes positional infomation for certain hash H
pub(crate) struct AppFut 
{
    /// hash-value pairs
    inner_data: std::collections::HashMap<u64, task::JoinHandle<()>>,
    /// an sorted(ascending) list storing hash and time stamp
    inner_index: std::collections::LinkedList<(u64, f64)>
}
impl AppFut {
    /// create an instance
    pub(crate) fn new() -> Self {
        Self{
            inner_data: std::collections::HashMap::new(),
            inner_index: std::collections::LinkedList::new(),
        }
    }

    /// directly take an value out and feed it to a closure
    /// and update `inner_index`
    pub(crate) async fn direct_join(&mut self, mut ids: Vec<u64>,) {
        let mut raw_results = Vec::with_capacity(ids.len());
        ids.iter().for_each(|id| {
            if let Some(item) = self.inner_data.remove(id) {
                raw_results.push(item);
            }
        }); 
        let mut item_cached = Vec::new();
        while let Some(item) = self.inner_index.pop_front() {
            if ids.contains(&item.0) {
                let p = ids.iter().position(|&x| x == item.0).unwrap();
                ids.remove(p);
            }else {
                item_cached.push(item);
            }
            if ids.is_empty() || self.inner_index.is_empty() {
                break;
            }

        }
        item_cached.into_iter().for_each(|item| self.inner_index.push_back(item));
        Client::join_all(raw_results).await;
    }

    /// execute results from `get_idel` and feed it to a callback
    /// and update `inner_index`
    pub(crate) async fn await_join(&mut self,  gap: f64, capacity: usize) {
        let idels = self.get_idel(gap, capacity);
        if !idels.is_empty() {
            log::info!(
                "joining {} / {} for Response.",
                idels.len(),
                self.inner_index.len() + idels.len(),
            );
            let tasks = idels.into_iter().map(|idel| idel.2 ).collect::<Vec<task::JoinHandle<()>>>();
            Client::join_all(tasks).await;
        }
    }

    /// inset an item and update `inner_data` and `inner_index`
    pub(crate) fn insert(&mut self, item: task::JoinHandle<()>, hash: u64, stamp: f64,) 
    {
        self.inner_data.insert(hash, item);
        let now =  utils::now();
        self.inner_index.push_back(( hash, stamp));
        assert!(self.inner_index.front().unwrap_or(&(0, 0.0)).1 < now);
    }

    /// get no more than `capacity`s idels that longer than `gap`
    fn get_idel(&mut self, gap: f64, capacity: usize) -> Vec<(u64, f64, task::JoinHandle<()>)> {
        let now = utils::now();
        let mut items = Vec::with_capacity(capacity);
        while let Some(item) = self.inner_index.pop_front() {
            if item.1 + gap <= now && items.len() <= capacity{
                let ele = (item.0, item.1, self.inner_data.remove(&item.0).unwrap());
                items.push(ele);

            }else {
                self.inner_index.push_front(item);
                    break;
            }
        }
        if !items.is_empty() {
            log::debug!("availible response length: {}", items.len());
        }
        items
    }
}

/// An abstraction and collection of data flow of `Dyer`,  
pub struct App<E, T, P>
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
    P: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone,
{
    /// a vector of `Task`, store them into directory if too many
    /// in order to lower the memory
    pub task: Arc<Mutex<Vec<Task<T>>>>,
    /// cached `Task`to be used  
    pub task_tmp: Arc<Mutex<Vec<Task<T>>>>,
    /// a vector of `Profile`
    pub profile: Arc<Mutex<Vec<Profile<P>>>>,
    /// a vector of `Request`
    pub req: Arc<Mutex<Vec<Request<T, P>>>>,
    /// cached `Request`to be spawned  
    pub req_tmp: Arc<Mutex<Vec<Request<T, P>>>>,
    /// a vector of `Response`
    pub(crate) res: Arc<Mutex<Vec<Response<T, P>>>>,
    /// collected entities
    pub(crate) entities: Arc<Mutex<Vec<E>>>,
    /// some parse-failed `Response` for manual inspection
    pub(crate) yield_err: Arc<Mutex<Vec<String>>>,
    /// future `Response` with time stamp by which joined forcefully
    //pub fut_res: Arc<Mutex<Vec<(f64, task::JoinHandle<()>)>>>,
    pub(crate) fut_res: AppFut,
    /// future `Profile` with time stamp by which joined forcefully
    //pub fut_profile: Arc<Mutex<Vec<(f64, task::JoinHandle<()>)>>>,
    pub(crate) fut_profile: AppFut,
    /// Some argument to control the data flow
    pub args: ArgApp,
}

impl<'a, E, T, P> App<E, T, P>
where
    T: 'static + Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone + Sync + Send,
    P: 'static + Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone + Sync + Send,
    E: Serialize + std::fmt::Debug + Clone + Send + Sync,
{
    /// create an instance of `App`
    pub fn new() -> Self {
        App {
            task: Arc::new(Mutex::new(Vec::new())),
            task_tmp: Arc::new(Mutex::new(Vec::new())),
            profile: Arc::new(Mutex::new(Vec::new())),
            req: Arc::new(Mutex::new(Vec::new())),
            req_tmp: Arc::new(Mutex::new(Vec::new())),
            res: Arc::new(Mutex::new(Vec::new())),
            entities: Arc::new(Mutex::new(Vec::new())),
            yield_err: Arc::new(Mutex::new(Vec::new())),
            //fut_res: Arc::new(Mutex::new(Vec::new())),
            //fut_profile: Arc::new(Mutex::new(Vec::new())),
            fut_res: AppFut::new(),
            fut_profile: AppFut::new(),
            args: ArgApp::new(),
        }
    }

    /// get the overview of `App`
    pub fn info(&mut self) {
        let mut vs = Vec::new();
        vs.push("App overview: ".to_string());
        let len_task = self.task.lock().unwrap().len();
        if len_task != 0 {
            vs.push(format!("{} Task(s)", len_task));
        }
        let len_task_tmp = self.task_tmp.lock().unwrap().len();
        if len_task_tmp != 0 {
            vs.push(format!("{} cached Task(s)", len_task_tmp));
        }
        let len_profile = self.profile.lock().unwrap().len();
        if len_profile != 0 {
            vs.push(format!("{} Profile(s)", len_profile));
        }
        let len_req = self.req.lock().unwrap().len();
        if len_req != 0 {
            vs.push(format!("{} Request(s)", len_req));
        }
        let len_req_tmp = self.req_tmp.lock().unwrap().len();
        if len_req_tmp != 0 {
            vs.push(format!("{} cached Request(s)", len_req_tmp));
        }
        let len_res = self.res.lock().unwrap().len();
        if len_res != 0 {
            vs.push(format!("{} Response(s)", len_res));
        }
        let len_result = self.entities.lock().unwrap().len();
        if len_result != 0 {
            vs.push(format!("{} Result(s)", len_result));
        }
        let len_yield_err = self.yield_err.lock().unwrap().len();
        if len_yield_err != 0 {
            vs.push(format!("{} yield Error(s)", len_yield_err));
        }
        let len_fut_res = self.fut_res.inner_index.len();
        if len_fut_res != 0 {
            vs.push(format!("{} future Response(s)", len_fut_res));
        }
        let len_fut_profile = self.fut_profile.inner_index.len();
        if len_fut_profile != 0 {
            vs.push(format!("{} future Profile(s)", len_fut_profile));
        }
        if vs.len() == 1 {
            log::info!("{}  {}", vs.join("  "), "empty so far");
        } else {
            log::info!("{}", vs.join("  "));
        }
    }

    /// to see whether to generate `Profile`
    pub fn update_profile(&mut self, spd: &'a dyn Spider<E, T, P>) {
        log::trace!("step into update_profile");
        if let Some(ArgProfile {
            is_on: true,
            profile_min,
            profile_max,
        }) = self.args.arg_profile
        {
            // profile customization is on
            let rd1 = (utils::now() * 3000.0 ) % 1.0;
            let profile_len = self.profile.lock().unwrap().len()
                + self.fut_profile.inner_index.len()
                + self.req.lock().unwrap().len()
                + self.req_tmp.lock().unwrap().len();
            let less = profile_len <= profile_min;
            let exceed = !less && profile_len <= profile_max && rd1 <= 0.333;
            let fut_exceed = profile_len < profile_max;
            let mut emer = false;
            let rd2 = (utils::now() * 3000.0 ) % 1.0;
            if profile_len < self.task.lock().unwrap().len() && rd2 <= 0.01 {
                emer = true;
            }
            if (less || exceed) && fut_exceed || emer {
                let now = utils::now();
                let profile = self.profile.clone();
                log::info!("{} requests spawned for Profile", 3);
                let f = spd.entry_profile();
                let req = f.req.as_ref().expect("Request to generate profile cannot be none when profile customization enabled");
                let salt = [&req.task.uri, &req.task.method, &req.task.able.to_string(), &req.task.trys.to_string()];
                let hash = utils::hash(salt.iter());
                let joinhandle = task::spawn(async move {
                    match Profile::exec_one::<E, T>(f).await {
                        Ok(item) => profile.lock().unwrap().push(item),
                        Err(e) =>log::error!("generate profile failed for: {}", e.desc), 
                    }
                });
                self.fut_profile.insert(joinhandle, hash, now);
            }
        } else {
            // profile customization is off
            // seemingly nothing to do.
        }
    }

    /// drive and consume extracted Entity into `PipeLine`
    pub async fn plineout<C>(&mut self, pipeline: &PipeLine<'a, E, C>)
    where
        C: Send + 'a,
    {
        log::trace!("step into plineout");
        if self.yield_err.lock().unwrap().len() > self.args.round_yield_err {
            log::debug!("pipeline put out yield_parse_err");
            (pipeline.process_yerr)(&mut self.yield_err).await;
        }
        if self.entities.lock().unwrap().len() > self.args.round_entity {
            self.info();
            log::debug!(
                "pipeline put out {} Results",
                self.entities.lock().unwrap().len()
            );
            (pipeline.process_item)(&mut self.entities).await;
        }
    }

    /// load and balance `Request`
    pub async fn update_req(&mut self, middleware: &MiddleWare<'a, E, T, P>) {
        log::trace!("step into update_req");
        let now = utils::now();
        let len_req_tmp = self.req_tmp.lock().unwrap().len();
        if len_req_tmp <= self.args.round_req_min {
            // cached request is not enough
            let len_req = self.req.lock().unwrap().len();
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
            let (buf_task, buf_pfile) = (middleware.handle_req)(&mut requests, self).await;
            let req_len = requests.len();
            if req_len != 0 {
                self.req_tmp.lock().unwrap().extend(requests);
                log::debug!("take {} request from request to cached request", req_len);
            }
            self.task.lock().unwrap().extend(buf_task);
            if let Some(ArgProfile { is_on: true, .. }) = self.args.arg_profile {
                self.profile.lock().unwrap().extend(buf_pfile);
            }
        }
    }

    /// construct request
    pub fn gen_req(&mut self) {
        log::trace!("step into gen_req");
        let use_device = if let Some(ArgProfile { is_on: true, .. }) = self.args.arg_profile {
            true
        } else {
            false
        };
        let len_task = self.task.lock().unwrap().len();
        let len_profile = self.profile.lock().unwrap().len();
        if (use_device && len_task != 0 && len_profile != 0) || (!use_device && len_task != 0) {
            let round_task = self.args.round_task;
            let reqs = Request::gen(
                self.profile.clone(),
                self.task.clone(),
                round_task,
                use_device,
            );
            self.req.lock().unwrap().extend(reqs);
        }
    }

    /// spawn polling `Request` as `tokio::task` and executing asynchronously,
    pub async fn spawn_task(&mut self) {
        log::trace!("step into spawn_task");
        if self.fut_res.inner_index.len() > self.args.spawn_task_max {
            log::warn!("enough Future Response, spawn no task.");
        } else {
            log::trace!("take request out to be executed.");
            //let now = utils::now();
            //let mut req_tmp = self.req_tmp.lock().unwrap();
            let len = self.args.round_req.min(self.req_tmp.lock().unwrap().len());
            let len_load = self.args.rate.lock().unwrap().get_len(None).min(len);
            if len_load > 0 {
                log::info!("{} requests spawned", len_load);
                std::iter::repeat(0).take(len_load).into_iter().for_each(|_| {
                    let now = utils::now();
                    let req = self.req_tmp.lock().unwrap().pop().unwrap();
                    let salt = [&req.task.uri, &req.task.method, &req.task.able.to_string(), &req.task.trys.to_string()];
                    let hash = utils::hash(salt.iter());
                    let app_arg = self.args.rate.clone();
                    let app_res = self.res.clone();
                    let joinhandle = task::spawn(async move {
                        let (res, gap) = Client::exec_one(req).await;
                        app_res.lock().unwrap().push(res);
                        app_arg.lock().unwrap().stamps.push(gap);
                    });
                    self.fut_res.insert(joinhandle, hash, now);
                });
            }
        }
    }

    ///join spawned task, once it exceed the timing `threshold`, then forcefully join it 
    pub async fn watch(&mut self) {
        log::trace!("step into watch");
        let threshold_tokio_task = self.args.join_gap;
        let capacity = self.args.round_req;
        if !self.fut_res.inner_index.is_empty() {
            self.fut_res.await_join(threshold_tokio_task, capacity).await;
        }
        if !self.fut_profile.inner_index.is_empty() {
            self.fut_profile.await_join(threshold_tokio_task, capacity).await;
        }
    }

    /// load cached `Task` from caced directory
    pub fn buf_task(&self) -> Vec<usize> {
        log::trace!("step into buf_task");
        let path = format!("{}/tasks/", self.args.data_dir);
        let mut file_indexs: Vec<usize> = Vec::new();
        if let Ok(items) = std::fs::read_dir(path) {
            items.for_each(|name| {
                let index = name
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                file_indexs.push(index);
            });
        }
        file_indexs
    }

    /// update task in App
    pub fn update_task(&mut self) {
        log::trace!("step into update_task");
        let path = format!("{}/tasks/", self.args.data_dir);
        if self.task.lock().unwrap().is_empty() {
            let file_indexs: Vec<usize> = self.buf_task();
            if !file_indexs.is_empty() {
                let file = format!("{}{}", path, file_indexs[0]);
                log::warn!("remove used task in {}", file);
                std::fs::remove_file(&file).unwrap();
            }
            if file_indexs.len() <= 1 {
                let mut task_tmp = Vec::new();
                let mut tmp = self.task_tmp.lock().unwrap();
                for _ in 0..tmp.len() {
                    let tsk = tmp.pop().unwrap();
                    task_tmp.push(tsk);
                }
                if !task_tmp.is_empty() {
                    log::debug!(
                        "no task buffer file found. load {} tasks from task_tmp",
                        task_tmp.len()
                    );
                }
                self.task.lock().unwrap().extend(task_tmp);
            } else if file_indexs.len() >= 2 {
                let file_new = format!("{}tasks/{}", self.args.data_dir, file_indexs[1]);
                let tsks = Task::load(&file_new);
                log::info!("load {} new task in {}", tsks.len(), file_new);
                self.task.lock().unwrap().extend(tsks);
            }
        }
        if self.task_tmp.lock().unwrap().len() >= self.args.buf_task_tmp {
            log::debug!("pipeline out buffered task.");
            let files = self.buf_task();
            let file_name = format!(
                "{}tasks/{}",
                self.args.data_dir,
                1 + files.last().unwrap_or(&0)
            );
            Task::stored(&file_name, &mut self.task_tmp);
            self.task_tmp.lock().unwrap().clear();
        }
    }

    /// preparation before closing `Dyer`
    pub async fn close<'b, C>(
        &'a mut self,
        spd: &'a dyn Spider<E, T, P>,
        middleware: &'a MiddleWare<'b, E, T, P>,
        pipeline: &'a PipeLine<'b, E, C>,
    ) where
        C: Send + 'b,
    {
        log::trace!("step into close");
        self.info();
        Response::parse_all(self, usize::MAX, spd, middleware).await;
        log::info!("sending all of them into Pipeline");
        (pipeline.process_yerr)(&mut self.yield_err).await;
        (pipeline.process_item)(&mut self.entities).await;
        (pipeline.close_pipeline)().await;
        log::info!("All work is Done.");
    }

    /// drive `Dyer` into running.
    pub async fn run<'b, C>(
        &'a mut self,
        spd: &'a dyn Spider<E, T, P>,
        middleware: &'a MiddleWare<'b, E, T, P>,
        pipeline: PipeLine<'b, E, C>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        C: Send + 'a,
    {
        // signal handling initial
        let term = Arc::new(AtomicUsize::new(0));
        const SIGINT: usize = signal_hook::SIGINT as usize;
        signal_flag::register_usize(signal_hook::SIGINT, Arc::clone(&term), SIGINT).unwrap();

        // user defined preparation when open spider
        spd.open_spider(self);

        //skip the history and start new fields to staart with, some Profile required
        if self.args.skip_history {
            log::warn!("skipped the history.");
            if let Some(ArgProfile { is_on: true, .. }) = self.args.arg_profile {
                Profile::exec_all::<E, T>(self.profile.clone(), 3usize, spd.entry_profile()).await;
            }
            let tasks = spd.entry_task().unwrap();
            self.task.lock().unwrap().extend(tasks);
            log::info!("new session started");
            self.info();
        } else {
            log::warn!("use the history file.");
            let path = format!("{}request", self.args.data_dir);
            let reqs: Vec<Request<T, P>> = Request::load(&path);
            let path = format!("{}request_tmp", self.args.data_dir);
            let req_tmp: Vec<Request<T, P>> = Request::load(&path);
            let path = format!("{}profile", self.args.data_dir);
            let profile: Vec<Profile<P>> = Profile::load(&path);
            let files = self.buf_task();
            let file = format!("{}/{}", self.args.data_dir, files[0]);
            let task: Vec<Task<T>> = Task::load(&file);
            log::info!("{} loaded {} Task.", file, task.len());
            self.task = Arc::new(Mutex::new(task));
            self.req = Arc::new(Mutex::new(reqs));
            self.req_tmp = Arc::new(Mutex::new(req_tmp));
            self.profile = Arc::new(Mutex::new(profile));
            log::info!("the history files are loaded");
            self.info();
        }

        loop {
            match term.load(Ordering::Relaxed) {
                SIGINT => {
                    // receive the Ctrl+c signal
                    // by default  request  task profile and result yield err are going to stroed into
                    // file
                    log::info!("receive Ctrl+c signal, preparing ...");

                    //finish remaining futures
                    let capacity = self.args.round_req;
                    while !self.fut_res.inner_data.is_empty() {
                        self.fut_res.await_join(0.0, capacity).await;
                    }
                    log::info!("join all future response");

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
                        && self.fut_res.inner_index.is_empty()
                        //&& self.fut_profile.lock().unwrap().is_empty()
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
                    // TODO middleware return profile
                    self.update_req(middleware).await;

                    //take req out to finish
                    self.spawn_task().await;

                    // before we construct request check profile first
                    self.update_profile(spd);

                    // parse response
                    //extract the parseResult
                    let round_res = self.args.round_res;
                    Response::parse_all(self, round_res, spd, middleware).await;

                    //pipeline put out yield_parse_err and Entity
                    self.plineout(&pipeline).await;

                    // if task is running out, load them from nex buf_task
                    self.update_task();

                    // construct request
                    self.gen_req();

                    //join the older jobs
                    self.watch().await;

                    // update Rate
                    let updated = self.args.rate.lock().unwrap().update();

                    // update config file in each interval
                    if updated {
                        let data_dir =  self.args.data_dir.clone();
                        self.args.parse_config(Some(&data_dir), true);
                    }

                    // to backup history file or not
                    if self.args.rate.lock().unwrap().backup() {
                        self.close(spd, middleware, &pipeline).await;

                        log::info!("backup history...");
                        let path = format!("{}profile", self.args.data_dir);
                        Profile::stored(&path, &mut self.profile);
                        let path = format!("{}task", self.args.data_dir);
                        Task::stored(&path, &mut self.task);
                        let path = format!("{}task_tmp", self.args.data_dir);
                        Task::stored(&path, &mut self.task_tmp);
                        let path = format!("{}request", self.args.data_dir);
                        Request::stored(&path, &mut self.req);
                        let path = format!("{}request_tmp", self.args.data_dir);
                        Request::stored(&path, &mut self.req_tmp);
                    }
                }

                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
