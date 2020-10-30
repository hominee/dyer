use crate::spider::Entity;
use log::debug;
use std::sync::{Arc, Mutex};

pub fn database(
    entities: Arc<Mutex<Vec<Entity>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let len = entities.lock().unwrap().len();
    debug!("receive {} entities. and stored in database.", len);
    Ok(())
}
