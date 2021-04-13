// This example shows you a way to highly customize your app with rather that using dyer-cli
// Take Scrapy Tutorial as a guide to know the basics

extern crate dyer;
extern crate select;
extern crate serde;
extern crate simple_logger;
extern crate tokio;

use dyer::{log, plug, to_json, App, FutureExt, ProfileInfo};
use dyer::{MiddleWare, ParseResult, PipeLine, Request, Response, Spider, Task};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{LineWriter, Write};
use std::sync::{Arc, Mutex, Once};

type Stem<U> = Result<U, Box<dyn std::error::Error + Send + Sync>>;

// the data to be collected, make sure it is Deserializable and Serializable
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Quote {
    pub text: String,
    pub author: String,
    pub tags: Vec<String>,
}

// use `Entities` as a container of all possible Item.
#[derive(Serialize, Debug, Clone)]
pub enum Entities {
    Quote(Quote),
}

// uri is enough to get the data, so generic parameter of `Task` and `Profile` are not necessary
// leave them as empty for the sake of appearance
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Targ {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parg {}

// here `select` is implemented to extract the data embodied in the HTML
// for more infomation about how to do that,
// "https://github.com/utkarshkukreti/select.rs" is recommanded to explore
pub fn parse_quote(res: Response<Targ, Parg>) -> ParseResult<Entities, Targ, Parg> {
    let mut r = ParseResult::default();
    r.profile.push(res.profile);
    if res.content.is_none() {
        // for the `Response` with empty content, recycle profile
        return r;
    }
    let mut quotes = Vec::new();
    let doc = select::document::Document::from(res.content.as_ref().unwrap().as_str());
    for node in doc.find(select::predicate::Class("quote")) {
        let text = node
            .find(select::predicate::Class("text"))
            .next()
            .unwrap()
            .text();
        let author = node
            .find(select::predicate::Class("author"))
            .next()
            .unwrap()
            .text();
        let tags = node
            .find(select::predicate::Class("tag"))
            .map(|tag| tag.text())
            .collect::<Vec<String>>();
        let item = Quote { text, author, tags };
        quotes.push(Entities::Quote(item));
    }
    r.entities = quotes;

    // follow the next page if exists
    let mut next_node = doc.find(select::predicate::Class("next"));
    if let Some(nd) = next_node.next() {
        // next page exists
        let next_url = nd
            .find(select::predicate::Name("a"))
            .next()
            .unwrap()
            .attr("href")
            .unwrap();
        let mut task = Task::<Targ>::default();
        task.uri = format!("https://quotes.toscrape.com{}", next_url);
        task.parser = "parse_quote".to_string();
        r.task.push(task);
    }
    r
}

pub struct Spd {}
// implementing `Spider` for `Spd`
impl Spider<Entities, Targ, Parg> for Spd {
    // preparation before opening spider
    // nothing to do in this case
    fn open_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}

    // preparation before closing spider
    // nothing to do in this case
    fn close_spider(&self, _app: &mut App<Entities, Targ, Parg>) {}

    // set up parser that extracts `Quote` from the `Response`
    fn get_parser<'a>(
        &self,
        ind: String,
    ) -> Option<&'a dyn Fn(Response<Targ, Parg>) -> ParseResult<Entities, Targ, Parg>> {
        plug!(get_parser(ind; parse_quote))
    }

    // `Task` to be executed when starting `dyer`. Note that this function must reproduce a
    // non-empty vector, if not, the whole program will be left at blank.
    fn entry_task(&self) -> Stem<Vec<Task<Targ>>> {
        let mut task = Task::default();

        // all infomation needed is uri and parser
        task.uri = "https://quotes.toscrape.com".to_string();
        // parser is indexed by a `String` name, you can check that in the function `get_parser`.
        task.parser = "parse_quote".to_string();
        Ok(vec![task])
    }

    // the generator of `Profile`
    // `dyer` consume the returned `Request`, generate a `Response` fed to the closure
    // to generate a `Profile`
    fn entry_profile<'a>(&self) -> ProfileInfo<'a, Targ, Parg> {
        let mut req = Request::<Targ, Parg>::default();
        req.task.uri = "https://quotes.toscrape.com".to_string();
        // as the domain suggests this site is specially built for crawling,
        // you do not have to pretend as a real device,
        // leave the `Profile` processor as empty
        ProfileInfo {
            req: req,
            parser: None,
        }
    }
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

#[tokio::main]
async fn main() {
    // for the sake of clarification, use simple_logger to display some level-varied infomation
    // initialize simple_logger
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("dyer", log::LevelFilter::Debug)
        .init()
        .unwrap();
    let spd: Spd = Spd {};
    // since the `Quote` collected by parse_quote is complete, `MiddleWare` is not necessary,
    let middleware = plug!(MiddleWare < Entities, Targ, Parg > {});
    // writing a `PipeLine` to store them
    // for short, handle `Entities` only
    let pipeline = plug!(
        PipeLine < Entities,
        std::fs::File > { process_item: store_item }
    );
    // construct the app and start the crawler
    let mut app: App<Entities, Targ, Parg> = App::<Entities, Targ, Parg>::new();
    // donot use history, such as `Request`, `Profile`, `Task`
    app.rt_args.lock().unwrap().skip_history = true;
    app.run(&spd, &middleware, pipeline).await.unwrap();
}

// As you expected, It is Done.
// The output file `result.json` is in the root directory which contains file `Cargo.toml`, and
// its content displays
//
// ```json
// {"text":"“... a mind needs books as a sword needs a whetstone, if it is to keep its edge.”","author":"George R.R. Martin","tags":["books","mind"]}
// ...
// {"text":"“The world as we have created it is a process of our thinking. It cannot be changed without changing our thinking.”","author":"Albert Einstein",    "tags":["change","deep-thoughts","thinking","world"]}
// ```
