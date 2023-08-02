use core::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Token {
    Pipe,
    RBra,
    LBra,
    RRBra,
    LLBra,
    Word,
    NewLine,
}

use Token::*;

/// possible states of the state machine.
/// This implementation is almost a pure DFA
enum State {
    Default,
    AfterPipe,
    AfterOpen1,
    AfterOpen2,
    AfterOpen3,
    AfterClose1,
    AfterClose2,
    AfterClose3,
    AfterSymbol,
    AfterReturn,
}

impl Default for State {
    fn default() -> Self {
        State::Default
    }
}

impl State {
    /// `s.finalize()` returns the extra token that would be returned
    /// if the stream ended in the state `s`
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
            AfterSymbol => Word,
            AfterReturn => NewLine,
            Default => return None,
        })
    }
}


pub struct Lexer<'a> {
    /// the state of the automata
    state: State,

    /// the stream of characacters to parse
    source: core::str::Chars<'a>,

    /// the current position inside the original slice of text.
    /// Each time a char `c` is read, the cursor increase by the utf8 size of `c`
    cursor: usize,

    /// last time a token was returned
    last_token_end: usize,
}

impl<'a> Lexer<'a> {
    pub fn new_at(source: &'a str, index: usize) -> Lexer<'a> {
        Lexer {
            source: source.chars(),
            cursor: index,
            state: State::Default,
            last_token_end: index,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Range<usize>);

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
                (_, AfterSymbol) => (AfterSymbol, None),
                (_, s) => (AfterSymbol, Some(s))

            };

            self.state = new_state;

            let last_cursor = self.cursor.clone();
            self.cursor += c.len_utf8();

            if let Some(t) = state_to_finalize.and_then(|x| x.finalize()) {

                let position = Range {
                    end: last_cursor,
                    start: std::mem::replace(&mut self.last_token_end, last_cursor),
                };

                return Some((t, position));
            }
        }

        if let Some(t) = std::mem::take(&mut self.state).finalize() {
            let position = Range {
                start: std::mem::replace(&mut self.last_token_end, self.cursor.clone()),
                end: self.cursor,
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
        let stream: Vec<Token> = Lexer::new_at(source, 0)
            .map(|(t, _)| t)
            .collect();
        println!("{stream:?}");
        assert_eq!(stream, 
                   vec![
                       LBra, 
                       Word,
                       RBra, 
                       Word,
                       LLBra,  
                       Word,
                       RRBra, 
                       NewLine, 
                       Word,
                   ]
        );
    }

    #[wasm_test]
    fn test_stream_double_bracket(){
        let source = "[[[";
        let stream: Vec<(Token, _)> 
            = Lexer::new_at(source, 0).collect();

        println!("{stream:?}");
        assert_eq!(stream, 
                   vec![
                       (LLBra, 0..2), 
                       (LBra, 2..3), 
                   ]
        );
    }

    #[wasm_test]
    fn lexer_emoji(){
        let source = "[[the url| with a strange content |😈| inside]]";
        let stream : Vec<_> =
            Lexer::new_at(source, 0)
            .map(|(token, range)| (token, &source[range]))
            .collect();

        println!("{stream:?}");
        assert_eq!(stream,
                   vec![
                   (LLBra, "[["), 
                   (Word, "the url"), 
                   (Pipe, "|"), 
                   (Word, " with a strange content "), 
                   (Pipe, "|"), 
                   (Word, "😈"), 
                   (Pipe, "|"), 
                   (Word, " inside"), 
                   (RRBra, "]]")
                   ]);
    }
}

