//! Cranelift JIT backend – compiles HIR to native code via Cranelift.

use std::collections::HashMap;

use cranelift::codegen::ir::{
    AbiParam, Block, InstBuilder, MemFlags, Signature, StackSlotData, StackSlotKind,
};
use cranelift::codegen::isa::CallConv;
use cranelift::codegen::settings;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift::prelude::*;
use cranelift_jit::JITBuilder;
use cranelift_module::{FuncId, Linkage, Module};

use crate::error::{external_error, runtime_error, SoplangError};
use crate::hir::{BinOpKind, HirConst, HirFunction, HirInstr, HirModule, SlotNum, UnOpKind};
use crate::runtime;
use crate::semantic::SymbolTable;

fn compilation_error(error: impl std::fmt::Display) -> SoplangError {
    external_error(
        "Cranelift ma dhammaystiri karin ururinta barnaamijka.",
        error.to_string(),
    )
}

struct CompiledFnInfo {
    name: String,
    n_params: usize,
    func_id: FuncId,
}

pub struct CraneliftBackend {
    runtime: runtime::RuntimeContext,
    module: OwnedJitModule,
    ctx: cranelift::codegen::Context,
    fn_ctx: FunctionBuilderContext,
    main_func_id: Option<FuncId>,
    compiled_fns: Vec<CompiledFnInfo>,
    /// Names already published into the runtime function table.
    registered_fns: usize,
    /// Unique suffix for Cranelift function declarations in this module.
    fn_seq: u32,
}

struct OwnedJitModule(Option<cranelift_jit::JITModule>);

impl std::ops::Deref for OwnedJitModule {
    type Target = cranelift_jit::JITModule;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("JIT module is alive")
    }
}

impl std::ops::DerefMut for OwnedJitModule {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("JIT module is alive")
    }
}

impl Drop for OwnedJitModule {
    fn drop(&mut self) {
        if let Some(module) = self.0.take() {
            // The backend owns its function registry and exposes no code pointers.
            // Its runtime is dropped first, and synchronous execution has returned.
            unsafe { module.free_memory() };
        }
    }
}

fn register_runtime_symbols(builder: &mut JITBuilder) {
    macro_rules! sym {
        ($b:expr, $name:ident) => {
            #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
            $b.symbol(stringify!($name), runtime::windows_abi::$name as *const u8);
            #[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
            $b.symbol(stringify!($name), runtime::$name as *const u8);
        };
    }
    sym!(builder, soplang_set_location);
    sym!(builder, soplang_enter_roots);
    sym!(builder, soplang_gc_safepoint);
    sym!(builder, soplang_leave_roots);
    sym!(builder, soplang_int);
    sym!(builder, soplang_float);
    sym!(builder, soplang_str);
    sym!(builder, soplang_bool);
    sym!(builder, soplang_null);
    sym!(builder, soplang_add);
    sym!(builder, soplang_sub);
    sym!(builder, soplang_mul);
    sym!(builder, soplang_div);
    sym!(builder, soplang_mod);
    sym!(builder, soplang_neg);
    sym!(builder, soplang_not);
    sym!(builder, soplang_eq);
    sym!(builder, soplang_ne);
    sym!(builder, soplang_lt);
    sym!(builder, soplang_le);
    sym!(builder, soplang_gt);
    sym!(builder, soplang_ge);
    sym!(builder, soplang_and);
    sym!(builder, soplang_or);
    sym!(builder, soplang_truthy);
    sym!(builder, soplang_range_continue);
    sym!(builder, soplang_qor);
    sym!(builder, soplang_gelin);
    sym!(builder, soplang_nooc);
    sym!(builder, soplang_list_new);
    sym!(builder, soplang_list_push);
    sym!(builder, soplang_object_new);
    sym!(builder, soplang_get_index);
    sym!(builder, soplang_set_index);
    sym!(builder, soplang_get_prop);
    sym!(builder, soplang_set_prop);
    sym!(builder, soplang_call);
    sym!(builder, soplang_get_builtin);
    sym!(builder, soplang_store_global);
    sym!(builder, soplang_call_method);
    sym!(builder, soplang_check_type);
    sym!(builder, soplang_check_class);
    sym!(builder, soplang_mark_const);
    sym!(builder, soplang_new_instance);
    sym!(builder, soplang_make_closure);
    sym!(builder, soplang_capture_get);
    sym!(builder, soplang_register_class);
    sym!(builder, soplang_raise);
    sym!(builder, soplang_suspend_error);
    sym!(builder, soplang_restore_error);
    sym!(builder, soplang_discard_error);
    sym!(builder, soplang_try_begin);
    sym!(builder, soplang_try_end);
    sym!(builder, soplang_has_pending_error);
    sym!(builder, soplang_get_caught_error);
    sym!(builder, soplang_get_caught_error_value);
    sym!(builder, soplang_enter_function);
    sym!(builder, soplang_leave_function);
}

// JIT allocations can be farther apart than AArch64's direct branch range,
// especially across persistent REPL compilations. Request a far-call sequence.
fn declare_jit_function(
    module: &mut cranelift_jit::JITModule,
    id: FuncId,
    function: &mut cranelift::codegen::ir::Function,
) -> cranelift::codegen::ir::FuncRef {
    let reference = module.declare_func_in_func(id, function);
    function.dfg.ext_funcs[reference].colocated = false;
    reference
}

fn make_sig(n_params: usize, n_returns: usize) -> Signature {
    let mut sig = Signature::new(CallConv::SystemV);
    for _ in 0..n_params {
        sig.params.push(AbiParam::new(types::I64));
    }
    for _ in 0..n_returns {
        sig.returns.push(AbiParam::new(types::I64));
    }
    sig
}

fn max_slot_in_body(body: &[HirInstr]) -> usize {
    let mut mx = 0usize;
    for instr in body {
        match instr {
            HirInstr::Const { dst, .. }
            | HirInstr::Load { dst, .. }
            | HirInstr::Pop { dst }
            | HirInstr::BindError { dst }
            | HirInstr::BindErrorValue { dst } => {
                mx = mx.max(*dst + 1);
            }
            HirInstr::Copy { dst, src } => {
                mx = mx.max(*dst + 1).max(*src + 1);
            }
            HirInstr::Store { src, .. } => {
                mx = mx.max(*src + 1);
            }
            HirInstr::BinOp { dst, lhs, rhs, .. } => {
                mx = mx.max(*dst + 1).max(*lhs + 1).max(*rhs + 1);
            }
            HirInstr::UnOp { dst, src, .. } => {
                mx = mx.max(*dst + 1).max(*src + 1);
            }
            HirInstr::BuildList { dst, items } => {
                mx = mx.max(*dst + 1);
                for s in items {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::BuildObject { dst, pairs } => {
                mx = mx.max(*dst + 1);
                for (_, s) in pairs {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::GetIndex { dst, obj, idx } => {
                mx = mx.max(*dst + 1).max(*obj + 1).max(*idx + 1);
            }
            HirInstr::SetIndex { obj, idx, val } => {
                mx = mx.max(*obj + 1).max(*idx + 1).max(*val + 1);
            }
            HirInstr::GetProp { dst, obj, .. } => {
                mx = mx.max(*dst + 1).max(*obj + 1);
            }
            HirInstr::SetProp { obj, val, .. } => {
                mx = mx.max(*obj + 1).max(*val + 1);
            }
            HirInstr::Call { dst, callee, args } => {
                mx = mx.max(*dst + 1).max(*callee + 1);
                for s in args {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::CallMethod { dst, obj, args, .. } => {
                mx = mx.max(*dst + 1).max(*obj + 1);
                for s in args {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::Return { val } | HirInstr::Raise { src: val } => {
                mx = mx.max(*val + 1);
            }
            HirInstr::JumpIf { cond, .. } => {
                mx = mx.max(*cond + 1);
            }
            HirInstr::RangeContinue {
                dst,
                current,
                end,
                step,
            } => {
                mx = mx
                    .max(*dst + 1)
                    .max(*current + 1)
                    .max(*end + 1)
                    .max(*step + 1);
            }
            HirInstr::CheckType { src, .. } | HirInstr::CheckClass { src, .. } => {
                mx = mx.max(*src + 1);
            }
            HirInstr::NewInstance { dst, args, .. } => {
                mx = mx.max(*dst + 1);
                for s in args {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::MakeClosure { dst, captures, .. } => {
                mx = mx.max(*dst + 1);
                for s in captures {
                    mx = mx.max(*s + 1);
                }
            }
            _ => {}
        }
    }
    mx
}

impl CraneliftBackend {
    pub fn new() -> Result<Self, SoplangError> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(compilation_error)?;
        let isa_builder =
            cranelift_native::builder().map_err(compilation_error)?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(compilation_error)?;

        let mut jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        register_runtime_symbols(&mut jit_builder);

        let module = OwnedJitModule(Some(cranelift_jit::JITModule::new(jit_builder)));
        let ctx = module.make_context();

        Ok(Self {
            runtime: runtime::RuntimeContext::new(),
            module,
            ctx,
            fn_ctx: FunctionBuilderContext::new(),
            main_func_id: None,
            compiled_fns: Vec::new(),
            registered_fns: 0,
            fn_seq: 0,
        })
    }

    pub fn compile_module(
        &mut self,
        hir: &HirModule,
        sym: &SymbolTable,
    ) -> Result<(), SoplangError> {
        let _scope = self.runtime.activate();
        runtime::clear_class_registry();
        self.compile_new(hir)?;
        runtime::register_classes_from_sym(&sym.classes);
        Ok(())
    }

    /// Compile another snippet into this module without dropping earlier functions or classes.
    pub fn compile_incremental(
        &mut self,
        hir: &HirModule,
        sym: &SymbolTable,
    ) -> Result<(), SoplangError> {
        let _scope = self.runtime.activate();
        self.compile_new(hir)?;
        runtime::merge_classes_from_sym(&sym.classes);
        Ok(())
    }

    fn compile_new(&mut self, hir: &HirModule) -> Result<(), SoplangError> {
        for f in hir.functions.iter() {
            self.compile_function(f)?;
        }
        self.compile_top_level(&hir.top_level, &hir.top_nums)?;
        self.module
            .finalize_definitions()
            .map_err(compilation_error)?;
        self.register_compiled_fns();
        Ok(())
    }

    pub fn run_main(&self) -> Result<(), SoplangError> {
        let _scope = self.runtime.activate();
        let func_id = self
            .main_func_id
            .ok_or_else(|| runtime_error("Hawsha 'main' weli lama ururin.", 0, 0))?;
        let code = self.module.get_finalized_function(func_id);
        if code.is_null() {
            return Err(runtime_error("Hawsha 'main' weli diyaar uma aha fulin.", 0, 0));
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        let main_fn: extern "sysv64" fn() = unsafe { std::mem::transmute(code) };
        #[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
        let main_fn: extern "C" fn() = unsafe { std::mem::transmute(code) };
        self.runtime.execute(|| main_fn())
    }

    /// Inspect a global in this backend's execution environment.
    pub fn get_global(&self, name: &str) -> Option<runtime::Value> {
        self.runtime.get_global(name)
    }

    pub fn collect(&self) -> runtime::CollectionReport {
        self.runtime.collect()
    }

    pub fn memory_stats(&self) -> runtime::MemoryStats {
        self.runtime.memory_stats()
    }

    fn register_compiled_fns(&mut self) {
        let start = self.registered_fns;
        for fi in &self.compiled_fns[start..] {
            let ptr = self.module.get_finalized_function(fi.func_id);
            if !ptr.is_null() {
                let idx = runtime::register_compiled_fn(ptr, fi.n_params);
                let sv = runtime::SoplangValue {
                    tag: runtime::TAG_FUNC,
                    _pad: [0; 7],
                    payload: idx,
                };
                runtime::store_global(&fi.name, sv);
            }
        }
        self.registered_fns = self.compiled_fns.len();
    }

    fn compile_top_level(
        &mut self,
        body: &[HirInstr],
        slot_nums: &[SlotNum],
    ) -> Result<(), SoplangError> {
        self.fn_seq += 1;
        let seq = self.fn_seq;
        let sig = Signature::new(CallConv::SystemV);
        self.ctx.func = cranelift::codegen::ir::Function::with_name_signature(
            cranelift::codegen::ir::UserFuncName::user(0, seq),
            sig,
        );

        let fn_name_map: HashMap<String, usize> = self
            .compiled_fns
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        let num_slots = max_slot_in_body(body);
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.fn_ctx);
        let entry = builder.create_block();
        builder.switch_to_block(entry);

        let (var_tags, var_pays) = declare_slot_vars(&mut builder, num_slots);

        compile_body(
            &mut builder,
            &mut self.module,
            body,
            &var_tags,
            &var_pays,
            &self.compiled_fns,
            &fn_name_map,
            false,
            slot_nums,
        )?;

        builder.seal_all_blocks();
        builder.finalize();

        let id = self
            .module
            .declare_function(
                &format!("soplang_entry_{seq}"),
                Linkage::Export,
                &self.ctx.func.signature,
            )
            .map_err(compilation_error)?;
        self.main_func_id = Some(id);
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(compilation_error)?;
        self.module.clear_context(&mut self.ctx);
        Ok(())
    }

    fn compile_function(&mut self, f: &HirFunction) -> Result<(), SoplangError> {
        self.fn_seq += 1;
        let seq = self.fn_seq;
        let has_env = !f.capture_slots.is_empty();
        let env_params = if has_env { 1 } else { 0 };
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::I64)); // tagged argument vector
        sig.params.push(AbiParam::new(types::I64)); // number of values
        sig.returns.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));

        self.ctx.func = cranelift::codegen::ir::Function::with_name_signature(
            cranelift::codegen::ir::UserFuncName::user(0, seq),
            sig,
        );

        let fn_name_map: HashMap<String, usize> = self
            .compiled_fns
            .iter()
            .enumerate()
            .map(|(i, fi)| (fi.name.clone(), i))
            .collect();

        let num_slots = max_slot_in_body(&f.body).max(f.local_count);
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.fn_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);

        let (var_tags, var_pays) = declare_slot_vars(&mut builder, num_slots);

        let name_ptr = builder.ins().iconst(types::I64, f.name.as_ptr() as i64);
        let name_len = builder.ins().iconst(types::I64, f.name.len() as i64);
        call_rt_void(&mut builder, &mut self.module, "soplang_enter_function", &[name_ptr, name_len])?;
        let arguments = builder.block_params(entry)[0];
        if has_env {
            let env_ptr = builder.ins().load(types::I64, MemFlags::trusted(), arguments, 8);
            for (i, &slot) in f.capture_slots.iter().enumerate() {
                let idx = builder.ins().iconst(types::I64, i as i64);
                let (t, p) = call_rt(
                    &mut builder,
                    &mut self.module,
                    "soplang_capture_get",
                    &[env_ptr, idx],
                )?;
                builder.def_var(var_tags[slot], t);
                builder.def_var(var_pays[slot], p);
            }
        }
        for (i, &slot) in f.params.iter().enumerate() {
            let offset = ((env_params + i) * 16) as i32;
            let tag = builder.ins().load(types::I8, MemFlags::trusted(), arguments, offset);
            let tag = builder.ins().uextend(types::I64, tag);
            let payload = builder.ins().load(types::I64, MemFlags::trusted(), arguments, offset + 8);
            builder.def_var(var_tags[slot], tag);
            builder.def_var(var_pays[slot], payload);
        }

        compile_body(
            &mut builder,
            &mut self.module,
            &f.body,
            &var_tags,
            &var_pays,
            &self.compiled_fns,
            &fn_name_map,
            true,
            &f.local_nums,
        )?;

        builder.seal_all_blocks();
        builder.finalize();

        let name = format!("soplang_fn_{seq}");
        let id = self
            .module
            .declare_function(&name, Linkage::Export, &self.ctx.func.signature)
            .map_err(compilation_error)?;
        self.compiled_fns.push(CompiledFnInfo {
            name: f.name.clone(),
            n_params: f.params.len() + env_params,
            func_id: id,
        });
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(compilation_error)?;
        self.module.clear_context(&mut self.ctx);
        Ok(())
    }
}

fn declare_slot_vars(
    builder: &mut FunctionBuilder,
    num_slots: usize,
) -> (Vec<Variable>, Vec<Variable>) {
    let mut var_tags = Vec::with_capacity(num_slots);
    let mut var_pays = Vec::with_capacity(num_slots);
    let zero = builder.ins().iconst(types::I64, 0);
    for _ in 0..num_slots {
        let tv = builder.declare_var(types::I64);
        let pv = builder.declare_var(types::I64);
        builder.def_var(tv, zero);
        builder.def_var(pv, zero);
        var_tags.push(tv);
        var_pays.push(pv);
    }
    (var_tags, var_pays)
}

fn compile_body(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    body: &[HirInstr],
    var_tags: &[Variable],
    var_pays: &[Variable],
    compiled_fns: &[CompiledFnInfo],
    fn_name_map: &HashMap<String, usize>,
    is_function: bool,
    seed: &[SlotNum],
) -> Result<(), SoplangError> {
    // Keep a tagged shadow of SSA slots visible to collection safepoints.
    let root_slot = builder.func.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot, (var_tags.len().max(1) * 16) as u32, 3,
    ));
    let root_base = builder.ins().stack_addr(types::I64, root_slot, 0);
    for slot in 0..var_tags.len() {
        let tag = builder.use_var(var_tags[slot]);
        let payload = builder.use_var(var_pays[slot]);
        let offset = (slot * 16) as i32;
        builder.ins().store(MemFlags::trusted(), tag, root_base, offset);
        builder.ins().store(MemFlags::trusted(), payload, root_base, offset + 8);
    }
    let payload_base = builder.ins().iadd_imm(root_base, 8);
    let root_count = builder.ins().iconst(types::I64, var_tags.len() as i64);
    let stride = builder.ins().iconst(types::I64, 2);
    call_rt_void(builder, module, "soplang_enter_roots", &[root_base, payload_base, root_count, stride])?;
    let mut block_map: HashMap<usize, Block> = HashMap::new();
    for instr in body {
        if let HirInstr::Label(id) = instr {
            block_map.insert(*id, builder.create_block());
        }
    }

    // Track which slots hold known compiled function references
    let mut slot_compiled_fn: HashMap<usize, usize> = HashMap::new();
    let mut kinds = vec![SlotNum::Dyn; var_tags.len()];
    let mut seeded = vec![false; var_tags.len()];
    for (i, kind) in seed.iter().enumerate() {
        if i < kinds.len() {
            kinds[i] = *kind;
            seeded[i] = *kind != SlotNum::Dyn;
        }
    }
    let mut terminated = false;

    for instr in body {
        match instr {
            HirInstr::SetLocation(span) => {
                if terminated {
                    continue;
                }
                let file = span
                    .file
                    .as_ref()
                    .and_then(|p| p.to_str())
                    .unwrap_or("<command>");
                let args = [
                    file.as_ptr() as i64,
                    file.len() as i64,
                    span.line as i64,
                    span.col as i64,
                    span.end_col as i64,
                    span.source_line.as_ptr() as i64,
                    span.source_line.len() as i64,
                ]
                .map(|value| builder.ins().iconst(types::I64, value));
                let fid = module
                    .declare_function("soplang_set_location", Linkage::Import, &make_sig(7, 0))
                    .map_err(compilation_error)?;
                let fref = declare_jit_function(module, fid, builder.func);
                builder.ins().call(fref, &args);
            }

            HirInstr::Label(id) => {
                slot_compiled_fn.clear();
                let blk = block_map[id];
                if !terminated {
                    builder.ins().jump(blk, &[]);
                }
                builder.switch_to_block(blk);
                terminated = false;
            }

            HirInstr::Const { dst, val } => {
                if terminated {
                    continue;
                }
                let (t, p) = emit_const(builder, module, val)?;
                builder.def_var(var_tags[*dst], t);
                builder.def_var(var_pays[*dst], p);
                if *dst < seeded.len() && !seeded[*dst] {
                    kinds[*dst] = match val {
                        HirConst::Int(_) => SlotNum::Int,
                        HirConst::Float(_) => SlotNum::Float,
                        _ => SlotNum::Dyn,
                    };
                }
            }

            HirInstr::Copy { dst, src } => {
                if terminated {
                    continue;
                }
                let t = builder.use_var(var_tags[*src]);
                let p = builder.use_var(var_pays[*src]);
                builder.def_var(var_tags[*dst], t);
                builder.def_var(var_pays[*dst], p);
                if *dst < seeded.len() && !seeded[*dst] {
                    kinds[*dst] = kinds.get(*src).copied().unwrap_or(SlotNum::Dyn);
                }
                if let Some(&fn_idx) = slot_compiled_fn.get(src) {
                    slot_compiled_fn.insert(*dst, fn_idx);
                } else {
                    slot_compiled_fn.remove(dst);
                }
            }

            HirInstr::Load { dst, name } => {
                if terminated {
                    continue;
                }
                if let Some(&fn_idx) = fn_name_map.get(name.as_str()) {
                    slot_compiled_fn.insert(*dst, fn_idx);
                } else {
                    slot_compiled_fn.remove(dst);
                }
                if !seeded[*dst] {
                    kinds[*dst] = SlotNum::Dyn;
                }
                let ptr_val = builder.ins().iconst(types::I64, name.as_ptr() as i64);
                let len_val = builder.ins().iconst(types::I64, name.len() as i64);
                let (t, p) = call_rt(builder, module, "soplang_get_builtin", &[ptr_val, len_val])?;
                builder.def_var(var_tags[*dst], t);
                builder.def_var(var_pays[*dst], p);
            }

            HirInstr::Store { name, src } => {
                if terminated {
                    continue;
                }
                let st = builder.use_var(var_tags[*src]);
                let sp = builder.use_var(var_pays[*src]);
                let nptr = builder.ins().iconst(types::I64, name.as_ptr() as i64);
                let nlen = builder.ins().iconst(types::I64, name.len() as i64);
                emit_store_global(builder, module, nptr, nlen, st, sp)?;
            }

            HirInstr::BinOp {
                dst,
                op,
                lhs,
                rhs,
                typed,
            } => {
                if terminated {
                    continue;
                }
                let lt = builder.use_var(var_tags[*lhs]);
                let lp = builder.use_var(var_pays[*lhs]);
                let rt = builder.use_var(var_tags[*rhs]);
                let rp = builder.use_var(var_pays[*rhs]);
                let lk = kinds.get(*lhs).copied().unwrap_or(SlotNum::Dyn);
                let rk = kinds.get(*rhs).copied().unwrap_or(SlotNum::Dyn);
                let native = if *typed && lk != SlotNum::Dyn && rk != SlotNum::Dyn {
                    emit_native_binop(builder, module, *op, lk, rk, lt, lp, rt, rp)?
                } else {
                    None
                };
                let (dt, dp, result_kind) = if let Some((dt, dp, result_kind)) = native {
                    (dt, dp, result_kind)
                } else {
                    let fname = match op {
                        BinOpKind::Add => "soplang_add",
                        BinOpKind::Sub => "soplang_sub",
                        BinOpKind::Mul => "soplang_mul",
                        BinOpKind::Div => "soplang_div",
                        BinOpKind::Mod => "soplang_mod",
                        BinOpKind::Eq => "soplang_eq",
                        BinOpKind::Ne => "soplang_ne",
                        BinOpKind::Lt => "soplang_lt",
                        BinOpKind::Le => "soplang_le",
                        BinOpKind::Gt => "soplang_gt",
                        BinOpKind::Ge => "soplang_ge",
                        BinOpKind::And => "soplang_and",
                        BinOpKind::Or => "soplang_or",
                    };
                    let (dt, dp) = call_rt(builder, module, fname, &[lt, lp, rt, rp])?;
                    (dt, dp, SlotNum::Dyn)
                };
                builder.def_var(var_tags[*dst], dt);
                builder.def_var(var_pays[*dst], dp);
                if *dst < seeded.len() && !seeded[*dst] {
                    kinds[*dst] = result_kind;
                }
            }

            HirInstr::UnOp { dst, op, src } => {
                if terminated {
                    continue;
                }
                let st = builder.use_var(var_tags[*src]);
                let sp = builder.use_var(var_pays[*src]);
                let sk = kinds.get(*src).copied().unwrap_or(SlotNum::Dyn);
                let native = if *op == UnOpKind::Neg && sk != SlotNum::Dyn {
                    Some(emit_native_neg(builder, sk, sp))
                } else {
                    None
                };
                let (dt, dp, result_kind) = if let Some((dt, dp, result_kind)) = native {
                    (dt, dp, result_kind)
                } else {
                    let fname = match op {
                        UnOpKind::Neg => "soplang_neg",
                        UnOpKind::Not => "soplang_not",
                    };
                    let (dt, dp) = call_rt(builder, module, fname, &[st, sp])?;
                    (dt, dp, SlotNum::Dyn)
                };
                builder.def_var(var_tags[*dst], dt);
                builder.def_var(var_pays[*dst], dp);
                if *dst < seeded.len() && !seeded[*dst] {
                    kinds[*dst] = result_kind;
                }
            }

            HirInstr::BuildList { dst, items } => {
                if terminated {
                    continue;
                }
                let (mut lt, mut lp) = call_rt(builder, module, "soplang_list_new", &[])?;
                for &item in items {
                    let it = builder.use_var(var_tags[item]);
                    let ip = builder.use_var(var_pays[item]);
                    let r = call_rt(builder, module, "soplang_list_push", &[lt, lp, it, ip])?;
                    lt = r.0;
                    lp = r.1;
                }
                builder.def_var(var_tags[*dst], lt);
                builder.def_var(var_pays[*dst], lp);
            }

            HirInstr::BuildObject { dst, pairs } => {
                if terminated {
                    continue;
                }
                let (mut ot, mut opay) = call_rt(builder, module, "soplang_object_new", &[])?;
                for (key, slot) in pairs {
                    let vt = builder.use_var(var_tags[*slot]);
                    let vp = builder.use_var(var_pays[*slot]);
                    let kptr = builder.ins().iconst(types::I64, key.as_ptr() as i64);
                    let klen = builder.ins().iconst(types::I64, key.len() as i64);
                    let r = call_rt(
                        builder,
                        module,
                        "soplang_set_prop",
                        &[ot, opay, kptr, klen, vt, vp],
                    )?;
                    ot = r.0;
                    opay = r.1;
                }
                builder.def_var(var_tags[*dst], ot);
                builder.def_var(var_pays[*dst], opay);
            }

            HirInstr::GetIndex { dst, obj, idx } => {
                if terminated {
                    continue;
                }
                let ot = builder.use_var(var_tags[*obj]);
                let opay = builder.use_var(var_pays[*obj]);
                let it = builder.use_var(var_tags[*idx]);
                let ip = builder.use_var(var_pays[*idx]);
                let (dt, dp) = call_rt(builder, module, "soplang_get_index", &[ot, opay, it, ip])?;
                builder.def_var(var_tags[*dst], dt);
                builder.def_var(var_pays[*dst], dp);
            }

            HirInstr::SetIndex { obj, idx, val } => {
                if terminated {
                    continue;
                }
                let ot = builder.use_var(var_tags[*obj]);
                let opay = builder.use_var(var_pays[*obj]);
                let it = builder.use_var(var_tags[*idx]);
                let ip = builder.use_var(var_pays[*idx]);
                let vt = builder.use_var(var_tags[*val]);
                let vp = builder.use_var(var_pays[*val]);
                let _ = call_rt(
                    builder,
                    module,
                    "soplang_set_index",
                    &[ot, opay, it, ip, vt, vp],
                )?;
            }

            HirInstr::GetProp { dst, obj, prop } => {
                if terminated {
                    continue;
                }
                let ot = builder.use_var(var_tags[*obj]);
                let opay = builder.use_var(var_pays[*obj]);
                let kptr = builder.ins().iconst(types::I64, prop.as_ptr() as i64);
                let klen = builder.ins().iconst(types::I64, prop.len() as i64);
                let (dt, dp) =
                    call_rt(builder, module, "soplang_get_prop", &[ot, opay, kptr, klen])?;
                builder.def_var(var_tags[*dst], dt);
                builder.def_var(var_pays[*dst], dp);
            }

            HirInstr::SetProp { obj, prop, val } => {
                if terminated {
                    continue;
                }
                let ot = builder.use_var(var_tags[*obj]);
                let opay = builder.use_var(var_pays[*obj]);
                let vt = builder.use_var(var_tags[*val]);
                let vp = builder.use_var(var_pays[*val]);
                let kptr = builder.ins().iconst(types::I64, prop.as_ptr() as i64);
                let klen = builder.ins().iconst(types::I64, prop.len() as i64);
                let _ = call_rt(
                    builder,
                    module,
                    "soplang_set_prop",
                    &[ot, opay, kptr, klen, vt, vp],
                )?;
            }

            HirInstr::Call { dst, callee, args } => {
                if terminated {
                    continue;
                }
                let direct_fn = slot_compiled_fn
                    .get(callee)
                    .copied()
                    .filter(|&idx| args.len() == compiled_fns[idx].n_params);
                if let Some(fn_idx) = direct_fn {
                    let fi = &compiled_fns[fn_idx];
                    let args: Vec<(Value, Value)> = args.iter()
                        .map(|slot| (builder.use_var(var_tags[*slot]), builder.use_var(var_pays[*slot])))
                        .collect();
                    let (dt, dp) = emit_direct_call(builder, module, &fi.func_id, &args)?;
                    builder.def_var(var_tags[*dst], dt);
                    builder.def_var(var_pays[*dst], dp);
                } else {
                    let ct = builder.use_var(var_tags[*callee]);
                    let cp = builder.use_var(var_pays[*callee]);
                    let arg_vals: Vec<(Value, Value)> = args
                        .iter()
                        .map(|a| (builder.use_var(var_tags[*a]), builder.use_var(var_pays[*a])))
                        .collect();
                    let (dt, dp) = emit_soplang_call(builder, module, ct, cp, &arg_vals)?;
                    builder.def_var(var_tags[*dst], dt);
                    builder.def_var(var_pays[*dst], dp);
                }
            }

            HirInstr::CallMethod {
                dst,
                obj,
                method,
                args,
            } => {
                if terminated {
                    continue;
                }
                let ot = builder.use_var(var_tags[*obj]);
                let opay = builder.use_var(var_pays[*obj]);
                let arg_vals: Vec<(Value, Value)> = args
                    .iter()
                    .map(|a| (builder.use_var(var_tags[*a]), builder.use_var(var_pays[*a])))
                    .collect();
                let (dt, dp) = emit_call_method(builder, module, ot, opay, method, &arg_vals)?;
                builder.def_var(var_tags[*dst], dt);
                builder.def_var(var_pays[*dst], dp);
            }

            HirInstr::Jump(target) => {
                if terminated {
                    continue;
                }
                builder.ins().jump(block_map[target], &[]);
                terminated = true;
            }

            HirInstr::JumpIf {
                cond,
                on_true,
                on_false,
            } => {
                if terminated {
                    continue;
                }
                let t = builder.use_var(var_tags[*cond]);
                let p = builder.use_var(var_pays[*cond]);
                let (_, p) = call_rt(builder, module, "soplang_truthy", &[t, p])?;
                let zero = builder.ins().iconst(types::I64, 0);
                let test = builder.ins().icmp(IntCC::NotEqual, p, zero);
                builder
                    .ins()
                    .brif(test, block_map[on_true], &[], block_map[on_false], &[]);
                terminated = true;
            }

            HirInstr::RangeContinue {
                dst,
                current,
                end,
                step,
            } => {
                if terminated {
                    continue;
                }
                let args = [
                    builder.use_var(var_tags[*current]),
                    builder.use_var(var_pays[*current]),
                    builder.use_var(var_tags[*end]),
                    builder.use_var(var_pays[*end]),
                    builder.use_var(var_tags[*step]),
                    builder.use_var(var_pays[*step]),
                ];
                let (t, p) = call_rt(builder, module, "soplang_range_continue", &args)?;
                builder.def_var(var_tags[*dst], t);
                builder.def_var(var_pays[*dst], p);
            }

            HirInstr::Return { val } => {
                if terminated {
                    continue;
                }
                let t = builder.use_var(var_tags[*val]);
                let p = builder.use_var(var_pays[*val]);
                call_rt_void(builder, module, "soplang_leave_function", &[])?;
                call_rt_void(builder, module, "soplang_leave_roots", &[])?;
                builder.ins().return_(&[t, p]);
                terminated = true;
            }

            HirInstr::Break(target) | HirInstr::Continue(target) => {
                if terminated {
                    continue;
                }
                builder.ins().jump(block_map[target], &[]);
                terminated = true;
            }

            HirInstr::CheckType {
                src,
                type_tag,
                name,
            } => {
                if terminated {
                    continue;
                }
                let st = builder.use_var(var_tags[*src]);
                let sp = builder.use_var(var_pays[*src]);
                let et = builder.ins().iconst(types::I64, *type_tag as i64);
                let nptr = builder.ins().iconst(types::I64, name.as_ptr() as i64);
                let nlen = builder.ins().iconst(types::I64, name.len() as i64);
                let sig = make_sig(5, 0);
                let fid = module
                    .declare_function("soplang_check_type", Linkage::Import, &sig)
                    .map_err(compilation_error)?;
                let fref = declare_jit_function(module, fid, builder.func);
                builder.ins().call(fref, &[st, sp, et, nptr, nlen]);
            }

            HirInstr::CheckClass {
                src,
                class_name,
                var_name,
            } => {
                if terminated {
                    continue;
                }
                let st = builder.use_var(var_tags[*src]);
                let sp = builder.use_var(var_pays[*src]);
                let cptr = builder.ins().iconst(types::I64, class_name.as_ptr() as i64);
                let clen = builder.ins().iconst(types::I64, class_name.len() as i64);
                let nptr = builder.ins().iconst(types::I64, var_name.as_ptr() as i64);
                let nlen = builder.ins().iconst(types::I64, var_name.len() as i64);
                let sig = make_sig(6, 0);
                let fid = module
                    .declare_function("soplang_check_class", Linkage::Import, &sig)
                    .map_err(compilation_error)?;
                let fref = declare_jit_function(module, fid, builder.func);
                builder.ins().call(fref, &[st, sp, cptr, clen, nptr, nlen]);
            }

            HirInstr::MarkConst { name } => {
                if terminated {
                    continue;
                }
                let nptr = builder.ins().iconst(types::I64, name.as_ptr() as i64);
                let nlen = builder.ins().iconst(types::I64, name.len() as i64);
                let sig = make_sig(2, 0);
                let fid = module
                    .declare_function("soplang_mark_const", Linkage::Import, &sig)
                    .map_err(compilation_error)?;
                let fref = declare_jit_function(module, fid, builder.func);
                builder.ins().call(fref, &[nptr, nlen]);
            }

            HirInstr::NewInstance {
                dst,
                class_name,
                args,
            } => {
                if terminated {
                    continue;
                }
                let arg_vals: Vec<(Value, Value)> = args
                    .iter()
                    .map(|a| (builder.use_var(var_tags[*a]), builder.use_var(var_pays[*a])))
                    .collect();
                let (dt, dp) = emit_new_instance(builder, module, class_name, &arg_vals)?;
                builder.def_var(var_tags[*dst], dt);
                builder.def_var(var_pays[*dst], dp);
            }

            HirInstr::Pop { .. } => {}

            HirInstr::Raise { src } => {
                if terminated { continue; }
                let tag = builder.use_var(var_tags[*src]);
                let pay = builder.use_var(var_pays[*src]);
                call_rt(builder, module, "soplang_raise", &[tag, pay])?;
            }
            HirInstr::SuspendError | HirInstr::RestoreError | HirInstr::DiscardError => {
                if terminated { continue; }
                let name = match instr {
                    HirInstr::SuspendError => "soplang_suspend_error",
                    HirInstr::RestoreError => "soplang_restore_error",
                    _ => "soplang_discard_error",
                };
                call_rt_void(builder, module, name, &[])?;
            }

            HirInstr::TryBegin { catch: _ } => {
                if terminated {
                    continue;
                }
                call_rt_void(builder, module, "soplang_try_begin", &[])?;
            }

            HirInstr::TryEnd => {
                if terminated {
                    continue;
                }
                call_rt_void(builder, module, "soplang_try_end", &[])?;
            }

            HirInstr::CheckThrow { catch } => {
                if terminated {
                    continue;
                }
                let catch_block = catch.map(|label| block_map[&label]).unwrap_or_else(|| builder.create_block());
                let continue_block = builder.create_block();
                let pending = call_rt_i64(builder, module, "soplang_has_pending_error", &[])?;
                let zero = builder.ins().iconst(types::I64, 0);
                let test = builder.ins().icmp(IntCC::NotEqual, pending, zero);
                builder
                    .ins()
                    .brif(test, catch_block, &[], continue_block, &[]);
                if catch.is_none() {
                    builder.switch_to_block(catch_block);
                    call_rt_void(builder, module, "soplang_leave_roots", &[])?;
                    if is_function {
                        let tag = builder.ins().iconst(types::I64, 8);
                        let payload = builder.ins().iconst(types::I64, 0);
                        call_rt_void(builder, module, "soplang_leave_function", &[])?;
                        builder.ins().return_(&[tag, payload]);
                    } else {
                        builder.ins().return_(&[]);
                    }
                }
                builder.switch_to_block(continue_block);
            }

            HirInstr::BindError { dst } | HirInstr::BindErrorValue { dst } => {
                if terminated {
                    continue;
                }
                let name = if matches!(instr, HirInstr::BindErrorValue { .. }) {
                    "soplang_get_caught_error_value"
                } else { "soplang_get_caught_error" };
                let (t, p) = call_rt(builder, module, name, &[])?;
                builder.def_var(var_tags[*dst], t);
                builder.def_var(var_pays[*dst], p);
            }

            HirInstr::MakeClosure {
                dst,
                fn_name,
                captures,
            } => {
                if terminated {
                    continue;
                }
                let arg_vals: Vec<(Value, Value)> = captures
                    .iter()
                    .map(|s| (builder.use_var(var_tags[*s]), builder.use_var(var_pays[*s])))
                    .collect();
                let (t, p) = emit_make_closure(builder, module, fn_name, &arg_vals)?;
                builder.def_var(var_tags[*dst], t);
                builder.def_var(var_pays[*dst], p);
            }
        }
        if !terminated {
            if let Some(slot) = instr.destination() {
                let tag = builder.use_var(var_tags[slot]);
                let payload = builder.use_var(var_pays[slot]);
                let offset = (slot * 16) as i32;
                builder.ins().store(MemFlags::trusted(), tag, root_base, offset);
                builder.ins().store(MemFlags::trusted(), payload, root_base, offset + 8);
            }
            if instr.may_allocate() { call_rt_void(builder, module, "soplang_gc_safepoint", &[])?; }
        }
    }

    if !terminated {
        call_rt_void(builder, module, "soplang_leave_roots", &[])?;
        if is_function {
            let nt = builder.ins().iconst(types::I64, 0);
            let np = builder.ins().iconst(types::I64, 0);
            call_rt_void(builder, module, "soplang_leave_function", &[])?;
            builder.ins().return_(&[nt, np]);
        } else {
            builder.ins().return_(&[]);
        }
    }

    Ok(())
}

/// Generic runtime call: arbitrary params → 2 returns (tag, payload).
fn call_rt(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    name: &str,
    args: &[Value],
) -> Result<(Value, Value), SoplangError> {
    let sig = make_sig(args.len(), 2);
    let fid = module
        .declare_function(name, Linkage::Import, &sig)
        .map_err(compilation_error)?;
    let fref = declare_jit_function(module, fid, builder.func);
    let call = builder.ins().call(fref, args);
    let r = builder.inst_results(call);
    Ok((r[0], r[1]))
}

fn call_rt_i64(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    name: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    let sig = make_sig(args.len(), 1);
    let fid = module
        .declare_function(name, Linkage::Import, &sig)
        .map_err(compilation_error)?;
    let fref = declare_jit_function(module, fid, builder.func);
    let call = builder.ins().call(fref, args);
    Ok(builder.inst_results(call)[0])
}

fn call_rt_void(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    name: &str,
    args: &[Value],
) -> Result<(), SoplangError> {
    let sig = make_sig(args.len(), 0);
    let fid = module
        .declare_function(name, Linkage::Import, &sig)
        .map_err(compilation_error)?;
    let fref = declare_jit_function(module, fid, builder.func);
    builder.ins().call(fref, args);
    Ok(())
}

fn emit_store_global(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    name_ptr: Value,
    name_len: Value,
    tag: Value,
    payload: Value,
) -> Result<(), SoplangError> {
    let sig = make_sig(4, 0);
    let fid = module
        .declare_function("soplang_store_global", Linkage::Import, &sig)
        .map_err(compilation_error)?;
    let fref = declare_jit_function(module, fid, builder.func);
    builder
        .ins()
        .call(fref, &[name_ptr, name_len, tag, payload]);
    Ok(())
}

fn payload_as_float(builder: &mut FunctionBuilder, kind: SlotNum, payload: Value) -> Value {
    match kind {
        SlotNum::Float => builder.ins().bitcast(types::F64, MemFlags::new(), payload),
        _ => builder.ins().fcvt_from_sint(types::F64, payload),
    }
}

fn float_to_bits(builder: &mut FunctionBuilder, val: Value) -> Value {
    builder.ins().bitcast(types::I64, MemFlags::new(), val)
}

fn emit_native_neg(
    builder: &mut FunctionBuilder,
    kind: SlotNum,
    payload: Value,
) -> (Value, Value, SlotNum) {
    match kind {
        SlotNum::Float => {
            let f = payload_as_float(builder, kind, payload);
            let n = builder.ins().fneg(f);
            (
                builder.ins().iconst(types::I64, 2),
                float_to_bits(builder, n),
                SlotNum::Float,
            )
        }
        _ => (
            builder.ins().iconst(types::I64, 1),
            builder.ins().ineg(payload),
            SlotNum::Int,
        ),
    }
}

fn emit_native_binop(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    op: BinOpKind,
    lk: SlotNum,
    rk: SlotNum,
    lt: Value,
    lp: Value,
    rt: Value,
    rp: Value,
) -> Result<Option<(Value, Value, SlotNum)>, SoplangError> {
    let use_float = matches!(op, BinOpKind::Div) || lk == SlotNum::Float || rk == SlotNum::Float;
    let result_kind = match op {
        BinOpKind::Div => SlotNum::Float,
        BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Mod if use_float => {
            SlotNum::Float
        }
        BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Mod => SlotNum::Int,
        _ => SlotNum::Dyn,
    };

    let produce = |builder: &mut FunctionBuilder| -> (Value, Value) {
        match op {
            BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul if !use_float => {
                let p = match op {
                    BinOpKind::Add => builder.ins().iadd(lp, rp),
                    BinOpKind::Sub => builder.ins().isub(lp, rp),
                    _ => builder.ins().imul(lp, rp),
                };
                (builder.ins().iconst(types::I64, 1), p)
            }
            BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Div => {
                let l = payload_as_float(builder, lk, lp);
                let r = payload_as_float(builder, rk, rp);
                let f = match op {
                    BinOpKind::Add => builder.ins().fadd(l, r),
                    BinOpKind::Sub => builder.ins().fsub(l, r),
                    BinOpKind::Mul => builder.ins().fmul(l, r),
                    _ => builder.ins().fdiv(l, r),
                };
                (
                    builder.ins().iconst(types::I64, 2),
                    float_to_bits(builder, f),
                )
            }
            BinOpKind::Mod if !use_float => (
                builder.ins().iconst(types::I64, 1),
                builder.ins().srem(lp, rp),
            ),
            BinOpKind::Mod => {
                let l = payload_as_float(builder, lk, lp);
                let r = payload_as_float(builder, rk, rp);
                let quot = builder.ins().fdiv(l, r);
                let whole = builder.ins().trunc(quot);
                let scaled = builder.ins().fmul(whole, r);
                let rem = builder.ins().fsub(l, scaled);
                (
                    builder.ins().iconst(types::I64, 2),
                    float_to_bits(builder, rem),
                )
            }
            BinOpKind::Eq
            | BinOpKind::Ne
            | BinOpKind::Lt
            | BinOpKind::Le
            | BinOpKind::Gt
            | BinOpKind::Ge => {
                let flag = if !use_float {
                    let cc = match op {
                        BinOpKind::Eq => IntCC::Equal,
                        BinOpKind::Ne => IntCC::NotEqual,
                        BinOpKind::Lt => IntCC::SignedLessThan,
                        BinOpKind::Le => IntCC::SignedLessThanOrEqual,
                        BinOpKind::Gt => IntCC::SignedGreaterThan,
                        _ => IntCC::SignedGreaterThanOrEqual,
                    };
                    builder.ins().icmp(cc, lp, rp)
                } else {
                    let l = payload_as_float(builder, lk, lp);
                    let r = payload_as_float(builder, rk, rp);
                    let cc = match op {
                        BinOpKind::Eq => FloatCC::Equal,
                        BinOpKind::Ne => FloatCC::NotEqual,
                        BinOpKind::Lt => FloatCC::LessThan,
                        BinOpKind::Le => FloatCC::LessThanOrEqual,
                        BinOpKind::Gt => FloatCC::GreaterThan,
                        _ => FloatCC::GreaterThanOrEqual,
                    };
                    builder.ins().fcmp(cc, l, r)
                };
                (
                    builder.ins().iconst(types::I64, 3),
                    builder.ins().uextend(types::I64, flag),
                )
            }
            _ => (
                builder.ins().iconst(types::I64, 0),
                builder.ins().iconst(types::I64, 0),
            ),
        }
    };

    if !matches!(op, BinOpKind::Div | BinOpKind::Mod) {
        let (t, p) = produce(builder);
        return Ok(Some((t, p, result_kind)));
    }

    let merge = builder.create_block();
    builder.append_block_param(merge, types::I64);
    builder.append_block_param(merge, types::I64);
    let zero_bb = builder.create_block();
    let ok_bb = builder.create_block();
    let is_zero = if rk == SlotNum::Int {
        let z = builder.ins().iconst(types::I64, 0);
        builder.ins().icmp(IntCC::Equal, rp, z)
    } else {
        let rv = payload_as_float(builder, rk, rp);
        let z = builder.ins().f64const(0.0);
        builder.ins().fcmp(FloatCC::Equal, rv, z)
    };
    builder.ins().brif(is_zero, zero_bb, &[], ok_bb, &[]);

    builder.switch_to_block(zero_bb);
    let fname = if op == BinOpKind::Div {
        "soplang_div"
    } else {
        "soplang_mod"
    };
    let (zt, zp) = call_rt(builder, module, fname, &[lt, lp, rt, rp])?;
    let zargs = [
        cranelift::codegen::ir::BlockArg::from(zt),
        cranelift::codegen::ir::BlockArg::from(zp),
    ];
    builder.ins().jump(merge, &zargs);

    builder.switch_to_block(ok_bb);
    let (nt, np) = produce(builder);
    let nargs = [
        cranelift::codegen::ir::BlockArg::from(nt),
        cranelift::codegen::ir::BlockArg::from(np),
    ];
    builder.ins().jump(merge, &nargs);

    builder.switch_to_block(merge);
    let params = builder.block_params(merge);
    Ok(Some((params[0], params[1], result_kind)))
}

fn emit_const(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    val: &HirConst,
) -> Result<(Value, Value), SoplangError> {
    match val {
        HirConst::Int(n) => Ok((
            builder.ins().iconst(types::I64, 1),
            builder.ins().iconst(types::I64, *n),
        )),
        HirConst::Float(x) => Ok((
            builder.ins().iconst(types::I64, 2),
            builder.ins().iconst(types::I64, x.to_bits() as i64),
        )),
        HirConst::Str(s) => {
            let ptr = builder.ins().iconst(types::I64, s.as_ptr() as i64);
            let len = builder.ins().iconst(types::I64, s.len() as i64);
            call_rt(builder, module, "soplang_str", &[ptr, len])
        }
        HirConst::Bool(b) => Ok((
            builder.ins().iconst(types::I64, 3),
            builder.ins().iconst(types::I64, if *b { 1 } else { 0 }),
        )),
        HirConst::Null => Ok((
            builder.ins().iconst(types::I64, 0),
            builder.ins().iconst(types::I64, 0),
        )),
    }
}

fn emit_direct_call(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    func_id: &FuncId,
    args: &[(Value, Value)],
) -> Result<(Value, Value), SoplangError> {
    let pointer = emit_arg_vector(builder, args);
    let count = builder.ins().iconst(types::I64, args.len() as i64);
    let fref = declare_jit_function(module, *func_id, builder.func);
    let call = builder.ins().call(fref, &[pointer, count]);
    let result = builder.inst_results(call);
    Ok((result[0], result[1]))
}

fn emit_arg_vector(builder: &mut FunctionBuilder, args: &[(Value, Value)]) -> Value {
    if args.is_empty() {
        return builder.ins().iconst(types::I64, 0);
    }
    let slot = builder.func.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        (args.len() * 16) as u32,
        3,
    ));
    let base = builder.ins().stack_addr(types::I64, slot, 0);
    for (index, (tag, payload)) in args.iter().enumerate() {
        let offset = (index * 16) as i32;
        builder.ins().store(MemFlags::trusted(), *tag, base, offset);
        builder
            .ins()
            .store(MemFlags::trusted(), *payload, base, offset + 8);
    }
    base
}

fn emit_make_closure(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    fn_name: &str,
    captures: &[(Value, Value)],
) -> Result<(Value, Value), SoplangError> {
    let nptr = builder.ins().iconst(types::I64, fn_name.as_ptr() as i64);
    let nlen = builder.ins().iconst(types::I64, fn_name.len() as i64);
    let n = captures.len();
    let n_val = builder.ins().iconst(types::I64, n as i64);
    if n == 0 {
        let null_ptr = builder.ins().iconst(types::I64, 0);
        return call_rt(
            builder,
            module,
            "soplang_make_closure",
            &[nptr, nlen, n_val, null_ptr],
        );
    }
    let size = (n * 16) as u32;
    let slot = builder.func.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        size,
        3,
    ));
    let base = builder.ins().stack_addr(types::I64, slot, 0);
    for (i, (t, p)) in captures.iter().enumerate() {
        let off = (i * 16) as i32;
        builder.ins().store(MemFlags::trusted(), *t, base, off);
        builder.ins().store(MemFlags::trusted(), *p, base, off + 8);
    }
    call_rt(
        builder,
        module,
        "soplang_make_closure",
        &[nptr, nlen, n_val, base],
    )
}

fn emit_soplang_call(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    ct: Value,
    cp: Value,
    args: &[(Value, Value)],
) -> Result<(Value, Value), SoplangError> {
    let n = args.len();
    if n == 0 {
        let null_ptr = builder.ins().iconst(types::I64, 0);
        let zero = builder.ins().iconst(types::I64, 0);
        return call_rt(builder, module, "soplang_call", &[ct, cp, null_ptr, zero]);
    }
    let size = (n * 16) as u32;
    let slot = builder.func.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        size,
        3,
    ));
    let base = builder.ins().stack_addr(types::I64, slot, 0);
    for (i, (t, p)) in args.iter().enumerate() {
        let off = (i * 16) as i32;
        builder.ins().store(MemFlags::trusted(), *t, base, off);
        builder.ins().store(MemFlags::trusted(), *p, base, off + 8);
    }
    let n_val = builder.ins().iconst(types::I64, n as i64);
    call_rt(builder, module, "soplang_call", &[ct, cp, base, n_val])
}

fn emit_call_method(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    ot: Value,
    opay: Value,
    method: &str,
    args: &[(Value, Value)],
) -> Result<(Value, Value), SoplangError> {
    let n = args.len();
    let mptr = builder.ins().iconst(types::I64, method.as_ptr() as i64);
    let mlen = builder.ins().iconst(types::I64, method.len() as i64);

    let (args_ptr, n_val) = if n == 0 {
        (
            builder.ins().iconst(types::I64, 0),
            builder.ins().iconst(types::I64, 0),
        )
    } else {
        let size = (n * 16) as u32;
        let slot = builder.func.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            size,
            3,
        ));
        let base = builder.ins().stack_addr(types::I64, slot, 0);
        for (i, (t, p)) in args.iter().enumerate() {
            let off = (i * 16) as i32;
            builder.ins().store(MemFlags::trusted(), *t, base, off);
            builder.ins().store(MemFlags::trusted(), *p, base, off + 8);
        }
        (base, builder.ins().iconst(types::I64, n as i64))
    };

    call_rt(
        builder,
        module,
        "soplang_call_method",
        &[ot, opay, mptr, mlen, args_ptr, n_val],
    )
}

fn emit_new_instance(
    builder: &mut FunctionBuilder,
    module: &mut cranelift_jit::JITModule,
    class_name: &str,
    args: &[(Value, Value)],
) -> Result<(Value, Value), SoplangError> {
    let cptr = builder.ins().iconst(types::I64, class_name.as_ptr() as i64);
    let clen = builder.ins().iconst(types::I64, class_name.len() as i64);
    let n = args.len();
    let (args_ptr, n_val) = if n == 0 {
        (
            builder.ins().iconst(types::I64, 0),
            builder.ins().iconst(types::I64, 0),
        )
    } else {
        let size = (n * 16) as u32;
        let slot = builder.func.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            size,
            3,
        ));
        let base = builder.ins().stack_addr(types::I64, slot, 0);
        for (i, (t, p)) in args.iter().enumerate() {
            let off = (i * 16) as i32;
            builder.ins().store(MemFlags::trusted(), *t, base, off);
            builder.ins().store(MemFlags::trusted(), *p, base, off + 8);
        }
        (base, builder.ins().iconst(types::I64, n as i64))
    };
    call_rt(
        builder,
        module,
        "soplang_new_instance",
        &[cptr, clen, args_ptr, n_val],
    )
}
