// Take Scrapy Tutorial as a guide to know the basics

extern crate dyer;
extern crate select;
extern crate serde;
extern crate simple_logger;
extern crate tokio;

mod entity;
mod middleware;
mod parser;
mod pipeline;
mod spider;

use dyer::{log, App, Spider};
use entity::*;
use middleware::make_middleware;
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
    let spd: MySpider = MySpider::new();
    // initialize the middleware
    let middleware = make_middleware();
    // initialize the pipeline
    let pipeline = get_pipeline();
    // construct the app and start the crawler
    let mut app: App<Entities, Targ, Parg> = App::<Entities, Targ, Parg>::new();
    // ArgApp configuration, custiomize your app including:
    // rate control, history file usage, app load balance and so on
    // more details see https://docs.rs/dyer/engine/struct.ArgApp.html
    app.args.skip_history = true;
    app.run(&spd, &middleware, pipeline).await.unwrap();
}

// As you expected, It is Done.
// The output file `result.json` is in the root directory which contains file `Cargo.toml`, and
// its content displays
//
// ```json
// {"text":"“... a mind needs books as a sword needs a whetstone, if it is to keep its edge.”","author":"George R.R. Martin","tags":["books","mind"]}
// ...
// {"text":"“The world as we have created it is a process of our thinking. It cannot be changed without changing our thinking.”","author":"Albert Einstein",    "tags":["change","deep-thoughts","thinking","world"]}
// ```
