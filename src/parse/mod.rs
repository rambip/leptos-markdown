use markdown::{Constructs, ParseOptions, mdast, mdast::Node};
mod token;
use token::{TokenStream, Token, Point, Position};
use core::iter::Peekable;

pub fn default_constructs() -> Constructs {
    Constructs {
        // enabled by default
        html_flow: true,
        list_item: true,
        math_flow: true,
        math_text: true,
        attention: true,
        thematic_break: true,
        frontmatter: true,
        block_quote: true,
        character_escape: true,
        code_fenced: true,
        code_text: true,
        character_reference: true,
        gfm_strikethrough: true,
        autolink: true,
        gfm_autolink_literal: true,
        heading_atx: true,
        heading_setext: true,
        label_start_image: true,
        label_start_link: true,
        label_end: true,
        gfm_table: true,
        // see `preprocess_hardbreaks`
        hard_break_trailing: true,

        // TODO
        definition: false,
        gfm_task_list_item: false,
        gfm_footnote_definition: false,
        gfm_label_start_footnote: false,

        // maybe one day
        mdx_esm: false,
        mdx_expression_flow: false,
        mdx_expression_text: false,
        mdx_jsx_flow: false,
        mdx_jsx_text: false,

        // not supported by default
        code_indented: false,
        hard_break_escape: false,
        html_text: false,

    }
}

pub fn new_parse_options(constructs: markdown::Constructs) -> ParseOptions {
    ParseOptions{
        constructs,
        gfm_strikethrough_single_tilde: true,
        math_text_single_dollar: true,
        mdx_expression_parse: None,
        mdx_esm_parse: None,
    }
}

pub fn parse(source: &str, parse_options: &markdown::ParseOptions, wikilinks: bool) -> Node {
    let mut ast = markdown::to_mdast(&source.to_string(), parse_options).expect("unable to parse markdown");
    postprocess(source, &mut ast, wikilinks);
    ast
}

fn postprocess(source: &str, ast: &mut Node, wikilinks: bool) {
    match ast {
        Node::Text(mdast::Text{position, ..}) if wikilinks => {
            *ast = Node::Paragraph(mdast::Paragraph{
                position: position.clone(),
                children: Parser::new_at(source, position.clone().unwrap()).collect()
            })
        },
        Node::InlineMath(m) if is_inline_latex(source, &m) => 
            *ast = Node::Math(mdast::Math{
                value: m.value.clone(),
                position: m.position.clone(),
                meta: None,
            }),
        x => {
            for c in x.children_mut().into_iter().flatten() {
                postprocess(source, c, wikilinks)
            }
        }
    }
}

fn text_content(s: String, p: Position) -> Node {
    Node::Text(mdast::Text {
        value: s,
        position: Some(p),
    })
}

type PeekTokenStream<'a> = Peekable<TokenStream<'a>>;

struct Parser<'a> {
    tokens: PeekTokenStream<'a>
}

impl<'a> Parser<'a> {
    fn new_at(source: &'a str, position: Position) -> Self {
        let text = &source[position.start.offset..position.end.offset];
        Self {
            tokens: TokenStream::new_at(&text, position.start).peekable(),
        }
    }
}

#[derive(Debug)]
struct ParseError(String, Option<Point>);
use Token::*;

fn parse_wikilink_first_field<'a>(tokens: &mut PeekTokenStream<'a>) -> Result<(String, Position), ParseError> {
    let start : Point = match tokens.peek(){
        Some((_, x)) => x.start.clone(),
        None => return Err(ParseError(String::new(), None))
    };
    let mut name = String::new();
    let mut end: Point = start.clone();
    loop {
        match tokens.peek() {
            Some((Pipe, _))| Some((RRBra, _)) => break Ok((name, Position {start, end})),
            Some((t, _)) => {
                name.push_str(&t.to_string());
                end = tokens.next().unwrap().1.end;
            }
            None => return Err(ParseError(name, Some(end))),
        }
    }
}

fn parse_wikilink_alias<'a>(tokens: &mut PeekTokenStream<'a>) -> Result<mdast::Text, ParseError>{
    let start : Point = match tokens.peek(){
        Some((_, x)) => x.start.clone(),
        None => return Err(ParseError(String::new(), None))
    };
    let mut name = String::new();
    let mut end: Point = start.clone();
    loop {
        match tokens.peek() {
            Some((RRBra, _)) => break Ok(mdast::Text {
                value: name,
                position: Some(Position{start, end}),
            }),
            Some((t, _)) => {
                name.push_str(&t.to_string());
                end = tokens.next().unwrap().1.end;
            }
            None => return Err(ParseError(name, Some(end))),
        }
    }
}

fn parse_wikilink<'a>(tokens: &mut PeekTokenStream<'a>) -> Result<mdast::Link, ParseError> {
    let start = tokens.next().unwrap().1.start;
    let (url, url_pos) = parse_wikilink_first_field(tokens)?;
    match tokens.next() {
        Some((RRBra, x)) => Ok(mdast::Link {
            title: Some("wiki".into()),
            url: url.clone(),
            children: vec![text_content(url, url_pos)],
            position: Some(Position {start, end: x.end.clone()}),
        }),
        Some((Pipe, _)) => {
            let alias = parse_wikilink_alias(tokens)?;
            let end = tokens.next().unwrap().1.end;
            Ok(mdast::Link {
                title: Some("wiki".into()),
                url: url.clone(),
                children: vec![Node::Text(alias)],
                position: Some(Position {start, end}),
            })
        }
        _ => unreachable!()
    }
}

fn parse_text<'a>(tokens: &mut Peekable<TokenStream<'a>>) -> mdast::Text {
    let start = tokens.peek().unwrap().1.start.clone();
    let mut end = start.clone();
    let mut name = String::new();
    loop {
        match tokens.peek() {
            Some((LLBra, _)) | None => break mdast::Text {
                value: name,
                position: Some(Position{start, end}),
            },
            Some((t, _)) => {
                name.push_str(&t.to_string());
                end = tokens.next().unwrap().1.end;
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.peek()? {
            (LLBra, x) => {
                let start = x.start.clone();
                match parse_wikilink(&mut self.tokens) {
                    Ok(l) => Some(Node::Link(l)),
                    Err(ParseError(s, end)) => 
                        Some(text_content(s, 
                                Position {end: end.unwrap_or(start.clone()), start}))
                }
            },
            (NewLine, _) => {
                self.tokens.next();
                self.next()
            }
            _ => {
                Some(Node::Text(
                        parse_text(&mut self.tokens)
                ))
            }
        }
    }
}

fn is_inline_latex(source: &str, m: &mdast::InlineMath) -> bool {
    match &m.position {
        Some(p) => {
            source.get(p.start.offset..p.start.offset+2) == Some("$$") 
            && source.get(p.end.offset-2..p.end.offset) == Some("$$")
        }
        None => false
    }
}

#[cfg(test)]
mod tests {
    use wasm_test::*;
    use super::*;

    #[wasm_test]
    fn test_offset_dollar(){
        let s = "$$x$$";
        assert!(s.get(0..2)==Some("$$"));
        assert!(s.get(s.len()-2..s.len())==Some("$$"));
    }

    #[wasm_test]
    fn test_offset2(){
        let s = "12345";
        let _parser = Parser::new_at(s, Position::new(1,1,0, 1,6,5));
    }

    #[wasm_test]
    fn parse_wiki_no_alias() {
        let s = "here is a wikilink: [[link]]";
        let nodes: Vec<Node> =
            Parser::new_at(s, Position::new(1,1,0, 1,29,28))
            .collect();
        println!("{nodes:?}");
        assert_eq!(nodes[0], text_content("here is a wikilink: ".into(), Position::new(1,1,0, 1,21,20)));
        assert_eq!(nodes[1],
                Node::Link(mdast::Link { 
                    children: vec![ text_content("link".into(), Position::new(1,23,22,1,27,26))], 
                    position: Some(Position::new(1,21,20, 1,29,28)), 
                    url: "link".into(), 
                    title: Some("wiki".into()) 
                })
       );
    }

    #[wasm_test]
    fn parse_wiki_alias() {
        let s = "[[url|an alias with a | in it]]";
        let nodes: Vec<Node> =
            Parser::new_at(s, Position::new(1,1,0, 1,32,31))
            .collect();
        println!("{:?}", nodes[0]);
        assert_eq!(
            nodes[0],
            Node::Link(mdast::Link { 
                children: vec![ 
                    text_content("an alias with a | in it".into(), Position::new(1,7,6, 1,30,29))
                ], 
                position: Some(Position::new(1,1,0, 1,32,31)), 
                url: "url".into(), 
                title: Some("wiki".into()) 
            })
       );
    }
}
