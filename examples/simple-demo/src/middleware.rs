use crate::entity::Entities;
use dyer::dyer_macros::middleware;
use dyer::{Affix, App, Request, Task};

#[middleware(handle_affix)]
pub async fn handle_affix(_affixs: &mut Vec<Affix>, _app: &mut App<Entities>) {}

#[middleware(handle_task)]
pub async fn handle_task(_tasks: &mut Vec<Task>, _app: &mut App<Entities>) {}

#[middleware(handle_req)]
pub async fn handle_req(_reqs: &mut Vec<Request>, _app: &mut App<Entities>) {}
