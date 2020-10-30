extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate log4rs;
extern crate config;
extern crate log;
extern crate signal_hook;
extern crate hyper_timeout;

mod item;
mod spider;
mod middleware;
mod pipeline;

use pipeline::{database, yield_parse_err};
use config::Config;
use item::{Profile, ResError,  Request, Response, Task,RawTask};
use log::{debug, error, info, trace, warn};
use spider::{App, fake, Entry, Entity, Parse};
use hyper::{Client as hClient, body::Body as hBody, client::HttpConnector};
use hyper_tls::HttpsConnector;
use hyper_timeout::TimeoutConnector;
use futures::future::join_all;
use futures::executor::block_on;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering} };
use signal_hook::flag as signal_flag;
use tokio::task;
use tokio::task::JoinHandle;
use tokio::runtime::Builder;
use futures::Future;

#[ tokio::main ]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //init log4rs "Hello  rust"
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // signal handling initial
    let term = Arc::new( AtomicUsize::new(0) );
    const SIGINT: usize = signal_hook::SIGINT as usize;
    signal_flag::register_usize(signal_hook::SIGINT, Arc::clone( &term ), SIGINT).unwrap();

    // hyper client intial
    let https: HttpsConnector<HttpConnector> = HttpsConnector::new();
    let mut conn = hyper_timeout::TimeoutConnector::new(https);
    conn.set_connect_timeout(Some(std::time::Duration::from_secs(7)));
    conn.set_read_timeout(Some(std::time::Duration::from_secs(23)));
    conn.set_write_timeout(Some(std::time::Duration::from_secs(7)));
    let client: hClient<TimeoutConnector<  HttpsConnector<HttpConnector> >> = hClient::builder().build::<_,hBody>(conn);

    let app = App::init();
    let base_reqs: Arc<Mutex< Vec<Request> >> = Arc::new(Mutex::new( Vec::new() )); 
    let base_reqs_tmp: Arc<Mutex< Vec<Request> >> = Arc::new(Mutex::new( Vec::new() )); 
    let base_tasks: Arc<Mutex< Vec<Task> >> = Arc::new(Mutex::new( Vec::new() ));
    let base_profile: Arc<Mutex< Vec<Profile> >> = Arc::new(Mutex::new( Vec::new() ));
    let base_res: Arc<Mutex< Vec<Response> >> = Arc::new(Mutex::new( Vec::new() ));
    let base_yield_err:Arc<Mutex< Vec<String> >> = Arc::new(Mutex::new( Vec::new() )); 
    let base_result: Arc<Mutex< Vec<Entity> >> = Arc::new(Mutex::new( Vec::new() ));
    let fut_res: Arc<Mutex< Vec<task::JoinHandle< () >> >> = Arc::new(Mutex::new( Vec::new() ));
    let fut_profile: Arc<Mutex< Vec<task::JoinHandle< () >> >> = Arc::new(Mutex::new( Vec::new() ));
    //number that once for a concurrent future poll
    let round_req: usize = 100;  // consume req one time
    let round_req_min: usize= 300; // cache request minimal length
    let round_req_max: usize = 700; // cache request maximal length
    let round_task: usize = 100; // construct req from task one time
    let round_task_min: usize = 7; // minimal task(profile) consumed per round
    let round_res: usize = 100; // consume response once upon a time
    let profile_min: usize = 3000; // minimal profile number
    let profile_max: usize = 10000; // maximal profile number
    let round_yield_err: usize = 100; // consume yield_err once upon a time
    let round_result: usize = 100; // consume Entity once upon a time
    let len_out_yield_err: usize = 70;

    let mut setting = Config::default();
    setting.merge(config::File::with_name("setting")).unwrap();
    println!("{:?}", setting);
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
                d
                .into_iter()
                .for_each(| req|{
                    t.push(req);
                });
                info!("load Request successfully.");
            }
            None => {
                error!("cannot load Request, check the setting.yaml for path_requst to settle this.");
                panic!("cannotload Request from file or not configurated.");
            }
        }
        //load `Task`
        let is_base_tasks = Task::load();
        match is_base_tasks {
            Some(d) => {
                let mut t = base_tasks.lock().unwrap();
                d.into_iter().for_each(| task | {
                    t.push( task );
                });
                info!("load Task successfully.");
            },
            None => {
                error!("cannot load ,Task check the setting.yaml for path_Task settle this.");
                panic!("cannotload Task from file or not configurated.")
            }
        }
        //load `Profile`
        let is_base_profile = Profile::load();
        match is_base_profile {
            Some(d) => {
                let mut t = base_profile.lock().unwrap();
                d.into_iter().for_each(| profile | {
                    t.push( profile );
                });
                info!("load Profile successfully.");
            }
            None => {
                error!("cannot load , Profile check the setting.yaml for path_profile settle this.");
                panic!("cannotload Profile from file or not configurated.")
            }
        }

    } else {
        //skip the history and start new fields 
        //to staart with, some Profile required
       let mut prof = Vec::new();
       for _ in 0..77 {
           let cf = base_profile.clone(); 
           prof.push(fake(&client, cf ));
        }
       block_on(  join_all(prof) );
       let cfut_res = base_res.clone();
       app.start_request( client.clone(), cfut_res );
       let cfut_res = base_res.clone();
       let len_res = cfut_res.lock().unwrap().len();
       vec![0; len_res].iter().for_each(|_| {
           let res = cfut_res.lock().unwrap().pop().unwrap(); 
           let result = App::parse(res);
           match result {
               Err(_e) => {
                   // res has err code (non-200) and cannot handled by error handle
                   // discard the response that without task or profile.
               }
               Ok(d) => {
                   if let Some(da) = d.profile {
                       base_profile.lock().unwrap().push(da);
                   }
                   if let Some(ta) = d.task {
                       base_tasks.lock().unwrap().extend(ta);
                   }
                   if let Some(re) = d.req {
                       base_reqs.lock().unwrap().push(re);
                   }
                   if let Some(en) = d.entities {
                       // pipeline out put the entities
                       base_result.lock().unwrap().extend(en.into_iter());
                       //database(en);
                   }
               }
           }
       });
        
    } 

    loop {

        let cbase_reqs = Arc::clone( &base_reqs );
        let cbase_reqs_tmp = Arc::clone( &base_reqs_tmp );
        let cbase_res = Arc::clone( &base_res );
        let cbase_tasks =Arc::clone(   &base_tasks );
        let cbase_yield_err = Arc::clone( &base_yield_err );
        let cbase_profile = Arc::clone(  &base_profile ); 
        let cbase_result = Arc::clone( &base_result );
        let cfut_res = Arc::clone( &fut_res );
        let cfut_profile = Arc::clone( &fut_profile );
        let cclient = client.clone();

        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u64;

        match term.load(Ordering::Relaxed) {
            SIGINT => {
                // receive the Ctrl+c signal 
                // by default  request  task profile and result yield err are going to stroed into
                // file

                //finish remaining futures
                let mut v = Vec::new();
                while let Some(res) = cfut_res.lock().unwrap().pop() {
                    //res.await;
                    v.push(res);
                };
                join_all(v).await;

                // dispath them 
                
                //store them
                Request::stored(cbase_reqs);
                Task::stored(cbase_tasks);
                Profile::stored(cbase_profile);
                database(cbase_result).unwrap();
                yield_parse_err(cbase_yield_err);

            }
            0 => {
                // if all task request and other things are done the quit
                if cbase_yield_err.lock().unwrap().is_empty() && cbase_reqs.lock().unwrap().is_empty() && cbase_tasks.lock().unwrap().is_empty() && cbase_result.lock().unwrap().is_empty() && cbase_profile.lock().unwrap().is_empty() {
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
                let len = round_req.min(cbase_reqs_tmp.lock().unwrap().len() );
                vec![0;len].iter().for_each( |_| {
                    let req = cbase_reqs_tmp.lock().unwrap().pop().unwrap();
                    futs.push( req );
                });
                let tbase_res = cbase_res.clone();
                let john = task::spawn( async move {
                    Response::exec_all( futs, cclient,tbase_res ).await;
                });
                cfut_res.lock().unwrap().push(john);

                // before we construct request check profile first
                let less = cbase_profile.lock().unwrap().len() <= profile_min;
                let exceed = !less && cbase_profile.lock().unwrap().len() <= profile_max && now %3 == 1;
                if exceed || less {
                    let fclient = client.clone();
                    let tbase_profile = base_profile.clone();
                    let johp = task::spawn( async move {
                        fake(&fclient, tbase_profile).await;
                    } );
                    cfut_profile.lock().unwrap().push( johp );
                }

                // parse response
                //extract the parseResult 
                let len_res = round_res.min( cbase_res.lock().unwrap().len() );
                let mut v_res = Vec::new();
                vec![0; len_res].iter().for_each(|_| {
                    v_res.push( cbase_res.lock().unwrap().pop().unwrap() );
                });
                v_res.into_iter().for_each(|response| {
                    let r = App::parse(response);
                    match r {
                        Ok( pas ) => {
                            match pas.req {
                                Some(req) => cbase_reqs.lock().unwrap().push(req),
                                None => {}
                            
                            }

                            match pas.task {
                                Some(tasks) => cbase_tasks.lock().unwrap().extend(tasks),
                                None => {}
                            }

                            match pas.profile {
                                Some(p) => cbase_profile.lock().unwrap().push(p),
                                None => {}
                            }

                            match pas.entities {
                                Some(es) => cbase_result.lock().unwrap().extend(es),
                                None => {}
                            }

                            match pas.yield_err {
                                Some(y) => cbase_yield_err.lock().unwrap().push(y),
                                None => {}
                            }
                        }
                        Err(_) => {}
                    }});

                //pipeline put out yield_parse_err and Entity 
                if cbase_yield_err.lock().unwrap().len() > len_out_yield_err {
                    yield_parse_err(cbase_yield_err);
                }
                if cbase_result.lock().unwrap().len() > len_out_yield_err {
                    database(cbase_result);
                }

                // count for profiles length if not more than round_task_min
                if round_task_min > cbase_profile.lock().unwrap().len() {
                    // not enough profile to construct request
                    // await the spawned task doe
                    let jh = cfut_profile.lock().unwrap().pop().unwrap();
                    block_on(jh).unwrap();
                }
                // construct request
                let len_profile = cbase_profile.lock().unwrap().len();
                let cbase_tasks = base_tasks.clone();
                let len_task = cbase_tasks.lock().unwrap().len();
                let mut task_run: Vec<Task> = Vec::new();
                let len_t = len_profile.min(len_task) as usize;
                
                vec![0; len_t].iter()
                    .for_each(|_| {
                        let t = cbase_tasks.lock().unwrap().pop().unwrap();
                        if t.able <= now {
                            task_run.push(t);
                        }
                    });
                vec![0; task_run.len() ].iter().for_each(|_| {
                    let tk = task_run.pop().unwrap();
                    let pf = cbase_profile.lock().unwrap().pop().unwrap();
                    let mut req = Request::default();
                    req.from_task(tk);
                    req.from_profile(pf);
                    cbase_reqs.lock().unwrap().push( req );
                });

            }
            _ => unreachable!(),
        }


    }
    
    Ok(())
}
