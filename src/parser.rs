use crate::common::{DataType, Token, TokenKind};
use crate::tables::TYPE_KEYWORDS;

// We use two separate macros because we generally want to check a token's kind
// to check for unexpected tokens, but it can also be done for other purposes
macro_rules! check_kind {
    ( $parser: expr, $token: expr, $( $token_type: pat ),+ $(,)? ) => {{
        match $token.kind {
            $( $token_type => true ),+,
            _ => false
        }
    }};
}

// This macro wrapper simply avoids repetition
macro_rules! expect_kind {
    ( $parser: expr, $token: expr, $( $token_type: pat ),+ $(,)? ) => {{
            //TODO: find a way to match the string representation with a clean string output
            if !check_kind!( $parser, $token, $( $token_type ),+ ) {
                let expected: String = 
                [$( stringify!($token_type) ),+]
                .map(|tt| tt.replace("TokenKind::", ""))
                .join(" | ");
            
            $parser.push_diagnostic(
                $token,
                format!("Unexpected token kind: expected {}, found {}", expected, $token.kind)
            );
        }}
    };
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    scope: Scope,
    context: PContext,
    subcontext: u8,
    diagnostics: Vec<Diagnostic>,
}

#[derive(PartialEq)]
enum Scope {
    Global,
    FnBody,
}

#[derive(Clone, Copy, PartialEq)]
enum PContext {
    Default,
    IdentDecl,
    FnDecl,
    FnCall,
    Operation,
    Cast,
}

#[derive(Debug, PartialEq)]
pub struct Diagnostic {
    msg: String,
    line: usize,
    col_start: usize,
    col_end: usize,
}

impl Diagnostic {
    pub fn new(msg: String, line: usize, col_start: usize, col_end: usize) -> Diagnostic {
        return Diagnostic {
            msg,
            line,
            col_start,
            col_end,
        };
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.saturating_add(1);
        return match self.tokens.get(self.index) {
            Some(token) => Some(*token),
            None => None,
        };
    }
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Parser<'a> {
        return Parser {
            tokens: tokens,
            index: 0,
            scope: Scope::Global,
            context: PContext::Default,
            subcontext: 0,
            diagnostics: Vec::with_capacity(256),
        };
    }

    pub fn get_diagnostics(mut self) -> Vec<Diagnostic> {
        while let Some(curr) = self.next() {
            match self.context {
                PContext::IdentDecl => {
                    self.ident_decl_branch(curr);
                }
                PContext::FnDecl => {
                    self.fn_decl_branch(curr);
                }
                PContext::FnCall => {}
                PContext::Operation => {}
                PContext::Cast => {}
                PContext::Default => match (curr.kind, curr.value) {
                    (TokenKind::TypeKeyword, _) | (TokenKind::MiscKeyword, "uniform") | (TokenKind::MiscKeyword, "const") => self.context = PContext::IdentDecl,
                    _ => {}
                },
            }

            //TODO: check if function declarations can be nested in GDSL
            if self.scope == Scope::FnBody && curr.value == "}" {
                self.scope = Scope::Global;
            }
        }

        return self.diagnostics;
    }

    fn peek(&'a self, n: usize) -> Option<&'a Token<'a>> {
        let index = self.index.saturating_add(n);
        return self.tokens.get(index);
    }

    fn peek_back(&'a self, n: usize) -> Option<&'a Token<'a>> {
        let index = self.index.saturating_sub(n);
        return self.tokens.get(index);
    }

    fn push_diagnostic(&mut self, token: Token, msg: String) {
        let diagnostic = Diagnostic {
            msg: msg,
            line: token.line,
            col_start: token.tail,
            col_end: token.tail + token.len(),
        };

        self.diagnostics.push(diagnostic);
    }

    fn expect_value(&mut self, token: Token, value: &'a str) {
        if token.value != value {
            self.push_diagnostic(token, format!("Unexpected token value: expected '{}', found '{}'", value, token.value));
        }
    }

    fn reset_context(&mut self) -> () {
        self.context = PContext::Default;
        self.subcontext = 0;
    }

    fn ident_decl_branch(&mut self, mut token: Token<'a>) -> () {
        match self.subcontext {
            0 => {
                expect_kind!(self, token, TokenKind::Ident(DataType::Unknown));
                //TODO: understand this error
                // token.is_mut = match self.peek_back(1) {
                //     Some(t) if ["uniform", "const"].contains(&t.value) => true,
                //     _ => false,
                // };
                self.subcontext += 1;
            },
            1 => {
                if token.value == "(" {
                    self.context = PContext::FnDecl;
                    self.subcontext = 0;
                    return;
                }

                self.expect_value(token, "=");
                self.subcontext += 1;
            },
            2 => {
                let var_type: DataType = match self.peek_back(3) {
                    Some(token) if token.kind == TokenKind::TypeKeyword => {
                        *TYPE_KEYWORDS.get(token.value).unwrap_or_else(|| {
                            panic!("Type keyword has an unexpected value: {}", token.value)
                        })
                    }
                    _ => DataType::Unknown,
                };

                match var_type {
                    _data_type @ (DataType::I8 | DataType::I16 | DataType::I32) => expect_kind!(
                        self,
                        token,
                        TokenKind::IntLit,
                        TokenKind::Ident(_data_type)
                    ),
                    _data_type @ (DataType::F8 | DataType::F16 | DataType::F32) => expect_kind!(
                        self,
                        token,
                        TokenKind::FloatLit,
                        TokenKind::Ident(_data_type)
                    ),
                    DataType::Unknown => {
                        self.push_diagnostic(token, String::from("Use of ident before assignment"));
                    }
                    _data_type => expect_kind!(self, token, TokenKind::Ident(_data_type)),
                }

                self.subcontext += 1;
            },
            3 => {
                self.expect_value(token, ";");
                self.subcontext += 1;
            }
            _ => {
                self.reset_context();
            }
        }
    }

    fn fn_decl_branch(&mut self, token: Token) -> () {
        match self.subcontext {
            0 => {
                if token.value == ")" {
                    self.subcontext = 3;
                    return;
                }
                
                expect_kind!(self, token, TokenKind::TypeKeyword);
                self.subcontext += 1;
            },
            1 => {
                expect_kind!(self, token, TokenKind::Ident(DataType::Unknown));
                self.subcontext += 1;
            },
            2 => {
                if token.value == "," {
                    self.subcontext = 0;
                    return;
                }

                self.expect_value(token, ")");
                self.subcontext += 1;
            },
            3 => {
                self.expect_value(token, "{");
                self.scope = Scope::FnBody;

                self.reset_context();
            },
            _ => {
                self.reset_context();
            }
        }
    }
}
