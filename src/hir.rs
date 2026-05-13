//! Compatibility module for the execution-shaped IR now owned by `mir`.
//!
//! Existing callers may still import `muga::hir` or use AST lowering here, but
//! new backend work should use `muga::mir` directly.

use std::collections::HashMap;

use crate::{
    ast, known_enum,
    span::Span,
    symbol::{Symbol, SymbolTable},
};

pub use crate::mir::*;

pub fn lower(program: &ast::Program) -> Program {
    let mut lowerer = Lowerer {
        functions: Vec::new(),
        symbols: SymbolTable::default(),
        enum_variants: HashMap::new(),
    };
    lowerer.collect_enum_variants(program);
    let statements = program
        .statements
        .iter()
        .filter_map(|statement| lowerer.lower_stmt(statement))
        .collect();
    Program {
        entry: Body {
            statements,
            result: None,
            span: Span::default(),
        },
        functions: lowerer.functions,
        symbols: lowerer.symbols,
    }
}

struct Lowerer {
    functions: Vec<Function>,
    symbols: SymbolTable,
    enum_variants: HashMap<String, EnumVariantLowering>,
}

#[derive(Clone, Copy)]
struct EnumVariantLowering {
    has_payload: bool,
}

impl Lowerer {
    fn collect_enum_variants(&mut self, program: &ast::Program) {
        for known in [known_enum::option_enum(), known_enum::result_enum()] {
            for variant in known.variants {
                self.enum_variants.insert(
                    known.qualified_variant(*variant),
                    EnumVariantLowering {
                        has_payload: variant.has_payload,
                    },
                );
            }
        }

        for statement in &program.statements {
            if let ast::Stmt::EnumDecl(enumeration) = statement {
                for variant in &enumeration.variants {
                    self.enum_variants.insert(
                        format!("{}::{}", enumeration.name, variant.name),
                        EnumVariantLowering {
                            has_payload: variant.payload.is_some(),
                        },
                    );
                }
            }
        }
    }

    fn lower_stmt(&mut self, statement: &ast::Stmt) -> Option<Stmt> {
        Some(match statement {
            ast::Stmt::Assign(stmt) => Stmt::Assign(AssignStmt {
                mutable: stmt.mutable,
                name: self.symbol(&stmt.name),
                value: self.lower_expr(&stmt.value),
                span: stmt.span,
            }),
            ast::Stmt::RecordDecl(_) => return None,
            ast::Stmt::EnumDecl(_) => return None,
            ast::Stmt::FuncDecl(stmt) => Stmt::Function(FunctionStmt {
                name: self.symbol(&stmt.name),
                function: self.lower_function_decl(stmt),
                span: stmt.span,
            }),
            ast::Stmt::If(stmt) => Stmt::If(IfStmt {
                condition: self.lower_expr(&stmt.condition),
                then_branch: self.lower_block(&stmt.then_branch),
                else_branch: stmt
                    .else_branch
                    .as_ref()
                    .map(|branch| self.lower_block(branch)),
                span: stmt.span,
            }),
            ast::Stmt::While(stmt) => Stmt::While(WhileStmt {
                condition: self.lower_expr(&stmt.condition),
                body: self.lower_block(&stmt.body),
                span: stmt.span,
            }),
            ast::Stmt::Expr(stmt) => Stmt::Expr(ExprStmt {
                expr: self.lower_expr(&stmt.expr),
                span: stmt.span,
            }),
        })
    }

    fn lower_block(&mut self, block: &ast::Block) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .filter_map(|statement| self.lower_stmt(statement))
                .collect(),
            span: block.span,
        }
    }

    fn lower_value_block(&mut self, block: &ast::ValueBlock) -> ValueBlock {
        ValueBlock {
            statements: block
                .statements
                .iter()
                .filter_map(|statement| self.lower_stmt(statement))
                .collect(),
            expr: Box::new(self.lower_expr(&block.expr)),
            span: block.span,
        }
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> Expr {
        match expr {
            ast::Expr::Int(expr) => Expr::Int(IntExpr {
                value: expr.value,
                span: expr.span,
            }),
            ast::Expr::Bool(expr) => Expr::Bool(BoolExpr {
                value: expr.value,
                span: expr.span,
            }),
            ast::Expr::String(expr) => Expr::String(StringExpr {
                value: expr.value.clone(),
                span: expr.span,
            }),
            ast::Expr::Ident(expr) => {
                if self
                    .enum_variants
                    .get(&expr.name)
                    .is_some_and(|variant| !variant.has_payload)
                {
                    let (enum_name, variant_name) =
                        split_variant_name(&expr.name).unwrap_or((&expr.name, ""));
                    Expr::EnumVariant(EnumVariantExpr {
                        enum_name: self.symbol(enum_name),
                        variant_name: self.symbol(variant_name),
                        payload: None,
                        span: expr.span,
                    })
                } else {
                    Expr::Ident(IdentExpr {
                        name: self.symbol(&expr.name),
                        span: expr.span,
                    })
                }
            }
            ast::Expr::ListLit(expr) => Expr::ListLit(ListLitExpr {
                items: expr
                    .items
                    .iter()
                    .map(|item| self.lower_expr(item))
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Index(expr) => Expr::Index(IndexExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                index: Box::new(self.lower_expr(&expr.index)),
                span: expr.span,
            }),
            ast::Expr::RecordLit(expr) => Expr::RecordLit(RecordLitExpr {
                type_name: self.symbol(&expr.type_name),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: self.symbol(&field.name),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Field(expr) => Expr::Field(FieldExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                field: self.symbol(&expr.field),
                span: expr.span,
            }),
            ast::Expr::RecordUpdate(expr) => Expr::RecordUpdate(RecordUpdateExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: self.symbol(&field.name),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Unary(expr) => Expr::Unary(UnaryExpr {
                op: match expr.op {
                    ast::UnaryOp::Neg => UnaryOp::Neg,
                    ast::UnaryOp::Not => UnaryOp::Not,
                },
                expr: Box::new(self.lower_expr(&expr.expr)),
                span: expr.span,
            }),
            ast::Expr::Binary(expr) => Expr::Binary(BinaryExpr {
                op: match expr.op {
                    ast::BinaryOp::Add => BinaryOp::Add,
                    ast::BinaryOp::Sub => BinaryOp::Sub,
                    ast::BinaryOp::Mul => BinaryOp::Mul,
                    ast::BinaryOp::Div => BinaryOp::Div,
                    ast::BinaryOp::Lt => BinaryOp::Lt,
                    ast::BinaryOp::LtEq => BinaryOp::LtEq,
                    ast::BinaryOp::Gt => BinaryOp::Gt,
                    ast::BinaryOp::GtEq => BinaryOp::GtEq,
                    ast::BinaryOp::EqEq => BinaryOp::EqEq,
                    ast::BinaryOp::BangEq => BinaryOp::BangEq,
                },
                left: Box::new(self.lower_expr(&expr.left)),
                right: Box::new(self.lower_expr(&expr.right)),
                span: expr.span,
            }),
            ast::Expr::Call(expr) => {
                if let ast::Expr::Ident(callee) = expr.callee.as_ref() {
                    if self
                        .enum_variants
                        .get(&callee.name)
                        .is_some_and(|variant| variant.has_payload)
                        && expr.args.len() == 1
                    {
                        let (enum_name, variant_name) =
                            split_variant_name(&callee.name).unwrap_or((&callee.name, ""));
                        return Expr::EnumVariant(EnumVariantExpr {
                            enum_name: self.symbol(enum_name),
                            variant_name: self.symbol(variant_name),
                            payload: Some(Box::new(self.lower_expr(&expr.args[0]))),
                            span: expr.span,
                        });
                    }
                }
                Expr::Call(CallExpr {
                    callee: Box::new(self.lower_expr(&expr.callee)),
                    args: expr.args.iter().map(|arg| self.lower_expr(arg)).collect(),
                    span: expr.span,
                })
            }
            ast::Expr::If(expr) => Expr::If(IfExpr {
                condition: Box::new(self.lower_expr(&expr.condition)),
                then_branch: self.lower_value_block(&expr.then_branch),
                else_branch: self.lower_value_block(&expr.else_branch),
                span: expr.span,
            }),
            ast::Expr::Match(expr) => Expr::Match(MatchExpr {
                value: Box::new(self.lower_expr(&expr.value)),
                arms: expr
                    .arms
                    .iter()
                    .map(|arm| MatchArm {
                        pattern: match &arm.pattern {
                            ast::MatchPattern::Variant(pattern) => {
                                MatchPattern::Variant(EnumVariantPattern {
                                    enum_name: self.symbol(&pattern.enum_name),
                                    variant_name: self.symbol(&pattern.variant_name),
                                    binding: pattern
                                        .binding
                                        .as_ref()
                                        .map(|binding| self.symbol(binding)),
                                    span: pattern.span,
                                })
                            }
                        },
                        value: self.lower_expr(&arm.value),
                        span: arm.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Fn(expr) => Expr::Closure(ClosureExpr {
                function: self.lower_fn_expr(expr),
                span: expr.span,
            }),
        }
    }

    fn lower_function_decl(&mut self, stmt: &ast::FuncDecl) -> FunctionId {
        let id = self.functions.len();
        let name = self.symbol(&stmt.name);
        let params = stmt
            .params
            .iter()
            .map(|param| self.symbol(&param.name))
            .collect();
        self.functions.push(Function {
            id,
            name: Some(name),
            params,
            body: placeholder_body(stmt.span),
            span: stmt.span,
        });
        let body = body_from_value_block(self.lower_value_block(&stmt.body));
        self.functions[id].body = body;
        id
    }

    fn lower_fn_expr(&mut self, expr: &ast::FnExpr) -> FunctionId {
        let id = self.functions.len();
        let params = expr
            .params
            .iter()
            .map(|param| self.symbol(&param.name))
            .collect();
        self.functions.push(Function {
            id,
            name: None,
            params,
            body: placeholder_body(expr.span),
            span: expr.span,
        });
        let body = body_from_value_block(self.lower_value_block(&expr.body));
        self.functions[id].body = body;
        id
    }

    fn symbol(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }
}

fn body_from_value_block(block: ValueBlock) -> Body {
    Body {
        statements: block.statements,
        result: Some(block.expr),
        span: block.span,
    }
}

fn placeholder_body(span: Span) -> Body {
    Body {
        statements: Vec::new(),
        result: Some(Box::new(Expr::Int(IntExpr { value: 0, span }))),
        span,
    }
}

fn split_variant_name(name: &str) -> Option<(&str, &str)> {
    name.rsplit_once("::")
}
