//! the end of data flow, plugin that consume the extracted `Entity`, In general, the default
//! method does nothing, and customization is requird to store the data. An example:
//!
//! ```no_run
//! # use crate::dyer::{plug,engine::vault::Vault, PipeLine};
//! #
//! pub struct E;
//! pub struct C;
//! #
//! # async fn pl_open() ->  Option<&'a C> {
//! #     None
//! # }
//! # async fn pl_close() {}
//! # async fn pl_item(_item: Vec<E>) {}
//! # async fn pl_yerr(_item: Vec<String>) {}
//!
//! // to initialize a `PipeLine`
//! let pipeline = PipeLine::<E,C>::builder()
//!     .initializer(&pl_close)
//!     .disposer(&pl_close)
//!     .entity(&pl_item)
//!     .yerr(&pl_yerr)
//!     .build("pipeline_id_here".into());
//! // pipeline is created with default `rank` 0
//! // if you want it has higher privilege to be executed
//! // increase it
//! assert_eq!(pipeline.rank(), 0);
//! ```

use crate::component::MetaResponse;
use crate::{App, BoxFuture, Response};
use http::Extensions;

macro_rules! builder {
    ($f:ident, $var: ident, $hd: ident, $ref:ident, $ret: ty, $bd: expr, $($item:ty,)*) => {
        #[doc = "Set the `"]
        #[doc = stringify!($hd)]
        #[doc = "` of `PipeLine`, if not called, the default value is `None`"]
        #[doc = ""]
        #[doc = "# Examples"]
        #[doc = ""]
        #[doc = "```rust"]
        #[doc = "# use dyer::pipeline::*;"]
        #[doc = concat!("async fn ", stringify!($hd), "(", stringify!($(_: &mut Vec<$item>, )*), "_: &mut App<E>) {}" )]
        #[doc = concat!("let pipeline = ", stringify!($bd))]
        #[doc = concat!("    ", stringify!(.$f),"(&", stringify!($hd), ")" ) ]
        #[doc = "    .build(\"marker\".into());"]
        #[doc = stringify!(assert!(pipeline.$hd.is_some()) )]
        #[doc = "```"]
        pub fn $f(
            mut self,
            $var: &'pl dyn for<'a> Fn($(Vec<$item>,)* &'a mut App<E>) -> BoxFuture<'a, $ret>,
        ) -> Self
        {
            self.$hd = Some($var );
            self
        }

        #[doc = "Get the shared reference of `"]
        #[doc = stringify!($hd)]
        #[doc = "` of `PipeLine`, if not set before, `None` is returned"]
        #[doc = ""]
        #[doc = "# Examples"]
        #[doc = ""]
        #[doc = "```rust"]
        #[doc = "# use dyer::pipeline::*;"]
        #[doc = concat!("async fn ", stringify!($hd), "(", stringify!($(_: &mut Vec<$item>, )*), "_: &mut App<E>) {}" )]
        #[doc = concat!("let pipeline = ", stringify!($bd))]
        #[doc = concat!("    ", stringify!(.$f),"(&", stringify!($hd), ")" ) ]
        #[doc = "    .build(\"marker\".into());"]
        #[doc = stringify!(assert_eq!(pipeline.$ref(), Some($hd)) )]
        #[doc = "```"]
        pub fn $ref(&self) ->
             Option<&'pl dyn for<'a> Fn($(Vec<$item>,)* &'a mut App<E>) -> BoxFuture<'a, $ret>>
             //Option<&'pl dyn for<'a> Fn(&'a mut Vec<$item>, &'a mut App<E>) -> BoxFuture<'a, ()>>
        {
            self.$hd
        }
    };
}

/// Represents a medium that manipulates the collected data structure of [App]
///
/// In practise, it processes the collected data and consume it
/// according to the `marker` and `rank`
pub struct PipeLine<'pl, E, C> {
    pub(crate) initializer: Option<&'pl dyn for<'a> Fn(&'a mut App<E>) -> BoxFuture<'a, Option<C>>>,

    pub(crate) disposer: Option<&'pl dyn for<'a> Fn(&'a mut App<E>) -> BoxFuture<'a, ()>>,

    pub(crate) process_entity:
        Option<&'pl dyn for<'a> Fn(Vec<E>, &'a mut App<E>) -> BoxFuture<'a, ()>>,

    pub(crate) process_yerr: Option<
        &'pl dyn for<'a> Fn(
            Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        ) -> BoxFuture<'a, ()>,
    >,
    pub marker: String,
    pub rank: i16,
    /// additional arguments for extensive application
    pub extensions: Extensions,
}

impl<'pl, E, C> PipeLine<'pl, E, C> {
    /// Create an instance of `PipeLinesBuilder` that used to build a `PipeLine`  
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// async fn process_entity(&mut Vec<E>, &mut App<E>) {}
    /// let pipeline = Pipeline::builder()
    ///     .process_entity(process_entity)
    ///     .build("marker");
    /// ```
    pub fn builder() -> PipeLineBuilder<'pl, E, C> {
        PipeLineBuilder::new()
    }

    /// get the rank of `PipeLine`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// async fn process_entity(&mut Vec<E>, &mut App<E>) {}
    /// let pipeline = Pipeline::builder()
    ///     .process_entity(process_entity)
    ///     .build("marker");
    /// assert_eq!(pipeline.rank(), 0);
    /// ```
    pub fn rank(&self) -> i16 {
        self.rank
    }

    /// get mutable reference to rank of `PipeLine`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// async fn process_entity(&mut Vec<E>, &mut App<E>) {}
    /// let pipeline = Pipeline::builder()
    ///     .process_entity(process_entity)
    ///     .build("marker");
    /// pipeline.rank_mut() = 3;
    /// assert_eq!(pipeline.rank(), 3);
    /// ```
    pub fn rank_mut(&mut self) -> &mut i16 {
        &mut self.rank
    }

    /// mutate the extensions of `PipeLine`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// let mut pipeline = Pipeline::builder()
    ///     .extensions(1i32)
    ///     .build("marker");
    /// pipeline.extension_mut().insert(2i32);
    /// ```
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    /// get extensions of `PipeLine`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// let pipeline = Pipeline::builder()
    ///     .extensions(1i32)
    ///     .build("marker");
    /// assert_eq!(pipeline.extensions().get::<i32>(), 1);
    /// ```
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    builder!(
        initializer_mut,
        initials,
        initializer,
        initializer,
        Option<C>,
        PipeLines::builder(),
    );
    builder!(
        disposer_mut,
        dispose,
        disposer,
        disposer,
        (),
        PipeLines::builder(),
    );
    builder!(
        entity_mut,
        entity,
        process_entity,
        entity,
        (),
        PipeLines::builder(),
        E,
    );
    builder!(
        yerr_mut,
        yerr,
        process_yerr,
        yerr,
        (),
        PipeLines::builder(),
        Result<Response, MetaResponse>,
    );
}

/// Serve as an medium to create an instance of [PipeLine]
///
/// This type can be used to construct an instance or [PipeLine]
/// through a builder-like pattern.
pub struct PipeLineBuilder<'pl, E, C>
where
    C: 'static,
{
    initializer: Option<&'pl dyn for<'a> Fn(&'a mut App<E>) -> BoxFuture<'a, Option<C>>>,

    disposer: Option<&'pl dyn for<'a> Fn(&'a mut App<E>) -> BoxFuture<'a, ()>>,

    process_entity: Option<&'pl dyn for<'a> Fn(Vec<E>, &'a mut App<E>) -> BoxFuture<'a, ()>>,

    process_yerr: Option<
        &'pl dyn for<'a> Fn(
            Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        ) -> BoxFuture<'a, ()>,
    >,
    rank: i16,
    extensions: Extensions,
}

impl<'pl, E, C> PipeLineBuilder<'pl, E, C> {
    /// Create an instance of `PipeLine`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeline::*;
    /// let pipeline = PipeLineBuilder::new();
    /// assert!(pipeline.entity_ref().is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            initializer: None,
            process_entity: None,
            process_yerr: None,
            disposer: None,
            rank: 0,
            extensions: Extensions::new(),
        }
    }

    /// Consume it and return an instance of PipeLine
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeline::*;
    /// async fn process_entity(_: &mut Vec<E>, _: &mut App<E>) {}
    /// let pipeline = PipeLineBuilder::new();
    ///     .entity(process_entity)
    ///     .build("marker");
    /// ```
    pub fn build<T: Into<String>>(self, marker: T) -> PipeLine<'pl, E, C> {
        let all = self.initializer.is_some()
            || self.disposer.is_some()
            || self.process_yerr.is_some()
            || self.process_entity.is_some();
        assert!(all, "None of pipeline has been specified");
        PipeLine {
            initializer: self.initializer,
            disposer: self.disposer,
            process_entity: self.process_entity,
            process_yerr: self.process_yerr,
            marker: marker.into(),
            rank: self.rank,
            extensions: self.extensions,
        }
    }

    /// set the extensions of `PipeLineBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// let pipeline = Pipeline::builder()
    ///     .extensions(1i32)
    ///     .build("marker");
    /// assert_eq!(pipeline.extensions().get::<i32>(), 1);
    /// ```
    pub fn extensions<S>(mut self, extensions: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.extensions.insert(extensions);
        self
    }

    /// get extensions of `PipeLineBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// let pipeline = Pipeline::builder()
    ///     .extensions(1i32)
    ///     .build("marker");
    /// assert_eq!(pipeline.extensions_ref().get::<i32>(), 1);
    /// ```
    pub fn extensions_ref(&self) -> &Extensions {
        &self.extensions
    }

    /// set the rank of `PipeLineBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// let pipeline = Pipeline::builder()
    ///     .rank(1)
    ///     .build("marker");
    /// assert_eq!(pipeline.rank(), 1);
    /// ```
    pub fn rank(mut self, rank: i16) -> Self {
        self.rank = rank;
        self
    }

    /// get rank of `PipeLineBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::pipeLine::*;
    /// async fn process_entity(&mut Vec<E>, &mut App<E>) {}
    /// let pipeline = Pipeline::builder()
    ///     .rank(1)
    ///     .build("marker");
    /// assert_eq!(pipeline.rank_ref(), 1);
    /// ```
    pub fn rank_ref(&self) -> i16 {
        self.rank
    }

    builder!(
        initializer,
        initials,
        initializer,
        initializer_ref,
        Option<C>,
        PipeLines::builder(),
    );
    builder!(
        disposer,
        dispose,
        disposer,
        disposer_ref,
        (),
        PipeLines::builder(),
    );
    builder!(
        entity,
        entity,
        process_entity,
        entity_ref,
        (),
        PipeLines::builder(),
        E,
    );
    builder!(
        yerr,
        yerr,
        process_yerr,
        yerr_ref,
        (),
        PipeLines::builder(),
        Result<Response, MetaResponse>,
    );
}
