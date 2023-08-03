use pulldown_cmark_wikilink::{Tag, TagEnd};
use std::rc::Rc;

pub fn as_closing_tag(t: &Tag) -> TagEnd {
    match t {
        Tag::Paragraph => TagEnd::Paragraph,
        Tag::Heading{level, ..} => TagEnd::Heading(*level),
        Tag::BlockQuote => TagEnd::BlockQuote,
        Tag::CodeBlock(_) => TagEnd::CodeBlock,
        Tag::List(b) => TagEnd::List(b.is_some()),
        Tag::Item => TagEnd::Item ,
        Tag::FootnoteDefinition(_) => TagEnd::FootnoteDefinition,
        Tag::Table(_) => TagEnd::Table,
        Tag::TableHead => TagEnd::TableHead ,
        Tag::TableRow => TagEnd::TableRow ,
        Tag::TableCell => TagEnd::TableCell ,
        Tag::Emphasis => TagEnd::Emphasis ,
        Tag::Strong => TagEnd::Strong ,
        Tag::Strikethrough => TagEnd::Strikethrough ,
        Tag::Link(_, _, _) => TagEnd::Link,
        Tag::Image(_, _, _) => TagEnd::Image,
        Tag::MetadataBlock(k) => TagEnd::MetadataBlock(*k),
    }
}

#[derive(Clone)]
pub struct Callback<In,Out=()>(Rc<dyn Fn(In) -> Out>);

impl <A,B> Callback<A,B> {
    pub fn new<F: Fn(A) -> B + 'static>(f: F) -> Self {
        Callback(Rc::new(f))
    }

    pub fn call(&self, value: A) -> B {
        self.0(value)
    }
}

impl<A,B,F> From<F> for Callback<A,B> 
where F: Fn(A) -> B + 'static {
    fn from(value: F) -> Callback<A,B> {
        Callback(Rc::new(value))
    }
}
