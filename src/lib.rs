use rust_web_markdown::{
    render_markdown, ElementAttributes, HtmlElement, Context,
    MdComponentProps as VMdComponentProps,
    CowStr
};

pub type MdComponentProps = VMdComponentProps<View>;

pub use rust_web_markdown::{
    LinkDescription, Options,
};

use web_sys::MouseEvent;

use leptos::*;
use leptos::html::AnyElement;

use std::collections::HashMap;
use core::ops::Range;


#[cfg(feature="debug")]
pub mod debug {
    use super::*;
    #[derive(Copy, Clone)]
    pub struct EventInfo(pub WriteSignal<Vec<String>>);
}

#[derive(Clone, Debug)]
pub struct MarkdownMouseEvent {
    /// the original mouse event triggered when a text element was clicked on
    pub mouse_event: MouseEvent,

    /// the corresponding range in the markdown source, as a slice of [`u8`][u8]
    pub position: Range<usize>,

    // TODO: add a clonable tag for the type of the element
    // pub tag: pulldown_cmark::Tag<'a>,
}

impl<'a> Context<'a, 'static> for &'a __MdProps {
    type View = View;

    type HtmlCallback<T: 'static> = Callback<T, leptos::HtmlElement<AnyElement>>;

    type Handler<T: 'static> = Callback<T, ()>;

    type Setter<T: 'static> = WriteSignal<T>;

    type MouseEvent = MouseEvent;

    fn props(self) -> rust_web_markdown::MarkdownProps<'a, 'static, Self> {
        rust_web_markdown::MarkdownProps {
            components: &self.components,
            frontmatter: self.frontmatter.as_ref(),
            hard_line_breaks: self.hard_line_breaks.get(),
            wikilinks: self.wikilinks.get(),
            parse_options: self.parse_options.as_ref(),
            render_links: self.render_links.as_ref(),
            theme: self.theme.as_deref(),
        }
    }

    fn set<T: 'static>(self, setter: &WriteSignal<T>, value: T) {
        setter.set(value)
    }

    #[cfg(feature="debug")]
    fn send_debug_info(self, info: Vec<String>) {
        let set_event_info = use_context::<debug::EventInfo>();

        if let Some(setter) = set_event_info {
            setter.0.set(info);
        }
    }

    #[cfg(not(feature="debug"))]
        fn send_debug_info(self, _info: Vec<String>) {
    }

    fn el_with_attributes(
        self,
        e: HtmlElement,
        inside: Self::View,
        attributes: ElementAttributes<Callback<MouseEvent>>,
    ) -> Self::View {
        let mut r: leptos::HtmlElement<AnyElement> = match e {
            HtmlElement::Div => html::div().into_any(),
            HtmlElement::Span => html::span().into_any(),
            HtmlElement::Paragraph => html::p().into_any(),
            HtmlElement::BlockQuote => html::blockquote().into_any(),
            HtmlElement::Ul => html::ul().into_any(),
            HtmlElement::Ol(s) => html::ol().attr("start", s).into_any(),
            HtmlElement::Li => html::li().into_any(),
            HtmlElement::Heading(1) => html::h1().into_any(),
            HtmlElement::Heading(2) => html::h2().into_any(),
            HtmlElement::Heading(3) => html::h3().into_any(),
            HtmlElement::Heading(4) => html::h4().into_any(),
            HtmlElement::Heading(5) => html::h5().into_any(),
            HtmlElement::Heading(6) => html::h6().into_any(),
            HtmlElement::Heading(_) => panic!(),
            HtmlElement::Table => html::table().into_any(),
            HtmlElement::Thead => html::thead().into_any(),
            HtmlElement::Trow => html::tr().into_any(),
            HtmlElement::Tcell => html::td().into_any(),
            HtmlElement::Italics => html::i().into_any(),
            HtmlElement::Bold => html::b().into_any(),
            HtmlElement::StrikeThrough => html::s().into_any(),
            HtmlElement::Pre => html::pre().into_any(),
            HtmlElement::Code => html::code().into_any(),
        };

        r = r.child(inside);
        if let Some(s) = attributes.style {
            r = r.attr("style", s.to_string())
        }
        if let Some(c) = attributes.on_click {
            r = r.on(ev::click, move |e| Callable::call(&c, e));
        }
        r = r.classes(attributes.classes.join(" "));
        r.into_view()
    }

    fn el_span_with_inner_html(self, inner_html: String, attributes: ElementAttributes<Callback<MouseEvent>>) -> Self::View {
        let mut r = view!{
            <span inner_html=inner_html></span>
        }.into_any();

        if let Some(s) = attributes.style {
            r = r.attr("style", s.to_string())
        }
        if let Some(c) = attributes.on_click {
            r = r.on(ev::click, move |e| Callable::call(&c, e));
        }
        r = r.classes(attributes.classes.join(" "));
        r.into_view()
    }

    fn el_hr(self, attributes: ElementAttributes<Callback<MouseEvent>>) -> Self::View {
        let mut r = html::hr();

        if let Some(s) = attributes.style {
            r = r.attr("style", s.to_string())
        }
        if let Some(c) = attributes.on_click {
            r = r.on(ev::click, move |e| Callable::call(&c, e));
        }
        r = r.classes(attributes.classes.join(" "));
        r.into_view()
    }

    fn el_br(self) -> Self::View {
        view! {<br/>}.into_view()
    }

    fn el_fragment(self, children: Vec<Self::View>) -> Self::View {
        children.into_iter().collect()
    }

    fn el_a(self, children: Self::View, href: String) -> Self::View {
        view! {<a href={href.to_string()}>{children}</a>}.into_view()
    }

    fn el_img(self, src: String, alt: String) -> Self::View {
        view! {<img src={src.to_string()} alt={alt.to_string()}/>}.into_view()
    }

    fn el_text(self, text: CowStr<'a>) -> Self::View {
        text.to_string().into_view()
    }

    fn mount_dynamic_link(self, rel: &str, href: &str, integrity: &str, crossorigin: &str) {
        let document = document();

        let link = document.create_element("link").unwrap();

        link.set_attribute("rel", rel).unwrap();
        link.set_attribute("href", href).unwrap();
        link.set_attribute("integrity", integrity).unwrap();
        link.set_attribute("crossorigin", crossorigin).unwrap();

        document.head()
            .unwrap()
            .append_child(&link).unwrap();
    }

    fn el_input_checkbox(self, checked: bool, attributes: ElementAttributes<Callback<MouseEvent>>) -> Self::View {
        let mut r = html::input()
            .attr("type", "checkbox")
            .attr("checked", checked)
        ;
        if let Some(c) = attributes.on_click {
            r = r.on(ev::click, move |e| Callable::call(&c, e));
        }
        r = r.classes(attributes.classes.join(" "));
        if let Some(s) = attributes.style {
            r = r.attr("style", s.to_string())
        }
        r.into_view()
    }

    fn call_handler<T: 'static>(callback: &Self::Handler<T>, input: T) {
        Callable::call(callback, input)
    }

    fn call_html_callback<T: 'static>(callback: &Self::HtmlCallback<T>, input: T) -> Self::View {
        Callable::call(callback, input).into_view()
    }

    fn make_handler<T: 'static, F: Fn(T) + 'static>(
        self,
        f: F,
    ) -> Self::Handler<T> {
        Callback::new(f)
    }

    fn make_md_handler(self, position: Range<usize>, stop_propagation: bool) -> Self::Handler<MouseEvent> {
        match self.on_click {
            Some(f) => {
                let position = position.clone();
                Callback::new(move |e: MouseEvent| {
                    if stop_propagation {
                        e.stop_propagation()
                    }
                    let report = MarkdownMouseEvent {
                        position: position.clone(),
                        mouse_event: e
                    };
                    Callable::call(&f, report)
                })
            }
            None => Callback::new(move |_| ())
        }
    }
}


#[component]
#[allow(unused)]
pub fn __Md(
    /// the markdown text to render
    #[prop(into)]
    src: MaybeSignal<String>,

    /// the callback called when a component is clicked.
    /// if you want to controll what happens when a link is clicked,
    /// use [`render_links`][render_links]
    #[prop(optional, into)]
    on_click: Option<Callback<MarkdownMouseEvent>>,

    /// 
    #[prop(optional, into)]
    render_links: Option<Callback<LinkDescription<View>, leptos::HtmlElement<AnyElement>>>,

    /// the name of the theme used for syntax highlighting.
    /// Only the default themes of [syntect::Theme] are supported
    #[prop(optional, into)]
    theme: Option<String>,

    /// wether to enable wikilinks support.
    /// Wikilinks look like [[shortcut link]] or [[url|name]]
    #[prop(optional, into)]
    wikilinks: MaybeSignal<bool>,

    /// wether to convert soft breaks to hard breaks.
    #[prop(optional, into)]
    hard_line_breaks: MaybeSignal<bool>,

    /// pulldown_cmark options.
    /// See [`Options`][pulldown_cmark_wikilink::Options] for reference.
    #[prop(optional, into)]
    parse_options: Option<Options>,

    #[prop(optional, into)]
    components: HashMap<String, Callback<VMdComponentProps<View>, leptos::HtmlElement<AnyElement>>>,

    #[prop(optional, into)]
    frontmatter: Option<WriteSignal<String>>
) -> impl IntoView {
    ()
}


#[allow(non_snake_case)]
pub fn Markdown(props: __MdProps) -> impl IntoView {
    move || render_markdown(&props, &props.src.get())
}
