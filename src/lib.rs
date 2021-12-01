//! [dyer] is designed for reliable, flexible and fast Request-Response based service, including data processing, web-crawling and so on, providing some friendly, flexible, comprehensive  features without compromising speed.
//!
//! `dyer` provides some high-level features:  
//!
//! * asynchronous, lock-free, concurrent streaming and I/O, make the best of thread pool, network, and system
//! resource.
//! * Event-driven, once you set the initials and recursive generator, `dyer` will handle
//! the rest of it.
//! * User-friendly and flexible, `dyer` offers high-level, easy to use wrappers and APIs what does a lot for you.    
//!
//! **Get started** by installing [dyer-cli] and looking over the [examples].

//!
//! [dyer]: https://github.com/HomelyGuy/dyer
//! [examples]: https://github.com/HomelyGuy/dyer/tree/master/examples/
//! [dyer-cli]: https://github.com/HomelyGuy/dyer-cli
//!
//! # Quick Start
//!
//! [**dyer-cli**] is a handy tool for your easy and fast use of dyer, and recommanded to intergrate
//! ### Prerequisite
//!  - `rust` and `cargo` must be installed,
//!  - OpenSSL library is required to compile openssl-sys(if not installed, see [here])
//!
//! [here]: https://github.com/HomelyGuy/dyer/tree/master/examples/
//!
//!  the following code helps you get the tool:
//! ```bash
//! cargo install dyer-cli
//! ```
//! Once installed, run `dyer` in your terminal or cmd prompt, it prints some info like
//! ```bash
//! Handy tool for dyer
//! ```
//! then dyer-cli is successfully installed.
//!
//! ## Create A Project
//!
//! Dyer-cli generates a template that contains many useful instances and instructions when using dyer with
//! following code:
//! ```bash
//! dyer new myproject
//! ```
//! It will create a project called `myproject` and the files layout displays:
//! ```bash
//! |___Cargo.toml
//! |___Readme.md
//! |___data/
//! |___data/tasks/
//! |___src/
//!     |___src/affix.rs
//!     |___src/entity.rs
//!     |___src/parser.rs
//!     |___src/actor.rs
//!     |___src/middleware.rs
//!     |___src/pipeline.rs
//! ```    
//! Main functionality of each file:
//! * the `affix.rs` serves as an actor to adjust and satisfy additional requirement
//! * the `entity.rs` contains entities/data structure to be used/collected
//! * the `parser.rs` contains functions that extract entities from response
//! * the `actor.rs` contains initial when opening and final things to do when closing
//! * the `middleware.rs` contains Some middlewares that process data at runtime
//! * the `pipeline.rs` contains entities manipulation including data-storage, displsying and so on
//! * the `lib.rs` exports all modules inside the directory, just do nothing here normally
//! * `Cargo.toml` is the basic configuration of the project
//! * `README.md` contains some instructions of the project
//! * `data/` place to store/load files of `App` when load-balancing and backup
//!
//! Then it is your show time, basically there are simple example items(`function`, `enum`, `struct`)
//! in each file you can follow. After that check your code
//! ```bash
//! dyer check
//! ```
//! if you run it the first time, dyer-cli will download the crates and then check the code.
//! if some warning happens such as `unused import` or `dead code` the command does a lot for you:
//! ```bash
//! dyer fix
//! ```
//! A wraper of `cargo fix`,  if some warning happens such as `unused import` or `dead code` the command does a lot for you. However it won't help if some errors occur, if so, you have to debug the code manually.
//!
//! Edit `dyer.cfg` file in the root directory
//!
//! the file contains some configurations of `ArgApp` that will update periodically, for more details see
//! [dyer.cfg Configuration]
//!
//! When the program compiles, haha run it:
//! ```bash
//! dyer run
//! ```
//! [dyer.cfg Configuration]: crate::engine::arg
//!
//! Short as it seems, here represents the normal procedure to write your program. It is believed that learning by example is the best, and some [examples] are provided to illustrate how to use.
//!
//! # Features to be added
//!
//! - [x] Dyer-cli Command-Line Tool Support
//! - [x] Interrupt and Resume Support
//! - [x] Lock-free WorkFlow
//! - [ ]  Multiple Actors, MiddleWares, PipeLines
//! - [ ]  More to go
//!
//! # Problem And Feedback
//!
//! It is, of course, probable that bugs and errors lie in somewhere, and defects may appear in an
//! unexpected way, if you got any one, comments and suggestions are welcome, please new a issue in
//! [my github].
//!
//! [examples]: https://github.com/HomelyGuy/dyer/tree/master/examples/
//! [**dyer-cli**]: https://crates.io/crates/dyer-cli
//! [my github]: https://github.com/HomelyGuy

pub mod component;
pub mod engine;
pub mod plugin;

#[doc(inline)]
pub use component::{affix, body, client, couple, info, parsed, request, response, task, utils};
#[doc(inline)]
pub use component::{
    Affix, Body, Buf, Bytes, Client, Couple, Info, MetaRequest, MetaResponse, MetaTask, Parsed,
    Request, Response, Task,
};
#[doc(inline)]
pub use engine::{Actor, App, ArgAffix, ArgApp, ArgRate};
#[doc(inline)]
pub use plugin::deser::FNMAP;
#[doc(inline)]
pub use plugin::{Affixor, MiddleWare, MiddleWareBuilder, PipeLine, PipeLineBuilder};

#[doc(inline)]
pub use crate::plugin::{BoxFuture, LocalBoxFuture};
#[doc(inline)]
pub use async_trait::async_trait;
#[doc(inline)]
pub use dyer_macros::{self, actor, affix, entity, middleware, parser, pipeline};
#[doc(inline)]
pub use log;
