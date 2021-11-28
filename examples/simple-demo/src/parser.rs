// Parsers that extract entities from Response
// external tool may be used to achieve that

use crate::entity::*;
use dyer::dyer_macros::parser;
use dyer::*;
use dyer::{component::Buf, Parsed, Response, Task};

#[parser]
pub fn parse_quote(res: Response) -> Parsed<Entities> {
    let mut r = Parsed::new();
    if res.body.is_empty() {
        return r;
    }
    let mut quotes = Vec::new();
    let s = std::str::from_utf8(res.body.bytes()).unwrap();
    let doc = select::document::Document::from(s);
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
        let task = Task::builder()
            .uri(format!("https://quotes.toscrape.com{}", next_url))
            .parser(parse_quote)
            .body(Body::empty(), "quote".into())
            .unwrap();
        r.task.push(task);
    }
    r
}
