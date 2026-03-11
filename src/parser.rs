use std::collections::HashMap;
use std::sync::LazyLock;

use crate::common::{DataType, Function, Token, TokenKind};
use crate::tables::{BUILT_IN_FUNCTIONS, GLOBALS, TYPE_KEYWORDS};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    scope: Scope,
    context: PContext,
    subcontext: u8,
    idents: HashMap<&'a str, DataType>,
    functions: HashMap<&'a str, Function>,
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
            idents: HashMap::new(),
            functions: HashMap::new(),
            diagnostics: Vec::with_capacity(256),
        };
    }

    pub fn get_diagnostics(mut self) -> Vec<Diagnostic> {
        while let Some(curr) = self.next() {
            if curr.kind == TokenKind::Comment {
                continue;
            }
            
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
                    (TokenKind::TypeKeyword, _)
                    | (TokenKind::MiscKeyword, "uniform")
                    | (TokenKind::MiscKeyword, "const") => self.context = PContext::IdentDecl,
                    _ => {}
                },
            }

            //TODO: check if function declarations can be nested in GDSL
            if self.scope == Scope::FnBody && curr.value == "}" {
                self.scope = Scope::Global;
            }

            if self.can_recover(curr) {
                self.reset_context();
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

    fn reset_context(&mut self) -> () {
        self.context = PContext::Default;
        self.subcontext = 0;
    }

    fn resolve_fn_type(&self, fn_name: &str) -> TokenKind {
        return match BUILT_IN_FUNCTIONS.get(fn_name) {
            Some(func) => TokenKind::Ident(func.ret_type),
            None => match self.functions.get(fn_name) {
                Some(func) => TokenKind::Ident(func.ret_type),
                None => panic!("Attempting to read unknown function's signature: {}", fn_name)
            }
        }
    }
    
    fn expect_value(&mut self, token: Token, value: &'a str) -> () {
        if token.value != value {
            self.push_diagnostic(
                token,
                format!(
                    "Unexpected token value: expected '{}', found '{}'",
                    value, token.value
                ),
            );
        }
    }

    fn expect_kind(&mut self, token: Token, kind: TokenKind) -> () {
        let token_kind = if token.is_fn() {
            self.resolve_fn_type(token.value)
        } else {
            token.kind
        };
        
        if token_kind != kind {
            self.push_diagnostic(
                token,
                format!(
                    "Unexpected token kind: expected '{}', found '{}'",
                    kind, token.kind
                ),
            );
        }
    }

    fn expect_one_of_values(&mut self, token: Token, values: &[&str]) -> () {
        if !values.contains(&token.value) {
            let expected = values.join(" | ");
            self.push_diagnostic(
                token,
                format!(
                    "Unexpected token value: expected '{}', found '{}'",
                    expected, token.kind
                ),
            );
        }
    }

    fn expect_one_of_kinds(&mut self, token: Token, kinds: &[TokenKind]) -> () {
        let token_kind = if token.is_fn() {
            self.resolve_fn_type(token.value)
        } else {
            token.kind
        };

        if !kinds.contains(&token_kind) {
            let mut expected = String::new();
            let kinds = kinds.iter().enumerate();

            for (i, kind) in kinds {
                if i > 0 {
                    expected.push_str(" | ");
                }
                expected.push_str(&kind.to_string());
            }

            self.push_diagnostic(
                token,
                format!(
                    "Unexpected token kind: expected '{}', found '{}'",
                    expected, token.kind
                ),
            );
        }
    }

    fn ident_decl_branch(&mut self, mut token: Token<'a>) -> () {
        match self.subcontext {
            0 => {
                self.expect_kind(token, TokenKind::Ident(DataType::Unknown));
                //TODO: understand this error
                // token.is_mut = match self.peek_back(1) {
                //     Some(t) if ["uniform", "const"].contains(&t.value) => true,
                //     _ => false,
                // };
                self.subcontext += 1;
            }
            1 => {
                if token.value == "(" {
                    self.context = PContext::FnDecl;
                    self.subcontext = 0;
                    return;
                }

                self.expect_value(token, "=");
                self.subcontext += 1;
            }
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
                    data_type @ (DataType::I8 | DataType::I16 | DataType::I32) => {
                        self.expect_one_of_kinds(
                            token,
                            &[TokenKind::IntLit, TokenKind::Ident(data_type)],
                        )
                    },
                    data_type @ (DataType::F8 | DataType::F16 | DataType::F32) => {
                        self.expect_one_of_kinds(
                            token,
                            &[TokenKind::FloatLit, TokenKind::Ident(data_type)]
                        )
                    },
                    DataType::Unknown => {
                        self.push_diagnostic(token, String::from("Use of ident before assignment"));
                    }
                    data_type => self.expect_kind(token, TokenKind::Ident(data_type)),
                }

                if token.is_fn() {
                    self.context = PContext::FnCall;
                    self.subcontext = 0;
                } else {
                    self.subcontext += 1;
                }
            }
            3 => {
                //TODO handle expressions before semicolon
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

                self.expect_kind(token, TokenKind::TypeKeyword);
                self.subcontext += 1;
            }
            1 => {
                self.expect_kind(token, TokenKind::Ident(DataType::Unknown));
                self.subcontext += 1;
            }
            2 => {
                if token.value == "," {
                    self.subcontext = 0;
                    return;
                }

                self.expect_value(token, ")");
                self.subcontext += 1;
            }
            3 => {
                self.expect_value(token, "{");
                self.scope = Scope::FnBody;

                self.reset_context();
            }
            _ => {
                self.reset_context();
            }
        }
    }

    fn can_recover(&self, token: Token) -> bool {
        return token.value == match self.context {
            PContext::FnCall | PContext::Cast => ")",
            PContext::FnDecl => "{",
            _ => ";"
        };
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
}
