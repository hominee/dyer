//! A joints of a [Task] and [Affix]
//! in practice, it is used to reproduce a [Request]
//!
//! [Request]: crate::request::Request
use crate::component::{Affix, Task};
use crate::utils;
use std::hash::Hash;
use std::hash::Hasher;

/// Group a [Task] and a optional [Affix] to make this
/// it is the raw stuff before turned into a [Request]
///
/// [Request]: crate::request::Request
pub struct Couple {
    pub task: Task,
    pub affix: Option<Affix>,
    pub id: u64,
}

impl From<Task> for Couple {
    fn from(task: Task) -> Couple {
        let mut item = Couple {
            task,
            affix: None,
            id: 0,
        };
        item.id = utils::hash(&item);
        item
    }
}

impl Default for Couple {
    fn default() -> Self {
        Self {
            task: Task::default(),
            affix: None,
            id: 0,
        }
    }
}

impl Hash for Couple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.task.hash(state);
        self.affix.hash(state);
    }
}

impl Couple {
    pub fn new(task: Task, affix: Option<Affix>) -> Self {
        let mut item = Couple {
            id: 0,
            task: task,
            affix: affix,
        };
        let id = utils::hash(&item);
        item.id = id;
        item
    }
}
