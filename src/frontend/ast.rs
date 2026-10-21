//! Typed AST for Soplang. Expr/Stmt enums. IMPLEMENTATION_PLAN Phase 2.

use std::fmt;

/// Static type annotation, a class name, or dynamic (`door` / `madoor`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAnnotation {
    Abn,
    Jajab,
    Qoraal,
    Bool,
    Teed,
    Tuple,
    Set,
    SetOf(Box<TypeAnnotation>),
    TupleOf(Vec<TypeAnnotation>),
    ListOf(Box<TypeAnnotation>),
    ObjectOf(Box<TypeAnnotation>),
    Bytes,
    Walax,
    Dynamic,
    Null,
    Union(Vec<TypeAnnotation>),
    /// Instance of this class, or of a class that extends it.
    Class(String),
}

impl TypeAnnotation {
    pub(crate) fn same_contract(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ListOf(a), Self::ListOf(b))
            | (Self::ObjectOf(a), Self::ObjectOf(b))
            | (Self::SetOf(a), Self::SetOf(b)) => a.same_contract(b),
            (Self::TupleOf(a), Self::TupleOf(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.same_contract(b))
            }
            (Self::Union(a), Self::Union(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .all(|ty| b.iter().any(|other| ty.same_contract(other)))
            }
            _ => self == other,
        }
    }
    pub(crate) fn union(types: Vec<Self>) -> Self {
        let mut result = Vec::new();
        for ty in types {
            if ty == Self::Dynamic {
                return Self::Dynamic;
            }
            let alternatives = match ty {
                Self::Union(types) => types,
                ty => vec![ty],
            };
            for ty in alternatives {
                if !result
                    .iter()
                    .any(|existing: &Self| existing.same_contract(&ty))
                {
                    result.push(ty);
                }
            }
        }
        if result.len() == 1 {
            result.remove(0)
        } else {
            Self::Union(result)
        }
    }

    pub(crate) fn runtime_contract(&self) -> String {
        match self {
            Self::Class(name) => format!("@{name}"),
            Self::TupleOf(types) => format!(
                "xirmo[{}]",
                types
                    .iter()
                    .map(Self::runtime_contract)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::SetOf(inner) => format!("urur[{}]", inner.runtime_contract()),
            Self::ListOf(inner) => format!("teed[{}]", inner.runtime_contract()),
            Self::ObjectOf(inner) => format!("walax[qoraal,{}]", inner.runtime_contract()),
            Self::Union(types) => types
                .iter()
                .map(Self::runtime_contract)
                .collect::<Vec<_>>()
                .join("|"),
            ty => ty.to_string(),
        }
    }

    pub(crate) fn named(name: String) -> Self {
        if name == "urur" {
            Self::Set
        } else if name == "xirmo" {
            Self::Tuple
        } else if name == "baytyo" {
            Self::Bytes
        } else {
            Self::Class(name)
        }
    }
}

/// Literal values at parse time.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(x) => write!(f, "{}", x),
            Literal::Str(s) => write!(f, "{:?}", s),
            Literal::Bool(b) => write!(f, "{}", if *b { "run" } else { "been" }),
            Literal::Null => write!(f, "maran"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InterpolationPart {
    Text(String),
    Value {
        expression: Expr,
        format: String,
        span: Option<crate::error::SourceSpan>,
    },
}

/// Expressions (produce values).
#[derive(Debug, Clone)]
pub enum Expr {
    Located {
        span: crate::error::SourceSpan,
        expr: Box<Expr>,
    },
    Literal(Literal),
    Interpolated {
        parts: Vec<InterpolationPart>,
    },
    Identifier(String),
    NamedArgument {
        name: String,
        value: Box<Expr>,
    },
    BinaryOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: String,
        expr: Box<Expr>,
    },
    /// Function call: name(args)
    Call {
        name: String,
        args: Vec<Expr>,
    },
    /// Invoke any callable expression, including a returned closure.
    Invoke {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Anonymous function expression: hawl(params) { body }
    AnonymousFunc {
        params: Vec<Param>,
        return_ann: TypeAnnotation,
        body: Vec<Stmt>,
    },
    /// obj.method(args)
    MethodCall {
        obj: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// arr[idx]
    Index {
        obj: Box<Expr>,
        idx: Box<Expr>,
    },
    /// obj.prop
    Property {
        obj: Box<Expr>,
        prop: String,
    },
    List(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    /// cusub ClassName(args)
    New {
        class_name: String,
        args: Vec<Expr>,
    },
}

impl Expr {
    pub fn unlocated(&self) -> &Self {
        match self {
            Self::Located { expr, .. } => expr.unlocated(),
            other => other,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Located { expr, .. } => write!(f, "{}", expr),
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Identifier(s) => write!(f, "{}", s),
            Expr::NamedArgument { name, value } => write!(f, "{}: {}", name, value),
            Expr::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::UnaryOp { op, expr } => write!(f, "({} {})", op, expr),
            Expr::Call { name, args } => {
                write!(f, "{}(", name)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ")")
            }
            Expr::Interpolated { parts } => {
                write!(f, "f\"")?;
                for part in parts {
                    match part {
                        InterpolationPart::Text(text) => write!(
                            f,
                            "{}",
                            text.escape_debug()
                                .to_string()
                                .replace('{', "{{")
                                .replace('}', "}}")
                        )?,
                        InterpolationPart::Value {
                            expression, format, ..
                        } => {
                            write!(f, "{{{}", expression)?;
                            if !format.is_empty() {
                                write!(f, ":{}", format)?;
                            }
                            write!(f, "}}")?;
                        }
                    }
                }
                write!(f, "\"")
            }
            Expr::Invoke { callee, args } => {
                write!(f, "({})(", callee)?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::AnonymousFunc {
                params, return_ann, ..
            } => {
                write!(f, "hawl(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if p.variadic || p.keyword_rest {
                        write!(f, "{}", if p.keyword_rest { "**" } else { "*" })?;
                    }
                    write!(f, "{}", p.name)?;
                    if let Some(default) = &p.default {
                        write!(f, " = {}", default)?;
                    }
                }
                if return_ann != &TypeAnnotation::Dynamic {
                    write!(f, ") : {} {{ ... }}", type_annotation_name(return_ann))
                } else {
                    write!(f, ") {{ ... }}")
                }
            }
            Expr::MethodCall { obj, method, args } => {
                write!(f, "({}).{}(", obj, method)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ")")
            }
            Expr::Index { obj, idx } => write!(f, "{}[{}]", obj, idx),
            Expr::Property { obj, prop } => write!(f, "{}.{}", obj, prop),
            Expr::List(es) => {
                write!(f, "[")?;
                for (i, e) in es.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            }
            Expr::Object(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Expr::New { class_name, args } => {
                write!(f, "cusub {}(", class_name)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Abn => f.write_str("abn"),
            Self::Jajab => f.write_str("jajab"),
            Self::Qoraal => f.write_str("qoraal"),
            Self::Bool => f.write_str("bool"),
            Self::Teed => f.write_str("teed"),
            Self::Tuple => f.write_str("xirmo"),
            Self::Set => f.write_str("urur"),
            Self::TupleOf(types) => {
                f.write_str("xirmo[")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                f.write_str("]")
            }
            Self::SetOf(inner) => write!(f, "urur[{inner}]"),
            Self::ListOf(inner) => write!(f, "teed[{inner}]"),
            Self::ObjectOf(inner) => write!(f, "walax[qoraal, {inner}]"),
            Self::Walax => f.write_str("walax"),
            Self::Bytes => f.write_str("baytyo"),
            Self::Dynamic => f.write_str("door"),
            Self::Null => f.write_str("maran"),
            Self::Class(name) => f.write_str(name),
            Self::Union(types) => {
                if types.len() == 2 && types.contains(&Self::Null) {
                    let value = types.iter().find(|ty| **ty != Self::Null).unwrap();
                    write!(f, "{value}?")
                } else {
                    for (index, ty) in types.iter().enumerate() {
                        if index > 0 {
                            f.write_str(" | ")?;
                        }
                        write!(f, "{ty}")?;
                    }
                    Ok(())
                }
            }
        }
    }
}

fn type_annotation_name(ann: &TypeAnnotation) -> String {
    ann.to_string()
}

/// A required parameter or a value evaluated when the argument is omitted.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: TypeAnnotation,
    pub default: Option<Expr>,
    pub variadic: bool,
    pub keyword_rest: bool,
}

/// One name brought into the caller from a namespaced file.
#[derive(Debug, Clone)]
pub struct SelectedImport {
    pub name: String,
    pub alias: Option<String>,
}

/// A file import. `alias` keeps the file's public names on one value.
/// `selected` binds individual public names in the caller.
#[derive(Debug, Clone)]
pub struct ImportSpec {
    pub path: String,
    pub alias: Option<String>,
    pub selected: Vec<SelectedImport>,
}
#[derive(Debug, Clone)]
pub enum Stmt {
    Located {
        span: crate::error::SourceSpan,
        stmt: Box<Stmt>,
    },
    VarDecl {
        name: String,
        type_ann: TypeAnnotation,
        is_const: bool,
        value: Expr,
        #[allow(dead_code)] // Phase 3 error reporting
        line: usize,
        #[allow(dead_code)]
        col: usize,
    },
    Assign {
        target: Expr,
        value: Expr,
        #[allow(dead_code)] // Phase 3 error reporting
        line: usize,
        #[allow(dead_code)]
        col: usize,
    },
    FuncDef {
        name: String,
        params: Vec<Param>,
        return_ann: TypeAnnotation,
        body: Vec<Stmt>,
    },
    ClassDef {
        name: String,
        parent: Option<String>,
        body: Vec<Stmt>,
    },
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        elseifs: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    Switch {
        expr: Expr,
        cases: Vec<(Expr, Vec<Stmt>)>,
        default: Option<Vec<Stmt>>,
    },
    For {
        var: String,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    ForEach {
        vars: Vec<String>,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Throw(Expr),
    Break,
    Continue,
    TryCatch {
        try_body: Vec<Stmt>,
        err_var: Option<String>,
        err_details: Option<String>,
        catch_body: Vec<Stmt>,
        finally_body: Vec<Stmt>,
    },
    Import(ImportSpec),
    /// Names listed by `soosaar` in a namespaced file.
    Export(Vec<String>),
    Block(Vec<Stmt>),
    Expr(Expr),
}

impl Stmt {
    pub fn unlocated(&self) -> &Self {
        match self {
            Self::Located { stmt, .. } => stmt.unlocated(),
            other => other,
        }
    }

    /// Pretty-print with indentation (for debug AST output).
    pub fn fmt_with_depth(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        const INDENT: &str = "  ";
        let pad = INDENT.repeat(depth);
        let pad_inner = INDENT.repeat(depth + 1);
        match self {
            Stmt::Located { stmt, .. } => stmt.fmt_with_depth(f, depth),
            Stmt::VarDecl {
                name,
                type_ann,
                is_const,
                value,
                ..
            } => {
                let ty = if *type_ann == TypeAnnotation::Dynamic && *is_const {
                    "madoor".to_owned()
                } else {
                    type_ann.to_string()
                };
                writeln!(f, "{}{} {} = {};", pad, ty, name, value)
            }
            Stmt::Assign { target, value, .. } => writeln!(f, "{}{} = {};", pad, target, value),
            Stmt::FuncDef {
                name,
                params,
                return_ann,
                body,
            } => {
                write!(f, "{}hawl {}(", pad, name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    let param_ty = p.type_ann.to_string();
                    write!(
                        f,
                        "{} {}{}",
                        param_ty,
                        if p.keyword_rest {
                            "**"
                        } else if p.variadic {
                            "*"
                        } else {
                            ""
                        },
                        p.name
                    )?;
                    if let Some(default) = &p.default {
                        write!(f, " = {}", default)?;
                    }
                }
                let ret_ty = return_ann.to_string();
                writeln!(f, ") : {} {{", ret_ty)?;
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::ClassDef { name, parent, body } => {
                if let Some(p) = parent {
                    writeln!(f, "{}qaab {} dhaxal {} {{", pad, name, p)?;
                } else {
                    writeln!(f, "{}qaab {} {{", pad, name)?;
                }
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::If {
                cond,
                then_body,
                elseifs,
                else_body,
            } => {
                writeln!(f, "{}haddii ({}) {{", pad, cond)?;
                for s in then_body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                for (c, b) in elseifs {
                    writeln!(f, "{}haddii_kale ({}) {{", pad, c)?;
                    for s in b {
                        s.fmt_with_depth(f, depth + 2)?;
                    }
                    writeln!(f, "{}}}", pad)?;
                }
                if let Some(eb) = else_body {
                    writeln!(f, "{}ugudambeyn {{", pad)?;
                    for s in eb {
                        s.fmt_with_depth(f, depth + 1)?;
                    }
                    writeln!(f, "{}}}", pad)?;
                }
                Ok(())
            }
            Stmt::Switch {
                expr,
                cases,
                default,
            } => {
                writeln!(f, "{}dooro ({}) {{", pad, expr)?;
                for (v, stmts) in cases {
                    writeln!(f, "{}xaalad {} {{", pad_inner, v)?;
                    for s in stmts {
                        s.fmt_with_depth(f, depth + 2)?;
                    }
                    writeln!(f, "{}}}", pad_inner)?;
                }
                if let Some(d) = default {
                    writeln!(f, "{}ugudambeyn {{", pad_inner)?;
                    for s in d {
                        s.fmt_with_depth(f, depth + 2)?;
                    }
                    writeln!(f, "{}}}", pad_inner)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::For {
                var,
                start,
                end,
                step,
                body,
            } => {
                write!(f, "{}kuceli ({} {} ilaa {}", pad, var, start, end)?;
                if let Some(s) = step {
                    write!(f, " :: {}", s)?;
                }
                writeln!(f, ") {{")?;
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::ForEach {
                vars,
                iterable,
                body,
            } => {
                writeln!(f, "{}kuceli ({} ku {}) {{", pad, vars.join(", "), iterable)?;
                for stmt in body {
                    stmt.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::While { cond, body } => {
                writeln!(f, "{}intay ({}) {{", pad, cond)?;
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::Throw(e) => writeln!(f, "{}tuur {};", pad, e),
            Stmt::Return(Some(e)) => writeln!(f, "{}celi {};", pad, e),
            Stmt::Return(None) => writeln!(f, "{}celi;", pad),
            Stmt::Break => writeln!(f, "{}jooji;", pad),
            Stmt::Continue => writeln!(f, "{}soco;", pad),
            Stmt::TryCatch {
                try_body,
                err_var,
                err_details,
                catch_body,
                finally_body,
            } => {
                writeln!(f, "{}tijaabi {{", pad)?;
                for stmt in try_body {
                    stmt.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)?;
                if let Some(name) = err_var {
                    if let Some(details) = err_details {
                        writeln!(f, "{}qabo ({}, {}) {{", pad, name, details)?;
                    } else {
                        writeln!(f, "{}qabo ({}) {{", pad, name)?;
                    }
                    for stmt in catch_body {
                        stmt.fmt_with_depth(f, depth + 1)?;
                    }
                    writeln!(f, "{}}}", pad)?;
                }
                if !finally_body.is_empty() || err_var.is_none() {
                    writeln!(f, "{}dhammee {{", pad)?;
                    for stmt in finally_body {
                        stmt.fmt_with_depth(f, depth + 1)?;
                    }
                    writeln!(f, "{}}}", pad)?;
                }
                Ok(())
            }
            Stmt::Import(import) => {
                write!(f, "{pad}keen {:?}", import.path)?;
                if let Some(alias) = &import.alias {
                    write!(f, " sida {alias}")?;
                }
                if !import.selected.is_empty() {
                    write!(f, " ka ")?;
                    for (index, item) in import.selected.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", item.name)?;
                        if let Some(alias) = &item.alias {
                            write!(f, " sida {alias}")?;
                        }
                    }
                }
                writeln!(f, ";")
            }
            Stmt::Export(names) => writeln!(f, "{}soosaar {};", pad, names.join(", ")),
            Stmt::Block(stmts) => {
                writeln!(f, "{} {{", pad)?;
                for s in stmts {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::Expr(e) => writeln!(f, "{}{};", pad, e),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_depth(f, 0)
    }
}
