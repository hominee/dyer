use dyer::dyer_macros::entity;
use serde::{Deserialize, Serialize};

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
