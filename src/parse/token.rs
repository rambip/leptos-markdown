pub use markdown::unist::{Point, Position};

#[derive(Debug, PartialEq)]
pub enum Token {
    Pipe,
    RBra,
    LBra,
    RRBra,
    LLBra,
    Word(String),
    NewLine,
}

use Token::*;

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Pipe => "|".into(),
            RBra => "]".into(),
            LBra => "[".into(),
            RRBra => "]]".into(),
            LLBra => "[[".into(),
            Word(s) => s.clone(),
            NewLine => "\n".into(),
        }
    }
}

enum State {
    Default,
    AfterPipe,
    AfterOpen1,
    AfterOpen2,
    AfterOpen3,
    AfterClose1,
    AfterClose2,
    AfterClose3,
    AfterSymbol(String),
    AfterReturn,
}

impl Default for State {
    fn default() -> Self {
        State::Default
    }
}

impl State {
    fn finalize(self: State) -> Option<Token> {
        use State::*;

        Some(match self {
            AfterPipe => Pipe,
            AfterOpen1 => LBra,
            AfterOpen2 => LLBra,
            AfterOpen3 => LBra,
            AfterClose1 => RBra,
            AfterClose2 => RRBra,
            AfterClose3 => RBra,
            AfterSymbol(s) => Word(s),
            AfterReturn => NewLine,
            Default => return None,
        })
    }
}


fn advance(point: &mut Point, c: char) {
    if c == '\n' {
        point.line += 1;
        point.column = 1;
    }
    else {
        point.column += c.len_utf8();
    }
    point.offset += c.len_utf8();
}

pub struct TokenStream<'a> {
    source: core::str::Chars<'a>,
    cursor: Point,
    state: State,
    last_token_end: Point,
}

impl<'a> TokenStream<'a> {
    pub fn new_at(source: &'a str, point: Point) -> TokenStream<'a> {
        TokenStream {
            source: source.chars(),
            cursor: point.clone(),
            state: State::Default,
            last_token_end: point.clone()
        }
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = (Token, Position);

    fn next(&mut self) -> Option<Self::Item> {
        use State::*;

        for c in self.source.by_ref() {

            let state = std::mem::take(&mut self.state);

            let (new_state, state_to_finalize) = match (c, state) {
                ('\r', s)           => (s, None),
                ('\n', s)           => (AfterReturn, Some(s)),
                ('[', AfterOpen1)   => (AfterOpen2, None),
                ('[', s@AfterOpen2) => (AfterOpen3, Some(s)),
                ('[', s@AfterOpen3) => (AfterOpen3, Some(s)),
                ('[', s)            => (AfterOpen1, Some(s)),
                (']', AfterClose1)  => (AfterClose2, None),
                (']', s@AfterClose2)=> (AfterClose3, Some(s)),
                (']', s@AfterClose3)=> (AfterClose3, Some(s)),
                (']', s)            => (AfterClose1, Some(s)),
                ('|', s)            => (AfterPipe, Some(s)),
                (c, AfterSymbol(mut s)) => {
                    s.push(c);
                    (AfterSymbol(s), None)
                }
                (c, s) => (AfterSymbol(c.into()), Some(s))

            };

            self.state = new_state;

            let last_cursor = self.cursor.clone();
            advance(&mut self.cursor, c);

            if let Some(t) = state_to_finalize.and_then(|x| x.finalize()) {

                let position = Position {
                    end: last_cursor.clone(),
                    start: std::mem::replace(&mut self.last_token_end, last_cursor),
                };

                return Some((t, position));
            }
        }

        if let Some(t) = std::mem::take(&mut self.state).finalize() {
            let position = Position {
                start: std::mem::replace(&mut self.last_token_end, self.cursor.clone()),
                end: self.cursor.clone(),
            };
            return Some((t, position));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use wasm_test::*;
    use super::*;

    #[wasm_test]
    fn test_stream(){
        let source = "[abc] [[ d e]]\nb";
        let stream: Vec<Token> = TokenStream::new_at(source, Point::new(1,1,0))
            .map(|(t, _)| t)
            .collect();
        println!("{stream:?}");
        assert_eq!(stream, 
                   vec![
                       LBra, 
                       Word("abc".into()),
                       RBra, 
                       Word(" ".into()), 
                       LLBra,  
                       Word(" d e".into()), 
                       RRBra, 
                       NewLine, 
                       Word("b".into()), 
                   ]
        );
    }

    #[wasm_test]
    fn test_stream_double_bracket(){
        let source = "[[[";
        let stream: Vec<(Token, Position)> 
            = TokenStream::new_at(source, Point::new(1,1,0)).collect();

        println!("{stream:?}");
        assert_eq!(stream, 
                   vec![
                       (LLBra, Position::new(1,1,0, 1,3,2)), 
                       (LBra, Position::new(1,3,2, 1,4,3)), 
                   ]
        );
    }
}

