use crate::entity::*;
use dyer::dyer_macros::parser;
use dyer::{ParseResult, Response, Task};

#[parser]
pub fn parse_quote(res: Response<Targ, Parg>) -> ParseResult<Entities, Targ, Parg> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut r = ParseResult::new();
    if res.content.is_none() {
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

        let mut s = DefaultHasher::new();
        text.hash(&mut s);
        (dyer::utils::now() as u64).hash(&mut s);
        let ss = i64::abs(s.finish() as i64);
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
    let mut next_node = doc.find(select::predicate::Class("next"));
    if let Some(nd) = next_node.next() {
        // next page exists
        let next_url = nd
            .find(select::predicate::Name("a"))
            .next()
            .unwrap()
            .attr("href")
            .unwrap();
        let mut task = Task::<Targ>::new();
        task.uri = format!("https://quotes.toscrape.com{}", next_url);
        task.parser = "parse_quote".to_string();
        r.task.push(task);
    }
    r
}
