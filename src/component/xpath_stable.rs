//! implement the xpath for the response
//! more to see [Response::into_xpath]

use super::{ConcatText, Response};
use libxml::{
    parser::{ParseFormat, Parser},
    tree::{Document, Node as XmlNode},
    xpath::Context,
};
use std::error::Error as StdError;

/// Represents an data container for query response with xpath
/// including body string, context, xpath builder factory,
/// parsing package, parsed document.
///
/// it is obtained from method [Response::into_xpath] of Response instance,
/// and can make recursive xpath queries
#[cfg_attr(docsrs, doc(cfg(feature = "xpath-stable")))]
pub struct XpathResponse {
    pub(crate) context: Option<Context>,
    pub(crate) document: Document,
}

impl XpathResponse {
    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-stable")))]
    /// create an XpathResponse instance to make it ready for xpath query
    pub fn new<T>(body: T) -> Result<Self, Box<dyn StdError>>
    where
        T: AsRef<[u8]>,
    {
        //assert!(input.len() != 0, "body must not be empty");
        let parser = Parser {
            format: ParseFormat::HTML,
        };
        let document = parser.parse_string(body.as_ref())?;
        let mut xpath_res = XpathResponse {
            context: None,
            document,
        };
        let context = Context::new(&xpath_res.document).unwrap();
        xpath_res.context = Some(context);
        Ok(xpath_res)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-stable")))]
    /// make an xpath query wit xpath literal string `xpath_str`
    /// return a vector of `Node` if all is correct
    /// NOTE that empty vector is returned if any error happens
    /// the feature `xpath-stable` must be enabled
    ///
    /// ```rust
    /// let html = r#"<!doctype html>
    /// <html>
    ///     <title>Hello dyer!</title>
    ///     <body> ... </body>
    /// </html>"#;
    /// let body = Body::from(html);
    /// let res = Response::new(body);
    /// let doc = res.into_xpath().unwrap();
    /// let title = doc.xpath("//title/text()");
    /// assert_eq!(
    ///     title[0]
    ///         .get_content(),
    ///     "Hello dyer!"
    /// );
    /// ```
    pub fn xpath<'d>(&'d self, xpath_str: &str) -> Vec<Node> {
        if let Ok(nodes) = self.context.as_ref().unwrap().evaluate(xpath_str) {
            return nodes
                .get_nodes_as_vec()
                .into_iter()
                .map(|en| Node { inner: en })
                .collect::<Vec<_>>();
        }
        Vec::new()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "xpath-stable")))]
/// Represents an data node of the document for html/xml
/// with additional infomation: `Context`, xpath builder `Factory`
/// used when querying
///
/// it is obtained from method [XpathResponse::xpath] of XpathResponse instance,
/// and can make recursive xpath queries
pub struct Node {
    pub(crate) inner: XmlNode,
}

impl Node {
    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-stable")))]
    /// make an xpath query wit xpath literal string `xpath_str`
    /// return a vector of `Node` if all is correct
    /// NOTE that empty vector is returned if any error happens
    /// the feature `xpath-stable` must be enabled
    ///
    /// ```rust
    /// let html = r#"<!doctype html>
    /// <html>
    ///     <title>Hello dyer!</title>
    ///     <body> ... </body>
    /// </html>"#;
    /// let body = Body::from(html);
    /// let res = Response::new(body);
    /// let doc = res.into_xpath().unwrap();
    /// let title = doc.xpath("//title/text()");
    /// assert_eq!(
    ///     title[0]
    ///         .get_content(),
    ///     "Hello dyer!"
    /// );
    /// ```
    pub fn xpath(&self, xpath_str: &str) -> Vec<Node> {
        if let Ok(nodes) = self.inner.findnodes(xpath_str) {
            return nodes
                .into_iter()
                .map(|en| Node { inner: en })
                .collect::<Vec<_>>();
        }
        Vec::new()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-stable")))]
    /// get the string literal of the node
    pub fn get_content(&self) -> String {
        self.inner.get_content()
    }
}

impl ConcatText for Vec<Node> {
    /// get all node string and concat them
    /// as string
    fn get_all_content(&self) -> String {
        let mut s = String::new();
        if self.len() == 1 {
            return self[0].get_content();
        }
        self.iter().for_each(|e| {
            let content = e.get_content();
            s.push_str(&content);
        });
        s
    }
}

impl Response {
    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-stable")))]
    /// initialize the context for the root html and
    /// parse the ready response's body with xpath
    ///
    /// the feature `xpath-stable` must be enabled
    /// ```rust
    /// let html = r#"<!doctype html>
    /// <html>
    ///     <title>Hello dyer!</title>
    ///     <body> ... </body>
    /// </html>"#;
    /// let body = Body::from(html);
    /// let mut res = Response::new(body);
    /// let res_xpath = res.into_xpath().unwrap();
    /// let title = res_xpath.xpath("//title/text()");
    /// assert_eq!(
    ///     title[0]
    ///         .get_content(),
    ///     "Hello dyer!"
    /// );
    /// ```
    pub fn into_xpath<'d>(&'d self) -> Result<XpathResponse, Box<dyn StdError>> {
        XpathResponse::new(self.body.bytes())
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
    </ol >
    <ol class="ol">
        <li>Command line Support</li>
        <li>Session Backup</li>
        <li>Signal Handling</li>
    </ol>
    <h2>Guide</h2>
    <p><strong>Get started</strong> by installing <a href="https://github.com/hominee/dyer-cli">dyer-cli</a> and looking over the <a href="https://github.com/hominee/dyer/tree/master/examples/">examples</a>.</p>
    <p>The <a href="https://hominee.github.io/dyer/">Cookbook</a> gives a detailed view of dyer.</p>
    <p><a href="https://github.com/hominee/dyer">See Repository...</a></p>
</body>
</html>
    "#;

    let body = Body::from(html);
    let res = Response::new(body);
    let res_xpath = res.into_xpath().unwrap();
    let title = res_xpath.xpath("/html/body/h1");
    assert_eq!(title.len(), 1);
    assert_eq!(title[0].get_content(), "Hello dyer!");
    let item1 = res_xpath.xpath("//*[@class=\"class-wrapper\"]");
    assert_eq!(item1[0].get_content(), "Concurrency");
    let h2 = res_xpath.xpath("//h2");
    assert_eq!(
        h2.iter()
            .map(|node| node.get_content())
            .collect::<Vec<String>>(),
        vec!["Features", "Guide"]
    );
    let li = res_xpath.xpath("//ol");
    let mut nodes = Vec::new();
    for node in li.iter() {
        let slice = node
            .xpath("./li/text()")
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
            node.xpath("./li/text()")
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
    let nodes = res_xpath
        .xpath("//body//a/@href")
        .iter()
        .map(|node| node.get_content())
        .collect::<Vec<_>>();
    assert_eq!(
        nodes,
        vec![
            "https://github.com/hominee/dyer-cli",
            "https://github.com/hominee/dyer/tree/master/examples/",
            "https://hominee.github.io/dyer/",
            "https://github.com/hominee/dyer",
        ]
    );
}
