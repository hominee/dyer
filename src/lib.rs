//! dyer is designed for reliable, flexible and fast web-crawling, providing some high-level, comprehensive  features without compromising speed.
//!
//! By means of event-driven, non-blocking I/O [tokio] and powerful, reliable [Rust programming
//! language], inspired by [scrapy], `dyer` provides some high-level features:  
//!
//! * asynchronous, concurrent streamimg and I/O, make the best of thread pool, network, and system
//! resource.
//! * event-driven, once you set the initials and recursive generator of `Task`, `dyer` will handle
//! the rest of it.
//! * user-friendly, considering the philosophy of rust programming language, more source code,
//! proper architecture may set up yourself in a dilemma when efficiency and learning cost are taken
//! into consideration. `dyer` offers high-level,flexible wrappers and APIs what does a lot for you.    
//!
//! [tokio]: https://docs.rs/tokio
//! [scrapy]: https://scrapy.org
//! [Rust programming language]: https://www.rust-lang.org
//!
//! # Main Functionalities
//!
//! `Dyer` is newly developed rust library, and has achieved some basic functionalities for
//! building a crawer, web service and data processing. Nevertheless, it can tackle most common problems you meet.
//!
//! ## Real-browser Customization
//!
//! For each feeded `Task`, `dyer` will fake a `Profile` and combines them into a `Request` to
//! satisfy the requirement of the target site. By means of powerful `ffi` interface of and web
//! assemble of rust, intergration is not hard.
//!
//! ## Signal Handling
//!
//! Think about a scenario that errors, bugs and unexpected accidents are found when your app is running, what would you
//! do? Stop the app, the entire program and the data are corupted. Nope, the result is not
//! reliable. `dyer` will backup your history between certain gap, resumption is at your choice.
//!
//! ## Run-time Control
//!
//! Each `Task` and each `Profile` is scheduled with some gap, has a time stamp for validation,
//! only the expired can be feeded to engine of `dyer`. Nevertheless `dyer` will limit the
//! `Requests` sent to poll, the `Profile` to make, `Task` to load or store and so on [see `AppArg`].
//!
//! [see `AppArg`]: crate::engine::AppArg
//!
//! # Walk you through an example
//!
//! Take [Scrapy Tutorial] as a guide to know the basics, step by step.
//! * Add `dyer` as dependency in you `Cargo.toml`
//! * Writing a struct and implementing `Spider` Trait, customize parser to extract data and generate recursive `Task`
//! * Writing `PipeLine` and `MiddleWare` to process data if necessary
//!
//! [Scrapy Tutorial]: https://docs.scrapy.org/en/latest/intro/tutorial.html
//!
//! ## Add as Dependency
//!
//! `dyer` is written in rust, normally put the following in `Cargo.toml`, and other libraries
//! needed for further.
//!
//! ```
//! dyer = "*"
//! tokio = { version = "0.2", features = [ "macros", "rt-threaded" ] }
//! futures = "*"
//! serde = {version = "*", features = ["derive"] }
//! serde_json = "*"
//! select = "*"
//! log = "*"
//! simple_logger = "*"
//! ```
//!
//! ## Customization your code in your `src/main.rs`
//!
//! ```
//! extern crate dyer;
//! extern crate futures;
//! extern crate select;
//! extern crate serde;
//! extern crate serde_json;
//! extern crate tokio;
//! extern crate log;
//! extern crate simple_logger;
//!
//! use dyer::{
//!     App, MiddleWare, ParseResult, PipeLine, Profile, Request, ResError, Response, Spider, Task,
//! };
//! use futures::future::{BoxFuture, FutureExt};
//! use serde::{Deserialize, Serialize};
//! use std::fmt::Debug;
//! use std::fs::OpenOptions;
//! use std::io::{LineWriter, Write};
//! use std::sync::{Arc, Mutex, Once};
//! use simple_logger::SimpleLogger;
//!
//! type Stem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;
//!
//! // the data to be collected, make sure it is Deserializable and Serializable
//! #[derive(Deserialize, Serialize, Debug, Clone)]
//! pub struct Quote {
//!     pub text: String,
//!     pub author: String,
//!     pub tags: Vec<String>,
//! }
//!
//! // use `Items` as a container of all possible Item.
//! #[derive(Serialize, Debug, Clone)]
//! pub enum Items {
//!     Quote(Quote),
//! }
//!
//! // uri is enough to get the data, so generic parameter of `Task` and `Profile` are not necessary
//! // leave them as empty for the sake of appearance
//! #[derive(Serialize, Deserialize, Debug, Clone)]
//! pub struct Targ {}
//! #[derive(Serialize, Deserialize, Debug, Clone)]
//! pub struct Parg {}
//!
//! // here `select` is implemented to extract the data embodied in the HTML
//! // for more infomation about how to do that,
//! // "https://github.com/utkarshkukreti/select.rs" is recommanded to explore
//! pub fn parse_quote(res: Response<Targ, Parg>) -> ParseResult<Items, Targ, Parg>
//! {
//!     let mut r = ParseResult {
//!         task: vec![],
//!         profile: vec![res.profile],
//!         req: vec![],
//!         entities: vec![],
//!         yield_err: vec![],
//!     };
//!     if res.content.is_none() {
//!         // for the `Response` with empty content, recycle profile
//!         return r;
//!     }
//!     let mut quotes = Vec::new();
//!     let doc = select::document::Document::from(res.content.as_ref().unwrap().as_str());
//!     for node in doc.find(select::predicate::Class("quote")) {
//!         let text = node
//!             .find(select::predicate::Class("text"))
//!             .next()
//!             .unwrap()
//!             .text();
//!         let author = node
//!             .find(select::predicate::Class("author"))
//!             .next()
//!             .unwrap()
//!             .text();
//!         let tags = node
//!             .find(select::predicate::Class("tag"))
//!             .map(|tag| tag.text())
//!             .collect::<Vec<String>>();
//!         let item = Quote { text, author, tags };
//!         quotes.push(Items::Quote(item));
//!     }
//!     r.entities = quotes;
//!
//!     // follow the next page if exists
//!     let mut next_node = doc.find(select::predicate::Class("next"));
//!     if let Some(nd) = next_node.next() {
//!         // next page exists
//!         let next_url = nd
//!             .find(select::predicate::Name("a"))
//!             .next()
//!             .unwrap()
//!             .attr("href")
//!             .unwrap();
//!         let mut task = Task::<Targ>::default();
//!         task.uri = format!("https://quotes.toscrape.com{}", next_url);
//!         task.parser = "parse_quote".to_string();
//!         r.task.push(task);
//!     }
//!     r
//! }
//!
//! pub struct Spd {}
//! // implementing `Spider` for `Spd`
//! impl Spider<Items, Targ, Parg> for Spd {
//!     // preparation before opening spider
//!     // nothing to do in this case
//!     fn open_spider(&self, _app: &mut App<Items, Targ, Parg>) {}
//!
//!     // preparation before closing spider
//!     // nothing to do in this case
//!     fn close_spider(&self, _app: &mut App<Items, Targ, Parg>) {}
//!
//!     // set up parser that extracts `Quote` from the `Response`
//!     fn get_parser<'a>(
//!         &self,
//!         ind: String,
//!     ) -> Option<&'a dyn Fn(Response<Targ, Parg>) -> ParseResult<Items, Targ, Parg>> {
//!         if &ind == "parse_quote" {
//!             return Some(&parse_quote);
//!         }
//!         None
//!     }
//!
//!     // `Task` to be executed when starting `dyer`
//!     fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
//!         let mut task = Task::default();
//!
//!         // all infomation needed is uri and parser
//!         task.uri = "https://quotes.toscrape.com".to_string();
//!         task.parser = "parse_quote".to_string();
//!         Ok(vec![task])
//!     }
//!
//!     // the generator of `Profile`
//!     // `dyer` consume the returned `Request`, generate a `Response` fed to the closure
//!     // to generate a `Profile`
//!     fn entry_profile(
//!         &self,
//!     ) -> (
//!         Request<Targ, Parg>,
//!         Option<
//!             &(dyn Fn(&mut Response<Targ, Parg>) -> BoxFuture<'_, Result<Profile<Parg>, ResError>>
//!                   + Send
//!                   + Sync),
//!         >,
//!     ) {
//!         let mut req = Request::<Targ, Parg>::default();
//!         req.task.uri = "https://quotes.toscrape.com".to_string();
//!         // as the domain suggests this site is specially built for crawling,
//!         // you do not have to pretend as a real device
//!         // leave the `Profile` generator as empty
//!         (req, None)
//!     }
//! }
//!
//! // open a static file `result.json`
//! async fn open_file(path: &str) -> &'static Option<std::fs::File> {
//!     static INIT: Once = Once::new();
//!     static mut VAL: Option<std::fs::File> = None;
//!     unsafe {
//!         INIT.call_once(|| {
//!             let file = OpenOptions::new()
//!                 .create(true)
//!                 .write(true)
//!                 .append(true)
//!                 .open(path)
//!                 .unwrap();
//!             VAL = Some(file);
//!         });
//!         &VAL
//!     }
//! }
//! // store Items into file
//! async fn store_item(items: &mut Arc<Mutex<Vec<Items>>>) {
//!     let mut ser_items = Vec::new();
//!     let items_len = items.lock().unwrap().len();
//!     for _ in 0..items_len {
//!         let item = items.lock().unwrap().pop().unwrap();
//!         let s = serde_json::to_string(&item).unwrap();
//!         ser_items.push(s);
//!     }
//!     let stream = ser_items.join("\n");
//!     let mut writer = LineWriter::new(open_file("result.json").await.as_ref().unwrap());
//!     writer.write(&stream.as_bytes()).unwrap();
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // for the sake of clarification, use simple_logger to display some level-varied infomation
//!     // initialize simple_logger
//!     SimpleLogger::new()
//!     .with_level(log::LevelFilter::Info)
//!     .with_module_level("dyer", log::LevelFilter::Debug)
//!     .init()
//!     .unwrap();
//!     static SPD: Spd = Spd {};
//!     // since the `Quote` collected by parse_quote is complete, `MiddleWare` is not necessary,
//!     let middleware = MiddleWare::<Items, Targ, Parg>::builder().build();
//!     // writing a `PipeLine` to store them
//!     // for short, handle `Items` only
//!     let pipeline = PipeLine::<Items, std::fs::File>::builder()
//!         .process_item(&|items: &mut Arc<Mutex<Vec<Items>>>| store_item(items).boxed_local())
//!         .build();
//!
//!     // construct the app and start the crawler
//!     let mut app: App<Items, Targ, Parg> = App::<Items, Targ, Parg>::new();
//!     // donot use history, such as `Request`, `Profile`, `Task`
//!     app.rt_args.lock().unwrap().skip_history = true;
//!     app.run(&SPD, &middleware, pipeline).await;
//!     Ok(())
//! }
//! ```
//!
//! As you expected, It is Done.
//! The output file `result.json` is in the root directory which contains file `Cargo.toml`, and
//! its content displays
//!
//! ```
//! {"Quote":{"text":"“... a mind needs books as a sword needs a whetstone, if it is     to keep its edge.”","author":"George R.R. Martin","tags":["books","mind"]}}
//! ...
//! {"Quote":{"text":"“The world as we have created it is a process of our thinking.     It cannot be changed without changing our thinking.”","author":"Albert Einstein",    "tags":["change","deep-thoughts","thinking","world"]}}
//! ```
//!
//! #Features to be added
//!
//! * proxy support
//! * debugging support(not bad though for now)
//! * more signal support(Ctrl+c for now)
//! * autothrottling and more customized plugins support
//! * more to go

pub mod component;
pub mod engine;
pub mod plugin;

pub use component::*;
pub use engine::{App, AppArg};
pub use plugin::{MiddleWare, PipeLine, Spider};
