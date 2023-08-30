use leptos::*;

mod render;
use render::{Renderer, RenderContext};

pub use render::HtmlError;

use web_sys::MouseEvent;

use pulldown_cmark_wikilink::{Parser, Options, LinkType, Event};

mod utils;
use utils::{Callback, HtmlCallback};

use core::ops::Range;

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


#[component]
pub fn Markdown(
    cx: Scope,

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
    render_links: Option<HtmlCallback<LinkDescription>>,

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

    ) -> impl IntoView 
     {
    let context = RenderContext::new(
        cx,
        theme,
        on_click,
        render_links,
    );

    let options = parse_options.unwrap_or(Options::all());

    view! {cx,
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css" integrity="sha384-3UiQGuEI4TTMaFmGIZumfRPtfKQ3trwQE2JgosJxCnGmQpL/lJdjpcHkaaFwHlcI" crossorigin="anonymous"/>
        <div style="width:100%; padding-left: 10px"> 
            {move || src.with( |x| {
                let mut stream: Vec<_> = Parser::new_ext(x, options, wikilinks())
                    .into_offset_iter()
                    .collect();

                if hard_line_breaks() {
                    for (r, _) in &mut stream {
                        if *r == Event::SoftBreak {
                            *r = Event::HardBreak
                        }
                    }
                }

                Renderer::new(&context, &mut stream.into_iter()).collect_view(cx)
                })
            }
        </div>
    }
}

