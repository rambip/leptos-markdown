use leptos::*;
use leptos::html::AnyElement;

mod render;
use render::{Renderer, RenderContext};

pub use render::HtmlError;

use web_sys::MouseEvent;

use pulldown_cmark_wikilink::{ParserOffsetIter, Options, LinkType, Event};

mod utils;
mod component;

use core::ops::Range;
use std::collections::HashMap;

/// the description of a link, used to render it with a custom callback.
/// See [pulldown_cmark::Tag::Link] for documentation
pub struct LinkDescription {
    /// the url of the link
    pub url: String,

    /// the html view of the element under the link
    pub content: View,

    /// the title of the link. 
    /// If you don't know what it is, don't worry: it is ofter empty
    pub title: String,

    /// the type of link
    pub link_type: LinkType,

    /// wether the link is an image
    pub image: bool,
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

#[cfg(feature="debug")]
pub mod debug {
    use super::*;
    #[derive(Copy, Clone)]
    pub struct EventInfo(pub WriteSignal<Vec<String>>);
}

pub struct MdComponentProps {
    pub attributes: Vec<(String, String)>,
    pub children: View
}

#[derive(Default)]
pub struct ComponentMap (HashMap<&'static str, Callback<MdComponentProps, View>>);

impl ComponentMap {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add<F, I>(mut self, name: &'static str, f: F) -> Self 
    where F: Fn(MdComponentProps) -> I + 'static,
          I: IntoView {
        self.0.insert(name, Callback::new(move |props| f(props).into_view()));
        self
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
    render_links: Option<Callback<LinkDescription, HtmlElement<AnyElement>>>,

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
    parse_options: Option<pulldown_cmark_wikilink::Options>,

    #[prop(optional, into)]
    components: ComponentMap,

    #[prop(optional, into)]
    frontmatter: Option<WriteSignal<String>>

    ) -> impl IntoView 
     {
    let context = RenderContext::new(
        theme,
        on_click,
        render_links,
        components,
        frontmatter
    );

    let options = parse_options.unwrap_or(Options::all());

    #[cfg(feature="debug")]
    let set_debug_info = use_context::<debug::EventInfo>();

    view! {
        <link rel="stylesheet" 
            href="https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css" 
            integrity="sha384-3UiQGuEI4TTMaFmGIZumfRPtfKQ3trwQE2JgosJxCnGmQpL/lJdjpcHkaaFwHlcI" 
            crossorigin="anonymous"/>
            {move || src.with( |x| {
                let mut stream: Vec<_> = ParserOffsetIter::new_ext(x, options, wikilinks.get())
                    .collect();

                if hard_line_breaks.get() {
                    for (r, _) in &mut stream {
                        if *r == Event::SoftBreak {
                            *r = Event::HardBreak
                        }
                    }
                }

                #[cfg(feature="debug")]
                set_debug_info.map(|setter| (setter.0)(
                        stream.iter()
                                    .map(|(e, r)| format!("{r:?}: {e:?}"))
                                    .collect())
                );

                Renderer::new(&context, &mut stream.into_iter()).collect_view()
                })
            }
    }
}

