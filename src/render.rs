use leptos::*;
use leptos::html::AnyElement;

use std::rc::Rc;

use markdown::{mdast, mdast::{Node, AlignKind, TableRow, TableCell}, unist::Point};
use katex;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Theme};

use web_sys::MouseEvent;

type Html = HtmlElement<AnyElement>;

/// `mouse_event` -> the original mouse event triggered when a text element was clicked on
/// `start_position: Point` -> the corresponding starting position in the markdown source
/// `end_position: Point` -> the corresponding ending position in the markdown source
#[derive(Clone, Debug)]
pub struct MarkdownMouseEvent {
    pub mouse_event: MouseEvent,
    pub start_position: Point,
    pub end_position: Point,
}


pub fn make_callback(context: &RenderContext, position: &Option<markdown::unist::Position>) 
    -> impl Fn(MouseEvent) + 'static 
{
    let position = position.clone()
        .expect("unable to know from which position the markdown tree was build");
    let onclick = context.onclick.clone();
    move |x| {
        let click_event = MarkdownMouseEvent {
            mouse_event: x,
            start_position: position.start.clone(),
            end_position: position.end.clone(),
        };
        onclick(click_event)
    }
}


/// all the context needed to render markdown:
/// - `syntax_set`, `theme`, `onclick`, `render_links`, `katex_opts`
pub struct RenderContext {
    cx: Scope,

    /// syntax used for syntax highlighting
    syntax_set: SyntaxSet,

    /// theme used for syntax highlighting
    theme: Theme,

    /// callback to add interactivity to the rendered markdown
    onclick: Rc<dyn Fn(MarkdownMouseEvent)>,

    /// callback used to render links
    render_links: Option<Box<dyn Fn(Scope, mdast::Link) -> Result<Html, HtmlError>>>,
}


impl RenderContext
{
    pub fn new(cx: Scope, theme_name: Option<String>, 
               onclick: Option<Box<dyn Fn(MarkdownMouseEvent)>>,
               render_links: Option<Box<dyn Fn(Scope, mdast::Link) -> Result<Html,HtmlError>>>)
-> Self 
{
        let theme_set = ThemeSet::load_defaults();
        let theme_name = theme_name
            .unwrap_or("base16-ocean.light".to_string());
        let theme = theme_set.themes.get(&theme_name)
            .expect("unknown theme")
            .clone();

        let syntax_set = SyntaxSet::load_defaults_newlines();

        let onclick : Rc<dyn Fn(_)->_> = match onclick {
            Some(x) => Rc::new(x),
            None => Rc::new(|_| ())
        };

        RenderContext {
            cx,
            syntax_set,
            theme,
            onclick,
            render_links,
        }
    }
}

pub fn render(ast: mdast::Node, context: &RenderContext) 
    -> Html
{
    let cx = context.cx;
    match render_node(context, &ast) {
        Ok(x) => x,
        Err(HtmlError(s)) => 
            view!(cx,
                  <div style="border:2px solid red">
                    {s}
                  </div>
        ).into_any()
    }
}


/// convert the input string or &str directly to an html node
fn raw_html(cx: Scope, text: String) -> HtmlElement<AnyElement> {
    view!(cx,
          <div inner_html=text>
          </div>
    ).into_any()
}


/// `highlight_code(content, ss, ts)` render the content `content`
/// with syntax highlighting
fn highlight_code(content: &str, lang: Option<&str>, context: &RenderContext) -> Option<Html> {
    Some( raw_html(context.cx, 
        syntect::html::highlighted_html_for_string(
            content,
            &context.syntax_set, 
            context.syntax_set.find_syntax_by_token(lang?)?,
            &context.theme
            ).ok()?
    ))
}


/// `render_header(d, s)` returns the html corresponding to
/// the string `s` inside a html header with depth `d`
fn render_header<I: IntoView>(cx: Scope, depth: u8, content: I) -> Html {
    match depth {
        1 => view!{cx, <h1>{content}</h1>}.into_any(),
        2 => view!{cx, <h2>{content}</h2>}.into_any(),
        3 => view!{cx, <h3>{content}</h3>}.into_any(),
        4 => view!{cx, <h4>{content}</h4>}.into_any(),
        5 => view!{cx, <h5>{content}</h5>}.into_any(),
        6 => view!{cx, <h6>{content}</h6>}.into_any(),
        _ => panic!("maximum heading level exceeded")
    }
}


/// `render_maths(content)` returns a html node
/// with the latex content `content` compiled inside
fn render_maths(cx: Scope, content: &str, display_mode: bool) 
    -> Result<Html, HtmlError>{
    let opts = katex::Opts::builder()
        .display_mode(display_mode)
        .build()
        .unwrap();

    match katex::render_with_opts(content, opts){
        Ok(x) => Ok(raw_html(cx, x)),
        Err(_) => HtmlError::err("invalid math")
    }
}

// fn extract_text(node: &mdast::Node) -> String {
//     match node {
//         Node::Text(t) => t.value.clone(),
//         _ => node.children()
//             .iter()
//             .map(|x| x.iter())
//             .flatten()
//             .map(extract_text)
//             .collect()
//     }
// }

fn render_links(context: &RenderContext, link: mdast::Link) -> Result<Html, HtmlError> 
{
    let cx = context.cx;
    match &context.render_links {
        Some(f) => f(context.cx, link),
        None => Ok(view!{cx,
                <a href={link.url}>
                    {render_children(context, &link.children)?}
                </a>
            }.into_any())
    }
}


/// `align_string(align)` gives the css string
/// that is used to align text according to `align`
fn align_string(align: &AlignKind) -> &'static str {
    match align {
        AlignKind::Left => "text-align: left",
        AlignKind::Right => "text-align: right",
        AlignKind::Center => "text-align: center",
        AlignKind::None => "",
    }
}

/// `render_cell(cell, align, context)` renders cell as html,
/// and use `align` to 
fn render_cell<'a> (context: &RenderContext, cell: &'a TableCell, align: &'a AlignKind) -> Result<Html, HtmlError> {
    let cx = context.cx;

    let html_children = cell.children.iter()
        .map(|x| render_node(context, x))
        .collect::<Result<Vec<_>, _>>()?
        .collect_view(cx);

    Ok(view!{ cx,
        <td style={align_string(align)}>
            {html_children}
        </td>
    }.into_any())
}

/// `render_table_row(row, align, context)` 
/// renders the markdown element `row` to html 
/// using `align` to align each cell.
/// `context` is used to render the child components
fn render_table_row<'a> (
        context: &RenderContext,
        row: &'a TableRow, 
        aligns: &Vec<AlignKind>, 
    ) -> Html {
    use core::iter::zip;

    let unwrap_cell = |node: &'a Node| match node {
        Node::TableCell(t) => t, 
        _ => panic!("table row contains ... that is not a cell"),
    };

    let cells = row.children.iter()
        .map(|x| unwrap_cell(x));

    let cx = context.cx;
    view!{ cx,
        <tr>
            {zip(cells, aligns)
                .map(|(c, a)| render_cell(context, c, a))
                .collect::<Result<Vec<_>,_>>()
                .collect_view(cx)
            }
        </tr>
    }.into_any()
}

pub struct HtmlError(String);

impl HtmlError {
    fn err<T>(message: &str) -> Result<T, Self>{
        Err(HtmlError(message.to_string()))
    }
}

fn render_children(context: &RenderContext, children: &[Node]) -> Result<View, HtmlError> {
    let cx = context.cx;
    Ok(children
        .iter()
        .map(|x| render_node(context, x))
        .collect::<Result<Vec<_>, _>>()?
        .collect_view(cx)
    )
}

/// `render_node(node, context)` returns an html view
/// of the markdown abstract syntax tree `node`.
/// It uses all the context present in `context`
fn render_node<'a>(context: &RenderContext, node: &'a Node) -> Result<Html, HtmlError> 
{
    let cx = context.cx;

    Ok(match node {
        Node::Html(n) => raw_html(context.cx, n.value.clone()),

        Node::Text(n) => {
            let onclick = make_callback(context, &n.position);
            view!{cx,
            <span on:click=onclick>
                {n.value.clone()}
            </span>
            }.into_any()
        },

        Node::Root(n) => {
            let child = render_children(context, &n.children)?;
            // TODO: why fragment does not have into_any() ?
            view!{cx,
            <ul>
                {child}
            </ul>
            }.into_any()
        },
        Node::BlockQuote(n) => view!{cx,
            <blockquote> { render_children(context, &n.children)?} </blockquote>
        }.into_any(),

        Node::FootnoteDefinition(_) => todo!(),

        Node::Break(_) => view!{cx, <br/>}.into_any(),
        Node::Delete(n) => view!{cx,
            <s>{render_children(context, &n.children)?}</s>
        }.into_any(),
        Node::Emphasis(n) => view!{cx,
            <i>{render_children(context, &n.children)?}</i>
        }.into_any(),
        Node::Strong(n) => view!{cx,
            <b>{render_children(context, &n.children)?}</b>
        }.into_any(),

        Node::Heading(n) => render_header(
            context.cx,
            n.depth, 
            render_children(context, &n.children)?
        ),
        Node::ThematicBreak(_) => view!{cx, <hr/>}.into_any(),
        Node::Paragraph(n) => view!{cx, 
            <p>{render_children(context, &n.children)?}</p>
        }.into_any(),

        Node::List(n) if n.ordered => view!{cx,
            <ol start={n.start.unwrap_or(0).to_string()}>
            {render_children(context, &n.children)?} 
            </ol>
        }.into_any(),

        Node::List(n) => view!{cx,
            <ul> {render_children(context, &n.children)?} </ul>
        }.into_any(),
        Node::ListItem(n) => view!{cx,
            <li>{render_children(context, &n.children)?}</li>
        }.into_any(),

        Node::TableRow(n) => view!{cx,
            <tr>{render_children(context, &n.children)?}</tr>
        }.into_any(),
        Node::TableCell(n) => view!{cx,
            <td>{render_children(context, &n.children)?}</td>
        }.into_any(),

        Node::Image(n) => view!{cx,
            <img src={n.url.clone()} alt={n.alt.clone()}/>
        }.into_any(),
        // TODO: what to do about `n.title` ?
        Node::Link(n) => render_links(context, n.clone())?,

        Node::InlineCode(c) => {
            let onclick = make_callback(context, &c.position);

            view!{cx,
                <code on:click=onclick>
                    {c.value.clone()}
                </code>
            }
        }.into_any(),
        Node::Code(c) => {
            let code_content = 
                highlight_code(&c.value, c.lang.as_deref(), context) 
                .unwrap_or_else(|| raw_html(cx, c.value.clone()));

            let onclick = make_callback(context, &c.position);

            view!{cx,
                <code on:click=onclick>
                    <pre>{code_content}</pre>
                </code>
            }
        }.into_any(),

        Node::Math(m) => {
            let onclick = make_callback(context, &m.position);
            let math = render_maths(context.cx, &m.value, true)?;
            view!{cx,
                <div class={"math-flow"} on:click=onclick>
                    {math}
                </div>
            }
        }.into_any(),

        Node::InlineMath(m) => {
            let onclick = make_callback(context, &m.position);
            view!{cx,
                <span class={"math-inline"} on:click=onclick>
                {render_maths(cx, &m.value, false)?}
                </span>
            }.into_any()
        },

        Node::Table(t) => {
            let unwrap_row = |node: &'a Node| match node {
                Node::TableRow(x) => x,
                _ => panic!("the table contains something that is not a row"),
            };
            let rows = t.children.iter()
                .map(|c| render_table_row(context, &unwrap_row(c), &t.align))
                .collect_view(cx);

            view!{cx,
                <table>
                    {rows}
                </table>
            }.into_any()
        },
        Node::FootnoteReference(_) => return HtmlError::err("footnote: not implemented"),
        Node::ImageReference(_) => return HtmlError::err("image ref: not implemented"),
        Node::LinkReference(_) => return HtmlError::err("link ref: not implemented"),
        Node::Definition(_) => return HtmlError::err("definition: not implemented"),

        // invisible
        Node::Toml(_) |
        Node::Yaml(_) => view!{cx, <div></div>}.into_any(),

        Node::MdxJsxTextElement(_) |
        Node::MdxTextExpression(_) |
        Node::MdxjsEsm(_) |
        Node::MdxJsxFlowElement(_) |
        Node::MdxFlowExpression(_)
            => return HtmlError::err("this part contain Mdx syntax, not yet implemented")
    })
}

