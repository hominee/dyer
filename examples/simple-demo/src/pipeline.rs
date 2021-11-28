use crate::entity::Entities;
use dyer::dyer_macros::pipeline;
use dyer::App;

use std::fs::OpenOptions;
use std::io::{LineWriter, Write};
use std::sync::Once;

#[pipeline(initializer)]
pub async fn open_file<'r>(_app: &mut App<Entities>) -> Option<std::fs::File> {
    let path = "result.json";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .unwrap();
    Some(file)
}

// store Entities into file
#[pipeline(process_entity)]
pub async fn store_item(mut items: Vec<Entities>, app: &mut App<Entities>) {
    let mut ser_items = Vec::new();
    while let Some(Entities::Quote(item)) = items.pop() {
        let s = serde_json::to_string(&item).unwrap() + "\n";
        ser_items.push(s);
    }
    let stream = ser_items.join("");
    let mut writer = LineWriter::new(open_file(app).await.unwrap());
    writer.write(&stream.as_bytes()).unwrap();
}

// open a static file `result.json`
#[pipeline(initializer)]
pub async fn opener<'r, E>(_app: &'r mut App<E>) -> Option<&'static std::fs::File>
where
    E: Sized,
{
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
        VAL.as_ref()
    }
}

// the pipeline macro will wrap opener with un-moveable Box, it expands
/*
 *pub fn opener<'r, E>(
 *    _app: &'r mut App<E>,
 *) -> std::pin::Pin<Box<dyn std::future::Future<Output = ... > + 'r>>
 *where
 *    E: Sized,
 *{
 *    pub async fn opener<'r, E>(_app: &'r mut App<E>) -> ...
 *    where
 *        E: Sized,
 *    {
 *        // function body here
 *        ...
 *    }
 *
 *    std::pin::Pin::from(Box::new(opener(_app)))
 *}
 */
