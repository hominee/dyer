use crate::entity::Entities;
use dyer::*;

#[dyer::middleware]
pub async fn handle_entities(_items: &mut Vec<Entities>, _app: &mut App<Entities>) {}
