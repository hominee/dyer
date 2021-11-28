//! An instruction of [App] and its configuration [ArgApp]
//!
//! # OverView
//!
//! This module contain executor that drives the [App] into running and its configuration [ArgApp],
//! generally speaking, the executor integrate [Actor], [middleware], [pipeline] and other
//! components then continuously runs until all the work you assigned is done. Meanwhile the
//! configuration which controls the data flow updated by the function [`parse_config`]
//! periodically, which means in each [interval] the executor updates [ArgApp].
//!
//! # Set Up Configuration
//!
//! In order to start the `engine` a file named `config` in the root directory of the project is
//! required. So we new a `config` file in the root, inside the file we write the following lines:
//! ```json
//! skip: true
//! data_dir: data/
//! round_entity: 70
//! arg_affix.is_on: true
//! ```
//! these line means that skipping history file when starting the [Actor] and storing `Task`
//! `Request` `Affix` if any at `data/` directory when backup or interrupting or exiting, if
//! extracted entities are over 70 then consuming all of them.
//!
//! **Note that** if you set `arg_affix.is_on` to be true, this means you enable affix
//! customization, So you need to specify a non-None type for `AffixInfo.req` what is used to
//! generate `Affix`.
//!
//! For more details, the reference to [ArgApp] [ArgAffix] [ArgRate] is recommanded.
//!
//! [middleware]: crate::plugin::middleware
//! [pipeline]: crate::plugin::pipeline
//! [Actor]: crate::engine::Actor
//! [`parse_config`]: crate::engine::arg::ArgApp#method.parse_config
//! [engine]: crate::engine::engine
//! [App]: crate::engine::App
//! [ArgApp]: crate::engine::ArgApp
//! [ArgRate]: crate::engine::arg::ArgRate
//! [interval]: crate::engine::arg::ArgRate#structfield.interval
//! [ArgAffix]: crate::engine::arg::ArgAffix

pub mod actor;
pub(crate) mod appfut;
pub mod arg;
pub mod engine;
pub mod vault;

#[doc(inline)]
pub use actor::Actor;
#[doc(inline)]
pub use arg::{ArgAffix, ArgApp, ArgRate};
#[doc(inline)]
pub use engine::App;
#[doc(inline)]
pub use vault::{Vault, Vaulted};
