use std::sync::{Arc, Mutex, Once};
use crate::component::Entity;

/// pipeline out the items
pub trait Pipeline {
    fn open_pipeline(&self) -> &Vec<std::fs::File>;
    fn close_pipeline(&mut self) ;
    fn process_item(&self, item: Arc<Mutex<Vec<Entity>>> ) ;
    fn process_yielderr(&self, item: Arc<Mutex<Vec<String>>> ) ;
}

pub struct MPipeline{
    pub path: String,
}

//impl Pipeline<std::fs::File> for MPipeline {
impl Pipeline for MPipeline {
    fn open_pipeline(&self) -> &Vec<std::fs::File> {
        static INIT: Once = Once::new();
        static mut VAL: Vec<std::fs::File> = vec![];
        unsafe {
            INIT.call_once(||{
                let clt = std::fs::File::open(&self.path).unwrap();
                VAL.push( clt );
            });
            &VAL
        }
    }

    fn close_pipeline(&mut self) {

    }

    fn process_item(&self, item: Arc<Mutex<Vec<Entity>>>) {

    }

    fn process_yielderr(&self, item: Arc<Mutex<Vec<String>>>) {

    }
}
