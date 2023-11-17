use rust_web_markdown::{
    render_markdown, ElementAttributes, HtmlElement, WebFramework,
    MdComponentProps as WMdComponentProps
};

pub use rust_web_markdown::{
    LinkDescription, MarkdownMouseEvent, Options,
};


pub type MdComponentProps = WMdComponentProps<MarkdownContext>;

use leptos::*;
use leptos::html::AnyElement;

use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct MarkdownContext {}


#[cfg(feature="debug")]
pub mod debug {
    use super::*;
    #[derive(Copy, Clone)]
    pub struct EventInfo(pub WriteSignal<Vec<String>>);
}

impl WebFramework for MarkdownContext {
    type View = View;

    type HtmlCallback<T: 'static> = Callback<T, leptos::HtmlElement<AnyElement>>;

    type Callback<A: 'static, B: 'static> = Callback<A, B>;

    type Setter<T: 'static> = WriteSignal<T>;

    fn set<T: 'static>(&self, setter: &WriteSignal<T>, value: T) {
        setter.set(value)
    }

    #[cfg(feature="debug")]
    fn send_debug_info(&self, info: Vec<String>) {
        let set_event_info = use_context::<debug::EventInfo>();

        if let Some(setter) = set_event_info {
            setter.0.set(info);
        }
    }

    #[cfg(not(feature="debug"))]
    fn send_debug_info(&self, _info: Vec<String>) {
    }

    fn el_with_attributes(
        &self,
        e: HtmlElement,
        inside: Self::View,
        attributes: ElementAttributes<Self>,
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
        if let Some(i) = attributes.inner_html {
            r = r.inner_html(i.to_string());
        }
        r.into_view()
    }

    fn el_hr(&self, attributes: ElementAttributes<Self>) -> Self::View {
        let mut r = html::hr();

        if let Some(s) = attributes.style {
            r = r.attr("style", s.to_string())
        }
        if let Some(c) = attributes.on_click {
            r = r.on(ev::click, move |e| Callable::call(&c, e));
        }
        r = r.classes(attributes.classes.join(" "));
        if let Some(i) = attributes.inner_html {
            r = r.inner_html(i.to_string());
        }
        r.into_view()
    }

    fn el_br(&self) -> Self::View {
        view! {<br/>}.into_view()
    }

    fn el_fragment(&self, children: Vec<Self::View>) -> Self::View {
        children.into_iter().collect()
    }

    fn el_a(&self, children: Self::View, href: &str) -> Self::View {
        view! {<a href={href.to_string()}>{children}</a>}.into_view()
    }

    fn el_img(&self, src: &str, alt: &str) -> Self::View {
        view! {<img src={src.to_string()} alt={alt.to_string()}/>}.into_view()
    }

    fn el_text(&self, text: &str) -> Self::View {
        text.to_string().into_view()
    }

    fn mount_dynamic_link(&self, rel: &str, href: &str, integrity: &str, crossorigin: &str) {
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

    fn el_input_checkbox(&self, checked: bool, attributes: ElementAttributes<Self>) -> Self::View {
        let mut r = html::input()
            .attr("type", "checkbox")
            .attr("checked", checked)
        ;
        if let Some(c) = attributes.on_click {
            r = r.on(ev::click, move |e| Callable::call(&c, e));
        }
        r = r.classes(attributes.classes.join(" "));
        if let Some(i) = attributes.inner_html {
            r = r.inner_html(i.to_string());
        }
        if let Some(s) = attributes.style {
            r = r.attr("style", s.to_string())
        }
        r.into_view()
    }

    fn call_callback<A: 'static, B: 'static>(callback: &Self::Callback<A, B>, input: A) -> B {
        Callable::call(callback, input)
    }

    fn call_html_callback<T: 'static>(callback: &Self::HtmlCallback<T>, input: T) -> Self::View {
        Callable::call(callback, input).into_view()
    }

    fn make_callback<A: 'static, B: 'static, F: Fn(A) -> B + 'static>(
        f: F,
    ) -> Self::Callback<A, B> {
        Callback::new(f)
    }
}

#[component]
pub fn Markdown(
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
    render_links: Option<Callback<LinkDescription<MarkdownContext>, leptos::HtmlElement<AnyElement>>>,

    /// the name of the theme used for syntax highlighting.
    /// Only the default themes of [syntect::Theme] are supported
    #[prop(optional)] 
    theme: Option<String>,

    /// wether to enable wikilinks support.
    /// Wikilinks look like [[shortcut link]] or [[url|name]]
    #[prop(into, default=false.into())]
    wikilinks: MaybeSignal<bool>,

    /// wether to convert soft breaks to hard breaks.
    #[prop(into, default=false.into())]
    hard_line_breaks: MaybeSignal<bool>,

    /// pulldown_cmark options.
    /// See [`Options`][pulldown_cmark_wikilink::Options] for reference.
    #[prop(optional, into)]
    parse_options: Option<Options>,

    #[prop(optional, into)]
    components: HashMap<String, Callback<MdComponentProps, leptos::HtmlElement<AnyElement>>>,

    #[prop(optional, into)]
    frontmatter: Option<WriteSignal<String>>

    ) -> impl IntoView 
     {

    move || src.with(|s| {
         let props = rust_web_markdown::MarkdownProps {
             components: &components,
             frontmatter: frontmatter.as_ref(),
             hard_line_breaks: hard_line_breaks.get(),
             wikilinks: wikilinks.get(),
             on_click: on_click.as_ref(),
             parse_options: parse_options.as_ref(),
             render_links: render_links.as_ref(),
             theme: theme.as_deref(),
     };


        let cx = MarkdownContext {};
        render_markdown(cx, s, props)
    })
    


}

