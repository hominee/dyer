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
//! It is disabled by default, but As you wish you can enable it by specifying [`ArgProfile`]. In general, for each feeded [Task], `dyer` will fake a [Profile] and combines them into a [Request] to
//! meet the requirement of the target site. By means of [`ffi`] interface of and web
//! assemble of rust, combination with javascript or python script may do you a favor hopefully.
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
//! [Request] sent to poll, the [Profile] to make, [Task] to load or store and so on [see `ArgApp`].
//!
//! [see `ArgApp`]: crate::engine::ArgApp
//! [Task]: crate::component::Task
//! [Profile]: crate::component::Profile
//! [`ArgProfile`]: crate::engine::ArgProfile
//! [Request]: crate::component::Request
//! [`ffi`]: https://doc.rust-lang.org/nomicon/ffi.html
//!
//! # Quick Start
//!
//! [**dyer-cli**] is a handy tool for your easy and fast use of dyer, and recommanded to intergrate
//! with other dependencies. with `rustup` and `cargo` installed, the following code helps you get
//! the tool:
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
//!     |___src/entity.rs
//!     |___src/parser.rs
//!     |___src/spider.rs
//!     |___src/middleware.rs
//!     |___src/pipeline.rs
//!     |___src/lib.rs
//! ```    
//! Main functionality of each file:
//! * the `entity.rs` contains entities/data structure to be used/collected
//! * the `parser.rs` contains functions that extract entities from response
//! * the `spider.rs` contains initial when opening and final things to do when closing
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
//! Edit `config` file in the root directory
//!
//! the file contains some configurations of `ArgApp` that will update periodically, for more details see
//! [config Configuration]
//!
//! When the program compiles, haha run it:
//! ```bash
//! dyer run
//! ```
//! [config Configuration]: crate::engine::arg
//!
//! Short as it seems, here represents the normal procedure to write your program. It is believed that learning by example is the best, and some [examples] are provided to illustrate how to use.
//!
//! # Features to be added
//!
//! ✅ Profile customization
//!
//! ✅ Periodic Configuration Update
//!
//! ✅ Dyer-cli Command-Line Tool Support
//!
//! ✅ Interrupt and Resume Support
//!
//! ⬜️ Proxy Support
//!
//! ⬜️ RobotsTxt exclusion Support
//!
//! ⬜️ Debugging Support(not bad though for now)
//!
//! ⬜️  Autothrottling and more customized plugins support
//!
//! ⬜️  More to go
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
    get_cookie, Client, ParseError, ParseResult, Profile, ProfileError, ReqError, Request,
    ResError, Response, Task, TaskError,
};
#[doc(hidden)]
pub use engine::{App, ArgApp, ArgProfile, ArgRate, ProfileInfo, Spider};
#[doc(hidden)]
pub use plugin::{MiddleWare, PipeLine};

#[doc(hidden)]
pub use dyer_macros;
#[doc(hidden)]
pub use futures::future::{BoxFuture, FutureExt};
#[doc(hidden)]
pub use log;
#[doc(hidden)]
pub use serde_json as to_json;
