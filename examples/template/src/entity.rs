use serde::{Deserialize, Serialize};

// the Entity to be used
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Item1 {
    pub field1: String,
    pub field2: i32,
}

/* serve as a container for all entities, and generic parameter of dyer::App
 * attribute #[dyer::entity(entities)] mark the enum and use it as container to all data to be collected
 */
#[dyer::entity(entities)]
#[derive(Serialize, Debug, Clone)]
pub enum Entities {
    Item1(Item1),
}
