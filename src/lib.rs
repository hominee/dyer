extern crate config;
extern crate hyper_timeout;
extern crate log;
extern crate log4rs;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
extern crate tokio;

pub mod engine;
pub mod middleware;
pub mod pipeline;

use config::Config;
use futures::future::join_all;
use hyper::{body::Body as hBody, client::HttpConnector, Client as hClient};
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use engine::{Profile, Parse,  Entity, Request, Response, Task, UserAgent};
use log::{debug, error, info, trace, warn};
use pipeline::{database, yield_parse_err};
use signal_hook::flag as signal_flag;
use engine::{ MSpider, Spider, S as Sapp };
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::task;

pub async fn run( app: &'static Sapp) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  
{
    //init log4rs "Hello  rust"
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // signal handling initial
    let term = Arc::new(AtomicUsize::new(0));
    const SIGINT: usize = signal_hook::SIGINT as usize;
    signal_flag::register_usize(signal_hook::SIGINT, Arc::clone(&term), SIGINT).unwrap();

    // hyper client intial
    let https: HttpsConnector<HttpConnector> = HttpsConnector::new();
    let mut conn = hyper_timeout::TimeoutConnector::new(https);
    conn.set_connect_timeout(Some(std::time::Duration::from_secs(7)));
    conn.set_read_timeout(Some(std::time::Duration::from_secs(23)));
    conn.set_write_timeout(Some(std::time::Duration::from_secs(7)));
    let client: hClient<TimeoutConnector<HttpsConnector<HttpConnector>>> =
        hClient::builder().build::<_, hBody>(conn);

    let base_reqs: Arc<Mutex<Vec<Request>>> = Arc::new(Mutex::new(Vec::new()));
    let base_reqs_tmp: Arc<Mutex<Vec<Request>>> = Arc::new(Mutex::new(Vec::new()));
    let base_tasks: Arc<Mutex<Vec<Task>>> = Arc::new(Mutex::new(Vec::new()));
    let base_profile: Arc<Mutex<Vec<Profile>>> = Arc::new(Mutex::new(Vec::new()));
    let mut base_ua: Arc<Vec<UserAgent> > = Arc::new(Vec::new());
    let base_res: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(Vec::new()));
    let base_yield_err: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let base_result: Arc<Mutex<Vec<Entity>>> = Arc::new(Mutex::new(Vec::new()));
    let fut_res: Arc<Mutex<Vec< (u64, task::JoinHandle<()>) >>> = Arc::new(Mutex::new(Vec::new()));
    let fut_profile: Arc<Mutex<Vec< (u64, task::JoinHandle<()>) >>> = Arc::new(Mutex::new(Vec::new()));
    //number that once for a concurrent future poll
    let round_req: usize = 100; // consume req one time
    let round_req_min: usize = 300; // cache request minimal length
    let round_req_max: usize = 700; // cache request maximal length
    let round_task: usize = 100; // construct req from task one time
    let round_task_min: usize = 7; // minimal task(profile) consumed per round
    let round_res: usize = 100; // consume response once upon a time
    let profile_min: usize = 3000; // minimal profile number
    let profile_max: usize = 10000; // maximal profile number
    let round_yield_err: usize = 100; // consume yield_err once upon a time
    let round_result: usize = 100; // consume Entity once upon a time

    let mut setting = Config::default();
    setting.merge(config::File::with_name("setting")).unwrap();
    println!("{:?}", setting);
    // load User Agent
    let path_ua = setting.get_str("path_ua").unwrap();
    *Arc::get_mut( &mut base_ua ).unwrap() = UserAgent::load(path_ua);

    let skip_history = setting.get_bool("skip_history").unwrap_or(false);
    if !skip_history {
        warn!("skip history is not set, use default as false.");
        // step into start_request
        println!("skip history: {}", skip_history);

        // basically and roughly `Task` + `Profile` = `Request `
        // `Task` is struct that contains uri availible timing parser or some supporting info;
        // `Profile` is struct that mainly focus on cookies user-agent and some site-id related
        // key-value;  and  is result of `Task` and `Profile`
        //load  `Request` if empty construct from Task and Profile
        let is_base_reqs = Request::load();
        match is_base_reqs {
            Some(d) => {
                let mut t = base_reqs.lock().unwrap();
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
        let is_base_tasks = Task::load();
        match is_base_tasks {
            Some(d) => {
                let mut t = base_tasks.lock().unwrap();
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
        let is_base_profile = Profile::load();
        match is_base_profile {
            Some(d) => {
                let mut t = base_profile.lock().unwrap();
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
        //skip the history and start new fields
        //to staart with, some Profile required
        let uri = app.entry_profile().unwrap();
        let uas = base_ua.clone();
        Profile::exec_all(&client, base_profile.clone(), uri, 7, uas).await;
        panic!("{:?}", base_profile);



        let cfut_res = base_res.clone();
        let tasks = app.entry_task().unwrap();
        base_tasks.lock().unwrap().extend(tasks);
    }

    loop {
        let cbase_reqs = Arc::clone(&base_reqs);
        let cbase_reqs_tmp = Arc::clone(&base_reqs_tmp);
        let cbase_res = Arc::clone(&base_res);
        let cbase_tasks = Arc::clone(&base_tasks);
        let cbase_yield_err = Arc::clone(&base_yield_err);
        let cbase_profile = Arc::clone(&base_profile);
        let cbase_result = Arc::clone(&base_result);
        let cfut_res = Arc::clone(&fut_res);
        let cfut_profile = Arc::clone(&fut_profile);
        let cclient = client.clone();

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
                while let Some(res) = cfut_res.lock().unwrap().pop() {
                    //res.await;
                    v.push(res.1);
                }
                join_all(v).await;

                // dispath them
                Response::parse_all(cbase_res.clone(), cbase_reqs.clone(), cbase_tasks.clone(), cbase_profile.clone(), cbase_result.clone(), cbase_yield_err.clone(), 99999999, app);

                //store them
                Request::stored(cbase_reqs_tmp);
                Request::stored(cbase_reqs);
                Task::stored(cbase_tasks);
                Profile::stored(cbase_profile);
                database(cbase_result).unwrap();
                yield_parse_err(cbase_yield_err);
            }
            0 => {
                // if all task request and other things are done the quit
                if cbase_yield_err.lock().unwrap().is_empty()
                    && cbase_reqs.lock().unwrap().is_empty()
                    && cbase_tasks.lock().unwrap().is_empty()
                    && cbase_result.lock().unwrap().is_empty()
                    && cbase_profile.lock().unwrap().is_empty()
                {
                    info!("All work is Done. exit gracefully");
                    break;
                }

                // consume valid request in cbase_reqs_tmp
                // if not enough take them from cbase_reqs
                if cbase_reqs_tmp.lock().unwrap().len() <= round_req_min {
                    // cached request is not enough
                    for _ in 0..cbase_reqs.lock().unwrap().len() {
                        let req = cbase_reqs.lock().unwrap().pop().unwrap();
                        if req.able <= now {
                            // put the request into cbase_req_tmp
                            cbase_reqs_tmp.lock().unwrap().push(req);
                        }
                        if cbase_reqs_tmp.lock().unwrap().len() > round_req_max {
                            break;
                        }
                    }
                }


                //take req out to finish
                let mut futs = Vec::new();
                let len = round_req.min(cbase_reqs_tmp.lock().unwrap().len());
                vec![0; len].iter().for_each(|_| {
                    let req = cbase_reqs_tmp.lock().unwrap().pop().unwrap();
                    futs.push(req);
                });
                let tbase_res = cbase_res.clone();
                let john = task::spawn(async move {
                    Response::exec_all(futs, cclient, tbase_res).await;
                });
                cfut_res.lock().unwrap().push( (now, john) );

                // before we construct request check profile first
                let less = cbase_profile.lock().unwrap().len() <= profile_min;
                let exceed =
                    !less && cbase_profile.lock().unwrap().len() <= profile_max && now % 3 == 1;
                if exceed || less {
                    let fclient = client.clone();
                    let tbase_profile = base_profile.clone();
                    let uas = base_ua.clone();
                    let uri = app.entry_profile().unwrap();
                    let johp = task::spawn(async move {
                        Profile::exec_all(&fclient, tbase_profile, uri, 7, uas).await;
                    });
                    cfut_profile.lock().unwrap().push( (now, johp) );
                }

                // parse response
                //extract the parseResult
                Response::parse_all(cbase_res.clone(), cbase_reqs.clone(), cbase_tasks.clone(), cbase_profile.clone(), cbase_result.clone(), cbase_yield_err.clone(), round_res, app);

                //pipeline put out yield_parse_err and Entity
                if cbase_yield_err.lock().unwrap().len() > round_yield_err {
                    yield_parse_err(cbase_yield_err);
                }
                if cbase_result.lock().unwrap().len() > round_result {
                    database(cbase_result);
                }

                // count for profiles length if not more than round_task_min
                if round_task_min > cbase_profile.lock().unwrap().len() {
                    // not enough profile to construct request
                    // await the spawned task doe
                    let jh = cfut_profile.lock().unwrap().pop().unwrap();
                    jh.1.await.unwrap();
                }

                // construct request
                Request::gen(cbase_profile.clone(), cbase_tasks.clone(), cbase_reqs.clone(), now, round_task);

                //join the older tokio-task
                Response::join(cfut_res.clone(), cfut_profile.clone());

            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
