extern crate async_trait;

use async_trait::async_trait;
use std::sync::{Arc, Mutex, Once};

/// pipeline out the items
#[async_trait]
pub trait Pipeline<T, C>
where
    T: std::fmt::Debug,
{
    async fn open_pipeline(&self) -> &'static Option<Arc<C>>;
    async fn close_pipeline(&self);
    async fn process_item(&self, item: &mut Arc<Mutex<Vec<T>>>);
    async fn process_yielderr(&self, item: &mut Arc<Mutex<Vec<String>>>);
}

pub struct PipelineDefault<T> {
    data: std::marker::PhantomData<T>,
}

impl<T> PipelineDefault<T> {
    pub fn new() -> Self {
        PipelineDefault {
            data: std::marker::PhantomData::<T>,
        }
    }
}

#[async_trait]
impl<T> Pipeline<T, std::fs::File> for PipelineDefault<T>
where
    T: std::fmt::Debug + Send + Sync,
{
    async fn open_pipeline(&self) -> &'static Option<Arc<std::fs::File>> {
        static INIT: Once = Once::new();
        static mut VAL: Option<Arc<std::fs::File>> = None;
        unsafe {
            INIT.call_once(|| {
                let file = std::fs::File::open("result").unwrap();
                VAL = Some(Arc::new(file));
            });
            &VAL
        }
    }

    async fn close_pipeline(&self) {
        drop(self);
    }

    async fn process_item(&self, item: &mut Arc<Mutex<Vec<T>>>)
    where
        T: Send + Sync,
    {
        let len = item.lock().unwrap().len();
        log::info!("process {} item", len);
        for _ in 0..len {
            let itm = item.lock().unwrap().pop().unwrap();
            println!("pipeline out item: {:?}", itm)
        }
    }

    async fn process_yielderr(&self, item: &mut Arc<Mutex<Vec<String>>>) {
        let len = item.lock().unwrap().len();
        log::info!("process {} yield_err", len);
        for _ in 0..len {
            let itm = item.lock().unwrap().pop().unwrap();
            println!("pipeline out item: {:?}", itm)
        }
    }
}
