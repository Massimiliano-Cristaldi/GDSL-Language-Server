use core::panic;
use std::collections::HashMap;
use std::fmt::format;
use std::mem;
use std::sync::LazyLock;
use std::task::Context;

use crate::common::{DataType, Function, Token, TokenKind};
use crate::tables::{BUILT_IN_FUNCTIONS, GLOBALS, TYPE_KEYWORDS};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    curr_index: usize,
    scope: Scope,
    global_idents: HashMap<&'a str, DataType>,
    scope_idents: HashMap<&'a str, DataType>,
    prev_ctxs: Vec<ParserCtx>,
    functions: HashMap<String, Function>,
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
            global_idents: HashMap::with_capacity(32),
            scope_idents: HashMap::with_capacity(32),
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
            
            let task = match curr_ctx {
                ParserCtx::IdentDecl(ref mut ctx) => {
                    self.ident_decl_branch(token, ctx)
                }
                ParserCtx::IdentAssign(ref mut ctx) => {
                    self.ident_assign_branch(token, ctx)
                }
                ParserCtx::ArrDecl(ref mut ctx) => {
                    self.arr_decl_branch(token, ctx)
                }
                ParserCtx::ArrAssign(ref mut ctx) => {
                    self.arr_assign_branch(token, ctx)
                }
                ParserCtx::FnDecl(ref mut ctx) => {
                    self.fn_decl_branch(token, ctx)
                }
                ParserCtx::FnCall(ref mut ctx) => {
                    self.fn_call_branch(token, ctx)
                }
                ParserCtx::Expr(ref mut ctx) => {
                    self.expr_branch(token, ctx)
                }
                ParserCtx::Cast(ref mut ctx) => {
                    self.cast_branch(token, ctx)
                }
                ParserCtx::Default => {
                    self.default_branch(token)
                }
            };

            if let Some(task) = task {
                match task {
                    ParserTask::Enter(new_ctx) => {
                        let prev_ctx = mem::replace(&mut curr_ctx, new_ctx);
                        self.prev_ctxs.push(prev_ctx);
                    }
                    ParserTask::Switch(new_ctx) => {
                        curr_ctx = new_ctx;
                    }
                    ParserTask::Exit => {
                        self.exit_ctx(&mut curr_ctx);
                    }
                }
            }
            
            if self.scope == Scope::FnBody && token.value == "}" {
                self.scope_idents.clear();
                self.scope = Scope::Global;
            }

            //TODO: keep track of poisoned state
            if self.can_recover(&token, &curr_ctx) {
                self.exit_ctx(&mut curr_ctx);
            }
        }

        if self.scope == Scope::FnBody {
            //We can safely unwrap because we cannot change scope if the document is empty
            let token =
                self
                .tokens
                .last()
                .unwrap()
                .clone();
            
            self.push_diagnostic_at_head(
                &token,
                String::from("Expected '}'")
            );
        }

        return self.diagnostics;
    }

    fn ident_decl_branch(&mut self, token: Token<'a>, ctx: &mut IdentDeclCtx) -> Option<ParserTask> {
        match ctx.subctx {
            0 => {
                self.expect_kind(&token, TokenKind::TypeKeyword);
                ctx.ident_type = *TYPE_KEYWORDS.get(token.value).unwrap();
                ctx.subctx = 1;
                return None;
            }
            1 => {
                match token.kind {
                    TokenKind::Ident(_) => {
                        let is_fn = match self.peek(1) {
                            Some(next_token) => next_token.value == "(",
                            None => false
                        };

                        if is_fn {
                            if self.functions.contains_key(token.value) {
                                self.push_diagnostic(
                                    &token,
                                    format!("Cannot redeclare function {}", token.value)
                                );
                            } else {
                                self.functions.insert(
                                    String::from(token.value),
                                    Function::new(HashMap::new(), ctx.ident_type)
                                );
                            }

                            if self.scope == Scope::FnBody {
                                self.push_diagnostic(
                                    &token,
                                    String::from("Cannot declare functions inside other functions")
                                ); 
                            }

                            self.curr_index += 1;
                            let new_ctx = ParserCtx::new_fn_decl(String::from(token.value), ctx.ident_type);
                            return Some(ParserTask::Switch(new_ctx));
                        } 
                        
                        if self.scope_idents.contains_key(token.value) {
                            self.push_diagnostic(
                                &token,
                                format!("Cannot redeclare variable {}", token.value)
                            );

                            ctx.subctx = 2;
                            return None;
                        }
                        
                        
                        match self.scope {
                            Scope::FnBody => {
                                self.scope_idents.insert(
                                    token.value,
                                    ctx.ident_type
                                );
                            }
                            Scope::Global => {
                                self.global_idents.insert(
                                    token.value,
                                    ctx.ident_type
                                );
                            }
                        };

                        ctx.is_valid_decl = true;
                        ctx.subctx = 2;

                        return None;
                    }
                    TokenKind::Global(_) => {
                        self.push_diagnostic(
                            &token,
                            format!("Cannot redeclare global variable {}", token.value)
                        );

                        ctx.subctx = 2;
                        return None;
                    }
                    _ => {
                        self.push_generic_diagnostic(&token);
                        return Some(ParserTask::Exit);
                    }
                }
            }
            2 => {
                match token.value {
                    "[" => {
                        return Some(
                            ParserTask::Switch(ParserCtx::new_arr_decl())
                        );
                    }
                    "=" => {
                        let new_ctx = ParserCtx::new_expr(ctx.ident_type.clone());
                        return Some(ParserTask::Switch(new_ctx));
                    }
                    ";" => {
                        if ctx.is_valid_decl {
                            //TODO: peek back
                            self.scope_idents.insert(
                                token.value,
                                DataType::Unknown
                            );
                        }

                        return Some(ParserTask::Exit);
                    }
                    _ => {
                        self.push_generic_diagnostic(&token);
                        return Some(ParserTask::Exit);
                    }
                }
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn ident_assign_branch(&mut self, token: Token, ctx: &IdentAssignCtx) -> Option<ParserTask> {
        todo!()
    }
    
    fn arr_decl_branch(&mut self, token: Token, ctx: &ArrDeclCtx) -> Option<ParserTask> {
        todo!()
        // self.expect_one_of_kinds(
        //     &token,
        //     &[
        //         TokenKind::IntLit,
        //         TokenKind::Ident(DataType::I8),
        //         TokenKind::Ident(DataType::I16),
        //         TokenKind::Ident(DataType::I32)
        //     ]
        // );
    }

    fn arr_assign_branch(&mut self, token: Token, ctx: &mut ArrAssignCtx) -> Option<ParserTask> {
        todo!()
    }

    fn fn_decl_branch(&mut self, token: Token<'a>, ctx: &mut FnDeclCtx) -> Option<ParserTask> {
        match ctx.subctx {
            0 => {
                if token.value == ")" {
                    ctx.subctx = 3;
                    return None;
                }
                
                if self.expect_kind(&token, TokenKind::TypeKeyword) {
                    let data_type = *TYPE_KEYWORDS.get(token.value).unwrap();
                    
                    ctx.curr_arg_type = data_type;
                    ctx.subctx = 1;
                } else {
                    ctx.subctx = 3;
                }

                return None;
            }
            1 => {
                if self.expect_kind(&token, TokenKind::Ident(DataType::Unknown)) {
                    ctx.args.insert(
                        String::from(token.value),
                        ctx.curr_arg_type
                    );
                    
                    ctx.curr_arg_type = DataType::Unknown;
                }

                ctx.subctx = 2;
                return None;
            }
            2 => {
                if token.value == "," {
                    ctx.subctx = 0;
                    return None;
                }

                self.expect_value(&token, ")");

                ctx.subctx = 3;
                return None;
            }
            3 => {
                if self.expect_value(&token, "{") {
                    self.scope = Scope::FnBody;
                }

                let args = mem::take(&mut ctx.args);
                let fn_name = mem::take(&mut ctx.fn_name);
                
                self.functions.insert(
                    fn_name,
                    Function::new(args, ctx.ret_type)
                );

                return Some(ParserTask::Exit);
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn fn_call_branch(&mut self, token: Token, ctx: &FnCallCtx) -> Option<ParserTask> {
        match ctx.subctx {
            0 => {
                return None;
            },
            _ => {
                return Some(ParserTask::Exit);
            }
        }
    }

    fn expr_branch(&mut self, token: Token, ctx: &mut ExprCtx) -> Option<ParserTask> {
        match ctx.subctx {
            0 => {
                match token.kind {
                    TokenKind::Ident(data_type) => {
                        if let Some(next_token) = self.peek(1)
                        && next_token.value == "("
                        {
                            ctx.subctx = 1;
                            self.curr_index += 1;
                            return Some(ParserTask::Enter(ParserCtx::new_fn_call()));
                        }

                        if (data_type == DataType::Unknown) {
                            self.push_diagnostic(
                                &token,
                                String::from("Use of variable before declaration")
                            );
                            
                            ctx.subctx = 1;
                            return None;
                        }

                        if let Some(vec_type) = token.try_vec_type()
                        && vec_type == ctx.result_type
                        {
                            //TODO: vec component access subctx
                            ctx.subctx = 100;
                        } else if data_type != ctx.result_type {
                            ctx.subctx = 1;

                            self.push_diagnostic(
                                &token,
                                format!("Unexpected variable type: expected {}, found {}", ctx.result_type, data_type)
                            );
                        }

                        return None;
                    }
                    TokenKind::Operator => {
                        if self.expect_one_of_values(&token, &["+", "-"]) {
                            //TODO:: unary operation subctx
                        }

                        return None;
                    }
                    TokenKind::Symbol => {
                        match token.value {
                            "(" => {
                                ctx.brackets.push('(');
                            }
                            "[" => {
                                ctx.brackets.push('[');
                            }
                            ")" => {
                                match ctx.brackets.last() {
                                    Some(&'(') => {},
                                    _ => self.push_generic_diagnostic(&token)
                                }
                            }
                            "]" => {
                                match ctx.brackets.last() {
                                    Some(&'[') => {},
                                    _ => self.push_generic_diagnostic(&token)
                                }
                            }
                            _ => {
                                self.push_generic_diagnostic(&token);
                            }
                        }

                        ctx.subctx = 1;

                        return None;
                    }
                    TokenKind::TypeKeyword => {
                        if let Some(next_token) = self.peek(1) 
                        && next_token.value == "["
                        {
                            //TODO: array initialization subctx
                        }

                        return None;
                    }
                    _ => {
                        match ctx.result_type {
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

                        ctx.subctx = 1;
                        return None;
                    }
                }
            },
            1 => {
                match token.kind {
                    //TODO: check operator compatibility
                    TokenKind::Operator => {
                        ctx.subctx = 0;
                    }
                    _ => {
                        if token.value == ";" {
                            return Some(ParserTask::Exit);
                        } else {
                            self.push_generic_diagnostic(&token);
                        }
                    }
                }

                return None;
            }
            //TODO: Expect vector component access
            100 => {
                return None;
            }
            _ => {
                return Some(ParserTask::Exit);
            }
        }
    }

    fn cast_branch(&mut self, token: Token, ctx: &CastCtx) -> Option<ParserTask> {
        match ctx.subctx {
            0 => {
                return None;
            },
            _ => {
                return Some(ParserTask::Exit);
            }
        }
    }

    fn default_branch(&mut self, token: Token) -> Option<ParserTask> {
        match (token.kind, token.value) {
            (TokenKind::TypeKeyword, _) => {
                //We can safely unwrap because TypeKeyword tokens can only be created
                //when the tokenizer finds the token value in the TYPE_KEYWORDS map
                let ident_type = *TYPE_KEYWORDS.get(token.value).unwrap();
                return Some(ParserTask::Enter(
                    ParserCtx::new_ident_decl(1, ident_type, true)
                ));
            },
            (TokenKind::MiscKeyword, "uniform") |
            (TokenKind::MiscKeyword, "const") => {
                return Some(ParserTask::Enter(
                    ParserCtx::new_ident_decl(0, DataType::Unknown, false)
                ));
            }
            _ => {
                return None;
            }
        }
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

    fn get_ident_type(&self, ident_name: &'a str) -> DataType {
        match self.scope_idents.get(ident_name) {
            Some(data_type) => return *data_type,
            None => {
                match self.global_idents.get(ident_name) {
                    Some (data_type) => return *data_type,
                    None => DataType::Unknown
                }
            }
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
            token.head()
        );
        
        self.diagnostics.push(diagnostic);
    }

    fn push_generic_diagnostic(&mut self, token: &Token) -> () {
        self.push_diagnostic(
            &token, 
            format!("Unexpected token: '{}'", token.value)
        );
    }

    fn push_diagnostic_at_head(&mut self, token: &Token, msg: String) -> () {
        let diagnostic = Diagnostic {
            line: token.line,
            msg: msg,
            col_start: token.head(),
            col_end: token.head()
        };
        
        self.diagnostics.push(diagnostic);
    }

    fn debug_iteration(&self, ctx: &ParserCtx) -> () {
        if let Some(curr_token) = self.tokens.get(self.curr_index) {
            let (ctx_str, subctx) = match ctx {
                ParserCtx::ArrAssign(ctx) => ("AssAssign", ctx.subctx),
                ParserCtx::ArrDecl(ctx) => ("ArrDecl", ctx.subctx),
                ParserCtx::Cast(ctx) => ("Cast", ctx.subctx),
                ParserCtx::Expr(ctx) => ("Expr", ctx.subctx),
                ParserCtx::FnCall(ctx) => ("FnCall", ctx.subctx),
                ParserCtx::FnDecl(ctx) => ("FnDecl", ctx.subctx),
                ParserCtx::IdentAssign(ctx) => ("IdentAssign", ctx.subctx),
                ParserCtx::IdentDecl(ctx) => ("IdentDecl", ctx.subctx),
                ParserCtx::Default => ("Default", 0),
            };

            println!("curr_index: {} | {:?} | scope: {:?} | ctx: {} | subctx: {}", self.curr_index, curr_token, self.scope, ctx_str, subctx);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct IdentDeclCtx {
    subctx: usize,
    is_valid_decl: bool,
    ident_type: DataType,
    is_mut: bool
}

#[derive(Debug, Clone, PartialEq)]
struct IdentAssignCtx {
    subctx: usize,
    ident_type: DataType,
}

#[derive(Debug, Clone, PartialEq)]
struct ArrDeclCtx {
    subctx: usize
}

#[derive(Debug, Clone, PartialEq)]
struct ArrAssignCtx {
    subctx: usize
}

#[derive(Debug, Clone, PartialEq)]
struct FnDeclCtx {
    subctx: usize,
    fn_name: String,
    curr_arg_type: DataType,
    args: HashMap<String, DataType>,
    ret_type: DataType
}

#[derive(Debug, Clone, PartialEq)]
struct FnCallCtx {
    subctx: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct ExprCtx {
    subctx: usize,
    lhs: Option<DataType>,
    operator: &'static str,
    rhs: DataType,
    result_type: DataType,
    brackets: Vec<char>
}

#[derive(Debug, Clone, PartialEq)]
struct CastCtx {
    subctx: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum ParserCtx {
    Default,
    IdentDecl(IdentDeclCtx),
    IdentAssign(IdentAssignCtx),
    ArrDecl(ArrDeclCtx),
    ArrAssign(ArrAssignCtx),
    FnDecl(FnDeclCtx),
    FnCall(FnCallCtx),
    Expr(ExprCtx),
    Cast(CastCtx),
}

impl ParserCtx {
    fn new_default() -> ParserCtx {
        return ParserCtx::Default;
    }
    
    fn new_ident_decl(subctx: usize, ident_type: DataType, is_mut: bool) -> ParserCtx {
        return ParserCtx::IdentDecl(IdentDeclCtx {
            subctx: subctx,
            is_valid_decl: false,
            ident_type: ident_type,
            is_mut: is_mut
        });
    }

    fn new_ident_assign() -> ParserCtx {
        return ParserCtx::IdentAssign(IdentAssignCtx {
            subctx: 0 ,
            ident_type: DataType::Unknown,
        });
    }

    fn new_arr_decl() -> ParserCtx {
        return ParserCtx::ArrDecl(ArrDeclCtx {
            subctx: 0
        });
    }

    fn new_arr_assign() -> ParserCtx {
        return ParserCtx::ArrAssign(ArrAssignCtx {
            subctx: 0
        });
    }

    fn new_fn_decl(fn_name: String, ret_type: DataType) -> ParserCtx {
        return ParserCtx::FnDecl(FnDeclCtx {
            subctx: 0,
            fn_name: fn_name,
            curr_arg_type: DataType::Unknown,
            args: HashMap::new(),
            ret_type: ret_type
        });
    }

    fn new_fn_call() -> ParserCtx {
        return ParserCtx::FnCall(FnCallCtx {
            subctx: 0,
        });
    }

    fn new_expr(result_type: DataType) -> ParserCtx {
        return ParserCtx::Expr(ExprCtx {
            subctx: 0,
            lhs: None,
            operator: "",
            rhs: DataType::Unknown,
            result_type: result_type,
            brackets: Vec::with_capacity(10)
        });
    }

    fn new_cast() -> ParserCtx {
        return ParserCtx::Cast(CastCtx { subctx: 0 });
    }
}

enum ParserTask {
    Switch(ParserCtx),
    Enter(ParserCtx),
    Exit
}

#[derive(PartialEq, Debug)]
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