extern crate dyer;
extern crate serde;
extern crate simple_logger;
extern crate tokio;

mod entity;
mod middleware;
mod parser;
mod pipeline;
mod spider;

use dyer::{log, App};
use entity::*;
use middleware::get_middleware;
use pipeline::get_pipeline;
use spider::MySpider;

#[tokio::main]
async fn main() {
    // initialize simple_logger use simple_logger to display some level-varied infomation
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        //.with_module_level("dyer", log::LevelFilter::Debug) // log level varied from modules
        .init()
        .unwrap();
    let spd: MySpider = MySpider {};
    // initialize the middleware
    let middleware = get_middleware();
    // initialize the pipeline
    let pipeline = get_pipeline();
    // construct the app and start the crawler
    let mut app: App<Entities, Targ, Parg> = App::<Entities, Targ, Parg>::new();
    // AppArg configuration, custiomize your app including:
    // rate control, history file usage, app load balance and so on
    // more details see https://docs.rs/dyer/engine/struct.AppArg.html
    app.rt_args.lock().unwrap().skip_history = true;
    /*
     *app.rt_args.lock().unwrap().round_req_max = 100;
     *app.rt_args.lock().unwrap().gap = 10;
     */
    app.run(&spd, &middleware, pipeline).await.unwrap();
}