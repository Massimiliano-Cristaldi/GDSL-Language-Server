use core::panic;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::task::Context;

use crate::common::{DataType, Function, Token, TokenKind};
use crate::tables::{BUILT_IN_FUNCTIONS, GLOBALS, TYPE_KEYWORDS};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    scope: Scope,
    curr_context: PContext,
    prev_contexts: Vec<PContext>,
    idents: HashMap<&'a str, DataType>,
    functions: HashMap<&'a str, Function>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(PartialEq)]
enum Scope {
    Global,
    FnBody,
}

#[derive(Clone, PartialEq)]
enum PContextKind {
    Default,
    IdentDecl,
    FnDecl {
        args: Vec<DataType>,
    },
    FnCall {
        args: Vec<DataType>,
    },
    Operation {
        lhs: Option<DataType>,
        operator: &'static str,
        rhs: DataType,
    },
    Cast,
}

#[derive(Clone, PartialEq)]
struct PContext {
    kind: PContextKind,
    subcontext: usize
}

impl PContext {
    fn new_default() -> PContext {
        return PContext { kind: PContextKind::Default, subcontext: 0 }
    }
    
    fn new_ident_decl() -> PContext {
        return PContext { kind: PContextKind::IdentDecl, subcontext: 0 };
    }

    fn new_fn_decl() -> PContext {
        return PContext { kind: PContextKind::FnDecl { args: vec![] }, subcontext: 0 };
    }

    fn new_fn_call() -> PContext {
        return PContext { kind: PContextKind::FnCall { args: vec![] }, subcontext: 0 };
    }

    fn new_operation() -> PContext {
        return PContext { kind: PContextKind::Operation { lhs: None, operator: "", rhs: DataType::Unknown }, subcontext: 0 };
    }

    fn new_cast() -> PContext {
        return PContext { kind: PContextKind::Cast, subcontext: 0 };
    }

    fn get_arg(&self, index: usize) -> DataType {
        return match &self.kind {
            PContextKind::FnCall { args } | PContextKind::FnDecl { args }=> {
                *args.get(index).expect(&format!("Tried to access non existent function argument {index}"))
            },
            _ => panic!("Cannot get function signature data from non-function contexts")
        }
    }

    fn push_arg(&mut self, arg: DataType) -> () {
        return match &mut self.kind {
            PContextKind::FnCall { args } => {
                args.push(arg)
            },
            PContextKind::FnDecl { args } => {
                args.push(arg)
            },
            _ => panic!("Cannot modify function signature data in non-function contexts")
        }
    }
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
            curr_context: PContext::new_default(),
            prev_contexts: Vec::with_capacity(256),
            idents: HashMap::new(),
            functions: HashMap::new(),
            diagnostics: Vec::with_capacity(256),
        };
    }

    pub fn get_diagnostics(mut self) -> Vec<Diagnostic> {
        while let Some(token) = self.next() {
            //TODO: check if comment is at end of line
            if token.kind == TokenKind::Comment {
                continue;
            }
            
            match self.curr_context.kind {
                PContextKind::IdentDecl { .. } => {
                    self.ident_decl_branch(token);
                }
                PContextKind::FnDecl { .. } => {
                    self.fn_decl_branch(token);
                }
                PContextKind::FnCall { .. } => {
                    self.fn_call_branch(token);
                }
                PContextKind::Operation { .. } => {}
                PContextKind::Cast { .. } => {}
                PContextKind::Default => match (token.kind, token.value) {
                    (TokenKind::TypeKeyword, _) |
                    (TokenKind::MiscKeyword, "uniform") |
                    (TokenKind::MiscKeyword, "const") => self.push_context(PContext::new_ident_decl()),
                    _ => {}
                },
            }

            //TODO: check if function declarations can be nested in GDSL
            if self.scope == Scope::FnBody && token.value == "}" {
                self.scope = Scope::Global;
            }

            //TODO: keep track of poisoned state
            if self.can_recover(token) {
                self.pop_context();
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

    fn push_context(&mut self, new_context: PContext) -> () {
        self.prev_contexts.push(self.curr_context.clone());
        self.curr_context = new_context;
    } 
    
    fn pop_context(&mut self) -> () {
        self.curr_context = 
            self.prev_contexts
            .pop()
            .unwrap_or(PContext::new_default());
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

    fn expect_kind(&mut self, token: Token, kind: TokenKind) -> bool {
        let token_kind = if token.is_fn() {
            self.resolve_fn_type(token.value)
        } else {
            token.kind
        };
        
        if token_kind == kind {
            return true;
        } else {
            self.push_diagnostic(
                token,
                format!(
                    "Unexpected token kind: expected '{}', found '{}'",
                    kind, token.kind
                ),
            );

            return false;
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

    fn ident_decl_branch(&mut self, token: Token<'a>) -> () {
        match self.curr_context.subcontext {
            0 => {
                self.expect_kind(token, TokenKind::Ident(DataType::Unknown));
                //TODO: understand this error
                // token.is_mut = match self.peek_back(1) {
                //     Some(t) if ["uniform", "const"].contains(&t.value) => true,
                //     _ => false,
                // };
                self.curr_context.subcontext += 1;
            }
            1 => {
                if token.value == "(" {
                    self.push_context(PContext::new_fn_decl());
                    self.curr_context.subcontext = 0;
                    return;
                }

                self.expect_value(token, "=");
                self.curr_context.subcontext += 1;
            }
            2 => {
                //TODO: keep track of var_type as we iterate rather than peeking back
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
                    data_type => {
                        self.expect_kind(token, TokenKind::Ident(data_type));
                    }
                }

                if token.is_fn() {
                    self.push_context(PContext::new_fn_call());
                } else {
                    self.curr_context.subcontext += 1;
                }
            }
            3 => {
                //TODO handle expressions before semicolon
                self.expect_value(token, ";");
                self.curr_context.subcontext += 1;
            }
            _ => {
                self.pop_context();
            }
        }
    }

    fn fn_decl_branch(&mut self, token: Token) -> () {
        match self.curr_context.subcontext {
            0 => {
                if token.value == ")" {
                    self.curr_context.subcontext = 3;
                    return;
                }

                if self.expect_kind(token, TokenKind::TypeKeyword) {
                    self.curr_context.push_arg(*TYPE_KEYWORDS.get(token.value).unwrap());
                }

                self.curr_context.subcontext += 1;
            }
            1 => {
                self.expect_kind(token, TokenKind::Ident(DataType::Unknown));
                self.curr_context.subcontext += 1;
            }
            2 => {
                if token.value == "," {
                    self.curr_context.subcontext = 0;
                    return;
                }

                self.expect_value(token, ")");
                self.curr_context.subcontext += 1;
            }
            3 => {
                self.expect_value(token, "{");
                self.scope = Scope::FnBody;

                self.pop_context();
            }
            _ => {
                self.pop_context();
            }
        }
    }

    fn fn_call_branch(&mut self, token: Token) -> () {
        match self.curr_context.subcontext {
            0 => {

            },
            _ => {
                self.pop_context();
            }
        }
    }

    fn can_recover(&self, token: Token) -> bool {
        return token.value == match self.curr_context.kind {
            PContextKind::FnCall { .. } | PContextKind::Cast { .. } => ")",
            PContextKind::FnDecl { .. } => "{",
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
