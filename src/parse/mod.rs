use pulldown_cmark::{Options, Parser, Event, Tag, TagEnd, LinkType};
use core::ops::Range;

mod token;
use token::{Lexer, Token};

use Token::*;

use core::iter::Peekable;


pub fn default_options() -> Options {
    let mut r = Options::all();
    r.set(Options::ENABLE_FOOTNOTES, false);
    r
}


/// regroups adjacents text events.
/// if the events are [Text("a"), Text("b"), Link], the result will be
/// [Text("ab"), Link]
fn group_text<'a>(source: &'a str, events: Vec<(Event<'a>, Range<usize>)>) 
-> Vec<(Event<'a>, Range<usize>)> {
    let mut result = Vec::with_capacity(events.len());

    let mut last_text_range: Option<Range<usize>> = None;

    for (t, r) in events {
        last_text_range = match (t, std::mem::take(&mut last_text_range)) {
            (Event::Text(_), Some(last_range)) => {
                Some(last_range.start..r.end)
            },
            (Event::Text(_), None) => {
                Some(r)
            },
            (t, Some(last_range)) => {
                result.push((Event::Text(source[last_range.clone()].into()), last_range.clone()));
                result.push((t, r));
                None
            },
            (t, None) => {
                result.push((t, r));
                None
            }
        }
    }

    result
}


/// `parse(s, options, wikilinks)` returns a vector of [`Events`][pulldown_cmark::Event]
/// it adds inline wikilinks if the `wikilinks` flag is set to `true`
pub fn parse<'a>(source: &'a str, parse_options: &Options, wikilinks: bool) 
-> Vec<(Event<'a>, Range<usize>)>{
    let events = Parser::new_ext(source, parse_options.to_owned())
        .into_offset_iter()
        .collect::<Vec<_>>();

    let events_grouped = group_text(source, events);

    if !wikilinks {
        return events_grouped
    }

    let mut result = Vec::new();

    for item in events_grouped {
        match item {
            (Event::Text(_), r) => result.extend(MyParser::new_at(source, r)),
            _ => result.push(item)
        }
    }

    result
}

type PeekTokenStream<'a> = Peekable<Lexer<'a>>;

struct MyParser<'a> {
    source: &'a str,
    tokens: PeekTokenStream<'a>,
    buffer: std::array::IntoIter<(Event<'a>, Range<usize>), 3>
}


enum ParseError {
    Empty,
    ReParse(Range<usize>)
}

impl ParseError {
    /// `error.extend_before(start..end)` returns a new error
    /// that spans from start to the end of the error 
    /// (either end, either the original error end)
    fn extend_before(self, r: Range<usize>) -> ParseError {
        match self {
            Self::Empty => Self::ReParse(r),
            Self::ReParse(r2) => Self::ReParse(r.start..r2.end)
        }
    }
}


impl<'a> MyParser<'a> {
    fn new_at(source: &'a str, position: Range<usize>) -> Self {
        let text = &source[position.clone()];
        Self {
            source,
            tokens: Lexer::new_at(&text, position.start).peekable(),
            buffer: std::array::IntoIter::empty(),
        }
    }

    /// in `[[url|link]]`, returns `url` and don't consume the `|`
    fn parse_wikilink_first_field(&mut self) -> Result<Range<usize>, ParseError> {
        let start : usize = match self.tokens.peek(){
            Some((_, x)) => x.start,
            None => return Err(ParseError::Empty)
        };
        let mut end: usize = start.clone();
        loop {
            match self.tokens.peek() {
                Some((Pipe, _))| Some((RRBra, _)) => break Ok(start..end),
                Some((_, _)) => {
                    end = self.tokens.next().unwrap().1.end;
                }
                None => return Err(ParseError::ReParse(start..end)),
            }
        }
    }

    /// in `link]]`, returns `link` and don't consume the `]]`
    fn parse_wikilink_alias(&mut self) -> Result<Range<usize>, ParseError>{
        let start : usize = match self.tokens.peek(){
            Some((_, x)) => x.start.clone(),
            None => return Err(ParseError::Empty)
        };
        let mut end: usize = start.clone();
        loop {
            match self.tokens.peek() {
                Some((RRBra, _)) => return Ok(start..end),
                Some((_, _)) => {
                    end = self.tokens.next().unwrap().1.end;
                }
                None => return Err(ParseError::ReParse(start..end)),
            }
        }
    }

    /// parse an entire wikilink, ie one of
    /// - `[[a shortcut url]]`
    /// - `[[a url|with some displayed content]]`
    fn parse_wikilink(&mut self) -> Result<[(Event<'a>, Range<usize>); 3], ParseError> {
        let tag_pos = self.tokens.next().unwrap().1;
        let url_pos = self.parse_wikilink_first_field()
            .map_err(|x| x.extend_before(tag_pos.clone()))?;

        let opening_tag = Event::Start(Tag::Link(
                LinkType::Inline,
                self.source[url_pos.clone()].into(),
                "wiki".into(),
        ));

        let closing_tag = Event::End(TagEnd::Link);

        match self.tokens.next() {
            Some((RRBra, x)) => {
                Ok([
                    (opening_tag, tag_pos.start..x.end),
                    (Event::Text(self.source[url_pos.clone()].into()), url_pos),
                    (closing_tag, tag_pos.start..x.end),
                ])
            },
            Some((Pipe, _)) => {
                let alias_pos = self.parse_wikilink_alias()
                    .map_err(|x| x.extend_before(tag_pos.clone()))?;

                let end = self.tokens.next().unwrap().1.end;
                Ok([
                   (opening_tag, tag_pos.start..end),
                    (Event::Text(self.source[alias_pos.clone()].into()), alias_pos),
                   (closing_tag, tag_pos.start..end),
                ])
            }
            _ => unreachable!()
        }
    }

    // parse a text until the first `[[` (start of wikilink) is encountered.
    // don't consume the `[[`
    fn parse_text(&mut self) -> Range<usize> {
        let start = self.tokens.peek().unwrap().1.start.clone();
        let mut end = start.clone();
        loop {
            match self.tokens.peek() {
                Some((LLBra, _)) | None => return start..end,
                Some((_, _)) => {
                    end = self.tokens.next().unwrap().1.end;
                }
            }
        }
    }
}


impl<'a> Iterator for MyParser<'a> {
    type Item = (Event<'a>, Range<usize>);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((x, r)) = self.buffer.next() {
            return Some((x.clone(), r.clone()))
        };
        match self.tokens.peek()? {
            (LLBra, x) => {
                let _start = x.start.clone();
                match self.parse_wikilink() {
                    Ok(b) => {
                        self.buffer = b.into_iter();
                        self.next()
                    },
                    Err(e) => {
                        let r = match e {
                            ParseError::ReParse(r) => r,
                            _ => unreachable!(),
                        };
                        Some((Event::Text(self.source[r.clone()].into()), r))
                    }
                }
            },
            (NewLine, _) => self.next(),
            _ => {
                let r = self.parse_text();
                Some((Event::Text(self.source[r.clone()].into()), r))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use wasm_test::*;
    use super::*;
    use pulldown_cmark::TagEnd;

    use Event::*;
    use LinkType::*;

    #[wasm_test]
    fn test_offset(){
        let s = "12345";
        let _parser = MyParser::new_at(s, 0..5);
    }

    #[wasm_test]
    fn parse_wikilink_no_alias() {
        let s = "here is a wikilink: [[link]]";
        let tokens : Vec<_> = Lexer::new_at(s, 0).collect();
        println!("{tokens:?}");
        let events: Vec<_> =
            MyParser::new_at(s, 0..28)
            .collect();
        println!("{events:?}");
        assert_eq!(events, vec![
           (Text("here is a wikilink: ".into()), 0..20),
           (Start(Tag::Link(Inline, "link".into(), "wiki".into())), 20..28),
           (Text("link".into()), 22..26),
           (End(TagEnd::Link), 20..28)
        ]);
    }

    #[wasm_test]
    fn parse_wikilink_alias(){
        let s = "[[the url| with a strange content |😈| inside]]";

        let events: Vec<_> = 
            MyParser::new_at(s, 0..s.len())
            .map(|(t, _)| t)
            .collect();

        println!("{events:?}");
        assert_eq!(
            events,
            vec![
                Start(Tag::Link(Inline, "the url".into(), "wiki".into())), 
                Text(" with a strange content |😈| inside".into()), 
                End(TagEnd::Link)]
        );
    }

    #[wasm_test]
    fn parse(){
        let s = "[[the url| with a strange content |😈| inside]]";

        let events: Vec<_> = 
            parse(s, &default_options(), true)
            .into_iter()
            .map(|(t, _)| t)
            .collect();

        println!("{events:?}");
        assert_eq!(
            events,
            vec![
                Start(Tag::Paragraph),
                Start(Tag::Link(Inline, "the url".into(), "wiki".into())), 
                Text(" with a strange content |😈| inside".into()), 
                End(TagEnd::Link),
                End(TagEnd::Paragraph),
            ]
        );
    }

    #[wasm_test]
    fn group_by() {
        let slice = &[1, 2, 3, 1, 1, 1, 2, 1];
        let groups: Vec<_> = slice.group_by(|a,b| *a==1 && *b == 1)
            .collect();
        
        assert_eq!(groups, vec![vec![1], vec![2], vec![3], vec![1, 1, 1], vec![2], vec![1]])
    }
}
