use crate::entity::{Entities, Parg, Targ};
use dyer::dyer_macros::middleware;
use dyer::{App, Profile, Request, Task};

#[middleware(handle_profile)]
pub async fn handle_profile(
    _profiles: &mut Vec<Profile<Parg>>,
    _app: &mut App<Entities, Targ, Parg>,
) {
}
#[middleware(handle_task)]
pub async fn handle_task(_tasks: &mut Vec<Task<Targ>>, _app: &mut App<Entities, Targ, Parg>) {}
#[middleware(handle_req)]
pub async fn handle_req(
    _reqs: &mut Vec<Request<Targ, Parg>>,
    _app: &mut App<Entities, Targ, Parg>,
) -> (Vec<Task<Targ>>, Vec<Profile<Parg>>) {
    (vec![], vec![])
}
