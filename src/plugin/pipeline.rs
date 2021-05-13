use futures::future::{FutureExt, LocalBoxFuture};
use std::sync::{Arc, Mutex};
use typed_builder::TypedBuilder;

/// default method used when open the `PipeLine`
async fn pl_open<'a, C>() -> &'a Option<C>
where
    C: 'a,
{
    &None
}

/// default method used when close the `PipeLine`
async fn pl_close<'a, I, C>()
where
    I: Send + Sync + std::fmt::Debug,
    C: 'a,
{
    let _c = std::marker::PhantomData::<C>;
}

/// default method used to consume extracted Entities
async fn pl_item<I>(item: &mut Arc<Mutex<Vec<I>>>)
where
    I: Send + Sync + std::fmt::Debug,
{
    let len = item.lock().unwrap().len();
    log::info!("process {} item", len);
    for _ in 0..len {
        let itm = item.lock().unwrap().pop().unwrap();
        println!("pipeline out item: {:?}", itm)
    }
}

/// default method used to process parsed failure
async fn pl_yerr(item: &mut Arc<Mutex<Vec<String>>>) {
    let len = item.lock().unwrap().len();
    log::info!("process {} yield_err", len);
    for _ in 0..len {
        let itm = item.lock().unwrap().pop().unwrap();
        println!("pipeline out item: {:?}", itm)
    }
}

/// the end of data flow, plugin that consume the extracted `Entity`, In general, the default
/// method does nothing, and customization is requird to store the data. An example:
/// ```
/// use futures::future::FutureExt;
/// # use std::sync::{Arc, Mutex};
/// # use crate::dyer::{plug, PipeLine};
/// #
/// # #[derive(std::fmt::Debug)]
/// # pub struct I;
/// # unsafe impl Send for I {}
/// # unsafe impl Sync for I {}
/// # #[derive(std::fmt::Debug)]
/// # pub struct C;
/// #
/// # async fn pl_open<'a>() -> &'a Option<C> {
/// #     &None
/// # }
/// # async fn pl_close() {}
/// # async fn pl_item(_item: &mut Arc<Mutex<Vec<I>>>) {}
/// # async fn pl_yerr(_item: &mut Arc<Mutex<Vec<String>>>) {}
///
/// // the recommanded way to initialize a `PipeLine` by means of macro `plug!`
/// plug! {
///     PipeLine<I, C> {
///         close_pipeline: pl_close,
///         //open_pipeline:  pl_open,
///         process_entity:   pl_item,
///         process_yerr:   pl_yerr,
///     }
/// };
///
/// // the the second way is also at ease
/// let mut pl = PipeLine::<I, C>::builder().build();
/// pl.close_pipeline = &|| pl_close().boxed_local();
///
/// // the traditional way is supported as well
/// PipeLine::<I, C> {
///     open_pipeline: &|| pl_open().boxed_local(),
///     close_pipeline: &|| pl_close().boxed_local(),
///     process_entity: &|items: &mut Arc<Mutex<Vec<I>>>| pl_item(items).boxed_local(),
///     process_yerr: &|items: &mut Arc<Mutex<Vec<String>>>| pl_yerr(items).boxed_local(),
///     };
/// ```
/// the member not specified is by default assigned to the default method
#[derive(TypedBuilder)]
pub struct PipeLine<'pl, I, C>
where
    C: 'pl,
    I: Send + Sync + std::fmt::Debug + 'pl,
{
    #[builder(default_code = r#"& | | pl_open().boxed_local() "#)]
    pub open_pipeline: &'pl dyn Fn() -> LocalBoxFuture<'pl, &'pl Option<C>>,

    #[builder(default_code = r#"&|  | pl_close::<I, C>().boxed_local() "#)]
    pub close_pipeline: &'pl dyn Fn() -> LocalBoxFuture<'pl, ()>,

    #[builder(
        default_code = r#" &|items: &mut Arc<Mutex<Vec<I>>>| pl_item(items).boxed_local() "#
    )]
    pub process_entity: &'pl dyn Fn(&mut Arc<Mutex<Vec<I>>>) -> LocalBoxFuture<'_, ()>,

    #[builder(
        default_code = r#" &|yerrs: &mut Arc<Mutex<Vec<String>>>| pl_yerr(yerrs).boxed_local() "#
    )]
    pub process_yerr: &'pl dyn Fn(&mut Arc<Mutex<Vec<String>>>) -> LocalBoxFuture<'_, ()>,
}
