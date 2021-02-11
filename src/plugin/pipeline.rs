use futures::future::{FutureExt, LocalBoxFuture};
use std::sync::{Arc, Mutex};
use typed_builder::TypedBuilder;

/// default method used when open the `PipeLine`
async fn pl_open<'a, C>() -> &'a Option<Arc<C>>
where
    C: 'a,
{
    /*
     *static INIT: Once = Once::new();
     *static mut VAL: Option<Arc<C>> = None;
     *unsafe {
     *    INIT.call_once(|| {
     *        let file = std::fs::File::open("result").unwrap();
     *        VAL = Some(Arc::new(file));
     *    });
     *    &VAL
     *}
     */
    &None
}

/// default method used when close the `PipeLine`
async fn pl_close<'a, I, C>()
where
    I: Send + Sync + std::fmt::Debug,
{
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
/// async fn process_item(items: &mut Arc<Mutex<Vec<I>>>) {
///     let itms = items.lock().unwrap();
///     for _ in itms.len() {
///         itms.pop();
///     }
///     println!("consumed {} items", itms.len() )
/// }
/// let pipeline = PipeLine::builder().process_item(&|items: &mut Arc<Mutex<Vec<I>>>|
/// process_item(items).boxed_local() );
/// ```
/// the member not specified is by default assigned as the default method
#[derive(TypedBuilder)]
pub struct PipeLine<'pl, I, C>
where
    C: 'pl,
    I: Send + Sync + std::fmt::Debug + 'pl,
{
    #[builder(default_code = r#"& | | pl_open().boxed_local() "#)]
    pub open_pipeline: &'pl dyn Fn() -> LocalBoxFuture<'pl, &'pl Option<Arc<C>>>,

    #[builder(default_code = r#"&|  | pl_close::<I, C>().boxed_local() "#)]
    pub close_pipeline: &'pl dyn Fn() -> LocalBoxFuture<'pl, ()>,

    #[builder(
        default_code = r#" &|items: &mut Arc<Mutex<Vec<I>>>| pl_item::<I>(items).boxed_local() "#
    )]
    pub process_item: &'pl dyn Fn(&mut Arc<Mutex<Vec<I>>>) -> LocalBoxFuture<'_, ()>,

    #[builder(
        default_code = r#" &|yerrs: &mut Arc<Mutex<Vec<String>>>| pl_yerr(yerrs).boxed_local() "#
    )]
    pub process_yerr: &'pl dyn Fn(&mut Arc<Mutex<Vec<String>>>) -> LocalBoxFuture<'_, ()>,
}

/*
 * /// pipeline out the items
 *#[async_trait]
 *pub trait Pipeline<T, C>
 *where
 *    T: std::fmt::Debug,
 *{
 *    async fn open_pipeline(&self) -> &'static Option<Arc<C>>;
 *    async fn close_pipeline(&self);
 *    async fn process_item(&self, item: &mut Arc<Mutex<Vec<T>>>);
 *    async fn process_yielderr(&self, item: &mut Arc<Mutex<Vec<String>>>);
 *}
 *
 *pub struct PipelineDefault<T, C> {
 *    _t: std::marker::PhantomData<T>,
 *    _c: std::marker::PhantomData<C>,
 *}
 *
 *impl<T, C> PipelineDefault<T, C> {
 *    pub fn new() -> Self {
 *        PipelineDefault {
 *            _t: std::marker::PhantomData::<T>,
 *            _c: std::marker::PhantomData::<C>,
 *        }
 *    }
 *}
 */

/*
 *#[async_trait]
 *impl<T> Pipeline<T, std::fs::File> for PipelineDefault<T, std::fs::File>
 *where
 *    T: std::fmt::Debug + Send + Sync,
 *{
 *    async fn open_pipeline(&self) -> &'static Option<Arc<std::fs::File>> {
 *        static INIT: Once = Once::new();
 *        static mut VAL: Option<Arc<std::fs::File>> = None;
 *        unsafe {
 *            INIT.call_once(|| {
 *                let file = std::fs::File::open("result").unwrap();
 *                VAL = Some(Arc::new(file));
 *            });
 *            &VAL
 *        }
 *    }
 *
 *    async fn close_pipeline(&self) {
 *        drop(self);
 *    }
 *
 *    async fn process_item(&self, item: &mut Arc<Mutex<Vec<T>>>)
 *    where
 *        T: Send + Sync,
 *    {
 *        let len = item.lock().unwrap().len();
 *        log::info!("process {} item", len);
 *        for _ in 0..len {
 *            let itm = item.lock().unwrap().pop().unwrap();
 *            println!("pipeline out item: {:?}", itm)
 *        }
 *    }
 *
 *    async fn process_yielderr(&self, item: &mut Arc<Mutex<Vec<String>>>) {
 *        let len = item.lock().unwrap().len();
 *        log::info!("process {} yield_err", len);
 *        for _ in 0..len {
 *            let itm = item.lock().unwrap().pop().unwrap();
 *            println!("pipeline out item: {:?}", itm)
 *        }
 *    }
 *}
 */
