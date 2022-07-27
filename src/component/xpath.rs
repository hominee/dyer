use super::Response;
use libxml::{
    parser::{ParseFormat, Parser},
    tree::Node,
    xpath::Context,
};

/// implement the xpath for the response
impl Response {
    /// initialize the context for the root html
    fn xpath_context(&mut self) {
        if self.context.0.is_none() || self.context.1.is_none() {
            let input = self.body.bytes();
            assert!(input.len() != 0, "body must not be empty");
            let parser = Parser {
                format: ParseFormat::HTML,
            };
            let doc = parser.parse_string(input).unwrap();
            self.context.0 = Some(doc);
            let context = Context::new(self.context.0.as_ref().unwrap()).unwrap();
            self.context.1 = Some(context);
        }
    }

    /// parse the ready response's body with xpath
    /// the feature `xpath` must be enabled
    /// ```rust
    /// let html = r#"<!doctype html>
    /// <html>
    ///     <title>Hello dyer!</title>
    ///     <body> ... </body>
    /// </html>"#;
    /// let body = Body::from(html);
    /// let mut res = Response::new(body);
    /// let title = res.xpath("//title/text()");
    /// assert_eq!(
    ///     title[0]
    ///         .get_content(),
    ///     "Hello dyer!"
    /// );
    /// ```
    pub fn xpath(&mut self, xpath: &str) -> Vec<Node> {
        self.xpath_context();
        self.context
            .1
            .as_ref()
            .unwrap()
            .evaluate(xpath)
            .unwrap()
            .get_nodes_as_vec()
    }
}

#[test]
fn test_xpath() {
    use super::*;
    let html = r#"<!doctype html>
<html>
<head>
    <title>Hello dyer!</title>
    <meta charset="utf-8" />
    <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
</head>

<body>
    <h1>Hello dyer!</h1>
    <p>
        dyer is designed for reliable, flexible and fast web crawling, providing some high-level, comprehensive features without compromising speed.
    </p>
    <h2>Features</h2>
    <ol class="ol">
        <li>Async</li>
        <li class="class-wrapper">Concurrency</li>
        <li id="id-wrapper">Multi-Thread</li>
    </ol class="ol">
    <ol>
        <li>Command line Support</li>
        <li>Session Backup</li>
        <li>Signal Handling</li>
    </ol>
    <h2>Guide</h2>
    <p><strong>Get started</strong> by installing <a href="https://github.com/homelyguy/dyer-cli">dyer-cli</a> and looking over the <a href="https://github.com/HomelyGuy/dyer/tree/master/examples/">examples</a>.</p>
    <p>The <a href="https://homelyguy.github.io/dyer/">Cookbook</a> gives a detailed view of dyer.</p>
    <p><a href="https://github.com/homelyguy/dyer">See Repository...</a></p>
</body>
</html>
    "#;

    let body = Body::from(html);
    let mut res = Response::new(body);
    let title = res.xpath("/html/body/h1");
    assert_eq!(title.len(), 1);
    assert_eq!(title[0].get_content(), "Hello dyer!");
    let item1 = res.xpath("//*[@class=\"class-wrapper\"]");
    assert_eq!(item1[0].get_content(), "Concurrency");
    let h2 = res.xpath("//h2");
    assert_eq!(
        h2.iter()
            .map(|node| node.get_content())
            .collect::<Vec<String>>(),
        vec!["Features", "Guide"]
    );
    let li = res.xpath("//ol");
    let mut nodes = Vec::new();
    for node in li.iter() {
        let slice = node
            .findnodes("./li/text()")
            .unwrap()
            .into_iter()
            .map(|n| n.get_content())
            .collect::<Vec<_>>();
        assert_eq!(slice.len(), 3);
        nodes.extend(slice);
    }
    assert_eq!(
        nodes,
        vec![
            "Async",
            "Concurrency",
            "Multi-Thread",
            "Command line Support",
            "Session Backup",
            "Signal Handling"
        ]
    );
    let nodes = li
        .into_iter()
        .flat_map(|node| {
            node.findnodes("./li/text()")
                .unwrap()
                .into_iter()
                .map(|n| n.get_content())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        nodes,
        vec![
            "Async",
            "Concurrency",
            "Multi-Thread",
            "Command line Support",
            "Session Backup",
            "Signal Handling"
        ]
    );
    let nodes = res
        .xpath("//body//a/@href")
        .iter()
        .map(|node| node.get_content())
        .collect::<Vec<_>>();
    assert_eq!(
        nodes,
        vec![
            "https://github.com/homelyguy/dyer-cli",
            "https://github.com/HomelyGuy/dyer/tree/master/examples/",
            "https://homelyguy.github.io/dyer/",
            "https://github.com/homelyguy/dyer",
        ]
    );
}
