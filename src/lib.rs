//! Dyer is designed for reliable, flexible and fast web-crawling, providing some high-level, comprehensive  features without compromising speed.
//!
//! By means of event-driven, non-blocking I/O [tokio] and powerful, reliable [Rust programming
//! language], inspired by [scrapy], `Dyer` provides some high-level features.  
//!
//! * asynchronous, concurrent streamimg and I/O, make the best of thread pool, network, and system
//! resource.
//! * event-driven, once you set the initials and recursive generator of `Task`, `Dyer` will handle
//! the rest of it.
//! * user-friendly, considering the philosophy of rust programming language, more source code,
//! proper framework may contribute the awkwardness when efficiency and learning cost are taken
//! into consideration. `Dyer` presents high-level wrapper for convenience.    
//!
//! [tokio]: https://docs.rs/tokio
//! [scrapy]: https://scrapy.org
//!
//! # Walk you through an example
//!
//! take [Scrapy Tutorial] as a guide to know the basics, step by step.
//! * Add `Dyer` as dependency in you `Cargo.toml`
//! * Writing a struct and implementing `Spider` Trait, customize parser to extract data and generate recursive `Task`
//! * Writing `PipeLine` and `MiddleWare` to process data if necessary
//!
//! [Scrapy Tutorial]: https://docs.scrapy.org/en/latest/intro/tutorial.html
//!
//! ## Add as Dependency
//!
//! `Dyer` is written in rust, normally put the following in `Cargo.toml`, and other libraries
//! needed for further.
//! ```
//! dyer = "0.1.0"
//! tokio = { version = "0.2", features = [ "macros", "rt-threaded" ] }
//! serde = {version = "*", features = ["derive"] }
//! serde_json = "*"
//! select = "*"
//! ```
//! ## Customization your code in your `src/main.rs`
//!
//! ```
//! extern crate dyer;
//! extern crate futures;
//! extern crate select;
//! extern crate serde;
//! extern crate serde_json;
//! extern crate tokio;
//!
//! use dyer::{
//!     App, MiddleWare, ParseResult, PipeLine, Profile, Request, ResError, Response, Spider, Task,
//! };
//! use futures::future::{BoxFuture, FutureExt};
//! use serde::{Deserialize, Serialize};
//! use std::fmt::Debug;
//! use std::io::{LineWriter, Write};
//! use std::sync::{Arc, Mutex, Once};
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
//! where
//!     Items: Serialize + Clone + Debug,
//!     Targ: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
//!     Parg: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
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
//!     if next_node.next().is_some() {
//!         // next page exists
//!         let next_url = next_node
//!             .next()
//!             .unwrap()
//!             .find(select::predicate::Name("a"))
//!             .next()
//!             .unwrap()
//!             .attr("href")
//!             .unwrap();
//!         let mut task = Task::<Targ>::default();
//!         task.uri = format!("https://quotes.toscrape.com{}", next_url);
//!         r.task.push(task);
//!     }
//!     r
//! }
//!
//! pub struct Spd {}
//! // implementing `Spider` for `Spd`
//! impl Spider<Items, Targ, Parg> for Spd {
//!     // preparation before opening spider
//!     fn open_spider(&self, _app: &mut App<Items, Targ, Parg>) {}
//!
//!     // preparation before closing spider
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
//!     // `Task` executed when starting `dyer`
//!     fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
//!         let mut task = Task::default();
//!
//!         // all infomation needed is uri and parser
//!         task.uri = "quotes.toscrape.com".to_string();
//!         task.parser = "parse_quote".to_string();
//!         Ok(vec![task])
//!     }
//!
//!     // the generator of `Profile`
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
//!         req.task.uri = "quotes.toscrape.com".to_string();
//!         // this site is specially for newbies, you do not have to pretend as a real device
//!         (req, None)
//!     }
//! }
//!
//! use std::fs::OpenOptions;
//! // open a static file
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
//!     let mut file = LineWriter::new(open_file("result.json").await.as_ref().unwrap());
//!     file.write(&stream.as_bytes()).unwrap();
//! }
//!
//! #[tokio::main]
//! async fn main() {
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
//!     app.run(&SPD, &middleware, pipeline).await;
//!     Ok(())
//! }
//! ```
//!
//! As you expected, It is Done.

pub mod component;
pub mod engine;
pub mod plugin;

pub use component::*;
pub use engine::{App, AppArg};
pub use plugin::{MiddleWare, PipeLine, Spider};
