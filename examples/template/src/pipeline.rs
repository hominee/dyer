// PipeLine that consume all entities, the end of data flow
// stdout the data as default, customaization is need for data storage

// there 4 methods availible:
//     1. open_pipeline
//     2. close_pipeline
//     3. process_item
//     4. process_yerr
// more details see https://docs.rs/dyer/plugin/pipeline/struct.PipeLine.html

use crate::entity::Entities;
use dyer::{plug, FutureExt, PipeLine};

// something to do before sending entities to pipeline
// note that this function only runs one time
async fn open_pipeline<'a>() -> &'a Option<std::fs::File> {
    &None
}

pub fn get_pipeline<'pl>() -> PipeLine<'pl, Entities, std::fs::File> {
    plug!(PipeLine<Entities, std::fs::File> {
        open_pipeline: open_pipeline,
    })
}