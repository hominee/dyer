use crate::entity::*;
use dyer::*;

#[dyer::parser]
pub fn parse_quote(mut res: Response) -> Parsed<Entities> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

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

        let mut s = DefaultHasher::new();
        text.hash(&mut s);
        let ss = s.finish() as i64;
        let len = text.len();
        let role = if len >= 70 { Roles::Long } else { Roles::Short };
        let tag = if tags.is_empty() {
            None
        } else {
            Some(Tags(tags))
        };
        let item = Quote {
            id: ss,
            role,
            text,
            author,
            tags: tag,
        };
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
            .parser(parse_quote)
            .body(Body::empty(), "quote")
            .unwrap();
        r.task.push(task);
    }
    r
}
