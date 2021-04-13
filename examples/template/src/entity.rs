// define data structure here to be used or collected
// all data structures got to be Serializable and Deserializable

use serde::{Deserialize, Serialize};

// the Entity to be used
/*
 *#[derive(Deserialize, Serialize, Debug, Clone)]
 *pub struct Item1 {
 *    pub field1: String,
 *    pub field2: i32,
 *}
 */

// serve as a placeholder for all entities, and generic parameter of dyer::App
#[derive(Serialize, Debug, Clone)]
pub enum Entities {
    //Item1(Item1),
}

// serve as a appendix/complement to dyer::Task
// providing more infomation for this Task, leave it empty if not necessary
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Targ {}

// serve as a appendix/complement to dyer::Profile
// providing more infomation for this Profile, empty as default
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Parg {}