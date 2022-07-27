//! the core of `Dyer`, that drives almost the events of data flow, including dispathing parser to
//! parse `Response`, generating `Affix`,
//! generating `Task`, preparation before opening actor, affairs before closing actor.  

use crate::component::{body::Body, couple::Couple, Affix, Poly, Request, Response, Task};
use crate::engine::Actor;
use crate::engine::{appfut::AppFut, arg::ArgAffix, vault::Vault, ArgApp};
use crate::plugin::Affixor;
use crate::plugin::{MiddleWare, PipeLine};
use crate::response::MetaResponse;
use crate::utils;
use crate::Parsed;
use http::Extensions;
use signal_hook::flag as signal_flag;
use std::collections::HashMap;
use std::error::Error;
use std::iter::FromIterator;
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicUsize, Ordering},
};

/// An abstraction and collection of data flow  
pub struct App<E> {
    /// a vector of `Task`, store them into directory if too many
    /// in order to lower the memory
    pub task: Vault<VecDeque<Task>>,
    /// cached `Task`to be used  
    pub task_tmp: Vault<Vec<Task>>,
    /// a vector of `Affix`
    pub affix: Vault<VecDeque<Affix>>,
    /// a vector of `Request`
    pub req: Vault<VecDeque<Request>>,
    /// cached `Request`to be spawned  
    pub req_tmp: Vault<Vec<Request>>,
    /// a vector of `Response`
    pub res: Vault<Vec<Result<Response, MetaResponse>>>,
    /// collected entities
    pub entities: Vault<Vec<E>>,
    /// some parse-failed `Response` for manual inspection
    pub errs: Vault<Vec<Result<Response, MetaResponse>>>,
    /// future `Response` with time stamp by which joined forcefully
    pub(crate) fut_res: AppFut,
    /// future `Affix` with time stamp by which joined forcefully
    pub(crate) fut_affix: AppFut,
    /// Some argument to control the data flow
    pub args: ArgApp,
    /// couples of task and affix,
    /// serving as a backup when the constructed request executed failed
    pub couple: Vault<HashMap<u64, Couple>>,
    /// periodically called to backup `Poly`
    /// NOTE that if `None` all `Poly` lost at runtime when interupts happens  
    pub session_storer: Option<Box<dyn for<'a> Fn(Poly, &'a ()) -> &'a str + Send>>,
    /// called to load `Poly` at resuming session or load `Task` to execute
    /// NOTE that if `None` App Starts with new session
    pub session_loader: Option<Box<dyn Fn(&str) -> Poly + Send>>,
    /// modify the body of [Task], [Affix] in [Couple]
    /// return the [Request]'s [Body]
    /// if not set, just simply concat the two of them
    pub body_modifier: Option<Box<dyn for<'c, 'd> Fn(&'c Body, Option<&'d Body>) -> Body + Send>>,
    pub(crate) exts_t_fn: Option<
        Box<dyn for<'c, 'd> Fn(&'c Extensions, &'d Extensions) -> (Extensions, Extensions) + Send>,
    >,
    pub(crate) exts_p_fn: Option<
        Box<dyn for<'c, 'd> Fn(&'c Extensions, &'d Extensions) -> (Extensions, Extensions) + Send>,
    >,
    //pool: ThreadPool,
}

impl<'a, E> App<E> {
    /// create an instance of `App`
    pub fn new() -> Self {
        App {
            task: Vault::new(VecDeque::new()),
            task_tmp: Vault::new(Vec::new()),
            affix: Vault::new(VecDeque::new()),
            req: Vault::new(VecDeque::new()),
            req_tmp: Vault::new(Vec::new()),
            res: Vault::new(Vec::new()),
            entities: Vault::new(Vec::new()),
            errs: Vault::new(Vec::new()),
            fut_res: AppFut::new(),
            fut_affix: AppFut::new(),
            couple: Vault::new(HashMap::new()),
            args: ArgApp::new(),
            session_storer: None,
            session_loader: None,
            body_modifier: None,
            exts_t_fn: None,
            exts_p_fn: None,
            //pool: ThreadPool::new().unwrap(),
        }
    }

    /// add an Actor
    pub fn add_actor<A>(&mut self, spd: &dyn Actor<E, A>)
    where
        A: Affixor + Send + 'static,
    {
        let _ = spd;
    }

    /// set the Session Loader
    pub fn session_loader(&mut self, loader: Box<dyn Fn(&str) -> Poly + Send>) {
        self.session_loader = Some(loader);
    }

    /// set the Session Storer
    pub fn session_storer(&mut self, storer: Box<dyn for<'b> Fn(Poly, &'b ()) -> &'b str + Send>) {
        self.session_storer = Some(storer);
    }

    /// set the task extension handler
    /// when make a couple of Task and Affix
    pub fn exts_t(
        &mut self,
        f: Box<
            dyn for<'c, 'd> Fn(&'c Extensions, &'d Extensions) -> (Extensions, Extensions) + Send,
        >,
    ) {
        self.exts_t_fn = Some(f);
    }

    /// set the affix extension handler
    /// when make a couple of Task and Affix
    pub fn exts_p(
        &mut self,
        f: Box<
            dyn for<'c, 'd> Fn(&'c Extensions, &'d Extensions) -> (Extensions, Extensions) + Send,
        >,
    ) {
        self.exts_p_fn = Some(f);
    }

    /// modify the body of [Task], [Affix] in [Couple]
    /// return the [Request]'s [Body]
    /// if not set, just simply concat the two of them
    /// get the overview of `App`
    pub fn body_modifier(
        &mut self,
        f: Box<dyn for<'c, 'd> Fn(&'c Body, Option<&'d Body>) -> Body + Send>,
    ) {
        self.body_modifier = Some(f);
    }

    fn info(&mut self) {
        let mut vs = Vec::new();
        vs.push("Stats Overview:".to_string());
        let len_task = self.task.as_ref().len();
        if len_task != 0 {
            vs.push(format!("{} Task(s)", len_task));
        }
        let len_task_tmp = self.task_tmp.as_ref().len();
        if len_task_tmp != 0 {
            vs.push(format!("{} Cached Task(s)", len_task_tmp));
        }
        let len_affix = self.affix.as_ref().len();
        if len_affix != 0 {
            vs.push(format!("{} Affix(s)", len_affix));
        }
        let len_req = self.req.as_ref().len();
        if len_req != 0 {
            vs.push(format!("{} Request(s)", len_req));
        }
        let len_req_tmp = self.req_tmp.as_ref().len();
        if len_req_tmp != 0 {
            vs.push(format!("{} Cached Request(s)", len_req_tmp));
        }
        let len_res = self.res.as_ref().len();
        if len_res != 0 {
            vs.push(format!("{} Response(s)", len_res));
        }
        let len_result = self.entities.as_ref().len();
        if len_result != 0 {
            vs.push(format!("{} Result(s)", len_result));
        }
        let len_errs = self.errs.as_ref().len();
        if len_errs != 0 {
            vs.push(format!("{} Yield Error(s)", len_errs));
        }
        let len_couple = self.couple.len();
        if len_couple != 0 {
            vs.push(format!("{} Buffered Couple(s)", len_couple));
        }
        let len_fut_res = self.fut_res.index.len();
        if len_fut_res != 0 {
            vs.push(format!("{} Future Response(s)", len_fut_res));
        }
        let len_fut_affix = self.fut_affix.index.len();
        if len_fut_affix != 0 {
            vs.push(format!("{} Future Affix(s)", len_fut_affix));
        }
        if vs.len() == 1 {
            log::info!("{}  {}", vs.join("  "), "empty so far");
        } else {
            log::info!("{}", vs.join("  "));
        }
    }

    /// to see whether to generate `Affix`
    async fn update_affix<A>(&mut self, spd: &'a mut dyn Actor<E, A>)
    where
        A: Affixor + Send + 'static,
    {
        log::trace!("Step into update_affix");
        if spd.entry_affix().await.is_none() || !self.args.affix_on() {
            return;
        }
        if let Some(ArgAffix {
            is_on: true,
            affix_min,
            affix_max,
        }) = self.args.arg_affix
        {
            // affix customization is on
            let rd1 = (utils::now() * 3000.0) % 1.0;
            let affix_len = self.affix.as_ref().len()
                + self.fut_affix.index.len()
                + self.req.as_ref().len()
                + self.req_tmp.as_ref().len();
            let less = affix_len <= affix_min;
            let exceed = !less && affix_len <= affix_max && rd1 <= 0.333;
            let fut_exceed = affix_len < affix_max;
            let mut emer = false;
            let rd2 = (utils::now() * 3000.0) % 1.0;
            if affix_len < self.task.as_ref().len() && rd2 <= 0.03 {
                emer = true;
            }
            if (less || exceed) && fut_exceed || emer {
                let now = utils::now();
                log::info!("{} requests spawned for Affix", 3);
                let mut actor = spd.entry_affix().await.unwrap();
                if let Some(mut req) = actor.invoke().await {
                    // use network-based way to generate affix
                    let mut affix = self.affix.clone();
                    let hash = req.metar.info.id;
                    actor.after_invoke().await;
                    let handle = tokio::spawn(async move {
                        //let handle = self .pool .spawn_with_handle(async move {
                        // generate one `Affix`
                        // construct a new reqeust
                        log::trace!("Request that to generate Affix: {:?}", req.inner);
                        let client = req.get_client();
                        let mut res = client.request(req).await;
                        actor.before_parse(Some(&mut res)).await;
                        if let Some(item) = actor.parse(Some(res)).await {
                            log::info!("Affix {}  generated", item.metap.info.id);
                            affix.as_mut().push_back(item);
                            actor.after_parse().await;
                        } else {
                            log::debug!("Affix not generated",);
                        }
                    });
                    self.fut_affix.insert(handle, hash, now);
                } else {
                    // non network-way
                    actor.after_invoke().await;
                    actor.before_parse(None).await;
                    if let Some(item) = actor.parse(None).await {
                        log::info!("Affix {}  generated", item.metap.info.id);
                        self.affix.as_mut().push_back(item);
                        actor.after_parse().await;
                    } else {
                        log::debug!("Affix not generated",);
                    }
                }
            }
        } else {
            // affix customization is off
            // seemingly nothing to do.
        }
    }

    /// drive and consume extracted Entity into `PipeLine`
    async fn plineout<'b, C>(&mut self, pipeline: &PipeLine<'b, E, C>) {
        log::trace!("Step into plineout");
        if self.errs.as_ref().len() > self.args.round_errs {
            log::info!("Pipeline put out yield_parse_err");
            let mut yerrs = Vec::new();
            self.errs.update(|es| {
                while let Some(e) = es.pop() {
                    yerrs.push(e);
                }
            });
            //std::mem::swap(&mut yerrs, &mut *self.errs);
            if let Some(ff) = pipeline.yerr() {
                ff(yerrs, self).await;
            }
        }
        if self.entities.as_ref().len() >= self.args.round_entity {
            self.info();
            log::info!("Data Pipeline Dumping: {} ", self.entities.as_ref().len());
            let mut ens = Vec::new();
            self.entities.update(|es| {
                while let Some(e) = es.pop() {
                    ens.push(e);
                }
            });
            if let Some(ff) = pipeline.entity() {
                ff(ens, self).await;
            }
        }
    }

    /// load and balance `Request`
    async fn update_req<'b>(&mut self, middleware: &'b MiddleWare<'b, E>) {
        log::trace!("Step into update_req");
        let now = utils::now();
        let len_req_tmp = self.req_tmp.as_ref().len();
        if len_req_tmp <= self.args.round_req_min {
            // cached request is not enough
            let len_req = self.req.as_ref().len();
            let mut requests = Vec::new();

            //  limit len_req and reqs that is availible by now
            for _ in 0..len_req {
                let request = self.req.as_mut().pop_front().unwrap();
                if request.metar.info.able <= now {
                    requests.push(request);
                } else {
                    self.req.as_mut().push_front(request);
                    break;
                }
            }
            if let Some(ff) = middleware.req() {
                ff(&mut requests, self).await;
            }
            let req_len = requests.len();
            if req_len > 0 {
                self.req_tmp.as_mut().extend(requests);
                log::debug!("Take {} request from request to cached request", req_len);
            }
        }
    }

    /// construct request
    fn gen_req<'b>(&mut self) {
        log::trace!("Step into gen_req");
        let affix_on = self.args.affix_on();
        let len_task = self.task.as_ref().len();
        let len_affix = self.affix.as_ref().len();
        let round_task = self.args.round_task;
        let len = usize::min(len_task, round_task);
        let mut reqs = Vec::new();
        if affix_on && len.min(len_affix) > 0 {
            let len = len.min(len_affix);
            log::debug!("Creating {} request", len);
            for _ in 0..len {
                let now = utils::now();
                let affix = self.affix.as_mut().pop_back().unwrap();
                if affix.metap.info.able > now {
                    // not available right now
                    self.affix.as_mut().push_front(affix);
                    break;
                }
                let task = self.task.as_mut().pop_back().unwrap();
                if task.metat.info.able > now {
                    // not available right now
                    self.task.as_mut().push_front(task);
                    break;
                }
                let couple = Couple::new(task, Some(affix));
                let req = Request::from_couple(
                    &couple,
                    self.body_modifier.as_ref(),
                    self.exts_t_fn.as_ref(),
                    self.exts_p_fn.as_ref(),
                );
                self.couple.insert(couple.id, couple);
                log::debug!("Created Request: {:?}", req);
                reqs.push(req);
            }
        } else if !affix_on && len > 0 {
            log::debug!("Creating {} request", len);
            for _ in 0..len {
                let now = utils::now();
                let task = self.task.as_mut().pop_back().unwrap();
                if task.metat.info.able > now {
                    // not available right now
                    self.task.as_mut().push_front(task);
                    break;
                }
                let couple = Couple::new(task, None);
                let req = Request::from_couple(
                    &couple,
                    self.body_modifier.as_ref(),
                    self.exts_t_fn.as_ref(),
                    self.exts_p_fn.as_ref(),
                );
                log::trace!("Created request: {:?}", req);
                reqs.push(req);
                self.couple.insert(couple.id, couple);
            }
        }
        self.req.as_mut().extend(reqs);
    }

    /// spawn polling `Request` as `tokio::task` and executing asynchronously,
    async fn spawn_task(&mut self) {
        log::trace!("Step into spawn_task");
        if self.fut_res.index.len() > self.args.spawn_task_max {
            if self.args.rate.as_mut().update() {
                log::warn!("Enough Future Response, spawn no task.");
            }
            return;
        }
        log::trace!("Take request out to be executed.");
        self.req_tmp
            .as_mut()
            .sort_by(|a, b| a.info().rank.cmp(&b.info().rank));
        let len = self.args.round_req.min(self.req_tmp.as_ref().len());
        let len_load = self.args.rate.as_mut().get_len(None).min(len);
        for _ in 0..len_load {
            let now = utils::now();
            let mut req = self.req_tmp.as_mut().pop().unwrap();
            let hash = req.metar.info.id;
            let mut app_arg = self.args.rate.clone();
            let mut app_res = self.res.clone();
            //let mut couple = self.couple.clone();
            let handle = tokio::spawn(async move {
                //let handle = self .pool .spawn_with_handle(async move {
                log::info!("Crawling requests: {} ", &req.inner.uri);
                let client = req.get_client();
                match client.request(req).await {
                    Ok(res) => {
                        app_arg.as_mut().stamps.push(res.metas.info.gap);
                        app_res.as_mut().push(Ok(res));
                    }
                    Err(mta) => {
                        log::error!("request Failed: {:?}", mta.info.from);
                        app_res.as_mut().push(Err(mta));
                    }
                }
            });
            self.fut_res.insert(handle, hash, now);
        }
    }

    /// specifically, dispose a `Response`, handle failed or corrupt `Response`, and return `Parsed` or `ParseError`.
    pub async fn parse<'b>(&self, res: Response) -> (Parsed<E>, u64) {
        log::info!("Successful crawled: {}", &res.metas.info.from.to_string());
        let hash = res.metas.info.id;
        let ptr = res.metas.parser.clone();
        let parser = unsafe { std::mem::transmute::<*const (), fn(Response) -> Parsed<E>>(ptr) };
        ((parser)(res), hash)
    }

    /// parse multiple `Response` in `App`, then drive all `Parsed` into `MiddleWare`
    pub async fn parse_all<'b>(&mut self, mware: &'b MiddleWare<'b, E>) {
        let round = self.args.round_res;
        let mut v = Vec::new();
        let mut tsks = Vec::new();
        let mut pfiles = Vec::new();
        let mut reqs = Vec::new();
        let mut yerr = Vec::new();
        let mut ens = Vec::new();
        let mut errs = Vec::new();
        let mut hashes = Vec::new();

        let len = self.res.as_ref().len().min(round);
        for _ in 0..len {
            match self.res.as_mut().pop().unwrap() {
                Ok(item) => {
                    let status = item.status().as_u16();
                    let id = item.metas.info.id;
                    if status >= 200 && status < 300 {
                        self.couple.remove(&id);
                        v.push(item);
                        continue;
                    }
                    errs.push(Ok(item));
                }
                Err(meta) => {
                    errs.push(Err(meta));
                }
            }
        }
        if !errs.is_empty() {
            if let Some(ff) = mware.err() {
                ff(&mut errs, self).await;
            }
        }
        if !v.is_empty() {
            if let Some(ff) = mware.res() {
                ff(&mut v, self).await;
            }
        }
        while let Some(res) = v.pop() {
            let (prs, hash) = self.parse(res).await;
            log::trace!("response parsed: {}", hash);
            hashes.push(hash);
            tsks.extend(prs.task);
            pfiles.extend(prs.affix);
            reqs.extend(prs.req);
            yerr.extend(prs.errs);
            ens.extend(prs.entities);
        }
        if !hashes.is_empty() {
            self.fut_res.direct_join(hashes).await;
        }
        if !reqs.is_empty() {
            if let Some(ff) = mware.req() {
                ff(&mut reqs, self).await;
            }
            self.req.as_mut().extend(reqs);
        }
        if !pfiles.is_empty() {
            if let Some(ff) = mware.affix() {
                ff(&mut pfiles, self).await;
            }
            self.affix.as_mut().extend(pfiles);
        }
        if !tsks.is_empty() {
            if let Some(ff) = mware.task() {
                ff(&mut tsks, self).await;
            }
            self.task_tmp.as_mut().extend(tsks);
        }
        if !ens.is_empty() {
            if let Some(ff) = mware.entity() {
                ff(&mut ens, self).await;
            }
            self.entities.as_mut().extend(ens);
        }
        if !yerr.is_empty() {
            self.errs.as_mut().extend(yerr);
        }
    }

    ///join spawned task, once it exceed the timing `join_gap`, then forcefully join it
    async fn watch(&mut self) {
        log::trace!("Step into watch");
        let threshold_tokio_task = self.args.join_gap;
        let capacity = self.args.round_req;
        if !self.fut_res.index.is_empty() {
            self.fut_res.cancell(threshold_tokio_task, capacity);
            //self.fut_res.all(threshold_tokio_task, capacity).await;
        }
        if !self.fut_affix.index.is_empty() {
            self.fut_affix.cancell(threshold_tokio_task, capacity);
            //self.fut_affix.all(threshold_tokio_task, capacity).await;
        }
    }

    /// load cached `Task` from caced directory
    fn buf_task(&self) -> Vec<usize> {
        log::trace!("Step into buf_task");
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
    fn update_task(&mut self) {
        log::trace!("Step into update_task");
        let path = format!("{}/tasks/", self.args.data_dir);
        if self.task.as_ref().is_empty() {
            let file_indexs: Vec<usize> = self.buf_task();
            if !file_indexs.is_empty() {
                let file = format!("{}{}", path, file_indexs[0]);
                log::warn!("Remove used task in {}", file);
                std::fs::remove_file(&file).unwrap();
            }
            if file_indexs.len() <= 1 {
                let mut task_tmp = Vec::new();
                let mut tmp = self.task_tmp.as_mut();
                for _ in 0..tmp.len() {
                    let tsk = tmp.pop().unwrap();
                    task_tmp.push(tsk);
                }
                if !task_tmp.is_empty() {
                    log::debug!(
                        "No task buffer file found. Load {} tasks from task_tmp",
                        task_tmp.len()
                    );
                }
                self.task.as_mut().extend(task_tmp);
            } else if file_indexs.len() >= 2 {
                let file_new = format!("{}tasks/{}", self.args.data_dir, file_indexs[1]);
                let tsks = utils::load(&file_new, self.session_loader.as_ref());
                log::info!("Load {} new task in {}", tsks.len(), file_new);
                self.task.as_mut().extend(tsks);
            }
        }
        if self.task_tmp.as_ref().len() >= self.args.buf_task {
            log::debug!(
                "Buffered Task Pipeline Dumping: {}",
                self.task_tmp.as_ref().len()
            );
            let files = self.buf_task();
            let file_name = format!(
                "{}tasks/{}",
                self.args.data_dir,
                1 + files.last().unwrap_or(&0)
            );
            utils::stored(&file_name, &mut self.task_tmp, self.session_storer.as_ref());
            self.task_tmp.as_mut().clear();
        }
    }

    /// check all necessary exit conditions
    fn exit(&self) -> bool {
        self.req.as_ref().is_empty()
            && self.req_tmp.as_ref().is_empty()
            && self.task.as_ref().is_empty()
            && self.task_tmp.as_ref().is_empty()
            && self.fut_res.index.is_empty()
            && self.res.as_ref().is_empty()
    }

    /// preparation before closing `Dyer`
    async fn close<'b, C, A>(
        &'a mut self,
        spd: &'a mut dyn Actor<E, A>,
        middleware: &'a MiddleWare<'b, E>,
        pipeline: &'a PipeLine<'b, E, C>,
    ) where
        A: Affixor + Send + 'static,
    {
        log::trace!("Step into close");
        self.info();
        if let Some(mut actor) = spd.entry_affix().await {
            actor.close().await;
        }
        self.parse_all(middleware).await;
        log::info!("Pipeline Data Dumping");
        let mut yerrs = Vec::new();
        self.errs.update(|es| {
            while let Some(e) = es.pop() {
                yerrs.push(e);
            }
        });
        if let Some(ff) = pipeline.yerr() {
            ff(yerrs, self).await;
        }
        let mut ens = Vec::new();
        self.entities.update(|es| {
            while let Some(e) = es.pop() {
                ens.push(e);
            }
        });
        if let Some(ff) = pipeline.entity() {
            ff(ens, self).await;
        }
        if let Some(ff) = pipeline.disposer() {
            ff(self).await;
        }
        log::info!("Clean the App");
    }

    /// drive `Dyer` into running.
    pub async fn run<'b, C, A>(
        &'a mut self,
        spd: &'a mut dyn Actor<E, A>,
        middleware: &'a MiddleWare<'b, E>,
        pipeline: &'a PipeLine<'b, E, C>,
    ) -> Result<(), Box<dyn Error>>
    where
        A: Affixor + Send + 'static,
    {
        // signal handling initial
        let term = std::sync::Arc::new(AtomicUsize::new(0));
        const SIGINT: usize = signal_hook::SIGINT as usize;
        signal_flag::register_usize(signal_hook::SIGINT, term.clone(), SIGINT).unwrap();

        // user defined preparation when open actor
        spd.open_actor(self).await;
        if let Some(mut actor) = spd.entry_affix().await {
            actor.init().await;
        }

        //skip the history and start new fields to staart with, some Affix required
        if self.args.skip {
            log::info!("New Session Started");
            let mut tasks = spd.entry_task().await.unwrap();
            if let Some(ff) = middleware.task() {
                ff(&mut tasks, self).await;
            }
            self.task.as_mut().extend(tasks);
            self.info();
        } else {
            log::info!("Resuming The Session");
            let path = format!("{}request", self.args.data_dir);
            let reqs: Vec<Request> = utils::load(&path, self.session_loader.as_ref());
            let path = format!("{}request_tmp", self.args.data_dir);
            let req_tmp: Vec<Request> = utils::load(&path, self.session_loader.as_ref());
            let path = format!("{}affix", self.args.data_dir);
            let affix: Vec<Affix> = utils::load(&path, self.session_loader.as_ref());
            let path = format!("{}couple", self.args.data_dir);
            let couples: Vec<(u64, Couple)> = utils::load(&path, self.session_loader.as_ref());
            let files = self.buf_task();
            let file = format!("{}/{}", self.args.data_dir, files[0]);
            let task: Vec<Task> = utils::load(&file, self.session_loader.as_ref());
            log::info!("{} loaded {} Task.", file, task.len());
            self.task
                .append(&mut task.into_iter().collect::<VecDeque<_>>());
            self.req
                .append(&mut reqs.into_iter().collect::<VecDeque<_>>());
            self.couple
                .replace(HashMap::<u64, Couple>::from_iter(couples));
            *self.req_tmp = req_tmp;
            self.affix
                .append(&mut affix.into_iter().collect::<VecDeque<_>>());
            log::info!("History Files Loaded");
            self.info();
        }

        loop {
            match term.load(Ordering::Relaxed) {
                SIGINT => {
                    // receive the Ctrl+c signal
                    // by default request task affix
                    // and result yield err are going to stroed into file
                    log::info!("Receive Ctrl+c Signal, Preparing Exit ...");

                    //finish remaining futures
                    log::info!("Joining All Futures");
                    let capacity = self.args.round_req;
                    while !self.fut_res.data.is_empty() {
                        self.fut_res.all(9999999.0, capacity).await;
                    }

                    // dispath them
                    log::info!("Closing Actor ...");
                    self.close(spd, middleware, &pipeline).await;
                    spd.close_actor(self).await;
                    log::info!("All Work Is Done, Exiting ...");
                    break;
                }

                0 => {
                    // if all task request and other things are done the quit
                    if self.exit() {
                        log::info!("Closing Actor ...");
                        self.close(spd, middleware, &pipeline).await;
                        spd.close_actor(self).await;
                        log::info!("All Work Is Done, Exiting ...");
                        break;
                    }

                    // before we update request check affix first
                    self.update_affix(spd).await;

                    // consume valid request in cbase_reqs_tmp
                    // if not enough take them from self.req
                    self.update_req(middleware).await;

                    //take req out to finish
                    self.spawn_task().await;

                    //pipeline put out yield_parse_err and Entity
                    self.plineout(&pipeline).await;

                    // if task is running out, load them from nex buf_task
                    self.update_task();

                    // construct request
                    self.gen_req();

                    //join the older jobs
                    self.watch().await;

                    self.parse_all(middleware).await;

                    // update Rate
                    let updated = self.args.rate.as_mut().update();

                    // update config file in each interval
                    if updated {
                        let data_dir = self.args.data_dir.clone();
                        self.args.parse_config(Some(&data_dir), true);
                    }

                    // to backup history file or not
                    if self.args.rate.as_mut().backup() && self.session_storer.is_some() {
                        self.close(spd, middleware, pipeline).await;
                        log::info!("Backup History...");
                        let path = format!("{}affix", self.args.data_dir);
                        utils::stored(&path, &mut self.affix, self.session_storer.as_ref());
                        let path = format!("{}task", self.args.data_dir);
                        utils::stored(&path, &mut self.task, None);
                        let path = format!("{}task_tmp", self.args.data_dir);
                        utils::stored(&path, &mut self.task_tmp, self.session_storer.as_ref());
                        let path = format!("{}request", self.args.data_dir);
                        utils::stored(&path, &mut self.req, self.session_storer.as_ref());
                        let path = format!("{}request_tmp", self.args.data_dir);
                        utils::stored(&path, &mut self.req_tmp, self.session_storer.as_ref());
                        let path = format!("{}couple", self.args.data_dir);
                        utils::stored(&path, &mut self.couple, self.session_storer.as_ref());
                    }
                }

                _ => unreachable!(),
            }
        }
        Ok(())
    }
}
