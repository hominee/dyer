//! plugin that process data flow in and out of `Actor` between component.
//!
//! ```no_run
//! # use crate::dyer::component::{Affix, Request, Response, Task};
//! # use crate::dyer::{plug, MiddleWare};
//! # use crate::dyer::engine::App;
//! pub struct E;
//! # async fn handle_affix(_affixs: &mut Vec<Affix>, _app: &mut App<E>) {}
//! # async fn handle_task(_tasks: &mut Vec<Task>, _app: &mut App<E,  >) {}
//! # async fn handle_req(
//! #     _reqs: &mut Vec<Request>,
//! #     _app: &mut App<E>,
//! # ) -> (Vec<Task>, Vec<Affix>) {
//! #     (vec![], vec![])
//! # }
//! # async fn handle_res(_res: &mut Vec<Response< >>, _app: &mut App<E>) {}
//! # async fn handle_item(_items: &mut Vec<E>, _app: &mut App<E>) {}
//! # async fn handle_err(_res: &mut Vec<Result<Response, MetaResponse>>, _app: &mut App<E>) {}
//! # async fn handle_yerr(_res: &mut Vec<Result<Response, MetaResponse>>, _app: &mut App<E,  >) {}
//! // to initialize a `MiddleWare`
//! let middleware = MiddleWare::<E>::builder()
//!     .affix(&handle_affix)
//!     .task(&handle_task)
//!     .err(&handle_err)
//!     .build("middleware_id_here");
//! // middleware is created with default `rank` 0
//! // if you want it has higher privilege to be executed
//! // increase it
//! assert_eq!(middleware.rank(), 0);
//! ```

use crate::component::{Affix, Request, Response, Task};
use crate::engine::App;
use crate::plugin::BoxFuture;
use crate::response::MetaResponse;
use http::Extensions;

/// default method for process `Affix` in `MiddleWare`
pub async fn haffix<E>(_affixs: &mut Vec<Affix>, _app: &mut App<E>) {}

/// default method for process `Task` in `MiddleWare`
pub async fn htask<'r, 's, E>(_tasks: &mut Vec<Task>, _app: &mut App<E>) {}

/// default method for process `Request` in `MiddleWare`
pub async fn hreq<E>(_reqs: &mut Vec<Request>, _app: &mut App<E>) {}

/// default method for process `Response` in `MiddleWare`
pub async fn hres<E>(_res: &mut Vec<Response>, _app: &mut App<E>) {}

/// default method for process `Item` in `MiddleWare`
pub async fn hitem<E>(_items: &mut Vec<E>, _app: &mut App<E>) {}

/// default method for process failed `Response` in `MiddleWare`
pub async fn herr<E>(_res: &mut Vec<Result<Response, MetaResponse>>, _app: &mut App<E>) {
    let tasks = Vec::new();
    let affixs = Vec::new();
    let mut reqs = Vec::new();
    let mut yerrs = Vec::new();
    while let Some(res) = _res.pop() {
        match res {
            Ok(mut item) => {
                log::error!(
                    "Response Failed: {}, uri: {}",
                    item.status().as_u16(),
                    item.metas.info.from.to_string()
                );
                let used = item.metas.info.used;
                if used > 2 {
                    /*
                     *let errs = format!(
                     *    "status: {}\turi: {}\tcontent: {:?}\n",
                     *    &item.status().as_str(),
                     *    &item.metas.info.from.to_string(),
                     *    item.body
                     *);
                     */
                    log::error!("Task Fails 3+ times. drop it.");
                    // remove affix  as default
                    yerrs.push(Ok(item));
                } else {
                    item.metas.info.used += 1;
                    log::error!("{} Times Failure, Reuse This Task.", used + 1);
                    if let Some(couple) = _app.couple.get(&item.metas.info.id) {
                        let req = Request::from_couple(
                            couple,
                            None,
                            _app.exts_t_fn.as_ref(),
                            _app.exts_p_fn.as_ref(),
                        );
                        reqs.push(req);
                    }
                }
            }
            Err(mut m) => {
                log::error!("Response Failed uri: {}", m.info.from.to_string());
                let used = m.info.used;
                if used > 2 {
                    /*
                     *let errs = format!(
                     *    "cannot make a request to uri: {}\n",
                     *    &m.info.from.to_string(),
                     *);
                     */
                    log::error!("Task Fails 3+ times. drop it.");
                    // remove affix  as default
                    yerrs.push(Err(m));
                } else {
                    m.info.used += 1;
                    log::error!("{} Times Failure, Reuse This Task.", used + 1);
                    if let Some(couple) = _app.couple.get(&m.info.id) {
                        let req = Request::from_couple(
                            couple,
                            None,
                            _app.exts_t_fn.as_ref(),
                            _app.exts_p_fn.as_ref(),
                        );
                        reqs.push(req);
                    }
                }
            }
        }
    }
    _app.task.as_mut().extend(tasks);
    _app.affix.as_mut().extend(affixs);
    _app.req.as_mut().extend(reqs);
    _app.errs.as_mut().extend(yerrs);
}

/// default method for failing parsing `Response` in `MiddleWare`
pub async fn hyerr<E>(_res: &mut Vec<Result<Response, MetaResponse>>, _app: &mut App<E>) {}

macro_rules! builder {
    ($f:ident, $var: ident, $hd: ident, $item:ty, $ref:ident, $bd: expr) => {
        #[doc = "Set the `"]
        #[doc = stringify!($item)]
        #[doc = "` handler of `MiddleWare`, if not called, the default value is `None`"]
        #[doc = ""]
        #[doc = "# Examples"]
        #[doc = ""]
        #[doc = "```rust"]
        #[doc = "# use dyer::middleware::*;"]
        #[doc = concat!("async fn ", stringify!($hd), "(_: &mut Vec<", stringify!($item), ">, _: &mut App<E>) {}" )]
        #[doc = concat!("let middleware = ", stringify!($bd))]
        #[doc = concat!("    ", stringify!(.$f),"(&", stringify!($hd), ")" ) ]
        #[doc = "    .build(\"marker\".into());"]
        #[doc = stringify!(assert!(middleware.$hd.is_some()) )]
        #[doc = "```"]
        pub fn $f(
            mut self,
            $var: &'md dyn for<'a> Fn(&'a mut Vec<$item>, &'a mut App<E>) -> BoxFuture<'a, ()>,
        ) -> Self
        {
            self.$hd = Some($var );
            self
        }

        #[doc = "Get the shared reference of `"]
        #[doc = stringify!($item)]
        #[doc = "` handler of `MiddleWare`, if not set before, `None` is returned"]
        #[doc = ""]
        #[doc = "# Examples"]
        #[doc = ""]
        #[doc = "```rust"]
        #[doc = "# use dyer::middleware::*;"]
        #[doc = concat!("async fn ", stringify!($hd), "(_: &mut Vec<", stringify!($item), ">, _: &mut App<E>) {}" )]
        #[doc = concat!("let middleware = ", stringify!($bd))]
        #[doc = concat!("    ", stringify!(.$f),"(&", stringify!($hd), ")" ) ]
        #[doc = "    .build(\"marker\".into());"]
        #[doc = stringify!(assert_eq!(middleware.$ref(), Some($hd)) )]
        #[doc = "```"]
        pub fn $ref(&self) ->
             Option<&'md dyn for<'a> Fn(&'a mut Vec<$item>, &'a mut App<E>) -> BoxFuture<'a, ()>>
        {
            self.$hd
        }
    };
}

/// Represents a medium that handles the dataflow of [App]
///
/// In practise, it handles every data structure's in-and-out during execution
/// according to the `marker` and `rank`
pub struct MiddleWare<'md, E> {
    pub(crate) handle_affix:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Affix>, &'a mut App<E>) -> BoxFuture<'a, ()>>,

    pub(crate) handle_task:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Task>, &'a mut App<E>) -> BoxFuture<'a, ()>>,

    pub(crate) handle_req:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Request>, &'a mut App<E>) -> BoxFuture<'a, ()>>,
    pub(crate) handle_res:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Response>, &'a mut App<E>) -> BoxFuture<'a, ()>>,
    pub(crate) handle_entity:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<E>, &'a mut App<E>) -> BoxFuture<'a, ()>>,
    pub(crate) handle_yerr: Option<
        &'md dyn for<'a> Fn(
            &'a mut Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        ) -> BoxFuture<'a, ()>,
    >,
    pub(crate) handle_err: Option<
        &'md dyn for<'a> Fn(
            &'a mut Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        ) -> BoxFuture<'a, ()>,
    >,
    pub marker: String,
    pub rank: i16,
    pub extensions: Extensions,
}

impl<'md, E> MiddleWare<'md, E> {
    /// Create an instance of `MiddleWareBuilder` that used to build a `MiddleWare`  
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// async fn handle_task(&mut Vec<Task>, &mut App<E>) {}
    /// let middleware = Middleware::builder()
    ///     .task(handle_task)
    ///     .build("marker".into());
    /// ```
    pub fn builder() -> MiddleWareBuilder<'md, E> {
        MiddleWareBuilder::new()
    }

    /// get the rank of `MiddleWare`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleWare::*;
    /// async fn handle_task(&mut Vec<Task>, &mut App<E>) {}
    /// let middleware = MiddleWare::builder()
    ///     .rank(1)
    ///     .build("marker".into());
    /// assert_eq!(middleware.rank(), 0);
    /// ```
    pub fn rank(&self) -> i16 {
        self.rank
    }

    /// get mutable reference to rank of `MiddleWare`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// async fn handle_task(&mut Vec<Task>, &mut App<E>) {}
    /// let middleware = MiddleWare::builder()
    ///     .task(handle_task)
    ///     .build("marker".into());
    /// middleware.rank_mut() = 3;
    /// assert_eq!(middleware.rank(), 3);
    /// ```
    pub fn rank_mut(&mut self) -> &mut i16 {
        &mut self.rank
    }

    /// mutate the extensions of `MiddleWare`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// let mut middleware = Middleware::builder()
    ///     .extensions(1i32)
    ///     .build("marker".into());
    /// middleware.extension_mut().insert(2i32);
    /// ```
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    /// get extensions of `MiddleWare`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// let middleware = Middleware::builder()
    ///     .extensions(1i32)
    ///     .build("marker".into());
    /// assert_eq!(middleware.extensions().get::<i32>(), 1);
    /// ```
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    builder!(
        task_mut,
        task,
        handle_task,
        Task,
        task,
        MiddleWare::builder()
    );
    builder!(
        affix_mut,
        affix,
        handle_affix,
        Affix,
        affix,
        MiddleWare::builder()
    );
    builder!(
        entity_mut,
        entity,
        handle_entity,
        E,
        entity,
        MiddleWare::builder()
    );
    builder!(
        req_mut,
        req,
        handle_req,
        Request,
        req,
        MiddleWare::builder()
    );
    builder!(
        res_mut,
        res,
        handle_res,
        Response,
        res,
        MiddleWare::builder()
    );
    builder!(err_mut, err, handle_err, Result<Response, MetaResponse>, err, MiddleWare::builder());
    builder!(yerr_mut, yerr, handle_yerr, Result<Response, MetaResponse>, yerr, MiddleWare::builder());
}

/// Serve as an medium to create an instance of [MiddleWare]
///
/// This type can be used to construct an instance or [MiddleWare]
/// through a builder-like pattern.
pub struct MiddleWareBuilder<'md, E> {
    handle_affix:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Affix>, &'a mut App<E>) -> BoxFuture<'a, ()>>,

    handle_task:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Task>, &'a mut App<E>) -> BoxFuture<'a, ()>>,

    handle_req:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Request>, &'a mut App<E>) -> BoxFuture<'a, ()>>,
    handle_res:
        Option<&'md dyn for<'a> Fn(&'a mut Vec<Response>, &'a mut App<E>) -> BoxFuture<'a, ()>>,
    handle_entity: Option<&'md dyn for<'a> Fn(&'a mut Vec<E>, &'a mut App<E>) -> BoxFuture<'a, ()>>,
    handle_yerr: Option<
        &'md dyn for<'a> Fn(
            &'a mut Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        ) -> BoxFuture<'a, ()>,
    >,
    handle_err: Option<
        &'md dyn for<'a> Fn(
            &'a mut Vec<Result<Response, MetaResponse>>,
            &'a mut App<E>,
        ) -> BoxFuture<'a, ()>,
    >,
    rank: i16,
    extensions: Extensions,
}

impl<'md, E> MiddleWareBuilder<'md, E> {
    /// Create an instance of `MiddleWare`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// let middleware = MiddlewareBuilder::new();
    /// assert!(middleware.entity_ref().is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            handle_entity: None,
            handle_req: None,
            handle_affix: None,
            handle_task: None,
            handle_res: None,
            handle_err: None,
            handle_yerr: None,
            rank: 0,
            extensions: Extensions::new(),
        }
    }

    /// Consume it and return an instance of MiddleWare
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// async fn handle_entity(_: &mut Vec<E>, _: &mut App<E>) {}
    /// let middleware = MiddlewareBuilder::new();
    ///     .entity(handle_entity)
    ///     .build("marker".into());
    /// ```
    pub fn build(self, marker: String) -> MiddleWare<'md, E> {
        let all = self.handle_task.is_some()
            || self.handle_affix.is_some()
            || self.handle_req.is_some()
            || self.handle_res.is_some()
            || self.handle_entity.is_some()
            || self.handle_err.is_some()
            || self.handle_yerr.is_some();
        assert!(all, "None of MiddleWare has been specified");
        MiddleWare {
            handle_task: self.handle_task,
            handle_affix: self.handle_affix,
            handle_req: self.handle_req,
            handle_res: self.handle_res,
            handle_entity: self.handle_entity,
            handle_err: self.handle_err,
            handle_yerr: self.handle_yerr,
            marker,
            rank: self.rank,
            extensions: self.extensions,
        }
    }

    /// set the extensions of `MiddleWareBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// let middleware = middleware::builder()
    ///     .extensions(1i32)
    ///     .build("marker".into());
    /// assert_eq!(middleware.extensions().get::<i32>(), 1);
    /// ```
    pub fn extensions<S>(mut self, extensions: S) -> Self
    where
        S: std::any::Any + Send + Sync + 'static,
    {
        self.extensions.insert(extensions);
        self
    }

    /// get extensions of `MiddleWareBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// let middleware = middleware::builder()
    ///     .extensions(1i32)
    ///     .build("marker".into());
    /// assert_eq!(middleware.extensions_ref().get::<i32>(), 1);
    /// ```
    pub fn extensions_ref(&self) -> &Extensions {
        &self.extensions
    }

    /// set the rank of `MiddleWareBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleWare::*;
    /// let middleware = MiddleWareBuilder::new()
    ///     .rank(1)
    ///     .build("marker".into());
    /// assert_eq!(middleware.rank_ref(), 1);
    /// ```
    pub fn rank(mut self, rank: i16) -> Self {
        self.rank = rank;
        self
    }

    /// get rank of `MiddleWareBuilder`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dyer::middleware::*;
    /// let middleware = MiddleWareBuilder::new()
    ///     .rank(1)
    ///     .build("marker".into());
    /// middleware.rank_mut() = 3;
    /// assert_eq!(middleware.rank_ref(), 3);
    /// ```
    pub fn rank_ref(&self) -> i16 {
        self.rank
    }

    builder!(
        task,
        task,
        handle_task,
        Task,
        task_ref,
        MiddleWareBuilder::new()
    );
    builder!(
        affix,
        affix,
        handle_affix,
        Affix,
        affix_ref,
        MiddleWareBuilder::new()
    );
    builder!(
        entity,
        entity,
        handle_entity,
        E,
        entity_ref,
        MiddleWareBuilder::new()
    );
    builder!(
        req,
        req,
        handle_req,
        Request,
        req_ref,
        MiddleWareBuilder::new()
    );
    builder!(
        res,
        res,
        handle_res,
        Response,
        res_ref,
        MiddleWareBuilder::new()
    );
    builder!(err, err, handle_err, Result<Response, MetaResponse>, err_ref, MiddleWareBuilder::new());
    builder!(yerr, yerr, handle_yerr, Result<Response, MetaResponse>, yerr_ref, MiddleWareBuilder::new());
}
