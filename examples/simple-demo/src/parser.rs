// Parsers that extract entities from Response
// external tool may be used to achieve that

use crate::entity::*;
use dyer::dyer_macros::parser;
use dyer::*;
use dyer::{Parsed, Response, Task};

#[parser]
pub fn parse_quote(mut res: Response) -> Parsed<Entities> {
    let mut r = Parsed::new();
    if res.body.is_empty() {
        return r;
    }
    let mut quotes = Vec::new();
    for node in res.xpath("//*[@class=\"quote\"]") {
        let text = node.findnodes(".//*[@class=\"text\"]/text()").unwrap()[0].get_content();
        let author = node.findnodes(".//*[@class=\"author\"]/text()").unwrap()[0].get_content();
        let tags = node
            .findnodes(".//*[@class=\"tag\"]/text()")
            .unwrap()
            .iter()
            .map(|node| node.get_content())
            .collect::<Vec<_>>();
        let item = Quote { text, author, tags };
        quotes.push(Entities::Quote(item));
    }
    r.entities = quotes;

    // follow the next page if exists
    let next_node = res.xpath("//*[@class=\"next\"]");
    if !next_node.is_empty() {
        // next page exists
        let nd = &next_node[0];
        let next_url = nd.findnodes(".//a/@href").unwrap()[0].get_content();
        let task = Task::builder()
            .uri(format!("https://quotes.toscrape.com{}", next_url))
            //.proxy("http://127.0.0.1:1080") // require feature `proxy` enabled
            .parser(parse_quote)
            .body(Body::empty(), "quote")
            .unwrap();
        r.task.push(task);
    }
    r
}
