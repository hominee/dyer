use std::sync::{Arc, Mutex, };

/// pipeline out the items
pub trait Pipeline<T, C> {
    fn open_pipeline(&self) -> Vec<C>;
    fn close_pipeline(&mut self);
    fn process_item(&self, item: &mut Arc<Mutex<Vec<T>>>);
    fn process_yielderr(&self, item: &mut Arc<Mutex<Vec<String>>>);
}

pub struct PipelineDefault<T> {
    data: std::marker::PhantomData<T>
}

impl<T> PipelineDefault<T> {
    pub fn new() -> Self {
        PipelineDefault{
            data: std::marker::PhantomData::<T>
        }
    }
}

impl<T> Pipeline<T, std::fs::File> for PipelineDefault<T> {
    fn open_pipeline(&self) -> Vec<std::fs::File> {
        /*
         *static INIT: Once = Once::new();
         *static mut VAL: Vec<C> = vec![];
         *unsafe {
         *    INIT.call_once(||{
         *        let clt = std::fs::File::open("").unwrap();
         *        VAL.push( clt );
         *    });
         *    &VAL
         *}
         */
        vec![]
    }

    fn close_pipeline(&mut self) {
        drop(self);
    }

    fn process_item(&self, item: &mut Arc<Mutex<Vec<T>>>) { 
        println!("process {} item", item.lock().unwrap().len());
    }

    fn process_yielderr(&self, item: &mut Arc<Mutex<Vec<String>>>) {
        println!("process {} yielderr", item.lock().unwrap().len());
    }
}
