use pulldown_cmark_wikilink::{Tag, TagEnd};
use std::rc::Rc;

use leptos::html::{HtmlElement, AnyElement, ElementDescriptor};

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
        Tag::Link{..} => TagEnd::Link,
        Tag::Image{..} => TagEnd::Image,
        Tag::MetadataBlock(k) => TagEnd::MetadataBlock(*k),
    }
}

#[derive(Clone)]
pub struct Callback<In,Out=()>(Rc<dyn Fn(In) -> Out>);

impl <In,Out> Callback<In,Out> {
    pub fn new<F: Fn(In) -> Out + 'static>(f: F) -> Self {
        Callback(Rc::new(f))
    }

    pub fn call(&self, value: In) -> Out {
        self.0(value)
    }
}

impl<In,Out,F> From<F> for Callback<In,Out> 
where F: Fn(In) -> Out + 'static {
    fn from(value: F) -> Callback<In,Out> {
        Callback::new(value)
    }
}

#[derive(Clone)]
pub struct HtmlCallback<In>(Rc<dyn Fn(In) -> HtmlElement<AnyElement>>);

impl<In> HtmlCallback<In> {
    pub fn new<F, H>(f: F) -> Self
    where H: ElementDescriptor + 'static,
          F: Fn(In) -> HtmlElement<H> + 'static
    {
        HtmlCallback(Rc::new(move |x| f(x).into_any()))
    }

    pub fn call(&self, value: In) -> HtmlElement<AnyElement> {
        self.0(value)
    }
}

impl<In,D,F> From<F> for HtmlCallback<In> 
where F: Fn(In) -> HtmlElement<D> + 'static,
      D: ElementDescriptor + 'static {
    fn from(value: F) -> HtmlCallback<In> {
        HtmlCallback::new(value)
    }
}
