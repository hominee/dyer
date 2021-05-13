use serde::{Deserialize, Serialize};
use dyer::dyer_macros::entity;

// the Entity to be used
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Quote {
    pub text: String,
    pub author: String,
    pub tags: Vec<String>,
}

// serve as a placeholder for all entities
#[entity(entities)]
#[derive(Serialize, Debug, Clone)]
pub enum Entities {
    Quote(Quote),
}

// appendix/complement to dyer::Task
// leave it empty if not necessary
#[entity(targ)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Targ {}

// appendix/complement to dyer::Profile
// empty as default
#[entity(parg)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Parg {}
