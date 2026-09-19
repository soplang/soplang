//! High-level IR (HIR) for the Soplang compiler.
//! Phase 2 of COMPILER_PLAN. Flat, backend-agnostic representation.

use std::fmt;

use crate::frontend::ast::{Expr, Literal, Stmt, TypeAnnotation};
use crate::semantic::{resolve_name, resolve_parent_method, SymbolTable, VarInfo};

pub type Slot = usize;
pub type LabelId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum HirConst {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOpKind {
    Neg,
    Not,
}

/// Static numeric shape of a slot. `Dyn` stays on the tagged runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlotNum {
    #[default]
    Dyn,
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub enum HirInstr {
    Const { dst: Slot, val: HirConst },
    Copy { dst: Slot, src: Slot },
    Load { dst: Slot, name: String },
    Store { name: String, src: Slot },
    BinOp { dst: Slot, op: BinOpKind, lhs: Slot, rhs: Slot, typed: bool },
    UnOp { dst: Slot, op: UnOpKind, src: Slot },
    BuildList { dst: Slot, items: Vec<Slot> },
    BuildObject { dst: Slot, pairs: Vec<(String, Slot)> },
    GetIndex { dst: Slot, obj: Slot, idx: Slot },
    SetIndex { obj: Slot, idx: Slot, val: Slot },
    GetProp { dst: Slot, obj: Slot, prop: String },
    SetProp { obj: Slot, prop: String, val: Slot },
    Label(LabelId),
    Jump(LabelId),
    JumpIf { cond: Slot, on_true: LabelId, on_false: LabelId },
    Call { dst: Slot, callee: Slot, args: Vec<Slot> },
    CallMethod { dst: Slot, obj: Slot, method: String, args: Vec<Slot> },
    Return { val: Slot },
    Break(LabelId),
    Continue(LabelId),
    TryBegin { catch: LabelId },
    TryEnd,
    BindError { dst: Slot },
    CheckThrow { catch: LabelId },
    Pop { dst: Slot }, // discard value (for expr stmt)
    CheckType { src: Slot, type_tag: u8, name: String },
    /// Value must be an instance of `class_name` or a subclass.
    CheckClass { src: Slot, class_name: String, var_name: String },
    MarkConst { name: String },
    NewInstance { dst: Slot, class_name: String, args: Vec<Slot> },
    /// Nested function value: compiled function plus the captured slots.
    MakeClosure { dst: Slot, fn_name: String, captures: Vec<Slot> },
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name:        String,
    pub params:      Vec<Slot>,
    pub local_count: usize,
    pub body:        Vec<HirInstr>,
    pub is_static:   bool,
    /// Numeric kind of local slots, indexed by slot. Temps are not included.
    pub local_nums:  Vec<SlotNum>,
    /// Locals filled from the closure environment. Empty means no env parameter.
    pub capture_slots: Vec<Slot>,
}

#[derive(Debug, Default, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub top_level:  Vec<HirInstr>,
    /// Numeric kind of top-level variable slots, indexed by slot.
    pub top_nums: Vec<SlotNum>,
}

impl fmt::Display for HirInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HirInstr::Const { dst, val } => write!(f, "  %{} = const {:?}", dst, val),
            HirInstr::Copy { dst, src } => write!(f, "  %{} = copy %{}", dst, src),
            HirInstr::Load { dst, name } => write!(f, "  %{} = load {}", dst, name),
            HirInstr::Store { name, src } => write!(f, "  store {} %{}", name, src),
            HirInstr::BinOp { dst, op, lhs, rhs, .. } => {
                write!(f, "  %{} = %{} {:?} %{}", dst, lhs, op, rhs)
            }
            HirInstr::UnOp { dst, op, src } => write!(f, "  %{} = {:?} %{}", dst, op, src),
            HirInstr::BuildList { dst, items } => {
                write!(f, "  %{} = list [", dst)?;
                for (i, s) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "%{}", s)?;
                }
                write!(f, "]")
            }
            HirInstr::BuildObject { dst, pairs } => {
                write!(f, "  %{} = object {{", dst)?;
                for (i, (k, s)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: %{}", k, s)?;
                }
                write!(f, "}}")
            }
            HirInstr::GetIndex { dst, obj, idx } => write!(f, "  %{} = %{}[%{}]", dst, obj, idx),
            HirInstr::SetIndex { obj, idx, val } => write!(f, "  %{}[%{}] = %{}", obj, idx, val),
            HirInstr::GetProp { dst, obj, prop } => write!(f, "  %{} = %{}.{}", dst, obj, prop),
            HirInstr::SetProp { obj, prop, val } => write!(f, "  %{}.{} = %{}", obj, prop, val),
            HirInstr::Label(id) => write!(f, "L{}:", id),
            HirInstr::Jump(id) => write!(f, "  jump L{}", id),
            HirInstr::JumpIf { cond, on_true, on_false } => {
                write!(f, "  jump_if %{} L{} L{}", cond, on_true, on_false)
            }
            HirInstr::Call { dst, callee, args } => {
                write!(f, "  %{} = call %{}(", dst, callee)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "%{}", a)?;
                }
                write!(f, ")")
            }
            HirInstr::CallMethod { dst, obj, method, args } => {
                write!(f, "  %{} = %{}.{}(", dst, obj, method)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "%{}", a)?;
                }
                write!(f, ")")
            }
            HirInstr::Return { val } => write!(f, "  return %{}", val),
            HirInstr::Break(id) => write!(f, "  break L{}", id),
            HirInstr::Continue(id) => write!(f, "  continue L{}", id),
            HirInstr::TryBegin { catch } => write!(f, "  try_begin L{}", catch),
            HirInstr::TryEnd => write!(f, "  try_end"),
            HirInstr::CheckThrow { catch } => write!(f, "  check_throw L{}", catch),
            HirInstr::BindError { dst } => write!(f, "  bind_error %{}", dst),
            HirInstr::Pop { dst } => write!(f, "  pop %{}", dst),
            HirInstr::CheckType { src, type_tag, name } => write!(f, "  check_type %{} {} {}", src, type_tag, name),
            HirInstr::CheckClass { src, class_name, var_name } => {
                write!(f, "  check_class %{} {} {}", src, class_name, var_name)
            }
            HirInstr::MarkConst { name } => write!(f, "  mark_const {}", name),
            HirInstr::NewInstance { dst, class_name, args } => {
                write!(f, "  %{} = new {}(", dst, class_name)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "%{}", a)?;
                }
                write!(f, ")")
            }
            HirInstr::MakeClosure { dst, fn_name, captures } => {
                write!(f, "  %{} = closure {} [", dst, fn_name)?;
                for (i, s) in captures.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "%{}", s)?;
                }
                write!(f, "]")
            }
        }
    }
}

fn bin_op_from_str(op: &str) -> Option<BinOpKind> {
    match op {
        "+" => Some(BinOpKind::Add),
        "-" => Some(BinOpKind::Sub),
        "*" => Some(BinOpKind::Mul),
        "/" => Some(BinOpKind::Div),
        "%" => Some(BinOpKind::Mod),
        "==" => Some(BinOpKind::Eq),
        "!=" => Some(BinOpKind::Ne),
        "<" => Some(BinOpKind::Lt),
        "<=" => Some(BinOpKind::Le),
        ">" => Some(BinOpKind::Gt),
        ">=" => Some(BinOpKind::Ge),
        "&&" => Some(BinOpKind::And),
        "||" => Some(BinOpKind::Or),
        _ => None,
    }
}

fn un_op_from_str(op: &str) -> Option<UnOpKind> {
    match op {
        "-" => Some(UnOpKind::Neg),
        "!" => Some(UnOpKind::Not),
        _ => None,
    }
}

/// Lowers AST to HIR using the symbol table.
pub struct HirLowering<'a> {
    sym:        &'a SymbolTable,
    body:       Vec<HirInstr>,
    next_slot:  Slot,
    next_label: LabelId,
    loop_stack: Vec<(LabelId, LabelId)>, // (break_label, continue_label)
    func_scope:  Option<&'a std::collections::HashMap<String, crate::semantic::VarInfo>>,
    catch_var:  Option<(String, Slot)>,
    /// Class whose method is being lowered. `waalid` starts lookup at its parent.
    enclosing_class: Option<&'a str>,
    /// Function being lowered. Nested `hawl` names are `parent::name`.
    enclosing_fn: Option<&'a str>,
    nums: Vec<SlotNum>,
    /// Variable slots keep the kind from their type annotation.
    fixed: Vec<bool>,
}

impl<'a> HirLowering<'a> {
    pub fn lower(sym: &'a SymbolTable, stmts: &[Stmt]) -> HirModule {
        let mut lower = HirLowering {
            sym,
            body: Vec::new(),
            next_slot: 0,
            next_label: 0,
            loop_stack: Vec::new(),
            func_scope: None,
            catch_var: None,
            enclosing_class: None,
            enclosing_fn: None,
            nums: Vec::new(),
            fixed: Vec::new(),
        };
        if let Some(scope) = sym.scopes.first() {
            lower.prepare_vars(&scope.vars);
        }
        let top_nums = lower.nums.clone();
        let func_bodies = collect_func_bodies(stmts);
        for s in stmts {
            match s {
                Stmt::FuncDef { .. } | Stmt::ClassDef { .. } => {}
                _ => lower.lower_stmt(s),
            }
        }
        let top_level = std::mem::take(&mut lower.body);
        let func_bodies_len = func_bodies.len();
        debug_assert_eq!(
            sym.functions.len(),
            func_bodies_len,
            "function metadata and bodies diverged"
        );
        let functions: Vec<HirFunction> = sym.functions
            .iter()
            .zip(func_bodies.into_iter())
            .map(|(fm, body_stmts)| {
                let mut inner = HirLowering {
                    sym,
                    body: Vec::new(),
                    next_slot: 0,
                    next_label: 0,
                    loop_stack: Vec::new(),
                    func_scope: Some(&fm.scope_vars),
                    catch_var: None,
                    enclosing_class: fm.class_name.as_deref(),
                    enclosing_fn: Some(fm.name.as_str()),
                    nums: Vec::new(),
                    fixed: Vec::new(),
                };
                inner.prepare_vars(&fm.scope_vars);
                let local_nums = inner.nums.clone();
                let capture_slots: Vec<Slot> = fm
                    .captures
                    .iter()
                    .filter_map(|name| fm.scope_vars.get(name).map(|info| info.slot))
                    .collect();
                for (name, info) in &fm.scope_vars {
                    if let TypeAnnotation::Class(class) = &info.type_ann {
                        inner.emit(HirInstr::CheckClass {
                            src: info.slot,
                            class_name: class.clone(),
                            var_name: name.clone(),
                        });
                    }
                }
                for s in &body_stmts {
                    inner.lower_stmt(s);
                }
                HirFunction {
                    name:        fm.name.clone(),
                    params:      fm.param_slots.clone(),
                    local_count: fm.local_count,
                    body:        inner.body,
                    is_static:   false,
                    local_nums,
                    capture_slots,
                }
            })
            .collect();
        HirModule {
            functions,
            top_level,
            top_nums,
        }
    }

    fn alloc_slot(&mut self) -> Slot {
        let s = self.next_slot;
        self.next_slot += 1;
        self.ensure_slot(s);
        s
    }

    fn ensure_slot(&mut self, slot: Slot) {
        if self.nums.len() <= slot {
            self.nums.resize(slot + 1, SlotNum::Dyn);
            self.fixed.resize(slot + 1, false);
        }
    }

    fn prepare_vars(&mut self, vars: &std::collections::HashMap<String, VarInfo>) {
        let n = vars.values().map(|v| v.slot + 1).max().unwrap_or(0);
        if n == 0 {
            return;
        }
        self.nums.resize(n, SlotNum::Dyn);
        self.fixed.resize(n, false);
        for v in vars.values() {
            self.nums[v.slot] = num_of_ann(&v.type_ann);
            self.fixed[v.slot] = true;
        }
        if self.next_slot < n {
            self.next_slot = n;
        }
    }

    fn set_temp_num(&mut self, slot: Slot, num: SlotNum) {
        self.ensure_slot(slot);
        if !self.fixed[slot] {
            self.nums[slot] = num;
        }
    }

    fn emit_copy(&mut self, dst: Slot, src: Slot) {
        self.ensure_slot(dst);
        self.ensure_slot(src);
        self.emit(HirInstr::Copy { dst, src });
        if !self.fixed[dst] {
            self.nums[dst] = self.nums[src];
        }
    }

    fn emit_const(&mut self, slot: Slot, val: HirConst) {
        let num = match &val {
            HirConst::Int(_) => SlotNum::Int,
            HirConst::Float(_) => SlotNum::Float,
            _ => SlotNum::Dyn,
        };
        self.emit(HirInstr::Const { dst: slot, val });
        self.set_temp_num(slot, num);
    }

    fn emit_binop(&mut self, dst: Slot, op: BinOpKind, lhs: Slot, rhs: Slot) {
        self.ensure_slot(lhs);
        self.ensure_slot(rhs);
        let lk = self.nums[lhs];
        let rk = self.nums[rhs];
        let numeric = matches!(
            op,
            BinOpKind::Add
                | BinOpKind::Sub
                | BinOpKind::Mul
                | BinOpKind::Div
                | BinOpKind::Mod
                | BinOpKind::Eq
                | BinOpKind::Ne
                | BinOpKind::Lt
                | BinOpKind::Le
                | BinOpKind::Gt
                | BinOpKind::Ge
        );
        let typed = numeric && lk != SlotNum::Dyn && rk != SlotNum::Dyn;
        self.emit(HirInstr::BinOp { dst, op, lhs, rhs, typed });
        let result = if !typed {
            SlotNum::Dyn
        } else {
            match op {
                BinOpKind::Div => SlotNum::Float,
                BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Mod => {
                    if lk == SlotNum::Float || rk == SlotNum::Float {
                        SlotNum::Float
                    } else {
                        SlotNum::Int
                    }
                }
                _ => SlotNum::Dyn,
            }
        };
        self.set_temp_num(dst, result);
    }

    fn emit_unop(&mut self, dst: Slot, op: UnOpKind, src: Slot) {
        self.ensure_slot(src);
        self.emit(HirInstr::UnOp { dst, op, src });
        let num = if op == UnOpKind::Neg { self.nums[src] } else { SlotNum::Dyn };
        self.set_temp_num(dst, num);
    }

    fn new_label(&mut self) -> LabelId {
        let id = self.next_label;
        self.next_label += 1;
        id
    }

    fn emit(&mut self, instr: HirInstr) {
        self.body.push(instr);
    }

    /// Direct call of an ancestor method, with `nafta` as the instance.
    fn lower_parent_call(&mut self, method: &str, args: &[Expr]) -> Option<Slot> {
        let class = self.enclosing_class?;
        let mangled = resolve_parent_method(self.sym, class, method).ok()?;
        let nafta = self.resolve_slot("nafta")?;
        let mut arg_slots = vec![nafta];
        arg_slots.extend(args.iter().map(|a| self.lower_expr(a)));
        let callee = self.alloc_slot();
        self.emit(HirInstr::Load { dst: callee, name: mangled });
        let dst = self.alloc_slot();
        self.emit(HirInstr::Call { dst, callee, args: arg_slots });
        Some(dst)
    }

    fn var_ann(&self, name: &str) -> Option<TypeAnnotation> {
        if self.func_scope.is_some() {
            return self.func_scope.unwrap().get(name).map(|v| v.type_ann.clone());
        }
        resolve_name(self.sym, name, None).map(|v| v.type_ann.clone())
    }

    fn emit_type_check(&mut self, src: Slot, ann: &TypeAnnotation, name: &str) {
        match ann {
            TypeAnnotation::Dynamic => {}
            TypeAnnotation::Class(class) => self.emit(HirInstr::CheckClass {
                src,
                class_name: class.clone(),
                var_name: name.to_string(),
            }),
            other => self.emit(HirInstr::CheckType {
                src,
                type_tag: type_ann_to_tag(other),
                name: name.to_string(),
            }),
        }
    }

    fn resolve_slot(&self, name: &str) -> Option<Slot> {
        if let Some((catch_name, slot)) = &self.catch_var {
            if catch_name == name {
                return Some(*slot);
            }
        }
        if self.func_scope.is_some() {
            // Inside a function: only resolve local variables, not globals
            self.func_scope.unwrap().get(name).map(|v| v.slot)
        } else if self.sym.session_globals.contains(name) {
            None
        } else {
            resolve_name(self.sym, name, None).map(|v| v.slot)
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, value, type_ann, is_const, .. } => {
                let val_slot = self.lower_expr(value);
                if let Some(slot) = self.resolve_slot(name) {
                    self.emit_copy(slot, val_slot);
                    if self.func_scope.is_none() {
                        self.emit(HirInstr::Store { name: name.clone(), src: val_slot });
                    }
                } else {
                    self.emit(HirInstr::Store { name: name.clone(), src: val_slot });
                }
                self.emit_type_check(val_slot, type_ann, name);
                if *is_const && self.func_scope.is_none() {
                    self.emit(HirInstr::MarkConst { name: name.clone() });
                }
            }
            Stmt::Assign { target, value, .. } => {
                let val_slot = self.lower_expr(value);
                match target {
                    Expr::Identifier(n) => {
                        if let Some(slot) = self.resolve_slot(n) {
                            self.emit_copy(slot, val_slot);
                            if self.func_scope.is_none() {
                                self.emit(HirInstr::Store { name: n.clone(), src: val_slot });
                            }
                        } else {
                            self.emit(HirInstr::Store { name: n.clone(), src: val_slot });
                        }
                        if let Some(TypeAnnotation::Class(class)) = self.var_ann(n) {
                            self.emit(HirInstr::CheckClass {
                                src: val_slot,
                                class_name: class,
                                var_name: n.clone(),
                            });
                        }
                    }
                    Expr::Index { obj, idx } => {
                        let obj_slot = self.lower_expr(obj);
                        let idx_slot = self.lower_expr(idx);
                        self.emit(HirInstr::SetIndex { obj: obj_slot, idx: idx_slot, val: val_slot });
                    }
                    Expr::Property { obj, prop } => {
                        let obj_slot = self.lower_expr(obj);
                        self.emit(HirInstr::SetProp { obj: obj_slot, prop: prop.clone(), val: val_slot });
                    }
                    _ => {}
                }
            }
            Stmt::ClassDef { .. } => {}
            Stmt::FuncDef { name, .. } => {
                let Some(parent) = self.enclosing_fn else {
                    return;
                };
                let Some(dst) = self.resolve_slot(name) else {
                    return;
                };
                let mangled = format!("{parent}::{name}");
                let caps = self
                    .sym
                    .functions
                    .iter()
                    .find(|f| f.name == mangled)
                    .map(|f| f.captures.clone())
                    .unwrap_or_default();
                if caps.is_empty() {
                    self.emit(HirInstr::Load { dst, name: mangled });
                } else {
                    let slots: Vec<Slot> = caps.iter().filter_map(|c| self.resolve_slot(c)).collect();
                    self.emit(HirInstr::MakeClosure { dst, fn_name: mangled, captures: slots });
                }
            }
            Stmt::If { cond, then_body, elseifs, else_body } => {
                let end_label = self.new_label();
                let else_start = self.new_label();
                let cond_slot = self.lower_expr(cond);
                let then_label = self.new_label();
                self.emit(HirInstr::JumpIf { cond: cond_slot, on_true: then_label, on_false: else_start });
                self.emit(HirInstr::Label(then_label));
                for s in then_body {
                    self.lower_stmt(s);
                }
                self.emit(HirInstr::Jump(end_label));
                self.emit(HirInstr::Label(else_start));
                for (c, body) in elseifs {
                    let next_else = self.new_label();
                    let next_then = self.new_label();
                    let cond2_slot = self.lower_expr(c);
                    self.emit(HirInstr::JumpIf { cond: cond2_slot, on_true: next_then, on_false: next_else });
                    self.emit(HirInstr::Label(next_then));
                    for s in body {
                        self.lower_stmt(s);
                    }
                    self.emit(HirInstr::Jump(end_label));
                    self.emit(HirInstr::Label(next_else));
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.lower_stmt(s);
                    }
                }
                self.emit(HirInstr::Label(end_label));
            }
            Stmt::Switch { expr, cases, default } => {
                let expr_slot = self.lower_expr(expr);
                let end_label = self.new_label();
                for (v, body) in cases {
                    let case_val = self.lower_expr(v);
                    let eq_slot = self.alloc_slot();
                    self.emit_binop(eq_slot, BinOpKind::Eq, expr_slot, case_val);
                    let match_label = self.new_label();
                    let next_case = self.new_label();
                    self.emit(HirInstr::JumpIf { cond: eq_slot, on_true: match_label, on_false: next_case });
                    self.emit(HirInstr::Label(match_label));
                    for s in body {
                        self.lower_stmt(s);
                    }
                    self.emit(HirInstr::Jump(end_label));
                    self.emit(HirInstr::Label(next_case));
                }
                if let Some(d) = default {
                    for s in d {
                        self.lower_stmt(s);
                    }
                }
                self.emit(HirInstr::Label(end_label));
            }
            Stmt::For { var, start, end, step, body } => {
                let break_lbl = self.new_label();
                let cond_lbl = self.new_label();
                let incr_lbl = self.new_label();
                self.loop_stack.push((break_lbl, incr_lbl));
                let start_slot = self.lower_expr(start);
                let end_slot = self.lower_expr(end);
                let step_slot = step.as_ref().map(|s| self.lower_expr(s));
                let var_slot = self.resolve_slot(var).unwrap_or_else(|| self.alloc_slot());
                self.emit_copy(var_slot, start_slot);
                self.emit(HirInstr::Label(cond_lbl));
                let cond_slot = self.alloc_slot();
                self.emit_binop(cond_slot, BinOpKind::Le, var_slot, end_slot);
                let body_start = self.new_label();
                self.emit(HirInstr::JumpIf { cond: cond_slot, on_true: body_start, on_false: break_lbl });
                self.emit(HirInstr::Label(body_start));
                for s in body {
                    self.lower_stmt(s);
                }
                self.emit(HirInstr::Label(incr_lbl));
                if let Some(ss) = step_slot {
                    let new_var = self.alloc_slot();
                    self.emit_binop(new_var, BinOpKind::Add, var_slot, ss);
                    self.emit_copy(var_slot, new_var);
                } else {
                    let one = self.alloc_slot();
                    self.emit_const(one, HirConst::Int(1));
                    let new_var = self.alloc_slot();
                    self.emit_binop(new_var, BinOpKind::Add, var_slot, one);
                    self.emit_copy(var_slot, new_var);
                }
                self.emit(HirInstr::Jump(cond_lbl));
                self.emit(HirInstr::Label(break_lbl));
                self.loop_stack.pop();
            }
            Stmt::While { cond, body } => {
                let break_lbl = self.new_label();
                let cont_lbl = self.new_label();
                self.loop_stack.push((break_lbl, cont_lbl));
                self.emit(HirInstr::Label(cont_lbl));
                let cond_slot = self.lower_expr(cond);
                let body_lbl = self.new_label();
                self.emit(HirInstr::JumpIf { cond: cond_slot, on_true: body_lbl, on_false: break_lbl });
                self.emit(HirInstr::Label(body_lbl));
                for s in body {
                    self.lower_stmt(s);
                }
                self.emit(HirInstr::Jump(cont_lbl));
                self.emit(HirInstr::Label(break_lbl));
                self.loop_stack.pop();
            }
            Stmt::Return(val) => {
                let slot = match val {
                    None => {
                        let s = self.alloc_slot();
                        self.emit_const(s, HirConst::Null);
                        s
                    }
                    Some(e) => self.lower_expr(e),
                };
                self.emit(HirInstr::Return { val: slot });
            }
            Stmt::Break => {
                if let Some((lbl, _)) = self.loop_stack.last() {
                    self.emit(HirInstr::Break(*lbl));
                }
            }
            Stmt::Continue => {
                if let Some((_, lbl)) = self.loop_stack.last() {
                    self.emit(HirInstr::Continue(*lbl));
                }
            }
            Stmt::TryCatch { try_body, err_var, catch_body } => {
                let catch_label = self.new_label();
                self.emit(HirInstr::TryBegin { catch: catch_label });
                for s in try_body {
                    self.lower_stmt(s);
                    self.emit(HirInstr::CheckThrow { catch: catch_label });
                }
                self.emit(HirInstr::TryEnd);
                let end_try = self.new_label();
                self.emit(HirInstr::Jump(end_try));
                self.emit(HirInstr::Label(catch_label));
                let err_slot = self.alloc_slot();
                self.catch_var = Some((err_var.clone(), err_slot));
                self.emit(HirInstr::BindError { dst: err_slot });
                self.emit(HirInstr::TryEnd);
                for s in catch_body {
                    self.lower_stmt(s);
                }
                self.catch_var = None;
                self.emit(HirInstr::Label(end_try));
            }
            Stmt::Import(_) => {}
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.lower_stmt(s);
                }
            }
            Stmt::Expr(e) => {
                let slot = self.lower_expr(e);
                self.emit(HirInstr::Pop { dst: slot });
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> Slot {
        match expr {
            Expr::Literal(lit) => {
                let slot = self.alloc_slot();
                let val = match lit {
                    Literal::Int(n) => HirConst::Int(*n),
                    Literal::Float(x) => HirConst::Float(*x),
                    Literal::Str(s) => HirConst::Str(s.clone()),
                    Literal::Bool(b) => HirConst::Bool(*b),
                    Literal::Null => HirConst::Null,
                };
                self.emit_const(slot, val);
                slot
            }
            Expr::Identifier(name) => {
                let slot = self.alloc_slot();
                if let Some(var_slot) = self.resolve_slot(name) {
                    self.emit_copy(slot, var_slot);
                } else {
                    self.emit(HirInstr::Load { dst: slot, name: name.clone() });
                }
                slot
            }
            Expr::BinaryOp { op, left, right } => {
                let lhs = self.lower_expr(left);
                let rhs = self.lower_expr(right);
                let dst = self.alloc_slot();
                if let Some(kind) = bin_op_from_str(op) {
                    self.emit_binop(dst, kind, lhs, rhs);
                }
                dst
            }
            Expr::UnaryOp { op, expr: e } => {
                let src = self.lower_expr(e);
                let dst = self.alloc_slot();
                if let Some(kind) = un_op_from_str(op) {
                    self.emit_unop(dst, kind, src);
                }
                dst
            }
            Expr::Call { name, args } => {
                let callee = self.alloc_slot();
                if let Some(slot) = self.resolve_slot(name) {
                    self.emit_copy(callee, slot);
                } else {
                    self.emit(HirInstr::Load { dst: callee, name: name.clone() });
                }
                let arg_slots: Vec<Slot> = args.iter().map(|a| self.lower_expr(a)).collect();
                let dst = self.alloc_slot();
                self.emit(HirInstr::Call { dst, callee, args: arg_slots });
                dst
            }
            Expr::MethodCall { obj, method, args } => {
                if let Expr::Identifier(recv) = obj.as_ref() {
                    if recv == "waalid" {
                        if let Some(slot) = self.lower_parent_call(method, args) {
                            return slot;
                        }
                    }
                }
                let obj_slot = self.lower_expr(obj);
                let arg_slots: Vec<Slot> = args.iter().map(|a| self.lower_expr(a)).collect();
                let dst = self.alloc_slot();
                self.emit(HirInstr::CallMethod { dst, obj: obj_slot, method: method.clone(), args: arg_slots });
                dst
            }
            Expr::Index { obj, idx } => {
                let obj_slot = self.lower_expr(obj);
                let idx_slot = self.lower_expr(idx);
                let dst = self.alloc_slot();
                self.emit(HirInstr::GetIndex { dst, obj: obj_slot, idx: idx_slot });
                dst
            }
            Expr::Property { obj, prop } => {
                let obj_slot = self.lower_expr(obj);
                let dst = self.alloc_slot();
                self.emit(HirInstr::GetProp { dst, obj: obj_slot, prop: prop.clone() });
                dst
            }
            Expr::List(exprs) => {
                let items: Vec<Slot> = exprs.iter().map(|e| self.lower_expr(e)).collect();
                let dst = self.alloc_slot();
                self.emit(HirInstr::BuildList { dst, items });
                dst
            }
            Expr::Object(pairs) => {
                let pair_slots: Vec<(String, Slot)> = pairs.iter().map(|(k, v)| (k.clone(), self.lower_expr(v))).collect();
                let dst = self.alloc_slot();
                self.emit(HirInstr::BuildObject { dst, pairs: pair_slots });
                dst
            }
            Expr::New { class_name, args } => {
                let arg_slots: Vec<Slot> = args.iter().map(|a| self.lower_expr(a)).collect();
                let dst = self.alloc_slot();
                self.emit(HirInstr::NewInstance { dst, class_name: class_name.clone(), args: arg_slots });
                dst
            }
        }
    }
}

fn num_of_ann(ann: &TypeAnnotation) -> SlotNum {
    match ann {
        TypeAnnotation::Abn => SlotNum::Int,
        TypeAnnotation::Jajab => SlotNum::Float,
        _ => SlotNum::Dyn,
    }
}

fn type_ann_to_tag(ann: &TypeAnnotation) -> u8 {
    match ann {
        TypeAnnotation::Abn => 1,
        TypeAnnotation::Jajab => 2,
        TypeAnnotation::Qoraal => 3,
        TypeAnnotation::Bool => 4,
        TypeAnnotation::Teed => 5,
        TypeAnnotation::Walax => 6,
        TypeAnnotation::Dynamic | TypeAnnotation::Class(_) => 0,
    }
}

/// Collect function bodies in the same order as `sym.functions`.
/// Nested functions are pushed before the function that contains them.
fn collect_func_bodies(stmts: &[Stmt]) -> Vec<Vec<Stmt>> {
    let mut out = Vec::new();
    collect_funcs(stmts, &mut out);
    out
}

fn collect_funcs(stmts: &[Stmt], out: &mut Vec<Vec<Stmt>>) {
    for s in stmts {
        collect_funcs_stmt(s, out);
    }
}

fn collect_funcs_stmt(stmt: &Stmt, out: &mut Vec<Vec<Stmt>>) {
    match stmt {
        Stmt::FuncDef { body, .. } => {
            collect_funcs(body, out);
            out.push(body.clone());
        }
        Stmt::ClassDef { body, .. } => collect_funcs(body, out),
        Stmt::If { then_body, elseifs, else_body, .. } => {
            collect_funcs(then_body, out);
            for (_, b) in elseifs {
                collect_funcs(b, out);
            }
            if let Some(b) = else_body {
                collect_funcs(b, out);
            }
        }
        Stmt::Switch { cases, default, .. } => {
            for (_, b) in cases {
                collect_funcs(b, out);
            }
            if let Some(b) = default {
                collect_funcs(b, out);
            }
        }
        Stmt::For { body, .. } | Stmt::While { body, .. } => collect_funcs(body, out),
        Stmt::TryCatch { try_body, catch_body, .. } => {
            collect_funcs(try_body, out);
            collect_funcs(catch_body, out);
        }
        Stmt::Block(body) => collect_funcs(body, out),
        _ => {}
    }
}
