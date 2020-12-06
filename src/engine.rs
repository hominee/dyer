extern crate config;
extern crate hyper_timeout;
extern crate log;
extern crate log4rs;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
extern crate tokio;

use config::Config;
use futures::future::join_all;
use crate::component::{Client, Profile, Request, Response, Task, UserAgent};
use crate::macros::{MiddleWare, Pipeline};
use log::{error, info, warn};
use signal_hook::flag as signal_flag;
use crate::macros::{ Spider, };
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::task;

pub struct App<Entity> {
    pub uas: Arc<  Vec<UserAgent> >,
    pub task: Arc<Mutex< Vec<Task> >>,
    pub profile: Arc<Mutex< Vec<Profile> >>,
    pub req: Arc<Mutex< Vec<Request> >>,
    pub req_tmp: Arc<Mutex< Vec<Request> >>,
    pub res: Arc<Mutex< Vec<Response> >>,
    pub result: Arc<Mutex< Vec<Entity> >>,
    pub yield_err: Arc<Mutex< Vec<String> >>,
    pub fut_res: Arc<Mutex< Vec<(u64, task::JoinHandle<()>)> >>,
    pub fut_profile: Arc<Mutex< Vec<(u64, task::JoinHandle<()>)> >>,
}

impl<Entity> App<Entity> {
    pub fn new() -> Self {
        App{
            uas: Arc::new(Vec::new()),
            task: Arc::new(Mutex::new(Vec::new())),
            profile: Arc::new(Mutex::new(Vec::new())),
            req: Arc::new(Mutex::new(Vec::new())),
            req_tmp: Arc::new(Mutex::new(Vec::new())),
            res: Arc::new(Mutex::new(Vec::new())),
            result: Arc::new(Mutex::new(Vec::new())),
            yield_err: Arc::new(Mutex::new(Vec::new())),
            fut_res: Arc::new(Mutex::new(Vec::new())),
            fut_profile: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// number that once for a concurrent future poll
pub struct AppArg {
    pub round_req: usize, // consume req one time
    pub round_req_min: usize , // cache request minimal length
    pub round_req_max: usize , // cache request maximal length
    pub round_task: usize , // construct req from task one time
    pub round_task_min: usize , // minimal task(profile) consumed per round
    pub round_res: usize , // consume response once upon a time
    pub profile_min: usize , // minimal profile number
    pub profile_max: usize , // maximal profile number
    pub round_yield_err: usize , //consume yield_err once upon a time
    pub round_result: usize , //consume Entity once upon a time
}

impl Default for AppArg {
    fn default() -> Self {
        AppArg {
            round_req: 100, 
            round_req_min: 300, 
            round_req_max: 700,
            round_task: 100,
            round_task_min: 7,
            round_res: 100,
            profile_min: 3000,
            profile_max: 10000,
            round_yield_err: 100,
            round_result: 100,
        }
    }
} 

pub async fn run<'a, Entity >( 
    app: &'static dyn Spider<Entity>, 
    mware: &'a dyn MiddleWare<Entity>, 
    pline: &'a dyn Pipeline<Entity>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  
{
    //init log4rs "Hello  rust"
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // signal handling initial
    let term = Arc::new(AtomicUsize::new(0));
    const SIGINT: usize = signal_hook::SIGINT as usize;
    signal_flag::register_usize(signal_hook::SIGINT, Arc::clone(&term), SIGINT).unwrap();

    let mut apk = App::new();
    let args = AppArg::default();
    let mut setting = Config::default();
    setting.merge(config::File::with_name("setting.rs")).unwrap();
    println!("{:?}", setting);
    // load User Agent
    let path_ua = setting.get_str("path_ua").unwrap();
    *Arc::get_mut( &mut apk.uas ).unwrap() = UserAgent::load(path_ua);

    let skip_history = setting.get_bool("skip_history").unwrap_or(false);
    if !skip_history {
        warn!("skip history is not set, use default as false.");
        // step into start_request
        println!("skip history: {}", skip_history);

        // basically and roughly `Task` + `Profile` = `Request `
        // `Task` is struct that contains uri availible timing parser or some supporting info;
        // `Profile` is struct that mainly focus on cookies user-agent and some site-id related
        // key-value;  and  is result of `Task` and `Profile`

        let (_uaes, is_base_reqs, is_base_tasks, is_base_profile) = app.open_spider();
        //load  `Request` if empty construct from Task and Profile
        match is_base_reqs {
            Some(d) => {
                let mut t = apk.req.lock().unwrap();
                d.into_iter().for_each(|req| {
                    t.push(req);
                });
                info!("load Request successfully.");
            }
            None => {
                error!(
                    "cannot load Request, check the setting.yaml for path_requst to settle this."
                );
                //panic!("cannotload Request from file or not configurated.");
            }
        }
        //load `Task`
        match is_base_tasks {
            Some(d) => {
                let mut t = apk.task.lock().unwrap();
                d.into_iter().for_each(|task| {
                    t.push(task);
                });
                info!("load Task successfully.");
            }
            None => {
                error!("cannot load ,Task check the setting.yaml for path_Task settle this.");
                //panic!("cannotload Task from file or not configurated.")
            }
        }
        //load `Profile`
        match is_base_profile {
            Some(d) => {
                let mut t = apk.profile.lock().unwrap();
                d.into_iter().for_each(|profile| {
                    t.push(profile);
                });
                info!("load Profile successfully.");
            }
            None => {
                error!(
                    "cannot load , Profile check the setting.yaml for path_profile settle this."
                );
                //panic!("cannotload Profile from file or not configurated.")
            }
        }
    } else {
        //skip the history and start new fields to staart with, some Profile required
        let uri = app.entry_profile().unwrap();
        let uas = apk.uas.clone();
        Profile::exec_all( apk.profile.clone(), uri, 7, uas).await;
        let tasks = app.entry_task().unwrap();
        apk.task.lock().unwrap().extend(tasks);
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

                //finish remaining futures
                let mut v = Vec::new();
                while let Some(res) = apk.fut_res.lock().unwrap().pop() {
                    //res.await;
                    v.push(res.1);
                }
                join_all(v).await;

                // dispath them
                Response::parse_all(&mut apk,  99999999, app, mware);

                //store them
                Request::stored(&apk.req_tmp);
                Request::stored(&apk.req);
                Task::stored(&apk.task);
                Profile::stored(&apk.profile);
                pline.process_item(&mut apk.result);
                pline.process_yielderr(&mut apk.yield_err);
            }

            0 => {
                // if all task request and other things are done the quit
                if apk.yield_err.lock().unwrap().is_empty()
                    && apk.req.lock().unwrap().is_empty()
                    && apk.task.lock().unwrap().is_empty()
                    && apk.result.lock().unwrap().is_empty()
                    && apk.profile.lock().unwrap().is_empty()
                {
                    info!("All work is Done. exit gracefully");
                    break;
                }

                // consume valid request in cbase_reqs_tmp
                // if not enough take them from cbase_reqs
                if apk.req_tmp.lock().unwrap().len() <= args.round_req_min {
                    // cached request is not enough
                    for _ in 0..apk.req.lock().unwrap().len() {
                        let req = apk.req.lock().unwrap().pop().unwrap();
                        if req.able <= now {
                            // put the request into cbase_req_tmp
                            apk.req_tmp.lock().unwrap().push(req);
                        }

                        if apk.req_tmp.lock().unwrap().len() > args.round_req_max {
                            break;
                        }
                    }
                }

                //take req out to finish
                let mut futs = Vec::new();
                let len = args.round_req.min(apk.req_tmp.lock().unwrap().len());
                vec![0; len].iter().for_each(|_| {
                    let req = apk.req_tmp.lock().unwrap().pop().unwrap();
                    futs.push(req);
                });
                let tbase_res = apk.res.clone();
                let john = task::spawn(async move {
                    Client::exec_all(futs, tbase_res).await;
                });
                apk.fut_res.lock().unwrap().push( (now, john) );

                // before we construct request check profile first
                let less = apk.profile.lock().unwrap().len() <= args.profile_min;
                let exceed =
                    !less && apk.profile.lock().unwrap().len() <= args.profile_max && now % 3 == 1;
                if exceed || less {
                    let uas = apk.uas.clone();
                    let uri = app.entry_profile().unwrap();
                    let pfile = apk.profile.clone();
                    let johp = task::spawn(async move {
                        Profile::exec_all(pfile, uri, 7, uas).await;
                    });
                    apk.fut_profile.lock().unwrap().push( (now, johp) );
                }

                // parse response
                //extract the parseResult
                Response::parse_all(&mut apk, args.round_res, app, mware);

                //pipeline put out yield_parse_err and Entity
                if apk.yield_err.lock().unwrap().len() > args.round_yield_err {
                    pline.process_yielderr(&mut apk.yield_err);
                }
                if apk.result.lock().unwrap().len() > args.round_result {
                    pline.process_item(&mut apk.result);
                }

                // count for profiles length if not more than round_task_min
                if args.round_task_min > apk.profile.lock().unwrap().len() {
                    // not enough profile to construct request
                    // await the spawned task doe
                    let jh = apk.fut_profile.lock().unwrap().pop().unwrap();
                    jh.1.await.unwrap();
                }

                // construct request
                Request::gen(&mut apk, now, args.round_task);

                //join the older tokio-task
                Client::join(apk.fut_res.clone(), apk.fut_profile.clone());
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}
