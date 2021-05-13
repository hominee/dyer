use crate::entity::Entities;
use dyer::dyer_macros::pipeline;
use dyer::to_json;

use std::fs::{OpenOptions };
use std::io::{LineWriter, Write};
use std::sync::{Arc, Mutex, Once};

// open a static file `result.json`
#[pipeline(open_pipeline)]
pub async fn open_file() -> &'static Option<std::fs::File> {
    static INIT: Once = Once::new();
    static mut VAL: Option<std::fs::File> = None;
    unsafe {
        INIT.call_once(|| {
            let path = "result.json";
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
#[pipeline(process_entity)]
pub async fn store_item(items: &mut Arc<Mutex<Vec<Entities>>>) {
    let mut ser_items = Vec::new();
    while let Some(Entities::Quote(item)) = items.lock().unwrap().pop() {
        let s = to_json::to_string(&item).unwrap() + "\n";
        ser_items.push(s);
    }
    let stream = ser_items.join("");
    let mut writer = LineWriter::new(open_file().await.as_ref().unwrap());
    writer.write(&stream.as_bytes()).unwrap();
}
