//! implement the xpath for the response
//! more to see [Response::into_xpath]

use super::{ConcatText, Response};
use regex::Regex;
use sxd_document::{
    //dom::Document,
    parser::{parse, Error},
    Package,
};
use sxd_xpath::{nodeset, Context, Factory, Value as SxdValue};

/// Represents an data node of the document for html/xml
/// with additional infomation: `Context`, xpath builder `Factory`
/// used when querying
///
/// it is obtained from method [XpathResponse::xpath] of XpathResponse instance,
/// and can make recursive xpath queries
#[cfg_attr(docsrs, doc(cfg(feature = "xpath-alpha")))]
pub struct Node<'d> {
    pub(crate) inner: nodeset::Node<'d>,
    pub(crate) context: &'d Context<'d>,
    pub(crate) factory: &'d Factory,
}

impl<'d> ConcatText for Vec<Node<'d>> {
    /// get all node string and concat them
    /// as string
    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-alpha")))]
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

impl<'d> Node<'d> {
    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-alpha")))]
    /// make an xpath query wit xpath literal string `xpath_str`
    /// return a vector of `Node` if all is correct
    /// NOTE that empty vector is returned if any error happens
    /// the feature `xpath-alpha` must be enabled
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
    pub fn xpath<'c>(&'c self, xpath_str: &str) -> Vec<Node<'c>> {
        if let Ok(Some(xpath)) = self.factory.build(xpath_str) {
            if let Ok(value) = xpath.evaluate(self.context, self.inner) {
                if let SxdValue::Nodeset(ns) = value {
                    return ns
                        .document_order()
                        .into_iter()
                        .map(|e| Node {
                            inner: e,
                            context: &self.context,
                            factory: &self.factory,
                        })
                        .collect();
                }
            }
        }
        Vec::new()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-alpha")))]
    /// get the string literal of the node
    pub fn get_content(&self) -> String {
        self.inner.string_value()
    }
}

/// Represents an data container for query response with xpath
/// including body string, context, xpath builder factory,
/// parsing package, parsed document.
///
/// it is obtained from method [Response::into_xpath] of Response instance,
/// and can make recursive xpath queries
pub struct XpathResponse<'a> {
    pub(crate) body: String,
    pub(crate) package: Option<Package>,
    //pub(crate) document: Option<Document<'a>>,
    pub(crate) context: Context<'a>,
    pub(crate) factory: Factory,
}

impl<'a> XpathResponse<'a> {
    fn tag_close_attr(&mut self) {
        let pattern = Regex::new(
            r"(?P<before><\w+[^<>]*?)\s(?P<name>[-\w&&[^=]]+)(?P<tail>[/\s*])(?P<after>[^<>]*?>)",
        )
        .unwrap();
        //let mut s = "<html>\n<head><div disabled itemscope class=\"button\" hidden></div></head>\n<img hidden/></html>" .to_owned();
        while pattern.is_match(&self.body) {
            self.body = pattern
                .replace_all(&self.body, "$before $tail $name=\"\" $after")
                .into_owned();
            //println!("{:?}", self.body);
        }
        /*
         *let pattern1 = Regex::new(r"(?P<before><\w+.*?)\s(?P<name>[\w&&[^=]]+)>").unwrap();
         *while pattern1.is_match(&self.body) {
         *    self.body = pattern1
         *        .replace_all(&self.body, "$before $name=\"\">")
         *        .into_owned();
         *}
         */
        //println!("{:?}", self.body);
        /*
         *if pattern.is_match(&s) {
         *    s = pattern
         *        .replace_all(&s, "$before $name=\"\"$tail$after")
         *        .into_owned();
         *    println!("{:?}", s);
         *}
         *if pattern.is_match(&s) {
         *    s = pattern
         *        .replace_all(&s, "$before $name=\"\"$tail$after")
         *        .into_owned();
         *    println!("{:?}", s);
         *}
         */
    }

    fn rm_html_entity(&mut self) {
        let pattern = Regex::new(r"&[\w&&[^&;]]*?;").unwrap();
        self.body = pattern.replace_all(&self.body, "").into_owned();
    }

    fn self_close_tag(&mut self) {
        let body = &self.body;
        let pattern = Regex::new(r"(?x)<(?P<tag>(area)|(link)|(base)|(br)|(col)|(embed)|(hr)|(img)|(input)|(meta)|(param)|(source)|(track)|(wbr)|(command)|(keygen)|(menuitem))(?P<content> [^<]*?)(?P<tails>[^/\s])\s*>").unwrap();
        //let s = "<html>\n<head>\n<area charset=\"UTF-8\"/>\n<link role=\"anything\" href=\"https://image.com/url/\">\n</head>\n</html>";
        //let ss = pattern.replace_all(s, "<$tag$content$tails />");
        if pattern.is_match(body) {
            log::debug!("Complete the self-closing tag for html");
        }
        let pattern0 = Regex::new(r"/\s*>").unwrap();
        let body_tag_compact = pattern0.replace_all(body, "/>").into_owned();
        /*
         *for cap in pattern.captures_iter(body) {
         *    println!("tails:{:?}, content: {:?}", &cap["tails"], &cap["content"]);
         *}
         */
        let pattern1 = Regex::new(r"(^\s*<!\s*[Dd][Oo][Cc][Tt][Yy][Pp][Ee]\s*.*?\s*>\s*)").unwrap();
        let body_no_doctype = pattern1.replace_all(&body_tag_compact, "").into_owned();
        self.body = pattern
            .replace_all(&body_no_doctype, "<$tag$content$tails />")
            .into_owned();
        //println!("{:?}", self.body);
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-alpha")))]
    /// create an XpathResponse instance to make it ready for xpath query
    pub fn new<T>(body: T) -> Result<XpathResponse<'a>, Error>
    where
        T: Into<String>,
    {
        let mut xpath_res = XpathResponse {
            body: body.into(),
            package: None,
            //document: None,
            context: Context::new(),
            factory: Factory::new(),
        };
        xpath_res.self_close_tag();
        xpath_res.tag_close_attr();
        xpath_res.rm_html_entity();
        let package = parse(xpath_res.body.as_str())?;
        //.map_err(|e| log::error!("Failed to parse response: {:?}", e))
        //.map_err(|e| println!("Failed to parse response: {:?}", e))
        xpath_res.package = Some(package);
        /*
         *unsafe {
         *    let pac_ref = xpath_res.package.as_ref().unwrap();
         *    let pac_ref_ext = std::mem::transmute::<&'_ Package, &'a Package>(pac_ref);
         *    let doc = pac_ref_ext.as_document();
         *    xpath_res.document = Some(doc);
         *}
         */
        log::debug!("body is parsed");
        //assert!(xpath_res.document.is_some(), "document is not parsed");
        Ok(xpath_res)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-alpha")))]
    /// make an xpath query wit xpath literal string `xpath_str`
    /// return a vector of `Node` if all is correct
    /// NOTE that empty vector is returned if any error happens
    /// the feature `xpath-alpha` must be enabled
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
    pub fn xpath<'c>(&'c self, xpath_str: &str) -> Vec<Node<'c>> {
        if let Ok(Some(xpath)) = self.factory.build(xpath_str) {
            //assert!(self.document.is_some(), "document must not be null");
            let doc = self.package.as_ref().unwrap().as_document();
            if let Ok(value) = xpath.evaluate(&self.context, doc.root()) {
                if let SxdValue::Nodeset(ns) = value {
                    return ns
                        .document_order()
                        .into_iter()
                        .map(|e| Node {
                            inner: e,
                            context: &self.context,
                            factory: &self.factory,
                        })
                        .collect();
                }
            }
        }
        Vec::new()
    }
}

#[test]
fn test_xpath_res() {
    //let s = "\n<!DOCTYPE xml>\n<html lang=\"en\">\n<head>\n\t<meta charset=\"UTF-8\" />\n\t<title>Quotes to Scrape</title>\n    <link rel=\"stylesheet\" href=\"/static/bootstrap.min.css\" />\n    <link rel=\"stylesheet\" href=\"/static/main.css\" />\n</head>\n<body><meta class=\"keywords\" itemprop=\"keywords\" content=\"change,deep-thoughts,thinking,world\" /><img hidden high/><div nodeset disabled alt=\"hi\" node></div></body>\n</html>";
    let s = r#"
<html >
<link role="link-role-1" /  >
<link role="link-role-2" disabled hidden  >
<div data-modal-content data-modal-align="right" data-modal-trigger="hover" data-modal-offset="10px"><!-- any desired syntax can go here --></div>
<input disabled>
<link disabl-ed /  >
<link disabl-ed >

<body>
    <div class="container"></div>
            <div class="col-md-4 tags-box"></div>
    <footer class="footer">
        <div class="container">
            <p class="text-muted"> Quotes by: <a href="https://www.goodreads.com/quotes">GoodReads.com</a> </p>
            <p class="copyright"> Made
                with <span class='sh-red'>\u{9d}¤</span> by <a href="https://scrapinghub.com">Scrapinghub</a> </p>
            <p>I will display &permil;</p>
            <p>I will display &rarr;</p>
            <p>I will display &#8240;</p>
            <p>I will display &#x2030;</p>
            <p class="cn-text">你好呀 dyer.</p>
            <span class="emoji" >😀 😃 😄 😁 😆 😅 😂 🤣 ☺️  😊 😇 🙂 🙃 😉 😌 😍 🥰 😘 😗 😙 😚 😋 😛 😝 😜 🤪 🤨 🧐 🤓 😎 🤩 🥳 </span>
        </div>
    </footer>
</body>

</html>
    "#;
    let _res = Response::new(s);
    let _doc = _res.into_xpath().unwrap();
    assert_eq!(
        _doc.xpath("//p[@class='text-muted']/a/@href")[0].get_content(),
        "https://www.goodreads.com/quotes".to_owned()
    );
    dbg!(_doc.xpath("//p[@class='cn-text']/text()")[0].inner);
    assert_eq!(
        _doc.xpath("//p[@class='cn-text']/text()")[0].get_content(),
        "你好呀 dyer.".to_owned()
    );
    assert_eq!(
        _doc.xpath("//span[@class='emoji']/text()")[0].get_content(),
        "😀 😃 😄 😁 😆 😅 😂 🤣 ☺️  😊 😇 🙂 🙃 😉 😌 😍 🥰 😘 😗 😙 😚 😋 😛 😝 😜 🤪 🤨 🧐 🤓 😎 🤩 🥳 ".to_owned()
    );
    let res = Response::new(
        "<html><meta charset=\"UTF-8\">
    <body>
        <h1>Hello dyer</h1>
        <ol>
            <li>Xpath Support</li>
            <li><span>3.1415926</span></li>
        </ol>
        <div class='img' id='img-id'>
          <p> img text </p>
          <span class=\"text\" itemprop=\"text\">A person&#39;s a person, no matter how small.</span>
          <img src=\"https://image.com/url\" alt='img-alt' />
        </div>
    </body> 
    </html>",
    );
    let doc = res.into_xpath().unwrap();
    //println!("{:?}", doc.body);

    let ds = doc.xpath("//*[@class='text']//text()");
    assert_eq!(
        ds.get_all_content(),
        "A person's a person, no matter how small.".to_owned()
    );
    assert_eq!(doc.xpath("//h1")[0].get_content(), "Hello dyer".to_owned());
    assert_eq!(
        doc.xpath("//ol//li")[0].get_content(),
        "Xpath Support".to_owned()
    );
    let node_img = doc.xpath("//div[@class='img']/img");

    fn nested_xpath<'d>(n: &'d Node<'d>) {
        assert_eq!(
            n.xpath("./@src")[0].get_content(),
            "https://image.com/url".to_owned()
        );
    }
    assert_eq!(
        node_img[0].xpath("./@alt")[0].get_content(),
        "img-alt".to_owned()
    );
    nested_xpath(&node_img[0]);
    //assert_eq!(1, 2);
}

impl Response {
    #[cfg_attr(docsrs, doc(cfg(feature = "xpath-alpha")))]
    /// initialize the context for the root html
    ///
    /// Currently dyer's xpath-alpha feature is experimental and unstable,
    /// dyer adopt xml parser and modify the html content beforehand to
    /// fullfil the xpath feature.
    ///
    /// the underlaying functionlity mainly relies on sxd_xpath and sxd-document
    /// what were designed for xml parsing.
    /// Right now dyer only use `self-closing` tag completion and tag inner
    /// attribute completion and html entity(`&rarr;` etc.) removing to
    /// slightly change the html content which is very basic
    ///
    /// this feature can parse most of html without errors, However,
    /// as the html get complicated, it may find it is too basic to use,
    ///
    /// if any error happens to you, feel free to open an issue
    /// anyway right now use it with caution
    ///
    /// the feature `xpath-alpha` must be enabled
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
    pub fn into_xpath<'d>(&'d self) -> Result<XpathResponse<'d>, Error> {
        XpathResponse::new(self.body.to_string())
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
                .collect::<Vec<_>>()
                .into_iter()
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
