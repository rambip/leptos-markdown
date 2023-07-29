use leptos::*;
use leptos::html::AnyElement;
use markdown::mdast;

mod render;
use render::{render, MarkdownMouseEvent, RenderContext, HtmlError};

mod parse;
use parse::{parse, default_constructs, new_parse_options};


#[component]
pub fn Markdown(
    cx: Scope,
    src: ReadSignal<String>,
    #[prop(optional)] 
    onclick: Option<Box<dyn Fn(MarkdownMouseEvent)>>,

    #[prop(optional)] 
    render_links: Option<Box<dyn Fn(mdast::Link) -> 
    Result<HtmlElement<AnyElement>, HtmlError>>>,

    #[prop(optional)] 
    theme: Option<String>,

    #[prop(default=false)]
    wikilinks: bool,

    #[prop(optional)]
    parse_options: Option<Box<dyn Fn(markdown::Constructs) -> markdown::Constructs>>,

    ) -> impl IntoView 
     {
    let context = RenderContext::new(
        cx,
        theme,
        onclick,
        render_links
    );

    let constructs = match parse_options {
        Some(f) => f(default_constructs()),
        None => default_constructs()
    };

    let parse_options = new_parse_options(constructs);


    view! {cx,
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css" integrity="sha384-3UiQGuEI4TTMaFmGIZumfRPtfKQ3trwQE2JgosJxCnGmQpL/lJdjpcHkaaFwHlcI" crossorigin="anonymous"/>
        <div style="width:100%"> 
            {move || src.with( |x| {
                    let ast = parse(x, &parse_options, wikilinks);
                    log!("{:?}", ast);
                    render(ast, &context)
                })
            }
        </div>
    }
}

