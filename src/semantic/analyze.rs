//! Semantic analysis: name resolution, variable slots, function/class metadata.
//! Phase 1 of COMPILER_PLAN. Consumes AST, produces SymbolTable for HIR lowering.

use std::collections::{HashMap, HashSet};

use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};
use crate::frontend::ast::{Expr, Literal, Stmt, TypeAnnotation};

/// Symbol table built by semantic analysis. Used by HIR lowering.
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Stack of scopes (inner to outer). Scope 0 = global.
    pub scopes:    Vec<Scope>,
    /// Per-function metadata (params, local count, captures).
    pub functions: Vec<FunctionMeta>,
    /// Class name -> metadata (methods, parent).
    pub classes:   HashMap<String, ClassMeta>,
    /// Globals carried in from a REPL session. Top-level uses load/store them
    /// instead of treating their old slots as locals of the new snippet.
    pub session_globals: HashSet<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub vars: HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub slot:        usize,
    pub type_ann:    TypeAnnotation,
    pub is_const:    bool,
    pub is_captured: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionMeta {
    pub name:        String,
    pub param_slots: Vec<usize>,
    pub param_types: Vec<TypeAnnotation>,
    pub return_ann:  TypeAnnotation,
    pub local_count: usize,
    pub captures:    Vec<String>,
    /// Variable name -> VarInfo for this function's scope (params + locals).
    pub scope_vars:  HashMap<String, VarInfo>,
    pub class_name:  Option<String>,
    pub is_method:   bool,
}

#[derive(Debug, Clone)]
pub struct ClassMeta {
    pub name:    String,
    pub parent:  Option<String>,
    /// Method name -> mangled function name (e.g. "salaam" -> "Bisad::salaam").
    pub methods: HashMap<String, String>,
}

/// Mangle a class method name for global registration.
pub fn mangle_method(class: &str, method: &str) -> String {
    format!("{}::{}", class, method)
}

/// Run semantic analysis on the AST. Returns a symbol table or an error.
pub fn analyze(stmts: &[Stmt]) -> Result<SymbolTable, SoplangError> {
    analyze_with_options(stmts, AnalyzeOptions::default())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnalyzeOptions {
    pub strict: bool,
    /// REPL: a later line may declare an existing global again.
    pub repl: bool,
}

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<TypeAnnotation>,
    ret:    TypeAnnotation,
}

/// Run semantic analysis with options (e.g. strict typing mode).
pub fn analyze_with_options(
    stmts: &[Stmt],
    opts: AnalyzeOptions,
) -> Result<SymbolTable, SoplangError> {
    let mut sym = SymbolTable::default();
    sym.scopes.push(Scope::default()); // global scope
    analyze_into(&mut sym, stmts, opts)?;
    Ok(sym)
}

/// Continue analysis on a symbol table that may already hold REPL globals and classes.
pub fn analyze_into(
    sym: &mut SymbolTable,
    stmts: &[Stmt],
    opts: AnalyzeOptions,
) -> Result<(), SoplangError> {
    if sym.scopes.is_empty() {
        sym.scopes.push(Scope::default());
    }
    let fn_sigs = collect_function_sigs(stmts);
    analyze_block(stmts, sym, &opts, &fn_sigs, None, None)?;
    validate_classes(sym)?;
    Ok(())
}

fn validate_classes(sym: &SymbolTable) -> Result<(), SoplangError> {
    for meta in sym.classes.values() {
        if let Some(ref parent) = meta.parent {
            if !sym.classes.contains_key(parent) {
                return Err(type_error_ex(
                    format!(
                        "Qaab '{}' wuxuu ka dhaxlay '{}' laakiin qaabkaas ma jiro",
                        meta.name, parent
                    ),
                    0,
                    0,
                    ErrorMeta::default()
                        .with_code(codes::E021_UNDECLARED)
                        .with_hint("Hubi in qaabka waalidka uu ka horreeyo qaabka ilmaha."),
                ));
            }
        }
    }
    Ok(())
}

fn analyze_block(
    stmts: &[Stmt],
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_return: Option<TypeAnnotation>,
    current_class: Option<&str>,
) -> Result<(), SoplangError> {
    for s in stmts {
        analyze_stmt(s, sym, opts, fn_sigs, current_return, current_class)?;
    }
    Ok(())
}

fn analyze_stmt(
    stmt: &Stmt,
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_return: Option<TypeAnnotation>,
    current_class: Option<&str>,
) -> Result<(), SoplangError> {
    match stmt {
        Stmt::VarDecl { name, type_ann, is_const, value, line, col } => {
            if opts.strict && *type_ann == TypeAnnotation::Dynamic {
                return Err(type_error_ex(
                    format!("'{}' waa dynamic laakiin 'strict mode' waa shidan yahay", name),
                    *line,
                    *col,
                    ErrorMeta::default()
                        .with_code(codes::E022_TYPE_MISMATCH)
                        .with_hint("Strict mode: isticmaal nooc cad oo sugan sida abn/jajab/qoraal/bool/teed/walax."),
                ));
            }
            let at_global = sym.scopes.len() == 1;
            let scope = sym.scopes.last_mut().unwrap();
            if let Some(existing) = scope.vars.get(name).cloned() {
                let allow_again = opts.repl && at_global && !existing.is_const;
                if !allow_again {
                    let (msg, hint) = if existing.is_const {
                        (
                            format!("Ma bedeli kartid qiimaha doorsamaha madoor '{}'", name),
                            "Doorsamaha madoor ma beddeli karaan qiimahooda. Isticmaal 'door' haddii aad rabto mid beddeli kara.",
                        )
                    } else {
                        (
                            format!("Magaca '{}' mar hore ayaa la qeexay", name),
                            "Magacan ayaa mar hore la isticmaalay. Fadlan isticmaal magac kale.",
                        )
                    };
                    return Err(type_error_ex(
                        msg,
                        *line,
                        *col,
                        ErrorMeta::default()
                            .with_code(codes::E020_REDECLARED)
                            .with_hint(hint),
                    ));
                }
                scope.vars.insert(
                    name.clone(),
                    VarInfo {
                        slot: existing.slot,
                        type_ann: *type_ann,
                        is_const: *is_const,
                        is_captured: false,
                    },
                );
            } else {
                let slot = scope.vars.len();
                scope.vars.insert(
                    name.clone(),
                    VarInfo {
                        slot,
                        type_ann: *type_ann,
                        is_const: *is_const,
                        is_captured: false,
                    },
                );
            }
            // Phase 2: basic static type check for initializer when we can infer a type.
            if *type_ann != TypeAnnotation::Dynamic {
                if let Some(expr_ty) = infer_expr_type(value, sym, fn_sigs) {
                    if !is_type_compatible(*type_ann, expr_ty) {
                        return Err(type_mismatch_error(
                            name,
                            *type_ann,
                            expr_ty,
                            *line,
                            *col,
                        ));
                    }
                }
            }
            analyze_expr(value, sym, fn_sigs, current_class)?;
        }
        Stmt::Assign { target, value, line, col } => {
            if let Expr::Identifier(name) = target {
                let var = resolve_name(sym, &name, None).ok_or_else(|| {
                    type_error_ex(
                        format!("Magaca '{}' lama aqoonsan", name),
                        *line,
                        *col,
                        ErrorMeta::default()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Doorsame cusub ku qeex 'door' ama 'madoor' ka hor intaadan isticmaalin."),
                    )
                })?;
                if var.is_const {
                    return Err(type_error_ex(
                        format!("Ma bedeli kartid qiimaha doorsamaha madoor '{}'", name),
                        *line,
                        *col,
                        ErrorMeta::default()
                            .with_code(codes::E020_REDECLARED)
                            .with_hint("Doorsamaha madoor ma beddeli karaan qiimahooda. Isticmaal 'door' haddii aad rabto mid beddeli kara."),
                    ));
                }
                if var.type_ann != TypeAnnotation::Dynamic {
                    if let Some(expr_ty) = infer_expr_type(value, sym, fn_sigs) {
                        if !is_type_compatible(var.type_ann, expr_ty) {
                            return Err(type_mismatch_error(
                                name,
                                var.type_ann,
                                expr_ty,
                                *line,
                                *col,
                            ));
                        }
                    }
                }
            } else {
                // Index or Property: resolve subexpressions only
                analyze_assign_target(target, sym, fn_sigs, current_class)?;
            }
            analyze_expr(value, sym, fn_sigs, current_class)?;
        }
        Stmt::FuncDef { name, params, return_ann, body } => {
            let is_method = current_class.is_some();
            if is_method {
                if params.iter().any(|p| p.name == "nafta") {
                    return Err(type_error_ex(
                        "'nafta' waa magac la xafiday oo si toos ah loo geliyaa qaabka gudihiisa",
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E020_REDECLARED)
                            .with_hint("Ha ku qorin 'nafta' liiska doodaha hawsha."),
                    ));
                }
            }
            if opts.strict {
                if *return_ann == TypeAnnotation::Dynamic {
                    return Err(type_error_ex(
                        format!("hawl '{}' ma leh nooc celi cad", name),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint("Strict mode: qor nooca celi sida ') : abn {' ama ') : qoraal {'."),
                    ));
                }
                if params.iter().any(|p| p.type_ann == TypeAnnotation::Dynamic) {
                    return Err(type_error_ex(
                        format!("hawl '{}' waxay leedahay param dynamic strict mode", name),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint("Strict mode: qor nooca param kasta, tusaale: hawl f(abn x, qoraal y)."),
                    ));
                }
            }
            let fn_name = if let Some(class) = current_class {
                mangle_method(class, name)
            } else {
                name.clone()
            };
            let class_name = current_class.map(str::to_string);
            let mut func_scope = Scope::default();
            let mut param_slots: Vec<usize> = Vec::new();
            if is_method {
                func_scope.vars.insert(
                    "nafta".to_string(),
                    VarInfo {
                        slot: 0,
                        type_ann: TypeAnnotation::Dynamic,
                        is_const: false,
                        is_captured: false,
                    },
                );
                param_slots.push(0);
            }
            for (i, p) in params.iter().enumerate() {
                let slot = if is_method { i + 1 } else { i };
                param_slots.push(slot);
                func_scope.vars.insert(
                    p.name.clone(),
                    VarInfo {
                        slot,
                        type_ann: p.type_ann,
                        is_const: false,
                        is_captured: false,
                    },
                );
            }
            sym.scopes.push(func_scope);
            analyze_block(body, sym, opts, fn_sigs, Some(*return_ann), current_class)?;
            let scope = sym.scopes.pop().unwrap();
            let local_count = scope.vars.len();
            sym.functions.push(FunctionMeta {
                name: fn_name,
                param_slots,
                param_types: params.iter().map(|p| p.type_ann).collect(),
                return_ann: *return_ann,
                local_count,
                captures: Vec::new(),
                scope_vars: scope.vars,
                class_name,
                is_method,
            });
        }
        Stmt::ClassDef { name, parent, body } => {
            if sym.classes.contains_key(name) {
                return Err(type_error_ex(
                    format!("Qaab '{}' mar hore ayaa la qeexay", name),
                    0,
                    0,
                    ErrorMeta::default()
                        .with_code(codes::E020_REDECLARED)
                        .with_hint("Isticmaal magac kale qaabka."),
                ));
            }
            let mut methods = HashMap::new();
            for s in body {
                if let Stmt::FuncDef { name: mname, .. } = s {
                    if methods.contains_key(mname) {
                        return Err(type_error_ex(
                            format!("Hawl '{}' ayaa laba jeer ku jirta qaabka '{}'", mname, name),
                            0,
                            0,
                            ErrorMeta::default()
                                .with_code(codes::E020_REDECLARED)
                                .with_hint("Magacyada hawlgallada qaabka waa in ay noqdaan kuwo gaar ah."),
                        ));
                    }
                    methods.insert(mname.clone(), mangle_method(name, mname));
                }
            }
            sym.classes.insert(
                name.clone(),
                ClassMeta {
                    name: name.clone(),
                    parent: parent.clone(),
                    methods,
                },
            );
            analyze_block(body, sym, opts, fn_sigs, current_return, Some(name))?;
        }
        Stmt::If { cond, then_body, elseifs, else_body } => {
            analyze_expr(cond, sym, fn_sigs, current_class)?;
            analyze_block(then_body, sym, opts, fn_sigs, current_return, current_class)?;
            for (c, b) in elseifs {
                analyze_expr(c, sym, fn_sigs, current_class)?;
                analyze_block(b, sym, opts, fn_sigs, current_return, current_class)?;
            }
            if let Some(eb) = else_body {
                analyze_block(eb, sym, opts, fn_sigs, current_return, current_class)?;
            }
        }
        Stmt::Switch { expr, cases, default } => {
            analyze_expr(expr, sym, fn_sigs, current_class)?;
            for (v, body) in cases {
                analyze_expr(v, sym, fn_sigs, current_class)?;
                analyze_block(body, sym, opts, fn_sigs, current_return, current_class)?;
            }
            if let Some(d) = default {
                analyze_block(d, sym, opts, fn_sigs, current_return, current_class)?;
            }
        }
        Stmt::For { var, start, end, step, body } => {
            // Loop variable is in scope for the body
            let scope = sym.scopes.last_mut().unwrap();
            if !scope.vars.contains_key(var) {
                let slot = scope.vars.len();
                scope.vars.insert(
                    var.clone(),
                    VarInfo {
                        slot,
                        type_ann: TypeAnnotation::Dynamic,
                        is_const: false,
                        is_captured: false,
                    },
                );
            }
            analyze_expr(start, sym, fn_sigs, current_class)?;
            analyze_expr(end, sym, fn_sigs, current_class)?;
            if let Some(s) = step {
                analyze_expr(s, sym, fn_sigs, current_class)?;
            }
            analyze_block(body, sym, opts, fn_sigs, current_return, current_class)?;
        }
        Stmt::While { cond, body } => {
            analyze_expr(cond, sym, fn_sigs, current_class)?;
            analyze_block(body, sym, opts, fn_sigs, current_return, current_class)?;
        }
        Stmt::Return(Some(e)) => {
            analyze_expr(e, sym, fn_sigs, current_class)?;
            if let Some(ret_ann) = current_return {
                if ret_ann != TypeAnnotation::Dynamic {
                    if let Some(expr_ty) = infer_expr_type(e, sym, fn_sigs) {
                        if !is_type_compatible(ret_ann, expr_ty) {
                            return Err(type_error_ex(
                                format!("nooca celi '{}' ma waafaqsana nooca hawsha '{}'", ty_str(expr_ty), ty_str(ret_ann)),
                                0,
                                0,
                                ErrorMeta::default()
                                    .with_code(codes::E022_TYPE_MISMATCH)
                                    .with_hint("Hubi in 'celi' uu la mid yahay nooca lagu qoray hawsha."),
                            ));
                        }
                    }
                }
            }
        }
        Stmt::Return(None) => {
            if let Some(ret_ann) = current_return {
                if ret_ann != TypeAnnotation::Dynamic {
                    return Err(type_error_ex(
                        format!("hawshu waxay rabtaa '{}' laakiin celi qiimo ma leh", ty_str(ret_ann)),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint("Ku celi qiime sax ah ama beddel nooca celi ee hawsha."),
                    ));
                }
            }
        }
        Stmt::Break | Stmt::Continue => {}
        Stmt::TryCatch { try_body, err_var, catch_body, .. } => {
            analyze_block(try_body, sym, opts, fn_sigs, current_return, current_class)?;
            sym.scopes.push(Scope::default());
            {
                let scope = sym.scopes.last_mut().unwrap();
                scope.vars.insert(
                    err_var.clone(),
                    VarInfo {
                        slot:        0,
                        type_ann:    TypeAnnotation::Dynamic,
                        is_const:    false,
                        is_captured: false,
                    },
                );
            }
            analyze_block(catch_body, sym, opts, fn_sigs, current_return, current_class)?;
            sym.scopes.pop();
        }
        Stmt::Import(_) => {}
        Stmt::Block(stmts) => analyze_block(stmts, sym, opts, fn_sigs, current_return, current_class)?,
        Stmt::Expr(e) => analyze_expr(e, sym, fn_sigs, current_class)?,
    }
    Ok(())
}

fn analyze_expr(
    expr: &Expr,
    sym: &mut SymbolTable,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_class: Option<&str>,
) -> Result<(), SoplangError> {
    match expr {
        Expr::Literal(_) => {}
        Expr::Identifier(name) => {
            if name == "nafta" && current_class.is_none() {
                return Err(type_error_ex(
                    "'nafta' waxaa loo isticmaali karaa gudaha hawlgallada qaabka kaliya",
                    0,
                    0,
                    ErrorMeta::default()
                        .with_code(codes::E021_UNDECLARED)
                        .with_hint("Isticmaal 'nafta' gudaha hawl ku jirta qaabka."),
                ));
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            analyze_expr(left, sym, fn_sigs, current_class)?;
            analyze_expr(right, sym, fn_sigs, current_class)?;
        }
        Expr::UnaryOp { expr: e, .. } => analyze_expr(e, sym, fn_sigs, current_class)?,
        Expr::Call { name, args } => {
            for a in args {
                analyze_expr(a, sym, fn_sigs, current_class)?;
            }
            if let Some(sig) = fn_sigs.get(name) {
                if sig.params.len() != args.len() {
                    return Err(type_error_ex(
                        format!(
                            "hawl '{}' waxay u baahan tahay {} dood, laakiin {} ayaa la bixiyay",
                            name,
                            sig.params.len(),
                            args.len()
                        ),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint("Hubi tirada doodaha markaad waceyso hawsha."),
                    ));
                }
                for (idx, (arg, expected)) in args.iter().zip(sig.params.iter()).enumerate() {
                    if *expected == TypeAnnotation::Dynamic {
                        continue;
                    }
                    if let Some(found) = infer_expr_type(arg, sym, fn_sigs) {
                        if !is_type_compatible(*expected, found) {
                            return Err(type_error_ex(
                                format!(
                                    "doodda {} ee '{}' waa '{}', laakiin '{}' ayaa la helay",
                                    idx + 1,
                                    name,
                                    ty_str(*expected),
                                    ty_str(found)
                                ),
                                0,
                                0,
                                ErrorMeta::default()
                                    .with_code(codes::E022_TYPE_MISMATCH)
                                    .with_hint("La jaanqaad noocyada doodaha ee qeexidda hawsha."),
                            ));
                        }
                    }
                }
            }
        }
        Expr::MethodCall { obj, args, .. } => {
            analyze_expr(obj, sym, fn_sigs, current_class)?;
            for a in args {
                analyze_expr(a, sym, fn_sigs, current_class)?;
            }
        }
        Expr::Index { obj, idx } => {
            analyze_expr(obj, sym, fn_sigs, current_class)?;
            analyze_expr(idx, sym, fn_sigs, current_class)?;
        }
        Expr::Property { obj, .. } => analyze_expr(obj, sym, fn_sigs, current_class)?,
        Expr::List(exprs) => {
            for e in exprs {
                analyze_expr(e, sym, fn_sigs, current_class)?;
            }
        }
        Expr::Object(pairs) => {
            for (_, e) in pairs {
                analyze_expr(e, sym, fn_sigs, current_class)?;
            }
        }
        Expr::New { class_name, args } => {
            if !sym.classes.contains_key(class_name) {
                return Err(type_error_ex(
                    format!("Qaab '{}' lama aqoonsan", class_name),
                    0,
                    0,
                    ErrorMeta::default()
                        .with_code(codes::E021_UNDECLARED)
                        .with_hint("Hubi in qaabka uu ka horreeyo isticmaalkiisa."),
                ));
            }
            for a in args {
                analyze_expr(a, sym, fn_sigs, current_class)?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Simple type inference for expressions (used for VarDecl/Assign checks)
// ---------------------------------------------------------------------------

fn infer_expr_type(
    expr: &Expr,
    sym: &SymbolTable,
    fn_sigs: &HashMap<String, FunctionSig>,
) -> Option<TypeAnnotation> {
    match expr {
        Expr::Literal(l) => Some(match l {
            Literal::Int(_) => TypeAnnotation::Abn,
            Literal::Float(_) => TypeAnnotation::Jajab,
            Literal::Str(_) => TypeAnnotation::Qoraal,
            Literal::Bool(_) => TypeAnnotation::Bool,
            Literal::Null => TypeAnnotation::Dynamic,
        }),
        Expr::Identifier(name) => resolve_name(sym, name, None).map(|v| v.type_ann),
        Expr::List(_) => Some(TypeAnnotation::Teed),
        Expr::Object(_) => Some(TypeAnnotation::Walax),
        Expr::UnaryOp { expr: inner, .. } => infer_expr_type(inner, sym, fn_sigs),
        Expr::BinaryOp { op, left, right } => {
            let lt = infer_expr_type(left, sym, fn_sigs)?;
            let rt = infer_expr_type(right, sym, fn_sigs)?;
            match op.as_str() {
                "+" | "-" | "*" | "%" => {
                    // Numeric ops: if any side is Jajab → Jajab, else Abn
                    if matches!(lt, TypeAnnotation::Jajab) || matches!(rt, TypeAnnotation::Jajab) {
                        Some(TypeAnnotation::Jajab)
                    } else if matches!(lt, TypeAnnotation::Abn) && matches!(rt, TypeAnnotation::Abn) {
                        Some(TypeAnnotation::Abn)
                    } else {
                        None
                    }
                }
                "/" => {
                    // Soplang division returns decimal even for integer operands.
                    if matches!(lt, TypeAnnotation::Abn | TypeAnnotation::Jajab)
                        && matches!(rt, TypeAnnotation::Abn | TypeAnnotation::Jajab)
                    {
                        Some(TypeAnnotation::Jajab)
                    } else {
                        None
                    }
                }
                "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" => Some(TypeAnnotation::Bool),
                _ => None,
            }
        }
        Expr::Call { name, .. } => fn_sigs.get(name).and_then(|s| {
            if s.ret == TypeAnnotation::Dynamic {
                None
            } else {
                Some(s.ret)
            }
        }),
        Expr::MethodCall { .. } | Expr::Index { .. } | Expr::Property { .. } | Expr::New { .. } => {
            // For now, calls/index/property/new are treated as dynamic
            None
        }
    }
}

fn is_type_compatible(target: TypeAnnotation, value: TypeAnnotation) -> bool {
    use TypeAnnotation::*;
    match target {
        Dynamic => true,
        // Interpreter allows assigning integral decimals into abn at runtime.
        // Keep semantic check permissive here to avoid false negatives.
        Abn => matches!(value, Abn | Jajab),
        Jajab => matches!(value, Abn | Jajab),
        Qoraal => matches!(value, Qoraal),
        Bool => matches!(value, Bool),
        Teed => matches!(value, Teed),
        Walax => matches!(value, Walax),
    }
}

fn type_mismatch_error(
    name: &str,
    expected: TypeAnnotation,
    found: TypeAnnotation,
    line: usize,
    col: usize,
) -> SoplangError {
    let msg = format!(
        "'{}' waa {} laakin qiimaheeda '{}' ma ahan {}",
        name,
        ty_str(expected),
        ty_str(found),
        ty_str(expected)
    );
    type_error_ex(
        msg,
        line,
        col,
        ErrorMeta::default()
            .with_code(codes::E022_TYPE_MISMATCH)
            .with_hint("Hubi in nooca qiimaha uu la mid yahay nooca lagu qeexay doorsamaha."),
    )
}

fn ty_str(a: TypeAnnotation) -> &'static str {
    match a {
        TypeAnnotation::Abn => "abn",
        TypeAnnotation::Jajab => "jajab",
        TypeAnnotation::Qoraal => "qoraal",
        TypeAnnotation::Bool => "bool",
        TypeAnnotation::Teed => "teed",
        TypeAnnotation::Walax => "walax",
        TypeAnnotation::Dynamic => "dynamic",
    }
}

fn analyze_assign_target(
    target: &Expr,
    sym: &mut SymbolTable,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_class: Option<&str>,
) -> Result<(), SoplangError> {
    match target {
        Expr::Index { obj, idx } => {
            analyze_expr(obj, sym, fn_sigs, current_class)?;
            analyze_expr(idx, sym, fn_sigs, current_class)?;
        }
        Expr::Property { obj, .. } => analyze_expr(obj, sym, fn_sigs, current_class)?,
        _ => {}
    }
    Ok(())
}

fn collect_function_sigs(stmts: &[Stmt]) -> HashMap<String, FunctionSig> {
    let mut out = HashMap::new();
    collect_function_sigs_rec(stmts, &mut out, None);
    out
}

fn collect_function_sigs_rec(
    stmts: &[Stmt],
    out: &mut HashMap<String, FunctionSig>,
    current_class: Option<&str>,
) {
    for s in stmts {
        match s {
            Stmt::FuncDef {
                name,
                params,
                return_ann,
                ..
            } => {
                let key = if let Some(class) = current_class {
                    mangle_method(class, name)
                } else {
                    name.clone()
                };
                out.insert(
                    key,
                    FunctionSig {
                        params: params.iter().map(|p| p.type_ann).collect(),
                        ret: *return_ann,
                    },
                );
            }
            Stmt::ClassDef { name, body, .. } => {
                collect_function_sigs_rec(body, out, Some(name));
            }
            Stmt::Block(body) => collect_function_sigs_rec(body, out, current_class),
            Stmt::If {
                then_body,
                elseifs,
                else_body,
                ..
            } => {
                collect_function_sigs_rec(then_body, out, current_class);
                for (_, b) in elseifs {
                    collect_function_sigs_rec(b, out, current_class);
                }
                if let Some(b) = else_body {
                    collect_function_sigs_rec(b, out, current_class);
                }
            }
            Stmt::Switch { cases, default, .. } => {
                for (_, b) in cases {
                    collect_function_sigs_rec(b, out, current_class);
                }
                if let Some(b) = default {
                    collect_function_sigs_rec(b, out, current_class);
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                collect_function_sigs_rec(body, out, current_class);
            }
            Stmt::TryCatch {
                try_body,
                catch_body,
                ..
            } => {
                collect_function_sigs_rec(try_body, out, current_class);
                collect_function_sigs_rec(catch_body, out, current_class);
            }
            _ => {}
        }
    }
}

/// Resolve a name to its VarInfo. If func_scope is given (for function body), check it first.
pub fn resolve_name(
    sym: &SymbolTable,
    name: &str,
    func_scope: Option<&HashMap<String, VarInfo>>,
) -> Option<VarInfo> {
    if let Some(scope) = func_scope {
        if let Some(info) = scope.get(name) {
            return Some(info.clone());
        }
    }
    for scope in sym.scopes.iter().rev() {
        if let Some(info) = scope.vars.get(name) {
            return Some(info.clone());
        }
    }
    None
}
