//! [dyer] is designed for reliable, flexible and fast web-crawling, providing some high-level, comprehensive  features without compromising speed.
//!
//! By means of event-driven, non-blocking I/O [tokio] and powerful, reliable [Rust programming
//! language], inspired by [scrapy], `dyer` provides some high-level features:  
//!
//! * asynchronous, concurrent streamimg and I/O, make the best of thread pool, network, and system
//! resource.
//! * event-driven, once you set the initials and recursive generator of [Task], `dyer` will handle
//! the rest of it.
//! * user-friendly, considering the philosophy of rust programming language, more source code,
//! proper architecture may set up yourself in a dilemma when efficiency and learning cost are taken
//! into consideration. `dyer` offers high-level,flexible wrappers and APIs what does a lot for you.    
//!
//! **Get Started** by installing [dyer-cli] and looking over the [examples].
//!
//! [dyer]: https://github.com/HomelyGuy/dyer
//! [dyer-cli]: https://github.com/HomelyGuy/dyer-cli
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
//! For each feeded [Task], `dyer` will fake a [Profile] and combines them into a [Request] to
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
//! Each [Task] and each [Profile] is scheduled with some gap, has a time stamp for validation,
//! only the expired can be feeded to engine of `dyer`. Nevertheless `dyer` will limit the
//! [Request] sent to poll, the [Profile] to make, [Task] to load or store and so on [see `AppArg`].
//!
//! [see `AppArg`]: crate::engine::AppArg
//! [Task]: crate::component::Task
//! [Profile]: crate::component::Profile
//! [Request]: crate::component::Request
//!
//! # Quick Start
//!
//! [**dyer-cli**] is a handy tool for your easy and fast use of dyer, and recommanded to intergrate
//! with other dependencies. with `rustup` and `cargo` installed, the following code helps you get
//! the tool:
//! ```bash
//! cargo install dyer-cli
//! ```
//! Once installed, run `dyer-cli` in your terminal or cmd prompt, it prints some info like
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
//! dyer-cli new myproject
//! ```
//! It will create a project called `myproject` and the files layout displays:
//! ```bash
//! |___Cargo.toml
//! |___Readme.md
//! |___data/
//! |___data/tasks/
//! |___src/
//!     |___src/entity.rs
//!     |___src/parser.rs
//!     |___src/spider.rs
//!     |___src/middleware.rs
//!     |___src/main.rs
//!     |___src/pipeline.rs
//! ```    
//! Main functionality of each file:
//! * the `entity.rs` contains entities/data structure to be used/collected
//! * the `parser.rs` contains functions that extract entities from response
//! * the `spider.rs` contains initial when opening and final things to do when closing
//! * the `middleware.rs` contains Some middlewares that process data at runtime
//! * the `pipeline.rs` contains entities manipulation including data-storage, displsying and so on
//! * the `main.rs` combines all modules then build them up into a programm
//! * `Cargo.toml` is the basic configuration of the project
//! * `README.md` contains some instructions of the project
//! * `data` folder balance the app load when data in app exceeds, and backup app data at certain
//! gap
//!
//! It is believed that learning by example is the best, and some [examples] are provided to illustrate how to use.
//!
//! # Features to be added
//!
//! * proxy support
//! * debugging support(not bad though for now)
//! * more signal support(Ctrl+c for now)
//! * autothrottling and more customized plugins support
//! * more to go
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

#[doc(hidden)]
pub use component::{client, profile, request, response, task, utils};
#[doc(hidden)]
pub use component::{
    get_cookie, Client, ParseError, ParseResult, Profile, Request, ResError, Response, Task,
};
#[doc(hidden)]
pub use engine::{App, AppArg};
#[doc(hidden)]
pub use plugin::{MiddleWare, PipeLine, ProfileInfo, Spider};

#[doc(hidden)]
pub use futures::future::{BoxFuture, FutureExt};
#[doc(hidden)]
pub use log;
#[doc(hidden)]
pub use serde_json as to_json;
