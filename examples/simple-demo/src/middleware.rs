use crate::entity::Entities;
use dyer::dyer_macros::middleware;
use dyer::{Affix, App, Request, Task};

#[middleware(handle_affix)]
pub async fn handle_affix(_affixs: &mut Vec<Affix>, _app: &mut App<Entities>) {}

#[middleware(handle_task)]
pub async fn handle_task(_tasks: &mut Vec<Task>, _app: &mut App<Entities>) {
    for itm in _tasks.iter() {
        if let Some(a) = itm.extensions().get::<i32>() {
            log::info!(
                "Task middleware get value from extensions: {} from: {}",
                a,
                itm.uri()
            );
        }
    }
}

#[middleware(handle_req)]
pub async fn handle_req(_reqs: &mut Vec<Request>, _app: &mut App<Entities>) {
    for itm in _reqs.iter() {
        if let Some(a) = itm.inner.extensions.0.get::<i32>() {
            log::info!("Request middleware get value from extensions: {}", a);
        }
    }
}
