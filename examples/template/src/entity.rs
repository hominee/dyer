use serde::{Deserialize, Serialize};
use dyer::dyer_macros::entity;

/*
 * the Entity to be used
 *
 *#[derive(Deserialize, Serialize, Debug, Clone)]
 *pub struct Item1 {
 *    pub field1: String,
 *    pub field2: i32,
 *}
 */

/* serve as a placeholder for all entities, and generic parameter of dyer::App
 * attribute #[entity(entities)] mark the enum and use it as container to all data to be collected
 */
#[entity(entities)]
#[derive(Serialize, Debug, Clone)]
pub enum Entities {
    //Item1(Item1),
}

// serve as a appendix/complement to dyer::Task,
// leave it empty if not necessary
// attribute #[entity(targ)] mark the struct and use it as generic type for `Task`
#[entity(targ)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Targ {}

// serve as a appendix/complement to dyer::Profile
// leave it empty as default
// attribute #[entity(parg)] mark the struct and use it as generic type for `Profile`
#[entity(parg)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Parg {}