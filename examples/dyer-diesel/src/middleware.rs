use crate::entity::{Entities, Parg, Targ};
use dyer::dyer_macros::middleware;
use dyer::App;

#[middleware(handle_entity)]
pub async fn handle_entities(_items: &mut Vec<Entities>, _app: &mut App<Entities, Targ, Parg>) {}
