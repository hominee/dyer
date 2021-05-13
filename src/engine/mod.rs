//! An instruction of [App] and its configuration [ArgApp] 
//!
//! # OverView
//!
//! This module contain executor that drives the [App] into running and its configuration [ArgApp],
//! generally speaking, the executor integrate [Spider], [middleware], [pipeline] and other
//! components then continuously runs until all the work you assigned is done. Meanwhile the
//! configuration which controls the data flow updated by the function [`parse_config`]
//! periodically, which means in each [interval] the executor updates [ArgApp]. 
//!
//! # Set Up Configuration
//!
//! In order to start the `engine` a file named `config` in the root directory of the project is
//! required. So we new a `config` file in the root, inside the file we write the following lines:
//! ```json
//! is_skip: true
//! data_dir: data/
//! round_entity: 70
//! arg_profile.is_on: true
//! ```
//! these line means that skipping history file when starting the [Spider] and storing `Task`
//! `Request` `Profile` if any at `data/` directory when backup or interrupting or exiting, if
//! extracted entities are over 70 then consuming all of them.
//!
//! **Note that** if you set `arg_profile.is_on` to be true, this means you enable profile
//! customization, So you need to specify a non-None type for `ProfileInfo.req` what is used to
//! generate `Profile`.
//!
//! For more details, the reference to [ArgApp] [ArgProfile] [ArgRate] is recommanded.
//!
//! [middleware]: crate::plugin::middleware
//! [pipeline]: crate::plugin::pipeline
//! [Spider]: crate::plugin::Spider
//! [`parse_config`]: crate::engine::arg::ArgApp#method.parse_config
//! [engine]: crate::engine::engine
//! [App]: crate::engine::App
//! [ArgApp]: crate::engine::ArgApp
//! [ArgRate]: crate::arg::ArgRate
//! [interval]: crate::arg::ArgRate#structfield.interval
//! [ArgProfile]: crate::arg::ArgProfile








pub mod arg;
pub mod engine;
pub mod spider;

#[doc(hidden)]
pub use arg::{ArgApp, ArgProfile, ArgRate};
#[doc(hidden)]
pub use engine::App;
#[doc(hidden)]
pub use spider::{ProfileInfo, Spider};
