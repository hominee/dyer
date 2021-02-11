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
////! * Writing `PipeLine` and `MiddleWare` to process data if necessary
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
//! extern crate tokio;
//! extern crate serde;
//! extern crate serde_json;
//! extern crate select;
//!
//! use serde::{Deserialize, Serialize};
//! use dyer::{Spider, App, Profile, Task, Request};
//! use std::fmt::Debug;
//!
//! type Stem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;
//!
//! #[derive(Deserialize, Serialize, Clone, Debug)]
//! struct Quote{
//!     pub text: String,
//!     pub author: String,
//!     pub tags: Vec<String>,
//! }
//!
//! // use `Items` as a container of all possible Item.
//! #[derive(Serialize, Clone, Debug)]
//! pub enum Items {
//!     Quote(Quote),
//! }
//!
//! // uri is enough to get the data, so generic parameter of `Task` and `Profile` are not necessary
//! // leave them as empty
//! struct Targ {};
//! struct Parg {};
//!
//! struct Spd {};
//! // implementing `Spider` for `Spd`
//! impl<Items, Parg, Targ> Spider for Spd {
//!     fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
//!         let mut task = Task::default();
//!         task.uri = "quotes.toscrape.com".to_string();
//!         Ok(task)
//!     }
//!     fn entry_profile(&self) -> (Request<Targ, Parg>,
//!            Option<
//!                &(dyn Fn(&mut Response<T, P>) -> LocalBoxFuture<'_, Result<Profile<P>, ResError>>
//!                      + Send
//!                      + Sync),
//!            >,
//!     ) {
//!     let mut req =Request<Targ, Parg>::default();
//!     req.uri = "quotes.toscrape.com";
//!     (req, None)
//!
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = App::new();
//!     let app.run::<Items, Targ, Parg>();
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
