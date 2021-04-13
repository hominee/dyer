// PipeLine that consume all entities, the end of data flow
// stdout the data as default, customaization is need for data storage

use crate::entity::Entities;
use dyer::to_json;
use dyer::{plug, FutureExt, PipeLine};
use std::fs::OpenOptions;
use std::io::{LineWriter, Write};
use std::sync::{Arc, Mutex, Once};

// something to do before sending entities to pipeline
// note that this function only runs one time
pub fn get_pipeline<'pl>() -> PipeLine<'pl, Entities, std::fs::File> {
    plug!(PipeLine<Entities, std::fs::File> {
        process_item: store_item,
    })
}

// open a static file `result.json`
async fn open_file(path: &str) -> &'static Option<std::fs::File> {
    static INIT: Once = Once::new();
    static mut VAL: Option<std::fs::File> = None;
    unsafe {
        INIT.call_once(|| {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)
                .unwrap();
            VAL = Some(file);
        });
        &VAL
    }
}
// store Entities into file
async fn store_item(items: &mut Arc<Mutex<Vec<Entities>>>) {
    let mut ser_items = Vec::new();
    while let Some(Entities::Quote(item)) = items.lock().unwrap().pop() {
        let s = to_json::to_string(&item).unwrap();
        ser_items.push(s);
    }
    let stream = ser_items.join("\n");
    let mut writer = LineWriter::new(open_file("result.json").await.as_ref().unwrap());
    writer.write(&stream.as_bytes()).unwrap();
}
