use crate::tables::{OPERATORS, SYMBOLS, MISC_KEYWORDS, TYPE_KEYWORDS, GLOBALS, BUILT_IN_FUNCTIONS};
use crate::common::{DataType, Token, TokenKind};

const BREAKPOINTS: [char; 13] = [
    ',', ';', '+', '-', '*', '/', '=', '(', ')', '[', ']', '{', '}',
];

trait TokenChar {
    fn is_blank(&self) -> bool;
    fn char_offset(&self) -> usize;
}

impl TokenChar for char {
    fn is_blank(&self) -> bool {
        return [' ', '\t', '\r'].contains(self);
    }

    //TODO: read tabsize from settings rather than using a hardcoded value
    fn char_offset(&self) -> usize {
        return if *self == '\t' { 4 } else { 1 };
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    src: &'a str,
    curr_line: usize,
    src_tail: Option<usize>,
    src_head: usize,
    line_tail: Option<usize>,
    line_head: usize,
    tokens: Vec<Token<'a>>,
    ctx: LexerCtx,
}

#[derive(PartialEq, Debug)]
enum LexerCtx {
    Default,
    IntDecl,
    FloatDecl,
    CommentLine,
    CommentBlock,
    CommentBlockEnd
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        return Lexer {
            src: src,
            curr_line: 0,
            src_tail: None,
            src_head: 0,
            line_tail: None,
            line_head: 0,
            tokens: Vec::new(),
            ctx: LexerCtx::Default,
        };
    }

    pub fn tokenize(mut self) -> Vec<Token<'a>> {
        let mut iterator = self.src.chars().peekable();

        while let Some(curr) = iterator.next() {
            if self.is_token_start(&curr) {
                self.src_tail = Some(self.src_head);
                self.line_tail = Some(self.line_head);

                if curr.is_digit(10) {
                    self.ctx = LexerCtx::IntDecl;
                }
            }

            if curr == '\n' {
                self.try_consume();
                self.curr_line += 1;

                self.src_head += curr.len_utf8();
                self.src_tail = None;

                self.line_head = 0;
                self.line_tail = None;

                if self.ctx != LexerCtx::CommentBlock {
                    self.ctx = LexerCtx::Default;
                }
                
                continue;
            }
            
            self.src_head += curr.len_utf8();
            self.line_head += curr.char_offset();

            let next = iterator.peek();
            
            if curr == '.' {
                match self.ctx {
                    LexerCtx::IntDecl => self.ctx = LexerCtx::FloatDecl,
                    LexerCtx::CommentLine | LexerCtx::CommentBlock | LexerCtx::FloatDecl => {}
                    _ => {
                        self.try_consume();
                        continue;
                    }
                }
            } else if let Some(next) = next {
                match (curr, next) {
                    ('/', '/') if self.ctx != LexerCtx::CommentBlock => {
                        self.ctx = LexerCtx::CommentLine;
                    },
                    ('/', '*') if self.ctx != LexerCtx::CommentLine  => {
                        self.ctx = LexerCtx::CommentBlock;
                    },
                    ('*', '/') if self.ctx == LexerCtx::CommentBlock => {
                        self.ctx = LexerCtx::CommentBlockEnd;
                    },
                    _ => {}
                }
            }

            if self.is_breakpoint(&curr, next) && !self.is_in_comment() {
                self.try_consume();
            }
        }

        return self.tokens;
    }

    fn is_token_start(&self, curr: &char) -> bool {
        return
            self.src_tail.is_none()
         && self.line_tail.is_none()
         && ![' ', '\t', '\r', '\n'].contains(curr);
    }

    fn is_breakpoint(&self, curr: &char, next: Option<&char>) -> bool {
        return match next {
            Some(next) => {
                BREAKPOINTS.contains(curr)
             || BREAKPOINTS.contains(next)
             || (*curr == '.' && self.ctx != LexerCtx::FloatDecl)
             || (*curr == '.' && self.ctx == LexerCtx::FloatDecl && !next.is_digit(10))
             || (*next == '.' && self.ctx != LexerCtx::IntDecl)
             || (*curr == '/' && self.ctx == LexerCtx::CommentBlockEnd)
             || next.is_blank()
            }
            None => true,
        };
    }

    fn is_in_comment(&self) -> bool {
        return [
            LexerCtx::CommentLine,
            LexerCtx::CommentBlock,
            LexerCtx::CommentBlockEnd
        ].contains(&self.ctx);
    }

    fn try_consume(&mut self) -> () {
        if let Some(src_tail) = self.src_tail
        && let Some(line_tail) = self.line_tail
        {
            let value = &self.src[src_tail..self.src_head];
            let kind = self.get_token_kind(value);

            let token = Token {
                value,
                kind,
                line: self.curr_line,
                tail: line_tail,
            };
            self.tokens.push(token);

            self.src_tail = None;
            self.line_tail = None;

            if self.ctx != LexerCtx::CommentBlock {
                self.ctx = LexerCtx::Default;
            }
        }
    }

    fn get_token_kind(&self, token_value: &str) -> TokenKind {
        return match self.ctx {
            LexerCtx::IntDecl => TokenKind::IntLit,
            LexerCtx::FloatDecl => TokenKind::FloatLit,
            LexerCtx::CommentBlock | LexerCtx::CommentBlockEnd | LexerCtx::CommentLine => TokenKind::Comment,
            _ => {
                //We match GLOBALS separately to avoid a double lookup (if contains_key => get)
                let value = token_value;
                if let Some(kind) = GLOBALS.get(&value) {
                    return TokenKind::Global(*kind);
                }

                match value {
                    value if MISC_KEYWORDS.contains(&value) => TokenKind::MiscKeyword,
                    value if TYPE_KEYWORDS.contains_key(&value) => TokenKind::TypeKeyword,
                    value if OPERATORS.contains(&value) => TokenKind::Operator,
                    value if SYMBOLS.contains(&value) => TokenKind::Symbol,
                    value if BUILT_IN_FUNCTIONS.contains_key(&value) => TokenKind::Ident(DataType::Fn),
                    _ => TokenKind::Ident(DataType::Unknown),
                }
            }
        };
    }

    fn debug_iteration(&self, curr: &char, next: Option<&char>) {
        println!(
            "src tail: {:?},\nsrc head: {},\nline tail: {:?},\nline head: {},\ncurr: {},\nnext {:?}\n",
            self.src_tail, self.src_head, self.line_tail, self.line_head, curr, next
        );
    }
}
