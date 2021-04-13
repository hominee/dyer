pub mod middleware;
pub mod pipeline;
pub mod spider;

#[doc(hidden)]
pub use middleware::MiddleWare;
#[doc(hidden)]
pub use pipeline::PipeLine;
#[doc(hidden)]
pub use spider::{ProfileInfo, Spider};

/// Macro that make a [plugin] at ease including `PipeLine` `MiddleWare`
///
/// [plugin]: crate::plugin
#[macro_export]
macro_rules! plug {
    // parts to tackle get_parser
    ( get_parser($index:ident; $field0: ident $(,)? $($field: ident $(,)?)*) ) => {
        if &$index == stringify!($field0) { Some(&$field0) }
        $(else if &$index == stringify!($field) { Some(&$field) })*
        else { None }
    };
    ( get_parser($($index:ident $(;)?)?) ) => {
        None
    };

    // parts to tackle `PipeLine`
    (open_pipeline, $open_pipeline: expr, $entity: ty,  $builder: ident) => {
        $builder.open_pipeline = &|| $open_pipeline().boxed_local()
    };
    (close_pipeline, $close_pipeline: expr, $entity: ty,  $builder: ident) => {
        $builder.close_pipeline = &|| $close_pipeline().boxed_local()
    };
    (process_item, $process_item: expr, $entity: ty,  $builder: ident) => {
        $builder.process_item = &|items: &mut Arc<Mutex<Vec<$entity>>>| $process_item(items).boxed_local()
    };
    (process_yerr, $process_yerr: expr, $entity: ty, $builder: ident) => {
        $builder.process_yerr = &|items: &mut Arc<Mutex<Vec<String>>>| $process_yerr(items).boxed_local()
    };
    (PipeLine< $entity:ty, $itm: ty> {
        $($field: ident: $value: expr $(,)?)*
    }) => {{
        let mut builder = PipeLine::<$entity, $itm>::builder().build();
        $(
            plug!($field, $value, $entity,  builder);
        )*
        builder
    }};

    // parts to tackle `MiddleWare`
    (hand_profile, $hand_profile: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.hand_profile = &|_profiles: &mut Vec<Profile<$p>>, _app: &mut App<$e, $t, $p>| $hand_profile(_profiles, _app).boxed_local()
    };
    (hand_task, $hand_task: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.hand_task = &|_tasks: &mut Vec<Task<$t>>, _app: &mut App<$e, $t, $p>| $hand_task(_tasks, _app).boxed_local()
    };
    (hand_req, $hand_req: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.hand_req = &|_reqs: &mut Vec<Request<$t,$p>>, _app: &mut App<$e, $t, $p>| $hand_req(_reqs, _app).boxed_local()
    };
    (hand_res, $hand_res: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.hand_res = &|_res: &mut Vec<Response<$t,$p>>, _app: &mut App<$e, $t, $p>| $hand_res(_res, _app).boxed_local()
    };
    (hand_item, $hand_item: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.hand_item = &|_items: &mut Vec<$e>, _app: &mut App<$e, $t, $p>| $hand_item(_items, _app).boxed_local()
    };
    (hand_yerr, $hand_yerr: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.hand_yerr = &|_yerrs: &mut Vec<Response<$t, $p>>, _app: &mut App<$e, $t, $p>| $hand_yerr(_yerrs, _app).boxed_local()
    };
    (hand_err, $hand_err: expr, $e: ty, $t: ty, $p: ty, $builder: ident ) => {
        $builder.hand_err = &|_errs: &mut Vec<Response<$t, $p>>, _app: &mut App<$e, $t, $p>| $hand_err(_errs, _app).boxed_local()
    };
    (MiddleWare<$e:ty,$t:ty,$p:ty> { $($field:ident: $value:expr $(,)?)* }) => {{
        let mut middleware = MiddleWare::<$e, $t, $p>::builder().build();
        $(
            plug!($field, $value, $e, $t, $p, middleware);
        )*
        middleware
    }};
}
