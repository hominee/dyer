//! Instructions of plugins including [middleware], [pipeline] and their usage.
//!
//! # OverView
//!
//! middleware serve as a processor between [components], processes data in-and-out.
//!
//! pipeline serve as the end of the data flow, data-storage happens here.
//!
//! [components]: crate::component
//! [middleware]: crate::plugin::middleware
//! [pipeline]: crate::plugin::pipeline
//!
pub mod middleware;
pub mod pipeline;

#[doc(hidden)]
pub use middleware::MiddleWare;
#[doc(hidden)]
pub use pipeline::PipeLine;

/// Macro that make a [plugin] at ease including `PipeLine` `MiddleWare`
///
/// [plugin]: crate::plugin
#[macro_export]
macro_rules! plug {
    // parts to tackle get_parser
    ( get_parser($index:ident; $field0: ident $(,)? $($field: ident $(,)?)*) ) => {
        if $index == stringify!($field0) { Some(&$field0) }
        $(else if &$index == stringify!($field) { Some(&$field) })*
        else { None }
    };
    ( get_parser($($index:ident $(;)?)?) ) => {
        None
    };

    // parts to tackle `PipeLine`
    (open_pipeline, $open_pipeline: expr, $entity: ty, $itm: ty,  $builder: ident) => {
        $builder.open_pipeline = &|| $open_pipeline().boxed_local()
    };
    (close_pipeline, $close_pipeline: expr, $entity: ty, $itm: ty, $builder: ident) => {
        $builder.close_pipeline = &|| $close_pipeline().boxed_local()
    };
    (process_entity, $process_entity: expr, $entity: ty, $itm: ty,  $builder: ident) => {
        $builder.process_entity = &|items: &mut Arc<Mutex<Vec<$entity>>>| $process_entity(items).boxed_local()
    };
    (process_yerr, $process_yerr: expr, $entity: ty,$itm: ty, $builder: ident) => {
        $builder.process_yerr = &|items: &mut Arc<Mutex<Vec<String>>>| $process_yerr(items).boxed_local()
    };
    (PipeLine< $entity:ty, $itm: ty> {
        $($field: ident: $value: expr $(,)?)*
    }) => {{
        let mut builder = PipeLine::<$entity, $itm>::builder().build();
        $(
            plug!($field, $value, $entity, $itm, builder);
        )*
            builder
    }};

    // parts to tackle `MiddleWare`
    (handle_profile, $handle_profile: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.handle_profile = &|_profiles: &mut Vec<Profile<$p>>, _app: &mut App<$e, $t, $p>| $handle_profile(_profiles, _app).boxed_local()
    };
    (handle_task, $handle_task: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.handle_task = &|_tasks: &mut Vec<Task<$t>>, _app: &mut App<$e, $t, $p>| $handle_task(_tasks, _app).boxed_local()
    };
    (handle_req, $handle_req: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.handle_req = &|_reqs: &mut Vec<Request<$t,$p>>, _app: &mut App<$e, $t, $p>| $handle_req(_reqs, _app).boxed_local()
    };
    (handle_res, $handle_res: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.handle_res = &|_res: &mut Vec<Response<$t,$p>>, _app: &mut App<$e, $t, $p>| $handle_res(_res, _app).boxed_local()
    };
    (handle_entity, $handle_entity: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.handle_entity = &|_items: &mut Vec<$e>, _app: &mut App<$e, $t, $p>| $handle_entity(_items, _app).boxed_local()
    };
    (handle_yerr, $handle_yerr: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.handle_yerr = &|_yerrs: &mut Vec<Response<$t, $p>>, _app: &mut App<$e, $t, $p>| $handle_yerr(_yerrs, _app).boxed_local()
    };
    (handle_err, $handle_err: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.handle_err = &|_errs: &mut Vec<Response<$t, $p>>, _app: &mut App<$e, $t, $p>| $handle_err(_errs, _app).boxed_local()
    };
    (MiddleWare$(::)?<$e:ty,$t:ty,$p:ty> { $($field:ident: $value:expr $(,)?)* }) => {{
        let mut middleware = MiddleWare::<$e, $t, $p>::builder().build();
        $(
            plug!($field, $value, $e, $t, $p, middleware);
        )*
        middleware
    }};

}
