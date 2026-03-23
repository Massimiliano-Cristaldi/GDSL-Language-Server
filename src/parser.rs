use core::panic;
use std::collections::HashMap;
use std::mem;
use std::sync::LazyLock;
use std::task::Context;

use crate::common::{DataType, Function, Token, TokenKind};
use crate::tables::{BUILT_IN_FUNCTIONS, GLOBALS, TYPE_KEYWORDS};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    curr_index: usize,
    scope: Scope,
    prev_ctxs: Vec<ParserCtx>,
    idents: HashMap<&'a str, DataType>,
    functions: HashMap<&'a str, Function>,
    diagnostics: Vec<Diagnostic>
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr_index = self.curr_index.saturating_add(1);
        return match self.tokens.get(self.curr_index) {
            Some(token) => Some(*token),
            None => None,
        };
    }
}

impl<'a> Parser<'a> {   
    pub fn new(tokens: Vec<Token<'a>>) -> Parser<'a> {
        return Parser {
            tokens: tokens,
            curr_index: 0,
            scope: Scope::Global,
            prev_ctxs: vec![],
            idents: HashMap::new(),
            functions: HashMap::new(),
            diagnostics: Vec::with_capacity(256)
        }
    }
    
    pub fn get_diagnostics(mut self) -> Vec<Diagnostic> {
        let mut curr_ctx = ParserCtx::new_default();

        while let Some(token) = self.next() {
            //TODO: check if comment is at end of line
            if token.kind == TokenKind::Comment {
                continue;
            }
            
            match curr_ctx {
                ParserCtx::IdentDecl(_) => {
                    self.ident_decl_branch(token, &mut curr_ctx)
                }
                ParserCtx::FnDecl(_) => {
                    self.fn_decl_branch(token, &mut curr_ctx)
                }
                ParserCtx::FnCall(_) => {
                    self.fn_call_branch(token, &mut curr_ctx)
                }
                ParserCtx::Expr(_) => {
                    self.expr_branch(token, &mut curr_ctx)
                }
                ParserCtx::Cast(_) => {
                    self.cast_branch(token, &mut curr_ctx)
                }
                ParserCtx::Default => {
                    self.default_branch(token, &mut curr_ctx)
                }
            };

            println!("{curr_ctx:?}");

            //TODO: check if function declarations can be nested in GDSL
            //TODO: do this in the FnDecl branch
            if self.scope == Scope::FnBody && token.value == "}" {
                self.scope = Scope::Global;
            }

            //TODO: keep track of poisoned state
            if self.can_recover(&token, &curr_ctx) {
                self.exit_ctx(&mut curr_ctx);
            }
        }

        return self.diagnostics;
    }

    fn ident_decl_branch(&mut self, token: Token, ctx: &mut ParserCtx) -> () {
        let ident_decl_ctx = ctx.as_ident_decl_ctx();
        
        match ident_decl_ctx.subcontext {
            0 => {
                self.expect_kind(&token, TokenKind::TypeKeyword);
                ident_decl_ctx.ident_type = *TYPE_KEYWORDS.get(token.value).unwrap();
                ident_decl_ctx.subcontext += 1;  
            }
            1 => {
                self.expect_kind(&token, TokenKind::Ident(DataType::Unknown));
                ident_decl_ctx.subcontext += 1;
            }
            2 => {
                if token.value == "(" {
                    *ctx = ParserCtx::new_fn_decl();
                    return;
                }

                if self.expect_value(&token, "=") {
                    let expr_result_type = ident_decl_ctx.ident_type.clone();
                    self.enter_ctx(ctx, ParserCtx::new_expr());
                    ctx.as_expr_ctx().result_type = expr_result_type;
                } else {
                    self.exit_ctx(ctx);
                }
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn fn_decl_branch(&mut self, token: Token, ctx: &mut ParserCtx) -> () {
        let fn_decl_ctx = ctx.as_fn_decl_ctx();
        
        match fn_decl_ctx.subcontext {
            0 => {
                if token.value == ")" {
                    fn_decl_ctx.subcontext = 3;
                    return;
                }
                
                if self.expect_kind(&token, TokenKind::TypeKeyword) {
                    //We can safely unwrap because TypeKeyword tokens can only be created
                    //when the tokenizer finds the token value in the TYPE_KEYWORDS map
                    fn_decl_ctx.args.push(*TYPE_KEYWORDS.get(token.value).unwrap());
                }

                fn_decl_ctx.subcontext += 1;
            }
            1 => {
                self.expect_kind(&token, TokenKind::Ident(DataType::Unknown));
                fn_decl_ctx.subcontext += 1;
            }
            2 => {
                if token.value == "," {
                    fn_decl_ctx.subcontext = 0;
                    return;
                }

                self.expect_value(&token, ")");
                fn_decl_ctx.subcontext += 1;
            }
            3 => {
                if self.expect_value(&token, "{") {
                    self.scope = Scope::FnBody;
                }

                self.exit_ctx(ctx);
            }
            _ => {
                self.exit_ctx(ctx);
                return;
            }
        }
    }

    fn fn_call_branch(&mut self, token: Token, ctx: &mut ParserCtx) -> () {
        let fn_call_ctx = ctx.as_fn_call_ctx();
        
        match fn_call_ctx.subcontext {
            0 => {
                return;
            },
            _ => {
                self.exit_ctx(ctx);
            }
        }
    }

    fn expr_branch(&mut self, token: Token, ctx: &mut ParserCtx) -> () {
        let expr_ctx = ctx.as_expr_ctx();
        
        match expr_ctx.subcontext {
            0 => {
                match token.kind {
                    TokenKind::Ident(data_type) => {
                        if let Some(next_token) = self.peek(1)
                        && next_token.value == "("
                        {
                            expr_ctx.subcontext = 1;

                            self.skip(1);
                            self.enter_ctx(ctx, ParserCtx::new_fn_call());
                            return;
                        }

                        if (data_type == DataType::Unknown) {
                            expr_ctx.subcontext = 1;

                            self.push_diagnostic(
                                &token,
                                String::from("Use of variable before declaration")
                            );
                            return;
                        }

                        //TODO: identifiers should be already classified as arrays by the tokenizer 
                        if let Some(vec_type) = token.try_vec_type()
                        && vec_type == expr_ctx.result_type
                        {
                            //TODO: vec component access subcontext
                            expr_ctx.subcontext = 100;
                        } else if data_type != expr_ctx.result_type {
                            expr_ctx.subcontext = 1;

                            self.push_diagnostic(
                                &token,
                                format!("Unexpected variable type: expected {}, found {}", expr_ctx.result_type, data_type)
                            );
                        }
                    }
                    TokenKind::Operator => {
                        if self.expect_one_of_values(&token, &["+", "-"]) {
                            //TODO:: unary operation subcontext
                        }
                    }
                    TokenKind::Symbol => {
                        match token.value {
                            "(" => {
                                expr_ctx.brackets.push('(');
                            }
                            "[" => {
                                expr_ctx.brackets.push('[');
                            }
                            "{" => {
                                //TODO: array initialization subcontext
                            }
                            ")" => {
                                match expr_ctx.brackets.last() {
                                    Some(&'(') => {},
                                    _ => self.push_generic_diagnostic(&token)
                                }
                            }
                            "]" => {
                                match expr_ctx.brackets.last() {
                                    Some(&'[') => {},
                                    _ => self.push_generic_diagnostic(&token)
                                }
                            }
                            _ => {
                                self.push_generic_diagnostic(&token);
                            }
                        }

                        expr_ctx.subcontext = 1;
                    }
                    TokenKind::TypeKeyword => {
                        if let Some(next_token) = self.peek(1) 
                        && next_token.value == "["
                        {

                            //TODO: array initialization subcontext
                        }
                    }
                    _ => {
                        match expr_ctx.result_type {
                            DataType::I8 | DataType::I16 | DataType::I32 => {
                                self.expect_kind(&token, TokenKind::IntLit);
                            }
                            DataType::F8 | DataType::F16 | DataType::F32 => {
                                self.expect_kind(&token, TokenKind::FloatLit);
                            }
                            _ => {
                                self.push_generic_diagnostic(&token);
                            }
                        }

                        expr_ctx.subcontext = 1;
                    }
                }
            },
            1 => {
                match token.kind {
                    TokenKind::Operator => {
                        expr_ctx.subcontext = 0;
                    }
                    _ => {
                        if token.value == ";" {
                            self.exit_ctx(ctx);
                        } else {
                            self.push_generic_diagnostic(&token);
                        }
                    }
                }
            }
            //Expect vector component access
            100 => {
                
            }
            _ => {
                self.exit_ctx(ctx);
                return;
            }
        }
    }

    fn cast_branch(&mut self, token: Token, ctx: &mut ParserCtx) -> () {
        let ctx_data = ctx.as_cast_ctx();
        
        match ctx_data.subcontext {
            0 => {
                return;
            },
            _ => {
                self.exit_ctx(ctx);
                return;
            }
        }
    }

    fn default_branch(&mut self, token: Token, ctx: &mut ParserCtx) -> () {
         match (token.kind, token.value) {
            (TokenKind::TypeKeyword, _) => {
                self.enter_ctx(ctx, ParserCtx::new_ident_decl());
                let ctx_data = ctx.as_ident_decl_ctx();
                ctx_data.ident_type = *TYPE_KEYWORDS.get(token.value).unwrap();
                ctx_data.subcontext = 1;
            },
            (TokenKind::MiscKeyword, "uniform") |
            (TokenKind::MiscKeyword, "const") => {
                self.enter_ctx(ctx, ParserCtx::new_ident_decl());
                let ctx_data = ctx.as_ident_decl_ctx();
                ctx_data.is_mut = false;
                ctx_data.subcontext = 0;
            }
            _ => {}
        }
    }

    fn enter_ctx(&mut self, curr_ctx: &mut ParserCtx, new_context: ParserCtx) -> () {
        let prev_ctx = mem::replace(curr_ctx, new_context);
        self.prev_ctxs.push(prev_ctx);
    } 
    
    fn exit_ctx(&mut self, curr_ctx: &mut ParserCtx) -> () {
        *curr_ctx = 
            self.prev_ctxs
            .pop()
            .unwrap_or(ParserCtx::new_default());
    }
    
    fn expect_value(&mut self, token: &Token, value: &str) -> bool {
        if token.value == value {
            return true;
        } else {
            self.push_diagnostic(
                token,
                format!("Unexpected token value: expected '{}', found '{}'", value, token.value)
            );

            return false;
        }
    }

            //
    fn expect_kind(&mut self, token: &Token, kind: TokenKind) -> bool {
        if self.resolve_token_kind(token) == kind {
            return true;
        } else {
            //TODO: expected should be stringified as generic (e.g. "float" instead of "float literal")
            self.push_diagnostic(
                token,
                format!("Unexpected token kind: expected '{}', found '{}'", kind, token.kind)
            );

            return false;
        }
    }

    fn expect_one_of_values(&mut self, token: &Token, values: &[&str]) -> bool {
        if values.contains(&token.value) {
            return true;
        } else {
            let expected = values.join(" | ");

            self.push_diagnostic(
                &token,
                format!("Unexpected token value: expected '{}', found '{}'", expected, token.kind)
            );

            return false;
        }
    }

    fn expect_one_of_kinds(&mut self, token: &Token, kinds: &[TokenKind]) -> bool {
        if kinds.contains(&self.resolve_token_kind(token)) {
            return true;
        } else {
            let mut expected = String::new();
            let kinds = kinds.iter().enumerate();

            for (i, kind) in kinds {
                if i > 0 {
                    expected.push_str(" | ");
                }
                expected.push_str(&kind.to_string());
            }

            self.push_diagnostic(
                &token,
                format!("Unexpected token kind: expected '{}', found '{}'", expected, token.kind)
            );

            return false;
        }
    }

    //TODO: functions can be declared after they're invoked
    fn resolve_token_kind(&self, token: &Token) -> TokenKind {
        if token.is_fn() {
            return match BUILT_IN_FUNCTIONS.get(token.value) {
                Some(func) => TokenKind::Ident(func.ret_type),
                None => match self.functions.get(token.value) {
                    Some(func) => TokenKind::Ident(func.ret_type),
                    None => panic!("Attempting to read unknown function's signature: {}", token.value)
                }
            }
        } else {
            return token.kind;
        }
    }

    fn can_recover(&self, token: &Token, curr_ctx: &ParserCtx) -> bool {
        return token.value == match curr_ctx {
            ParserCtx::FnCall(_) | ParserCtx::Cast(_) => ")",
            ParserCtx::FnDecl(_) => "{",
            _ => ";"
        };
    }

    fn peek(&'a self, n: usize) -> Option<&'a Token<'a>> {
        let index = self.curr_index.saturating_add(n);
        return self.tokens.get(index);
    }

    fn peek_back(&'a self, n: usize) -> Option<&'a Token<'a>> {
        let index = self.curr_index.saturating_sub(n);
        return self.tokens.get(index);
    }

    fn push_diagnostic(&mut self, token: &Token, msg: String) {
        let diagnostic = Diagnostic::new(
            msg,
            token.line,
            token.tail,
            token.tail + token.len()
        );
        
        self.diagnostics.push(diagnostic);
    }

    fn push_generic_diagnostic(&mut self, token: &Token) -> () {
        self.push_diagnostic(
            &token, 
            format!("Unexpected token: '{}'", token.value)
        );
    }
}

struct ParserEnv<'a> {
    scope: Scope,
    curr_ctx: ParserCtx,
    prev_ctxs: Vec<ParserCtx>,
    idents: HashMap<&'a str, DataType>,
    functions: HashMap<&'a str, Function>,
}

#[derive(Debug, Clone, PartialEq)]
struct IdentDeclCtx {
    subcontext: usize,
    ident_type: DataType,
    is_mut: bool
}

#[derive(Debug, Clone, PartialEq)]
struct FnDeclCtx {
    subcontext: usize,
    args: Vec<DataType>,
}

#[derive(Debug, Clone, PartialEq)]
struct FnCallCtx {
    subcontext: usize,
    args: Vec<DataType>,
}

#[derive(Debug, Clone, PartialEq)]
struct ExprCtx {
    subcontext: usize,
    lhs: Option<DataType>,
    operator: &'static str,
    rhs: DataType,
    result_type: DataType,
    brackets: Vec<char>
}

#[derive(Debug, Clone, PartialEq)]
struct CastCtx {
    subcontext: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum ParserCtx {
    Default,
    IdentDecl(IdentDeclCtx),
    FnDecl(FnDeclCtx),
    FnCall(FnCallCtx),
    Expr(ExprCtx),
    Cast(CastCtx),
}

impl ParserCtx {
    fn new_default() -> ParserCtx {
        return ParserCtx::Default;
    }
    
    fn new_ident_decl() -> ParserCtx {
        return ParserCtx::IdentDecl(IdentDeclCtx {
            subcontext: 0 ,
            ident_type: DataType::Unknown,
            is_mut: false
        });
    }

    fn new_fn_decl() -> ParserCtx {
        return ParserCtx::FnDecl(FnDeclCtx {
            subcontext: 0,
            args: vec![],
        });
    }

    fn new_fn_call() -> ParserCtx {
        return ParserCtx::FnCall(FnCallCtx {
            subcontext: 0,
            args: vec![],
        });
    }

    fn new_expr() -> ParserCtx {
        return ParserCtx::Expr(ExprCtx {
            subcontext: 0,
            lhs: None,
            operator: "",
            rhs: DataType::Unknown,
            result_type: DataType::Unknown,
            brackets: Vec::with_capacity(10)
        });
    }

    fn new_cast() -> ParserCtx {
        return ParserCtx::Cast(CastCtx { subcontext: 0 });
    }

    fn as_ident_decl_ctx(&mut self) -> &mut IdentDeclCtx {
        if let ParserCtx::IdentDecl(context) = self {
            return context;
        } else {
            panic!("Expected IdentDecl");
        }
    }

    fn as_fn_decl_ctx(&mut self) -> &mut FnDeclCtx {
        if let ParserCtx::FnDecl(context) = self {
            return context;
        } else {
            panic!("Expected FnDecl");
        }
    }

    fn as_fn_call_ctx(&mut self) -> &mut FnCallCtx {
        if let ParserCtx::FnCall(context) = self {
            return context;
        } else {
            panic!("Expected FnCall");
        }
    }
    
    fn as_expr_ctx(&mut self) -> &mut ExprCtx {
        if let ParserCtx::Expr(context) = self {
            return context;
        } else {
            panic!("Expected Operation");
        }
    }

    fn as_cast_ctx(&mut self) -> &mut CastCtx {
        if let ParserCtx::Cast(context) = self {
            return context;
        } else {
            panic!("Expected Operation");
        }
    }
}

#[derive(PartialEq)]
enum Scope {
    Global,
    FnBody,
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