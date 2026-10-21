//! Semantic analysis: name resolution, variable slots, function/class metadata.
//! Phase 1 of COMPILER_PLAN. Consumes AST, produces SymbolTable for HIR lowering.

use std::collections::{HashMap, HashSet};

use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};
use crate::frontend::ast::{Expr, InterpolationPart, Literal, Param, Stmt, TypeAnnotation};

/// Symbol table built by semantic analysis. Used by HIR lowering.
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Stack of scopes (inner to outer). Scope 0 = global.
    pub scopes: Vec<Scope>,
    /// Per-function metadata (params, local count, captures).
    pub functions: Vec<FunctionMeta>,
    /// Class name -> metadata (methods, parent).
    pub classes: HashMap<String, ClassMeta>,
    /// Anonymous expression address -> deterministic compiler-internal name.
    pub anonymous_functions: HashMap<usize, String>,
    /// Globals carried in from a REPL session. Top-level uses load/store them
    /// instead of treating their old slots as locals of the new snippet.
    pub session_globals: HashSet<String>,
    unavailable_parameters: Vec<(usize, HashSet<String>)>,
    pub class_spans: HashMap<String, crate::error::SourceSpan>,
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub vars: HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub slot: usize,
    pub type_ann: TypeAnnotation,
    pub is_const: bool,
    pub is_captured: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionMeta {
    pub name: String,
    pub param_slots: Vec<usize>,
    pub param_types: Vec<TypeAnnotation>,
    pub param_names: Vec<String>,
    pub required_params: usize,
    pub variadic: bool,
    pub keyword_rest: bool,
    pub return_ann: TypeAnnotation,
    pub local_count: usize,
    pub captures: Vec<String>,
    /// Variable name -> VarInfo for this function's scope (params + locals).
    pub scope_vars: HashMap<String, VarInfo>,
    pub class_name: Option<String>,
    pub is_method: bool,
}

#[derive(Debug, Clone)]
pub struct ClassMeta {
    pub name: String,
    pub parent: Option<String>,
    /// Method name -> mangled function name (e.g. "salaam" -> "Bisad::salaam").
    pub methods: HashMap<String, String>,
}

/// Mangle a class method name for global registration.
pub fn mangle_method(class: &str, method: &str) -> String {
    format!("{}::{}", class, method)
}

/// Method on an ancestor, starting at the parent of `class`. Skips `class` itself.
pub fn resolve_parent_method(
    sym: &SymbolTable,
    class: &str,
    method: &str,
) -> Result<String, SoplangError> {
    let parent = sym
        .classes
        .get(class)
        .and_then(|meta| meta.parent.clone())
        .ok_or_else(|| {
            type_error_ex(
                "'waalid' wuxuu u baahan yahay qaab ka dhaxla qaab kale",
                0,
                0,
                ErrorMeta::default()
                    .with_code(codes::E021_UNDECLARED)
                    .with_hint("Ku dar 'dhaxal' qaabka, kadibna wac waalid.hawl()."),
            )
        })?;
    let mut current = parent;
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err(type_error_ex(
                format!("Qaab '{}' wuxuu ka dhaxlaa meel-wareeg", class),
                0,
                0,
                ErrorMeta::default()
                    .with_code(codes::E021_UNDECLARED)
                    .with_hint("Qaabku ma dhaxli karo naftiisa ama qaab uu ka dhaxlo."),
            ));
        }
        let Some(meta) = sym.classes.get(&current) else {
            return Err(type_error_ex(
                format!(
                    "Qaab '{}' wuxuu ka dhaxlay '{}' laakiin qaabkaas ma jiro",
                    class, current
                ),
                0,
                0,
                ErrorMeta::default()
                    .with_code(codes::E021_UNDECLARED)
                    .with_hint("Hubi in qaabka waalidka uu ka horreeyo qaabka ilmaha."),
            ));
        };
        if let Some(mangled) = meta.methods.get(method).or_else(|| match method {
            "dhis" => meta.methods.get("dhaw"),
            "dhaw" => meta.methods.get("dhis"),
            _ => None,
        }) {
            return Ok(mangled.clone());
        }
        match meta.parent.clone() {
            Some(next) => current = next,
            None => {
                return Err(type_error_ex(
                    format!("Hawl '{}' lama helin waalidka qaabka '{}'", method, class),
                    0,
                    0,
                    ErrorMeta::default()
                        .with_code(codes::E021_UNDECLARED)
                        .with_hint("Hubi in hawlka uu ku jiro qaabka waalidka."),
                ));
            }
        }
    }
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
    names: Vec<String>,
    required_params: usize,
    variadic: bool,
    keyword_rest: bool,
    ret: TypeAnnotation,
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
    validate_control_flow(stmts, false, 0)?;
    let fn_sigs = collect_function_sigs(stmts);
    analyze_block(stmts, sym, &opts, &fn_sigs, None, None, None)?;
    validate_classes(sym)?;
    Ok(())
}

fn validate_control_flow(
    stmts: &[Stmt],
    in_function: bool,
    loop_depth: usize,
) -> Result<(), SoplangError> {
    for stmt in stmts {
        match stmt {
            Stmt::Located { span, stmt } => {
                validate_control_flow(std::slice::from_ref(stmt.as_ref()), in_function, loop_depth)
                    .map_err(|e| span.attach(e))?
            }
            Stmt::Return(value) => {
                if !in_function {
                    return Err(control_flow_error(
                        "'celi' waxaa lagu isticmaali karaa hawl gudaheeda kaliya",
                        "Geli 'celi' gudaha hawl.",
                    ));
                }
                if let Some(value) = value {
                    validate_control_expr(value)?;
                }
            }
            Stmt::Break => {
                if loop_depth == 0 {
                    return Err(control_flow_error(
                        "'jooji' waxaa lagu isticmaali karaa wareeg gudaheeda kaliya",
                        "Geli 'jooji' gudaha inta ama kuceli.",
                    ));
                }
            }
            Stmt::Continue => {
                if loop_depth == 0 {
                    return Err(control_flow_error(
                        "'soco' waxaa lagu isticmaali karaa wareeg gudaheeda kaliya",
                        "Geli 'soco' gudaha inta ama kuceli.",
                    ));
                }
            }
            Stmt::FuncDef { params, body, .. } => {
                validate_parameter_defaults(params)?;
                validate_control_flow(body, true, 0)?;
            }
            Stmt::ClassDef { body, .. } => validate_control_flow(body, false, 0)?,
            Stmt::If {
                cond,
                then_body,
                elseifs,
                else_body,
            } => {
                validate_control_expr(cond)?;
                validate_control_flow(then_body, in_function, loop_depth)?;
                for (cond, body) in elseifs {
                    validate_control_expr(cond)?;
                    validate_control_flow(body, in_function, loop_depth)?;
                }
                if let Some(body) = else_body {
                    validate_control_flow(body, in_function, loop_depth)?;
                }
            }
            Stmt::Switch {
                expr,
                cases,
                default,
            } => {
                validate_control_expr(expr)?;
                for (case, body) in cases {
                    validate_control_expr(case)?;
                    validate_control_flow(body, in_function, loop_depth)?;
                }
                if let Some(body) = default {
                    validate_control_flow(body, in_function, loop_depth)?;
                }
            }
            Stmt::For {
                start,
                end,
                step,
                body,
                ..
            } => {
                validate_control_expr(start)?;
                validate_control_expr(end)?;
                if let Some(step) = step {
                    validate_control_expr(step)?;
                }
                validate_control_flow(body, in_function, loop_depth + 1)?;
            }
            Stmt::ForEach { iterable, body, .. } => {
                validate_control_expr(iterable)?;
                validate_control_flow(body, in_function, loop_depth + 1)?;
            }
            Stmt::While { cond, body } => {
                validate_control_expr(cond)?;
                validate_control_flow(body, in_function, loop_depth + 1)?;
            }
            Stmt::TryCatch {
                try_body,
                catch_body,
                finally_body,
                ..
            } => {
                validate_control_flow(try_body, in_function, loop_depth)?;
                validate_control_flow(catch_body, in_function, loop_depth)?;
                validate_control_flow(finally_body, in_function, loop_depth)?;
            }
            Stmt::Block(body) => validate_control_flow(body, in_function, loop_depth)?,
            Stmt::VarDecl { value, .. } | Stmt::Expr(value) | Stmt::Throw(value) => validate_control_expr(value)?,
            Stmt::Assign { target, value, .. } => {
                validate_control_expr(target)?;
                validate_control_expr(value)?;
            }
            Stmt::Import(_) => {}
        }
    }
    Ok(())
}

fn validate_control_expr(expr: &Expr) -> Result<(), SoplangError> {
    match expr {
        Expr::Located { span, expr } => validate_control_expr(expr).map_err(|e| span.attach(e)),
        Expr::AnonymousFunc { params, body, .. } => {
            validate_parameter_defaults(params)?;
            validate_control_flow(body, true, 0)
        }
        Expr::BinaryOp { left, right, .. }
        | Expr::Index {
            obj: left,
            idx: right,
        } => {
            validate_control_expr(left)?;
            validate_control_expr(right)
        }
        Expr::NamedArgument { value: expr, .. }
        | Expr::UnaryOp { expr, .. }
        | Expr::Property { obj: expr, .. } => validate_control_expr(expr),
        Expr::Interpolated { parts } => {
            for part in parts {
                if let InterpolationPart::Value { expression, .. } = part {
                    validate_control_expr(expression)?;
                }
            }
            Ok(())
        }
        Expr::Call { args, .. } | Expr::New { args, .. } | Expr::List(args) => {
            for arg in args {
                validate_control_expr(arg)?;
            }
            Ok(())
        }
        Expr::MethodCall { obj, args, .. } | Expr::Invoke { callee: obj, args } => {
            validate_control_expr(obj)?;
            for arg in args {
                validate_control_expr(arg)?;
            }
            Ok(())
        }
        Expr::Object(pairs) => {
            for (_, value) in pairs {
                validate_control_expr(value)?;
            }
            Ok(())
        }
        Expr::Literal(_) | Expr::Identifier(_) => Ok(()),
    }
}

fn control_flow_error(message: &str, hint: &str) -> SoplangError {
    type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default()
            .with_code(codes::E022_TYPE_MISMATCH)
            .with_hint(hint),
    )
}

fn validate_classes(sym: &SymbolTable) -> Result<(), SoplangError> {
    let mut classes: Vec<_> = sym.classes.values().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    for meta in classes {
        let mut seen = HashSet::new();
        let mut current = Some(meta.name.clone());
        while let Some(name) = current {
            if !seen.insert(name.clone()) {
                return Err(attach_class_error(
                    sym,
                    &meta.name,
                    type_error_ex(
                        format!("Qaab '{}' wuxuu ka dhaxlaa meel-wareeg", meta.name),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Qaabku ma dhaxli karo naftiisa ama qaab uu ka dhaxlo."),
                    ),
                ));
            }
            let Some(node) = sym.classes.get(&name) else {
                break;
            };
            if let Some(ref parent) = node.parent {
                if !sym.classes.contains_key(parent) {
                    return Err(attach_class_error(
                        sym,
                        &meta.name,
                        type_error_ex(
                            format!(
                                "Qaab '{}' wuxuu ka dhaxlay '{}' laakiin qaabkaas ma jiro",
                                node.name, parent
                            ),
                            0,
                            0,
                            ErrorMeta::default()
                                .with_code(codes::E021_UNDECLARED)
                                .with_hint("Hubi in qaabka waalidka uu ka horreeyo qaabka ilmaha."),
                        ),
                    ));
                }
                current = Some(parent.clone());
            } else {
                current = None;
            }
        }
    }
    Ok(())
}

fn attach_class_error(sym: &SymbolTable, name: &str, error: SoplangError) -> SoplangError {
    match sym.class_spans.get(name) {
        Some(span) => span.attach(error),
        None => error,
    }
}

fn analyze_block(
    stmts: &[Stmt],
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_return: Option<&TypeAnnotation>,
    current_class: Option<&str>,
    enclosing_fn: Option<&str>,
) -> Result<(), SoplangError> {
    for s in stmts {
        analyze_stmt(
            s,
            sym,
            opts,
            fn_sigs,
            current_return,
            current_class,
            enclosing_fn,
        )?;
    }
    Ok(())
}

fn analyze_stmt(
    stmt: &Stmt,
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_return: Option<&TypeAnnotation>,
    current_class: Option<&str>,
    enclosing_fn: Option<&str>,
) -> Result<(), SoplangError> {
    match stmt {
        Stmt::Located { span, stmt } => {
            if let Stmt::ClassDef { name, .. } = stmt.unlocated() {
                sym.class_spans.insert(name.clone(), span.clone());
            }
            analyze_stmt(
                stmt,
                sym,
                opts,
                fn_sigs,
                current_return,
                current_class,
                enclosing_fn,
            )
            .map_err(|e| span.attach(e))?;
        }
        Stmt::VarDecl {
            name,
            type_ann,
            is_const,
            value,
            line,
            col,
        } => {
            if opts.strict && type_ann == &TypeAnnotation::Dynamic {
                return Err(type_error_ex(
                    format!("Habka adag wuxuu u baahan yahay nooc cad oo loo qoro '{}'.", name),
                    *line,
                    *col,
                    ErrorMeta::default()
                        .with_code(codes::E022_TYPE_MISMATCH)
                        .with_hint("Habka adag: isticmaal nooc cad sida abn, jajab, qoraal, bool, teed, ama walax."),
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
                        type_ann: type_ann.clone(),
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
                        type_ann: type_ann.clone(),
                        is_const: *is_const,
                        is_captured: false,
                    },
                );
            }
            // Phase 2: basic static type check for initializer when we can infer a type.
            ensure_known_class(sym, type_ann)?;
            if type_ann != &TypeAnnotation::Dynamic {
                if let Some(expr_ty) = infer_expr_type(value, sym, fn_sigs) {
                    if !is_type_compatible(sym, type_ann, &expr_ty) {
                        return Err(type_mismatch_error(name, type_ann, &expr_ty, *line, *col));
                    }
                }
            }
            analyze_expr(value, sym, opts, fn_sigs, current_class, enclosing_fn)?;
        }
        Stmt::Assign {
            target,
            value,
            line,
            col,
        } => {
            if let Expr::Identifier(name) = target.unlocated() {
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
                if &var.type_ann != &TypeAnnotation::Dynamic {
                    if let Some(expr_ty) = infer_expr_type(value, sym, fn_sigs) {
                        if !is_type_compatible(sym, &var.type_ann, &expr_ty) {
                            return Err(type_mismatch_error(
                                name,
                                &var.type_ann,
                                &expr_ty,
                                *line,
                                *col,
                            ));
                        }
                    }
                }
            } else {
                // Index or Property: resolve subexpressions only
                analyze_assign_target(target, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            }
            analyze_expr(value, sym, opts, fn_sigs, current_class, enclosing_fn)?;
        }
        Stmt::FuncDef {
            name,
            params,
            return_ann,
            body,
        } => {
            let is_method = current_class.is_some() && enclosing_fn.is_none();
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
                if return_ann == &TypeAnnotation::Dynamic {
                    return Err(type_error_ex(
                        format!("hawl '{}' ma leh nooc celi cad", name),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint(
                                "Habka adag: qor nooca qiimaha la celinayo sida ') : abn {' ama ') : qoraal {'.",
                            ),
                    ));
                }
                if params.iter().any(|p| p.type_ann == TypeAnnotation::Dynamic) {
                    return Err(type_error_ex(
                        format!("Habka adag wuxuu u baahan yahay in dood kasta oo hawsha '{}' leedahay loo qoro nooc.", name),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint(
                            "Habka adag: qor nooca dood kasta, tusaale: hawl f(abn x, qoraal y).",
                        ),
                    ));
                }
            }
            let fn_name = if is_method {
                mangle_method(current_class.unwrap(), name)
            } else if let Some(parent) = enclosing_fn {
                format!("{parent}::{name}")
            } else {
                name.clone()
            };
            let class_name = if is_method {
                current_class.map(str::to_string)
            } else {
                None
            };
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
                        type_ann: p.type_ann.clone(),
                        is_const: false,
                        is_captured: false,
                    },
                );
            }
            let fns_before = sym.functions.len();
            sym.scopes.push(func_scope);
            ensure_known_class(sym, return_ann)?;
            for p in params {
                ensure_known_class(sym, &p.type_ann)?;
            }
            analyze_parameter_defaults(params, sym, opts, fn_sigs, current_class, Some(&fn_name))?;
            analyze_block(
                body,
                sym,
                opts,
                fn_sigs,
                Some(return_ann),
                current_class,
                Some(fn_name.as_str()),
            )?;
            let mut captures = Vec::new();
            if enclosing_fn.is_some() {
                captures = collect_captures(body, sym);
                for param in params {
                    if let Some(default) = &param.default {
                        note_free_expr(default, sym, &mut captures);
                    }
                }
                let nested: Vec<Vec<String>> = sym.functions[fns_before..]
                    .iter()
                    .map(|nf| nf.captures.clone())
                    .collect();
                for caps in &nested {
                    for c in caps {
                        if !sym.scopes.last().unwrap().vars.contains_key(c)
                            && capture_from_enclosing(sym, c)
                        {
                            push_capture(&mut captures, c);
                        }
                    }
                }
                let missing: Vec<String> = captures
                    .iter()
                    .filter(|c| !sym.scopes.last().unwrap().vars.contains_key(c.as_str()))
                    .cloned()
                    .collect();
                for c in missing {
                    let scope = sym.scopes.last_mut().unwrap();
                    let slot = scope.vars.len();
                    scope.vars.insert(
                        c,
                        VarInfo {
                            slot,
                            type_ann: TypeAnnotation::Dynamic,
                            is_const: false,
                            is_captured: true,
                        },
                    );
                }
            }
            let scope = sym.scopes.pop().unwrap();
            let local_count = scope.vars.len();
            sym.functions.push(FunctionMeta {
                name: fn_name,
                param_slots,
                param_types: params.iter().map(|p| p.type_ann.clone()).collect(),
                param_names: params.iter().map(|p| p.name.clone()).collect(),
                required_params: required_parameters(params),
                variadic: params.iter().any(|param| param.variadic),
                keyword_rest: params.last().is_some_and(|param| param.keyword_rest),
                return_ann: return_ann.clone(),
                local_count,
                captures,
                scope_vars: scope.vars,
                class_name,
                is_method,
            });
            if enclosing_fn.is_some() {
                let scope = sym.scopes.last_mut().unwrap();
                if !scope.vars.contains_key(name) {
                    let slot = scope.vars.len();
                    scope.vars.insert(
                        name.clone(),
                        VarInfo {
                            slot,
                            type_ann: TypeAnnotation::Dynamic,
                            is_const: false,
                            is_captured: false,
                        },
                    );
                }
            }
        }
        Stmt::ClassDef { name, parent, body } => {
            if name == "baytyo" {
                return Err(type_error_ex(
                    "Magaca 'baytyo' waxaa loo adeegsadaa nooc hore u jiray.",
                    0,
                    0,
                    ErrorMeta::default().with_code(codes::E020_REDECLARED),
                ));
            }
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
                if let Stmt::FuncDef { name: mname, .. } = s.unlocated() {
                    if methods.contains_key(mname) {
                        return Err(type_error_ex(
                            format!("Hawl '{}' ayaa laba jeer ku jirta qaabka '{}'", mname, name),
                            0,
                            0,
                            ErrorMeta::default()
                                .with_code(codes::E020_REDECLARED)
                                .with_hint(
                                    "Magacyada hawlgallada qaabka waa in ay noqdaan kuwo gaar ah.",
                                ),
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
            analyze_block(body, sym, opts, fn_sigs, current_return, Some(name), None)?;
        }
        Stmt::If {
            cond,
            then_body,
            elseifs,
            else_body,
        } => {
            analyze_expr(cond, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            analyze_block(
                then_body,
                sym,
                opts,
                fn_sigs,
                current_return,
                current_class,
                enclosing_fn,
            )?;
            for (c, b) in elseifs {
                analyze_expr(c, sym, opts, fn_sigs, current_class, enclosing_fn)?;
                analyze_block(
                    b,
                    sym,
                    opts,
                    fn_sigs,
                    current_return,
                    current_class,
                    enclosing_fn,
                )?;
            }
            if let Some(eb) = else_body {
                analyze_block(
                    eb,
                    sym,
                    opts,
                    fn_sigs,
                    current_return,
                    current_class,
                    enclosing_fn,
                )?;
            }
        }
        Stmt::Switch {
            expr,
            cases,
            default,
        } => {
            analyze_expr(expr, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            for (v, body) in cases {
                analyze_expr(v, sym, opts, fn_sigs, current_class, enclosing_fn)?;
                analyze_block(
                    body,
                    sym,
                    opts,
                    fn_sigs,
                    current_return,
                    current_class,
                    enclosing_fn,
                )?;
            }
            if let Some(d) = default {
                analyze_block(
                    d,
                    sym,
                    opts,
                    fn_sigs,
                    current_return,
                    current_class,
                    enclosing_fn,
                )?;
            }
        }
        Stmt::For {
            var,
            start,
            end,
            step,
            body,
        } => {
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
            analyze_expr(start, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            analyze_expr(end, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            if let Some(s) = step {
                analyze_expr(s, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            }
            analyze_block(
                body,
                sym,
                opts,
                fn_sigs,
                current_return,
                current_class,
                enclosing_fn,
            )?;
        }
        Stmt::ForEach {
            vars,
            iterable,
            body,
        } => {
            analyze_expr(iterable, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            let scope = sym.scopes.last_mut().unwrap();
            for name in vars {
                if let Some(info) = scope.vars.get(name) {
                    if info.is_const {
                        return Err(type_error_ex(
                            format!("Ma bedeli kartid qiimaha doorsamaha madoor '{}'", name),
                            0,
                            0,
                            ErrorMeta::default().with_code(codes::E020_REDECLARED),
                        ));
                    }
                } else {
                    let slot = scope.vars.len();
                    scope.vars.insert(
                        name.clone(),
                        VarInfo {
                            slot,
                            type_ann: TypeAnnotation::Dynamic,
                            is_const: false,
                            is_captured: false,
                        },
                    );
                }
            }
            analyze_block(
                body,
                sym,
                opts,
                fn_sigs,
                current_return,
                current_class,
                enclosing_fn,
            )?;
        }
        Stmt::While { cond, body } => {
            analyze_expr(cond, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            analyze_block(
                body,
                sym,
                opts,
                fn_sigs,
                current_return,
                current_class,
                enclosing_fn,
            )?;
        }
        Stmt::Return(Some(e)) => {
            analyze_expr(e, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            if let Some(ret_ann) = current_return {
                if ret_ann != &TypeAnnotation::Dynamic {
                    if let Some(expr_ty) = infer_expr_type(e, sym, fn_sigs) {
                        if !is_type_compatible(sym, ret_ann, &expr_ty) {
                            return Err(type_error_ex(
                                format!(
                                    "nooca celi '{}' ma waafaqsana nooca hawsha '{}'",
                                    ty_str(&expr_ty),
                                    ty_str(ret_ann)
                                ),
                                0,
                                0,
                                ErrorMeta::default()
                                    .with_code(codes::E022_TYPE_MISMATCH)
                                    .with_hint(
                                        "Hubi in 'celi' uu la mid yahay nooca lagu qoray hawsha.",
                                    ),
                            ));
                        }
                    }
                }
            }
        }
        Stmt::Return(None) => {
            if let Some(ret_ann) = current_return {
                if !is_type_compatible(sym, ret_ann, &TypeAnnotation::Null) {
                    return Err(type_error_ex(
                        format!(
                            "hawshu waxay rabtaa '{}' laakiin celi qiimo ma leh",
                            ty_str(ret_ann)
                        ),
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
        Stmt::TryCatch {
            try_body,
            err_var,
            err_details,
            catch_body,
            finally_body,
            ..
        } => {
            analyze_block(
                try_body,
                sym,
                opts,
                fn_sigs,
                current_return,
                current_class,
                enclosing_fn,
            )?;
            sym.scopes.push(Scope::default());
            for err_var in err_var.iter().chain(err_details.iter()) {
                let scope = sym.scopes.last_mut().unwrap();
                scope.vars.insert(
                    err_var.clone(),
                    VarInfo {
                        slot: 0,
                        type_ann: TypeAnnotation::Dynamic,
                        is_const: false,
                        is_captured: false,
                    },
                );
            }
            analyze_block(
                catch_body,
                sym,
                opts,
                fn_sigs,
                current_return,
                current_class,
                enclosing_fn,
            )?;
            sym.scopes.pop();
            analyze_block(finally_body, sym, opts, fn_sigs, current_return, current_class, enclosing_fn)?;
        }
        Stmt::Import(_) => {}
        Stmt::Block(stmts) => analyze_block(
            stmts,
            sym,
            opts,
            fn_sigs,
            current_return,
            current_class,
            enclosing_fn,
        )?,
        Stmt::Expr(e) | Stmt::Throw(e) => analyze_expr(e, sym, opts, fn_sigs, current_class, enclosing_fn)?,
    }
    Ok(())
}

fn required_parameters(params: &[Param]) -> usize {
    params
        .iter()
        .take_while(|param| param.default.is_none() && !param.variadic && !param.keyword_rest)
        .count()
}

fn validate_parameter_defaults(params: &[Param]) -> Result<(), SoplangError> {
    for param in params {
        if let Some(default) = &param.default {
            validate_control_expr(default)?;
        }
    }
    Ok(())
}

fn check_default_dependency(name: &str, sym: &SymbolTable) -> Result<(), SoplangError> {
    let resolved = sym
        .scopes
        .iter()
        .rposition(|scope| scope.vars.contains_key(name));
    if sym
        .unavailable_parameters
        .iter()
        .any(|(scope, names)| resolved == Some(*scope) && names.contains(name))
    {
        return Err(type_error_ex(
            format!(
                "Qiimaha hordhaca ahi ma adeegsan karo doodda '{}' ka hor inta aan la diyaarin.",
                name
            ),
            0,
            0,
            ErrorMeta::default().with_code(codes::E022_TYPE_MISMATCH),
        ));
    }
    Ok(())
}

fn analyze_parameter_defaults(
    params: &[Param],
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_class: Option<&str>,
    enclosing_fn: Option<&str>,
) -> Result<(), SoplangError> {
    for (index, param) in params.iter().enumerate() {
        let Some(default) = &param.default else {
            continue;
        };
        sym.unavailable_parameters.push((
            sym.scopes.len() - 1,
            params[index..]
                .iter()
                .map(|param| param.name.clone())
                .collect(),
        ));
        let result = analyze_expr(default, sym, opts, fn_sigs, current_class, enclosing_fn);
        sym.unavailable_parameters.pop();
        result?;
        if param.type_ann != TypeAnnotation::Dynamic {
            if let Some(found) = infer_expr_type(default, sym, fn_sigs) {
                if !is_type_compatible(sym, &param.type_ann, &found) {
                    let error = type_mismatch_error(&param.name, &param.type_ann, &found, 0, 0);
                    return Err(match default {
                        Expr::Located { span, .. } => span.attach(error),
                        _ => error,
                    });
                }
            }
        }
    }
    Ok(())
}

fn analyze_expr(
    expr: &Expr,
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_class: Option<&str>,
    enclosing_fn: Option<&str>,
) -> Result<(), SoplangError> {
    match expr {
        Expr::Located { span, expr } => {
            analyze_expr(expr, sym, opts, fn_sigs, current_class, enclosing_fn)
                .map_err(|e| span.attach(e))?
        }
        Expr::Literal(_) => {}
        Expr::Identifier(name) => {
            check_default_dependency(name, sym)?;
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
            if name == "waalid" {
                return Err(type_error_ex(
                    "'waalid' waa wicitaan hawl, sida waalid.cod()",
                    0,
                    0,
                    ErrorMeta::default()
                        .with_code(codes::E021_UNDECLARED)
                        .with_hint("Wac hawl ka mid ah qaabka waalidka."),
                ));
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            analyze_expr(left, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            analyze_expr(right, sym, opts, fn_sigs, current_class, enclosing_fn)?;
        }
        Expr::NamedArgument { value: e, .. } | Expr::UnaryOp { expr: e, .. } => {
            analyze_expr(e, sym, opts, fn_sigs, current_class, enclosing_fn)?
        }
        Expr::Interpolated { parts } => {
            for part in parts {
                if let InterpolationPart::Value { expression, .. } = part {
                    analyze_expr(expression, sym, opts, fn_sigs, current_class, enclosing_fn)?;
                }
            }
        }
        Expr::Invoke { callee, args } => {
            analyze_expr(callee, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            for arg in args { analyze_expr(arg, sym, opts, fn_sigs, current_class, enclosing_fn)?; }
        }
        Expr::Call { name, args } => {
            check_default_dependency(name, sym)?;
            for a in args {
                analyze_expr(a, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            }
            if let Some(sig) = fn_sigs
                .get(name)
                .filter(|_| resolve_name(sym, name, None).is_none())
            {
                if has_named_arguments(args) {
                    validate_named_arguments(args, sig, sym, fn_sigs)?;
                    return Ok(());
                }
                if args.len() < sig.required_params
                    || (!sig.variadic
                        && args.len() > sig.params.len() - usize::from(sig.keyword_rest))
                {
                    return Err(type_error_ex(
                        format!(
                            "hawl '{}' waxay u baahan tahay {} dood, laakiin {} ayaa la bixiyay",
                            name,
                            crate::error::argument_count(
                                sig.required_params,
                                if sig.variadic {
                                    usize::MAX
                                } else {
                                    sig.params.len() - usize::from(sig.keyword_rest)
                                }
                            ),
                            args.len()
                        ),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint("Hubi tirada doodaha markaad waceyso hawsha."),
                    ));
                }
                for (idx, (arg, expected)) in args
                    .iter()
                    .zip(sig.params.iter().take(
                        sig.params.len()
                            - usize::from(sig.variadic)
                            - usize::from(sig.keyword_rest),
                    ))
                    .enumerate()
                {
                    if expected == &TypeAnnotation::Dynamic {
                        continue;
                    }
                    if let Some(found) = infer_expr_type(arg, sym, fn_sigs) {
                        if !is_type_compatible(sym, expected, &found) {
                            return Err(type_error_ex(
                                format!(
                                    "doodda {} ee '{}' waa '{}', laakiin '{}' ayaa la helay",
                                    idx + 1,
                                    name,
                                    ty_str(expected),
                                    ty_str(&found)
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
        Expr::MethodCall { obj, method, args } => {
            if let Expr::Identifier(recv) = obj.unlocated() {
                if recv == "waalid" {
                    let class = current_class.ok_or_else(|| {
                        type_error_ex(
                            "'waalid' waxaa loo isticmaali karaa gudaha hawlgallada qaabka kaliya",
                            0,
                            0,
                            ErrorMeta::default()
                                .with_code(codes::E021_UNDECLARED)
                                .with_hint("Isticmaal 'waalid' gudaha hawl ku jirta qaabka."),
                        )
                    })?;
                    let mangled = resolve_parent_method(sym, class, method)?;
                    if let Some(fm) = sym.functions.iter().find(|f| f.name == mangled) {
                        if has_named_arguments(args) {
                            validate_named_arguments(
                                args,
                                &FunctionSig {
                                    params: fm.param_types.clone(),
                                    names: fm.param_names.clone(),
                                    required_params: fm.required_params,
                                    variadic: fm.variadic,
                                    keyword_rest: fm.keyword_rest,
                                    ret: fm.return_ann.clone(),
                                },
                                sym,
                                fn_sigs,
                            )?;
                        } else if args.len() < fm.required_params
                            || (!fm.variadic
                                && args.len() > fm.param_types.len() - usize::from(fm.keyword_rest))
                        {
                            return Err(type_error_ex(
                                format!(
                                    "hawl '{}' waxay u baahan tahay {} dood, laakiin {} ayaa la bixiyay",
                                    method,
                                    crate::error::argument_count(fm.required_params, if fm.variadic { usize::MAX } else { fm.param_types.len() - usize::from(fm.keyword_rest) }),
                                    args.len()
                                ),
                                0,
                                0,
                                ErrorMeta::default()
                                    .with_code(codes::E022_TYPE_MISMATCH)
                                    .with_hint("Hubi tirada doodaha markaad wacayso waalidka."),
                            ));
                        }
                    }
                    for a in args {
                        analyze_expr(a, sym, opts, fn_sigs, current_class, enclosing_fn)?;
                    }
                    return Ok(());
                }
            }
            analyze_expr(obj, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            for a in args {
                analyze_expr(a, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            }
        }
        Expr::Index { obj, idx } => {
            analyze_expr(obj, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            analyze_expr(idx, sym, opts, fn_sigs, current_class, enclosing_fn)?;
        }
        Expr::Property { obj, .. } => {
            analyze_expr(obj, sym, opts, fn_sigs, current_class, enclosing_fn)?
        }
        Expr::List(exprs) => {
            for e in exprs {
                analyze_expr(e, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            }
        }
        Expr::Object(pairs) => {
            for (_, e) in pairs {
                analyze_expr(e, sym, opts, fn_sigs, current_class, enclosing_fn)?;
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
                analyze_expr(a, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            }
            let mut owner = Some(class_name.as_str());
            let mut constructor = None;
            let mut seen = HashSet::new();
            while let Some(name) = owner {
                if !seen.insert(name.to_string()) {
                    break;
                }
                let Some(class) = sym.classes.get(name) else {
                    break;
                };
                if let Some(mangled) = class
                    .methods
                    .get("dhis")
                    .or_else(|| class.methods.get("dhaw"))
                {
                    constructor = fn_sigs.get(mangled);
                    break;
                }
                owner = class.parent.as_deref();
            }
            if let Some(sig) = constructor.filter(|_| has_named_arguments(args)) {
                validate_named_arguments(args, sig, sym, fn_sigs)?;
                return Ok(());
            }
            if let Some(sig) = constructor {
                if args.len() < sig.required_params
                    || (!sig.variadic
                        && args.len() > sig.params.len() - usize::from(sig.keyword_rest))
                {
                    return Err(type_error_ex(
                        format!(
                            "dhaw() ee qaabka '{}' wuxuu u baahan yahay {} dood, laakiin {} ayaa la bixiyay",
                            class_name,
                            crate::error::argument_count(sig.required_params, if sig.variadic { usize::MAX } else { sig.params.len() - usize::from(sig.keyword_rest) }),
                            args.len()
                        ),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E022_TYPE_MISMATCH)
                            .with_hint("Hubi tirada doodaha aad siisay cusub."),
                    ));
                }
                for (index, (argument, expected)) in args
                    .iter()
                    .zip(sig.params.iter().take(
                        sig.params.len()
                            - usize::from(sig.variadic)
                            - usize::from(sig.keyword_rest),
                    ))
                    .enumerate()
                {
                    if expected == &TypeAnnotation::Dynamic {
                        continue;
                    }
                    if let Some(found) = infer_expr_type(argument, sym, fn_sigs) {
                        if !is_type_compatible(sym, expected, &found) {
                            return Err(type_error_ex(
                                format!(
                                    "doodda {} ee dhaw() waa '{}', laakiin '{}' ayaa la helay",
                                    index + 1,
                                    ty_str(expected),
                                    ty_str(&found)
                                ),
                                0,
                                0,
                                ErrorMeta::default()
                                    .with_code(codes::E022_TYPE_MISMATCH)
                                    .with_hint("La jaanqaad noocyada doodaha ee dhaw()."),
                            ));
                        }
                    }
                }
            } else if !args.is_empty() {
                return Err(type_error_ex(
                    format!("Qaabka '{}' ma laha dhaw() qaata doodo", class_name),
                    0,
                    0,
                    ErrorMeta::default()
                        .with_code(codes::E022_TYPE_MISMATCH)
                        .with_hint("Ka saar doodaha ama ku dar dhaw() qaabka."),
                ));
            }
        }
        Expr::AnonymousFunc {
            params,
            return_ann,
            body,
        } => {
            let key = expr as *const Expr as usize;
            let name = format!("#anonymous{}", sym.anonymous_functions.len());
            sym.anonymous_functions.insert(key, name.clone());
            analyze_anonymous_function(
                &name,
                params,
                return_ann,
                body,
                sym,
                opts,
                fn_sigs,
                current_class,
                enclosing_fn,
            )?;
        }
    }
    Ok(())
}

fn analyze_anonymous_function(
    name: &str,
    params: &[Param],
    return_ann: &TypeAnnotation,
    body: &[Stmt],
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_class: Option<&str>,
    enclosing_fn: Option<&str>,
) -> Result<(), SoplangError> {
    if opts.strict {
        if return_ann == &TypeAnnotation::Dynamic {
            return Err(type_error_ex(
                "hawl aan magac lahayn ma leh nooc celi cad",
                0,
                0,
                ErrorMeta::default()
                    .with_code(codes::E022_TYPE_MISMATCH)
                    .with_hint("Habka adag: qor nooca qiimaha la celinayo sida ') : abn {' ama ') : qoraal {'."),
            ));
        }
        if params.iter().any(|p| p.type_ann == TypeAnnotation::Dynamic) {
            return Err(type_error_ex(
                "Habka adag wuxuu u baahan yahay in dood kasta oo hawsha aan magaca lahayn leedahay loo qoro nooc.",
                0,
                0,
                ErrorMeta::default()
                    .with_code(codes::E022_TYPE_MISMATCH)
                    .with_hint(
                        "Habka adag: qor nooca dood kasta, tusaale: hawl(abn x, qoraal y).",
                    ),
            ));
        }
    }

    let fn_name = enclosing_fn
        .map(|parent| format!("{parent}::{name}"))
        .unwrap_or_else(|| name.to_string());
    let mut func_scope = Scope::default();
    let mut param_slots = Vec::with_capacity(params.len());
    for (slot, param) in params.iter().enumerate() {
        param_slots.push(slot);
        func_scope.vars.insert(
            param.name.clone(),
            VarInfo {
                slot,
                type_ann: param.type_ann.clone(),
                is_const: false,
                is_captured: false,
            },
        );
    }

    let fns_before = sym.functions.len();
    sym.scopes.push(func_scope);
    ensure_known_class(sym, return_ann)?;
    for param in params {
        ensure_known_class(sym, &param.type_ann)?;
    }
    analyze_parameter_defaults(params, sym, opts, fn_sigs, current_class, Some(&fn_name))?;
    analyze_block(
        body,
        sym,
        opts,
        fn_sigs,
        Some(return_ann),
        current_class,
        Some(fn_name.as_str()),
    )?;

    let mut captures = Vec::new();
    if sym.scopes.len() > 2 {
        captures = collect_captures(body, sym);
        for param in params {
            if let Some(default) = &param.default {
                note_free_expr(default, sym, &mut captures);
            }
        }
        let nested: Vec<Vec<String>> = sym.functions[fns_before..]
            .iter()
            .map(|function| function.captures.clone())
            .collect();
        for nested_captures in &nested {
            for capture in nested_captures {
                if !sym.scopes.last().unwrap().vars.contains_key(capture)
                    && capture_from_enclosing(sym, capture)
                {
                    push_capture(&mut captures, capture);
                }
            }
        }
        let missing: Vec<String> = captures
            .iter()
            .filter(|capture| {
                !sym.scopes
                    .last()
                    .unwrap()
                    .vars
                    .contains_key(capture.as_str())
            })
            .cloned()
            .collect();
        for capture in missing {
            let scope = sym.scopes.last_mut().unwrap();
            let slot = scope.vars.len();
            scope.vars.insert(
                capture,
                VarInfo {
                    slot,
                    type_ann: TypeAnnotation::Dynamic,
                    is_const: false,
                    is_captured: true,
                },
            );
        }
    }


    let scope = sym.scopes.pop().unwrap();
    sym.functions.push(FunctionMeta {
        name: fn_name,
        param_slots,
        param_types: params.iter().map(|param| param.type_ann.clone()).collect(),
        param_names: params.iter().map(|param| param.name.clone()).collect(),
        required_params: required_parameters(params),
        variadic: params.iter().any(|param| param.variadic),
        keyword_rest: params.last().is_some_and(|param| param.keyword_rest),
        return_ann: return_ann.clone(),
        local_count: scope.vars.len(),
        captures,
        scope_vars: scope.vars,
        class_name: None,
        is_method: false,
    });
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
        Expr::NamedArgument { value, .. } => infer_expr_type(value, sym, fn_sigs),
        Expr::Located { expr, .. } => infer_expr_type(expr, sym, fn_sigs),
        Expr::Literal(l) => Some(match l {
            Literal::Int(_) => TypeAnnotation::Abn,
            Literal::Float(_) => TypeAnnotation::Jajab,
            Literal::Str(_) => TypeAnnotation::Qoraal,
            Literal::Bool(_) => TypeAnnotation::Bool,
            Literal::Null => TypeAnnotation::Null,
        }),
        Expr::Identifier(name) => resolve_name(sym, name, None).map(|v| v.type_ann.clone()),
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
                    } else if matches!(lt, TypeAnnotation::Abn) && matches!(rt, TypeAnnotation::Abn)
                    {
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
        Expr::Interpolated { .. } => Some(TypeAnnotation::Qoraal),
        Expr::Call { name, .. } => fn_sigs
            .get(name)
            .filter(|_| resolve_name(sym, name, None).is_none())
            .and_then(|s| {
                if s.ret == TypeAnnotation::Dynamic {
                    None
                } else {
                    Some(s.ret.clone())
                }
            }),
        Expr::New { class_name, .. } => Some(TypeAnnotation::Class(class_name.clone())),
        Expr::AnonymousFunc { .. }
        | Expr::Invoke { .. }
        | Expr::MethodCall { .. }
        | Expr::Index { .. }
        | Expr::Property { .. } => None,
    }
}

fn is_type_compatible(sym: &SymbolTable, target: &TypeAnnotation, value: &TypeAnnotation) -> bool {
    use TypeAnnotation::*;
    match (target, value) {
        (Dynamic, _) => true,
        (_, Union(found)) => found.iter().all(|ty| is_type_compatible(sym, target, ty)),
        (Union(_) | Null | ListOf(_) | ObjectOf(_), Dynamic) => true,
        (Union(types), found) => types.iter().any(|ty| is_type_compatible(sym, ty, found)),
        (Null, Null) => true,
        // A door value has no static class. The runtime check accepts a real instance.
        (Class(_), Dynamic) => true,
        (Class(expected), Class(found)) => is_same_or_subclass(sym, found, expected),
        (Class(_), _) => false,
        // Integral decimals may be stored in abn. Keep that check permissive.
        (Abn, Abn | Jajab) => true,
        (Jajab, Abn | Jajab) => true,
        (Qoraal, Qoraal) => true,
        (Bool, Bool) => true,
        (Teed, Teed | ListOf(_)) => true,
        (Walax, ObjectOf(_)) => true,
        (ListOf(_), Teed) | (ObjectOf(_), Walax) => true,
        // Mutable element contracts are invariant; a dynamic alias can request
        // another contract after validating the actual shared contents.
        (ListOf(expected), ListOf(found)) | (ObjectOf(expected), ObjectOf(found)) => {
            expected.same_contract(found)
        }
        (Bytes, Bytes) => true,
        (Walax, Walax) => true,
        _ => false,
    }
}

/// `found` is the instance class. True when it is `expected` or extends it.
fn is_same_or_subclass(sym: &SymbolTable, found: &str, expected: &str) -> bool {
    let mut current = Some(found.to_string());
    let mut seen = HashSet::new();
    while let Some(name) = current {
        if name == expected {
            return true;
        }
        if !seen.insert(name.clone()) {
            return false;
        }
        current = sym.classes.get(&name).and_then(|meta| meta.parent.clone());
    }
    false
}

fn ensure_known_class(sym: &SymbolTable, ann: &TypeAnnotation) -> Result<(), SoplangError> {
    if let TypeAnnotation::ListOf(inner) | TypeAnnotation::ObjectOf(inner) = ann {
        return ensure_known_class(sym, inner);
    }
    if let TypeAnnotation::Union(types) = ann {
        for ty in types {
            ensure_known_class(sym, ty)?;
        }
        return Ok(());
    }
    let TypeAnnotation::Class(name) = ann else {
        return Ok(());
    };
    if sym.classes.contains_key(name) {
        return Ok(());
    }
    Err(type_error_ex(
        format!("Qaab '{}' lama aqoonsan", name),
        0,
        0,
        ErrorMeta::default()
            .with_code(codes::E021_UNDECLARED)
            .with_hint("Qeex qaabka ka hor intaadan u isticmaalin nooc ahaan."),
    ))
}

fn type_mismatch_error(
    name: &str,
    expected: &TypeAnnotation,
    found: &TypeAnnotation,
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

fn ty_str(a: &TypeAnnotation) -> String {
    if *a == TypeAnnotation::Dynamic {
        "dynamic".to_owned()
    } else {
        a.to_string()
    }
}

fn analyze_assign_target(
    target: &Expr,
    sym: &mut SymbolTable,
    opts: &AnalyzeOptions,
    fn_sigs: &HashMap<String, FunctionSig>,
    current_class: Option<&str>,
    enclosing_fn: Option<&str>,
) -> Result<(), SoplangError> {
    match target {
        Expr::Located { span, expr } => {
            analyze_assign_target(expr, sym, opts, fn_sigs, current_class, enclosing_fn)
                .map_err(|e| span.attach(e))?
        }
        Expr::Index { obj, idx } => {
            analyze_expr(obj, sym, opts, fn_sigs, current_class, enclosing_fn)?;
            analyze_expr(idx, sym, opts, fn_sigs, current_class, enclosing_fn)?;
        }
        Expr::Property { obj, .. } => {
            analyze_expr(obj, sym, opts, fn_sigs, current_class, enclosing_fn)?
        }
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
            Stmt::Located { stmt, .. } => {
                collect_function_sigs_rec(std::slice::from_ref(stmt.as_ref()), out, current_class)
            }
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
                        params: params.iter().map(|p| p.type_ann.clone()).collect(),
                        names: params.iter().map(|p| p.name.clone()).collect(),
                        required_params: required_parameters(params),
                        variadic: params.iter().any(|param| param.variadic),
                        keyword_rest: params.last().is_some_and(|param| param.keyword_rest),
                        ret: return_ann.clone(),
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
            Stmt::For { body, .. } | Stmt::ForEach { body, .. } | Stmt::While { body, .. } => {
                collect_function_sigs_rec(body, out, current_class);
            }
            Stmt::TryCatch {
                try_body,
                catch_body,
                finally_body,
                ..
            } => {
                collect_function_sigs_rec(try_body, out, current_class);
                collect_function_sigs_rec(catch_body, out, current_class);
                collect_function_sigs_rec(finally_body, out, current_class);
            }
            _ => {}
        }
    }
}

/// A name in an outer function, not a global and not a local of the current one.
fn capture_from_enclosing(sym: &SymbolTable, name: &str) -> bool {
    let last = sym.scopes.len().saturating_sub(1);
    for i in (0..sym.scopes.len()).rev() {
        if sym.scopes[i].vars.contains_key(name) {
            return i > 0 && i < last;
        }
    }
    false
}

fn push_capture(caps: &mut Vec<String>, name: &str) {
    if !caps.iter().any(|c| c == name) {
        caps.push(name.to_string());
    }
}

fn collect_captures(body: &[Stmt], sym: &SymbolTable) -> Vec<String> {
    let mut caps = Vec::new();
    note_free_stmts(body, sym, &mut caps);
    caps
}

fn note_free_stmts(stmts: &[Stmt], sym: &SymbolTable, caps: &mut Vec<String>) {
    for s in stmts {
        note_free_stmt(s, sym, caps);
    }
}

fn note_name(name: &str, sym: &SymbolTable, caps: &mut Vec<String>) {
    if capture_from_enclosing(sym, name) {
        push_capture(caps, name);
    }
}

fn note_free_expr(expr: &Expr, sym: &SymbolTable, caps: &mut Vec<String>) {
    match expr {
        Expr::Located { expr, .. } => note_free_expr(expr, sym, caps),
        Expr::Literal(_) => {}
        Expr::Identifier(name) => note_name(name, sym, caps),
        Expr::BinaryOp { left, right, .. } => {
            note_free_expr(left, sym, caps);
            note_free_expr(right, sym, caps);
        }
        Expr::NamedArgument { value: expr, .. } | Expr::UnaryOp { expr, .. } => {
            note_free_expr(expr, sym, caps)
        }
        Expr::Interpolated { parts } => {
            for part in parts {
                if let InterpolationPart::Value { expression, .. } = part {
                    note_free_expr(expression, sym, caps);
                }
            }
        }
        Expr::Call { name, args } => {
            note_name(name, sym, caps);
            for a in args {
                note_free_expr(a, sym, caps);
            }
        }
        Expr::AnonymousFunc { .. } => {}
        Expr::MethodCall { obj, args, .. } | Expr::Invoke { callee: obj, args } => {
            note_free_expr(obj, sym, caps);
            for a in args {
                note_free_expr(a, sym, caps);
            }
        }
        Expr::Index { obj, idx } => {
            note_free_expr(obj, sym, caps);
            note_free_expr(idx, sym, caps);
        }
        Expr::Property { obj, .. } => note_free_expr(obj, sym, caps),
        Expr::List(items) => {
            for e in items {
                note_free_expr(e, sym, caps);
            }
        }
        Expr::Object(pairs) => {
            for (_, e) in pairs {
                note_free_expr(e, sym, caps);
            }
        }
        Expr::New { args, .. } => {
            for a in args {
                note_free_expr(a, sym, caps);
            }
        }
    }
}

fn note_free_stmt(stmt: &Stmt, sym: &SymbolTable, caps: &mut Vec<String>) {
    match stmt {
        Stmt::Located { stmt, .. } => note_free_stmt(stmt, sym, caps),
        Stmt::VarDecl { value, .. } => note_free_expr(value, sym, caps),
        Stmt::Assign { target, value, .. } => {
            note_free_expr(target, sym, caps);
            note_free_expr(value, sym, caps);
        }
        Stmt::FuncDef { .. }
        | Stmt::ClassDef { .. }
        | Stmt::Break
        | Stmt::Continue
        | Stmt::Import(_)
        | Stmt::Return(None) => {}
        Stmt::If {
            cond,
            then_body,
            elseifs,
            else_body,
        } => {
            note_free_expr(cond, sym, caps);
            note_free_stmts(then_body, sym, caps);
            for (c, b) in elseifs {
                note_free_expr(c, sym, caps);
                note_free_stmts(b, sym, caps);
            }
            if let Some(b) = else_body {
                note_free_stmts(b, sym, caps);
            }
        }
        Stmt::Switch {
            expr,
            cases,
            default,
        } => {
            note_free_expr(expr, sym, caps);
            for (v, b) in cases {
                note_free_expr(v, sym, caps);
                note_free_stmts(b, sym, caps);
            }
            if let Some(b) = default {
                note_free_stmts(b, sym, caps);
            }
        }
        Stmt::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            note_free_expr(start, sym, caps);
            note_free_expr(end, sym, caps);
            if let Some(s) = step {
                note_free_expr(s, sym, caps);
            }
            note_free_stmts(body, sym, caps);
        }
        Stmt::ForEach { iterable, body, .. } => {
            note_free_expr(iterable, sym, caps);
            note_free_stmts(body, sym, caps);
        }
        Stmt::While { cond, body } => {
            note_free_expr(cond, sym, caps);
            note_free_stmts(body, sym, caps);
        }
        Stmt::Return(Some(e)) => note_free_expr(e, sym, caps),
        Stmt::TryCatch {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
            note_free_stmts(try_body, sym, caps);
            note_free_stmts(catch_body, sym, caps);
            note_free_stmts(finally_body, sym, caps);
        }
        Stmt::Block(b) => note_free_stmts(b, sym, caps),
        Stmt::Expr(e) | Stmt::Throw(e) => note_free_expr(e, sym, caps),
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

fn has_named_arguments(args: &[Expr]) -> bool {
    args.iter()
        .any(|argument| matches!(argument.unlocated(), Expr::NamedArgument { .. }))
}

fn validate_named_arguments(
    args: &[Expr],
    sig: &FunctionSig,
    sym: &SymbolTable,
    fn_sigs: &HashMap<String, FunctionSig>,
) -> Result<(), SoplangError> {
    let fixed = sig.params.len() - usize::from(sig.variadic) - usize::from(sig.keyword_rest);
    let mut filled = vec![false; fixed];
    let mut positional = 0;
    for arg in args {
        let (index, value) = match arg.unlocated() {
            Expr::NamedArgument { name, value } => {
                let index = sig.names[..fixed]
                    .iter()
                    .position(|parameter| parameter == name);
                if index.is_none() && !sig.keyword_rest {
                    return Err(attach_argument(
                        arg,
                        format!("Doodda magaca '{}' leh hawshani ma aqbasho.", name),
                    ));
                }
                (index, value.as_ref())
            }
            _ => {
                let index = positional;
                positional += 1;
                if index >= fixed && !sig.variadic {
                    return Err(attach_argument(
                        arg,
                        "Doodaha boosaska leh ayaa ka badan intii hawshu aqbali lahayd.",
                    ));
                }
                ((index < fixed).then_some(index), arg)
            }
        };
        let Some(index) = index else { continue };
        if filled[index] {
            return Err(attach_argument(
                arg,
                format!("Doodda '{}' laba jeer ayaa la bixiyay.", sig.names[index]),
            ));
        }
        filled[index] = true;
        if sig.params[index] != TypeAnnotation::Dynamic {
            if let Some(found) = infer_expr_type(value, sym, fn_sigs) {
                if !is_type_compatible(sym, &sig.params[index], &found) {
                    let error =
                        type_mismatch_error(&sig.names[index], &sig.params[index], &found, 0, 0);
                    return Err(match value {
                        Expr::Located { span, .. } => span.attach(error),
                        _ => error,
                    });
                }
            }
        }
    }
    for (index, supplied) in filled.iter().enumerate().take(sig.required_params) {
        if !supplied {
            return Err(type_error_ex(
                format!("Doodda '{}' ayaa ka maqan wicitaankan.", sig.names[index]),
                0,
                0,
                ErrorMeta::default().with_code(codes::E022_TYPE_MISMATCH),
            ));
        }
    }
    Ok(())
}

fn attach_argument(arg: &Expr, message: impl Into<String>) -> SoplangError {
    let error = type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E022_TYPE_MISMATCH),
    );
    match arg {
        Expr::Located { span, .. } => span.attach(error),
        _ => error,
    }
}
