use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;
use inkwell::IntPredicate;
use std::collections::HashMap;

use klc_mir::mir::*;
use klc_mir::ssa::{SsaFunction, SsaInst, SsaValueId};

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    fn_value_map: HashMap<String, inkwell::values::FunctionValue<'ctx>>,
    param_values: HashMap<usize, BasicValueEnum<'ctx>>,
    alloca_map: Vec<Option<PointerValue<'ctx>>>,
    alloca_types: HashMap<usize, BasicTypeEnum<'ctx>>,
    field_ptr_allocas: Vec<Option<PointerValue<'ctx>>>,
    field_ptr_types: HashMap<usize, BasicTypeEnum<'ctx>>,
    needs_main_wrapper: bool,
    /// Local IDs holding struct values via pointer (pass-by-reference semantics).
    /// Maps local_id → the original LLVM struct type (used for GEP/load).
    ref_param_struct_types: HashMap<usize, BasicTypeEnum<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    /// Load a local's value, always preferring the alloca for cross-block correctness.
    /// Falls back to last_value_map for values that weren't stored to an alloca.
    fn load_value(
        &self,
        id: usize,
        last_values: &HashMap<usize, BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if this is a field pointer: load GEP first, then load field value
        if id < self.field_ptr_allocas.len() && self.field_ptr_allocas[id].is_some() {
            if let Some(field_ptr_alloca) = self.field_ptr_allocas[id] {
                let gep = self.builder.build_load(
                    self.context.ptr_type(Default::default()),
                    field_ptr_alloca, "_fgepload"
                ).map_err(|e| format!("load_value fptr {}: {}", id, e))?;
                let field_type = self.field_ptr_types.get(&id).or_else(|| self.alloca_types.get(&id));
                if let Some(pointee_type) = field_type {
                    let loaded = self.builder.build_load(*pointee_type, gep.into_pointer_value(), "")
                        .map_err(|e| format!("load_value field {}: {}", id, e))?;
                    return Ok(loaded);
                }
            }
        }
        if let Some(Some(ptr)) = self.alloca_map.get(id) {
            if let Some(pointee_type) = self.alloca_types.get(&id) {
                let loaded = self.builder.build_load(*pointee_type, *ptr, "")
                    .map_err(|e| format!("load_value {}: {}", id, e))?;
                return Ok(loaded);
            }
        }
        if let Some(val) = last_values.get(&id) {
            return Ok(*val);
        }
        Ok(self.context.i32_type().const_zero().as_basic_value_enum())
    }

    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name);
        Self {
            context,
            builder,
            module,
            fn_value_map: HashMap::new(),
            param_values: HashMap::new(),
            alloca_map: Vec::new(),
            alloca_types: HashMap::new(),
            field_ptr_allocas: Vec::new(),
            field_ptr_types: HashMap::new(),
            needs_main_wrapper: false,
            ref_param_struct_types: HashMap::new(),
        }
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn compile(&mut self, mir_module: &MirModule) -> Result<(), String> {
        self.declare_runtime_externs();
        for func in &mir_module.functions {
            self.declare_function(func)?;
        }
        for func in &mir_module.functions {
            self.compile_function(func)?;
        }
        if self.needs_main_wrapper {
            self.generate_main_wrapper()?;
        }
        Ok(())
    }

    // ===================================================================
    // SSA Codegen (Phase 15 — experimental)
    // ===================================================================

    pub fn compile_with_ssa(&mut self, mir_module: &MirModule) -> Result<(), String> {
        let result = klc_mir::ssa::convert_module(mir_module);
        let ssa_fns = result.ssa_functions;
        let non_ssa_fns = result.non_ssa_functions;

        self.declare_runtime_externs();

        for ssa_fn in &ssa_fns {
            self.declare_ssa_function(ssa_fn)?;
        }
        for func in &non_ssa_fns {
            self.declare_function(func)?;
        }
        for ssa_fn in &ssa_fns {
            self.compile_ssa_function(ssa_fn)?;
        }
        for func in &non_ssa_fns {
            self.compile_function(func)?;
        }
        if self.needs_main_wrapper {
            self.generate_main_wrapper()?;
        }
        if let Err(e) = self.module.verify() {
            return Err(format!("SSA verify: {}", e));
        }
        Ok(())
    }

    fn declare_ssa_function(&mut self, func: &SsaFunction) -> Result<(), String> {
        let ret_type = self.llvm_type(&func.return_type);
        let ptr_ty = self.context.ptr_type(Default::default());
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = func.params
            .iter()
            .map(|p| if matches!(p, MirType::Struct(_, _)) { ptr_ty.into() }
                 else { self.llvm_type(p).into() })
            .collect();

        let fn_name = if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            self.needs_main_wrapper = true; "kyle_main"
        } else { &func.name };
        let fn_type = ret_type.fn_type(&param_types, false);
        let fn_value = self.module.add_function(fn_name, fn_type, None);
        self.fn_value_map.insert(fn_name.to_string(), fn_value);
        Ok(())
    }

    /// Compile an SSA function, generating pure SSA LLVM IR (no allocas for promoted vars).
    ///
    /// Key design:
    /// - `block_vals[bi]` maps SsaValueId → LLVM value for each block
    /// - Each instruction reads operands directly from `block_vals[bi]` (no stale snapshot)
    /// - `alloca_current` (global) tracks the current LLVM value for each promoted alloca
    /// - Phi nodes in LLVM carry cross-block values; `alloca_current` seeds per-block start
    fn compile_ssa_function(&mut self, func: &SsaFunction) -> Result<(), String> {
        let fn_name = if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            "kyle_main"
        } else { &func.name };
        let fn_value = *self.fn_value_map.get(fn_name)
            .ok_or_else(|| format!("SSA fn '{}' not declared", fn_name))?;
        let ptr_ty = self.context.ptr_type(Default::default());

        // Pre-scan non-promotable allocas (escaping: field_ptr, heap types)
        self.alloca_types.clear();
        self.ref_param_struct_types.clear();
        self.field_ptr_allocas.clear();
        self.field_ptr_types.clear();
        for block in &func.blocks {
            for inst in &block.insts {
                if let SsaInst::Alloca { dest, type_, .. } = inst {
                    let llvm_ty = self.llvm_type(type_);
                    let actual = if let MirType::Ptr(_) = type_ { ptr_ty.as_basic_type_enum() } else { llvm_ty };
                    self.alloca_types.entry(*dest).or_insert(actual);
                }
            }
        }

        // Create LLVM basic blocks and phi nodes
        let mut block_map: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();
        let mut phi_map: Vec<(usize, inkwell::values::PhiValue<'ctx>)> = Vec::new();
        for block in &func.blocks {
            let llvm_bb = self.context.append_basic_block(fn_value, &block.label);
            block_map.insert(block.label.clone(), llvm_bb);
        }

        // Entry block: allocate non-promotable stack slots + params
        self.alloca_map.clear();
        if let Some(entry) = func.blocks.first() {
            if let Some(&entry_bb) = block_map.get(&entry.label) {
                self.builder.position_at_end(entry_bb);
                for (&dest, &ty) in &self.alloca_types {
                    while self.alloca_map.len() <= dest { self.alloca_map.push(None); }
                    let ptr = self.builder.build_alloca(ty, "")
                        .map_err(|e| format!("ssa alloca {}: {}", dest, e))?;
                    self.alloca_map[dest] = Some(ptr);
                }
                for block in &func.blocks {
                    for inst in &block.insts {
                        if let SsaInst::FieldPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest { self.field_ptr_allocas.push(None); }
                            if self.field_ptr_allocas[*dest].is_none() {
                                self.field_ptr_allocas[*dest] = Some(
                                    self.builder.build_alloca(ptr_ty, "_fgep")
                                        .map_err(|e| format!("ssa fgep: {}", e))?
                                );
                            }
                        }
                    }
                }
                for (i, p) in fn_value.get_param_iter().enumerate() {
                    self.param_values.insert(i, p);
                }
            }
        }

        // Pass 1: create phi nodes at block starts
        for block in &func.blocks {
            if let Some(&llvm_bb) = block_map.get(&block.label) {
                self.builder.position_at_end(llvm_bb);
                for phi in &block.phis {
                    let phi_type = self.llvm_type(&phi.type_);
                    if let Ok(llvm_phi) = self.builder.build_phi(phi_type, "_phi") {
                        phi_map.push((phi.dest, llvm_phi));
                    }
                }
            }
        }

        // Pass 2: compile instructions — main SSA codegen
        // GLOBAL alloca_current: tracks current LLVM value for each promoted alloca.
        // This persists across ALL blocks so values flow through loops correctly.
        let mut alloca_current: HashMap<usize, BasicValueEnum<'ctx>> = HashMap::new();
        let mut block_vals: Vec<HashMap<usize, BasicValueEnum<'ctx>>> = vec![HashMap::new(); func.blocks.len()];

        for (bi, block) in func.blocks.iter().enumerate() {
            if let Some(&llvm_bb) = block_map.get(&block.label) {
                self.builder.position_at_end(llvm_bb);

                // Seed block_vals with phi values for THIS block
                // AND update alloca_current with phi values (for succeeding blocks)
                for &(phi_id, ref phi_val) in &phi_map {
                    let bv = phi_val.as_basic_value();
                    if let Some(phi) = block.phis.iter().find(|p| p.dest == phi_id) {
                        block_vals[bi].insert(phi_id, bv);
                        alloca_current.insert(phi.alloca_id, bv);
                    }
                }

                // Compile instructions.
                let insts = block.insts.clone();

                // Inline helper: read SsaValueId from block_vals, alloca_current, const_values
                macro_rules! ssa_read {
                    ($id:expr) => {{
                        let id: SsaValueId = $id;
                        // 1. Current block's computed values
                        if let Some(&v) = block_vals[bi].get(&id) { v }
                        // 2. alloca_current (cross-block vars via promoted stores/phis)
                        else if let Some(&v) = alloca_current.get(&id) { v }
                        // 3. Constant values from const_values
                        else if let Some(c) = func.const_values.get(&id) { self.constant_to_llvm(c) }
                        // 4. Default zero
                        else { self.context.i32_type().const_zero().as_basic_value_enum() }
                    }};
                }

                // Helper: resolve MirValue to LLVM value using block_local_map
                macro_rules! resolve_mir {
                    ($val:expr) => {{
                        let val: &MirValue = $val;
                        match val {
                            MirValue::Constant(c) => self.constant_to_llvm(c),
                            MirValue::Local(mir_id) => {
                                // Map MIR local → SsaValueId using block_local_map
                                if let Some(ssa_id) = func.block_local_map.get(bi).and_then(|m| m.get(mir_id)).copied() {
                                    ssa_read!(ssa_id)
                                } else {
                                    // Fallback: try alloca_current directly
                                    alloca_current.get(mir_id).copied()
                                        .unwrap_or_else(|| self.context.i32_type().const_zero().as_basic_value_enum())
                                }
                            }
                            MirValue::Param(id) => self.param_values.get(id).copied()
                                .unwrap_or_else(|| self.context.i32_type().const_zero().as_basic_value_enum()),
                        }
                    }};
                }

                for inst in &insts {
                    match inst {
                        SsaInst::Alloca { .. } => {}
                        SsaInst::Store { dest, value } => {
                            let val = ssa_read!(*value);
                            if *dest < self.field_ptr_allocas.len() && self.field_ptr_allocas[*dest].is_some() {
                                if let Some(fpa) = self.field_ptr_allocas[*dest] {
                                    let gep = self.builder.build_load(ptr_ty, fpa, "_fgepl")
                                        .map_err(|e| format!("sfst: {}", e))?;
                                    self.builder.build_store(gep.into_pointer_value(), val)
                                        .map_err(|e| format!("sfst2: {}", e))?;
                                }
                            } else if let Some(ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(ptr, val)
                                    .map_err(|e| format!("ssast: {}", e))?;
                            } else {
                                // Promoted alloca: track in global map AND block_vals
                                alloca_current.insert(*dest, val);
                                // KEY FIX: Also insert into block_vals so phi incomings can find it
                                block_vals[bi].insert(*value, val);
                            }
                        }
                        SsaInst::Load { dest, src } => {
                            let loaded = if *src < self.field_ptr_allocas.len() && self.field_ptr_allocas[*src].is_some() {
                                if let Some(fpa) = self.field_ptr_allocas[*src] {
                                    let gep = self.builder.build_load(ptr_ty, fpa, "_fgepl")
                                        .map_err(|e| format!("sfld: {}", e))?;
                                    let lt = self.field_ptr_types.get(src)
                                        .or_else(|| self.alloca_types.get(src))
                                        .copied().unwrap_or(self.context.i64_type().as_basic_type_enum());
                                    Some(self.builder.build_load(lt, gep.into_pointer_value(), "")
                                        .map_err(|e| format!("sfld2: {}", e))?)
                                } else { None }
                            } else if let Some(ptr) = self.alloca_map.get(*src).and_then(|p| *p) {
                                if let Some(lt) = self.alloca_types.get(src) {
                                    Some(self.builder.build_load(*lt, ptr, "")
                                        .map_err(|e| format!("ssald: {}", e))?)
                                } else { None }
                            } else {
                                // Promoted alloca: read from global map (no LLVM load)
                                alloca_current.get(src).copied()
                            };
                            if let Some(v) = loaded {
                                block_vals[bi].insert(*dest, v);
                            }
                        }
                        SsaInst::BinaryOp { dest, op, left, right } => {
                            let l = ssa_read!(*left);
                            let r = ssa_read!(*right);
                            let result = self.ssa_binop(*op, l, r)?;
                            block_vals[bi].insert(*dest, result);
                        }
                        SsaInst::UnaryOp { dest, op, operand } => {
                            let val = ssa_read!(*operand);
                            let result = match *op {
                                MirUnaryOp::Neg => self.builder.build_int_neg(self.to_int_value(val), ""),
                                MirUnaryOp::Not | MirUnaryOp::BitNot => self.builder.build_not(self.to_int_value(val), ""),
                            }.map_err(|e| format!("ssaun: {}", e))?.as_basic_value_enum();
                            block_vals[bi].insert(*dest, result);
                        }
                        SsaInst::Call { dest, name, args } => {
                            let runtime_name = match name.as_str() {
                                "print" => "kl_print", "println" => "kl_println",
                                "contains" => "kl_str_contains", "to_upper" => "kl_str_to_upper",
                                "to_lower" => "kl_str_to_lower", "trim" => "kl_str_trim",
                                "replace" => "kl_str_replace", "input" => "kl_input",
                                "open" => "kl_open", "read_str" => "kl_read_str",
                                "write_str" => "kl_write_str", "close" => "kl_close",
                                "sleep" => "kl_sleep", "now" => "kl_now",
                                "char_at" => "kl_char_at", "is_digit" => "kl_is_digit",
                                "is_alpha" => "kl_is_alpha", "is_alnum" => "kl_is_alnum",
                                "is_whitespace" => "kl_is_whitespace", "is_upper" => "kl_is_upper",
                                "is_lower" => "kl_is_lower", "ord" => "kl_ord",
                                "substr" => "kl_substr",
                                "json_parse" => "kl_json_parse", "json_stringify" => "kl_json_stringify",
                                "assert" => "kl_assert", "assert_eq" => "kl_assert_eq",
                                "assert_ne" => "kl_assert_ne", "assert_str" => "kl_assert_eq",
                                _ => name,
                            };
                            if let Some(callee) = self.module.get_function(runtime_name) {
                                let fn_ty = callee.get_type();
                                let param_tys = fn_ty.get_param_types();
                                let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = args.iter()
                                    .enumerate().map(|(i, a)| {
                                        let v = ssa_read!(*a);
                                        // Auto-cast i64 → ptr when callee expects ptr
                                        if i < param_tys.len() {
                                            if let inkwell::types::BasicMetadataTypeEnum::PointerType(_) = param_tys[i] {
                                                if let BasicValueEnum::IntValue(iv) = v {
                                                    let ptr_ty = self.context.ptr_type(Default::default());
                                                    if let Ok(pv) = self.builder.build_int_to_ptr(iv, ptr_ty, "_aptr") {
                                                        return pv.into();
                                                    }
                                                }
                                            }
                                        }
                                        v.into()
                                    }).collect();
                                let call_res = self.builder.build_call(callee, &llvm_args, "")
                                    .map_err(|e| format!("ssacl {}: {}", name, e))?;
                                if let Some(d) = dest {
                                    if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                        block_vals[bi].insert(*d, rv);
                                    }
                                }
                            }
                        }
                        SsaInst::Cast { dest, value, to_type } => {
                            let val = ssa_read!(*value);
                            let target = self.llvm_type(to_type);
                            let result = self.ssa_cast(val, &target)?;
                            block_vals[bi].insert(*dest, result);
                        }
                        SsaInst::FnAddr { dest, name } => {
                            if let Some(fv) = self.fn_value_map.get(name) {
                                block_vals[bi].insert(*dest, fv.as_global_value().as_basic_value_enum());
                            }
                        }
                        SsaInst::CallIndirect { dest, fn_ptr, ret_type, param_types, args } => {
                            let ptr_val = ssa_read!(*fn_ptr);
                            let llvm_ret = self.llvm_type(ret_type);
                            let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types.iter()
                                .map(|p| self.llvm_type(p).into()).collect();
                            let fn_ty = llvm_ret.fn_type(&llvm_params, false);
                            let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum> = args.iter()
                                .map(|a| { let v = ssa_read!(*a); v.into() }).collect();
                            let call_res = unsafe {
                                self.builder.build_indirect_call(fn_ty, ptr_val.into_pointer_value(), &llvm_args, "_sicl")
                                    .map_err(|e| format!("ssaic: {}", e))?
                            };
                            if let Some(d) = dest {
                                if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                    block_vals[bi].insert(*d, rv);
                                }
                            }
                        }
                        SsaInst::AsyncSpawn { dest, function_name, arg } => {
                            let arg_val = ssa_read!(*arg);
                            let spawn_fn = self.module.get_function("kl_spawn_task")
                                .ok_or_else(|| "no kl_spawn_task".to_string())?;
                            let fn_val = self.fn_value_map.get(function_name)
                                .ok_or_else(|| format!("async '{}' not found", function_name))?;
                            let args_m: Vec<inkwell::values::BasicMetadataValueEnum> = vec![
                                fn_val.as_global_value().as_pointer_value().into(),
                                arg_val.into(),
                            ];
                            let call_res = self.builder.build_call(spawn_fn, &args_m, "_ssasp")
                                .map_err(|e| format!("ssasp: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                block_vals[bi].insert(*dest, rv);
                            }
                        }
                        SsaInst::AsyncAwait { dest, handle } => {
                            let handle_val = ssa_read!(*handle);
                            let join_fn = self.module.get_function("kl_await_task")
                                .ok_or_else(|| "no kl_await_task".to_string())?;
                            let args_m: Vec<inkwell::values::BasicMetadataValueEnum> = vec![handle_val.into()];
                            let call_res = self.builder.build_call(join_fn, &args_m, "_ssaaw")
                                .map_err(|e| format!("ssaaw: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                block_vals[bi].insert(*dest, rv);
                            }
                        }
                        SsaInst::PtrOffset { dest, ptr, index } => {
                            if let Some(base) = self.alloca_map.get(*ptr).and_then(|p| *p) {
                                let idx = ssa_read!(*index);
                                let gep = unsafe {
                                    self.builder.build_gep(self.context.i8_type(), base, &[self.to_int_value(idx)], "_ssgep")
                                        .map_err(|e| format!("ssgep: {}", e))?
                                };
                                block_vals[bi].insert(*dest, gep.as_basic_value_enum());
                            }
                        }
                        SsaInst::FieldPtr { dest, ptr, field_index, struct_type } => {
                            if let Some(base) = self.alloca_map.get(*ptr).and_then(|p| *p) {
                                if let Some(fpa) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                    let st = self.llvm_type(struct_type);
                                    if let BasicTypeEnum::StructType(s) = st {
                                        let fts = s.get_field_types();
                                        if *field_index < fts.len() {
                                            self.field_ptr_types.insert(*dest, fts[*field_index]);
                                        }
                                    }
                                    let zero = self.context.i32_type().const_zero();
                                    let idx_v = self.context.i32_type().const_int(*field_index as u64, false);
                                    let gep = unsafe {
                                        self.builder.build_gep(st, base, &[zero, idx_v], "_ssfptr")
                                            .map_err(|e| format!("ssfptr: {}", e))?
                                    };
                                    self.builder.build_store(fpa, gep)
                                        .map_err(|e| format!("ssfptr2: {}", e))?;
                                }
                            }
                        }
                        SsaInst::Memcpy { dest_ptr_local, src_alloca_local, struct_type } => {
                            if let Some(dp) = self.alloca_map.get(*dest_ptr_local).and_then(|p| *p) {
                                if let Some(sp) = self.alloca_map.get(*src_alloca_local).and_then(|p| *p) {
                                    let st = self.llvm_type(struct_type);
                                    let src_val = self.builder.build_load(st, sp, "_ssmc")
                                        .map_err(|e| format!("ssmcld: {}", e))?;
                                    let dst_gep = self.builder.build_load(ptr_ty, dp, "_ssmd")
                                        .map_err(|e| format!("ssmcds: {}", e))?;
                                    self.builder.build_store(dst_gep.into_pointer_value(), src_val)
                                        .map_err(|e| format!("ssmcst: {}", e))?;
                                }
                            }
                        }
                    }
                }

                // Terminator
                match &block.terminator {
                    MirTerminator::Return(val) => {
                        let ret = resolve_mir!(val);
                        self.builder.build_return(Some(&ret))
                            .map_err(|e| format!("ssaret: {}", e))?;
                    }
                    MirTerminator::Br(label) => {
                        if let Some(&tb) = block_map.get(label) {
                            self.builder.build_unconditional_branch(tb)
                                .map_err(|e| format!("ssabr: {}", e))?;
                        }
                    }
                    MirTerminator::CondBr { cond, true_block, false_block } => {
                        let cond_val = resolve_mir!(cond);
                        let cond_int = self.to_int_value(cond_val);
                        let i1_cond = if cond_int.get_type().get_bit_width() > 1 {
                            let i1_ty = self.context.bool_type();
                            self.builder.build_int_truncate(cond_int, i1_ty, "")
                                .map_err(|e| format!("cond trunc: {}", e))?
                        } else {
                            cond_int
                        };
                        if let (Some(&tb), Some(&fb)) = (block_map.get(true_block), block_map.get(false_block)) {
                            self.builder.build_conditional_branch(i1_cond, tb, fb)
                                .map_err(|e| format!("ssacbr: {}", e))?;
                        }
                    }
                    MirTerminator::Unreachable => {}
                }
            }
        }

        // Pass 3: phi incomings — fill from block_vals + alloca_current
        for (bi, block) in func.blocks.iter().enumerate() {
            for phi in &block.phis {
                if let Some(phi_entry) = phi_map.iter().find(|(id, _)| *id == phi.dest) {
                    let incomings: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = phi.incomings.iter()
                        .filter_map(|(val_id, pred_label)| {
                            let pred_bb = block_map.get(pred_label).copied()?;
                            // Find predecessor index in function block list
                            let pred_bi = func.blocks.iter().position(|b| &b.label == pred_label)?;
                            // Search predecessor's block_vals only (avoids dominance violations)
                            let val = block_vals[pred_bi].get(val_id).copied()
                                .or_else(|| {
                                    // Check const_values for constants
                                    func.const_values.get(val_id)
                                        .map(|c| self.constant_to_llvm(c))
                                });
                            val.map(|v| (v, pred_bb))
                        })
                        .collect();
                    if !incomings.is_empty() {
                        let refs: Vec<(&dyn BasicValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
                            incomings.iter().map(|(v, b)| {
                                let bv: &dyn BasicValue = v;
                                (bv, *b)
                            }).collect();
                        phi_entry.1.add_incoming(&refs);
                    }
                }
            }
        }

        Ok(())
    }

    // ===================================================================
    // SSA Helper Functions (free functions to avoid borrow issues)
    // ===================================================================

    /// Helper: declare an extern runtime function with optional LLVM string attributes.
    fn add_runtime_extern(
        &self,
        name: &str,
        ft: inkwell::types::FunctionType<'ctx>,
        kv_attrs: &[(&str, &str)],
    ) -> FunctionValue<'ctx> {
        let func = self.module.add_function(name, ft, None);
        for &(key, val) in kv_attrs {
            let attr = self.context.create_string_attribute(key, val);
            func.add_attribute(AttributeLoc::Function, attr);
        }
        func
    }

    /// Declare external runtime functions that generated code can call.
    fn declare_runtime_externs(&mut self) {
        let void_ty = self.context.void_type();
        let i64_ty = self.context.i64_type();
        let i32_ty = self.context.i32_type();
        let f64_ty = self.context.f64_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        // void kl_print(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_print", ft, None);
        }
        // void kl_println(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_println", ft, None);
        }
        // ptr kl_alloc(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_alloc", ft, None);
        }
        // void kl_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_free", ft, None);
        }
        // ptr kl_i64_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_i64_to_str", ft, None);
        }
        // ptr kl_f64_to_str(f64)
        {
            let params = [f64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_f64_to_str", ft, None);
        }
        // i32 kl_strlen(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_strlen", ft, &[("memory", "read")]);
        }
        // ptr kl_concat(ptr, i32, ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_concat", ft, None);
        }
        // ptr kl_input()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("kl_input", ft, None);
        }
        // ptr kl_input_with_prompt(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_input_with_prompt", ft, None);
        }
        // i32 kl_str_contains(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_str_contains", ft, &[("memory", "read")]);
        }
        // ptr kl_str_to_upper(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_str_to_upper", ft, None);
        }
        // ptr kl_str_to_lower(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_str_to_lower", ft, None);
        }
        // ptr kl_str_trim(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_str_trim", ft, None);
        }
        // ptr kl_str_replace(ptr, ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_str_replace", ft, None);
        }
        // i32 kl_open(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("kl_open", ft, None);
        }
        // ptr kl_read_str(i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_read_str", ft, None);
        }
        // i32 kl_write_str(i32, ptr)
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("kl_write_str", ft, None);
        }
        // i32 kl_close(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("kl_close", ft, None);
        }
        // void kl_sleep(i32)
        {
            let params = [i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_sleep", ft, None);
        }
        // i64 kl_now()
        {
            let ft = i64_ty.fn_type(&[], false);
            self.module.add_function("kl_now", ft, None);
        }
        // i8 kl_char_at(ptr, i32) — readonly
        {
            let i8_ty = self.context.i8_type();
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = i8_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_char_at", ft, &[("memory", "read")]);
        }
        // ptr kl_list_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("kl_list_new", ft, None);
        }
        // void kl_list_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_list_free", ft, None);
        }
        // ptr kl_range(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_range", ft, None);
        }
        // void kl_list_push(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_list_push", ft, None);
        }
        // i64 kl_list_pop(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("kl_list_pop", ft, None);
        }
        // i64 kl_list_get(ptr, i64) — readonly
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_list_get", ft, &[("memory", "read")]);
        }
        // void kl_list_set(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_list_set", ft, None);
        }
        // i64 kl_list_len(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_list_len", ft, &[("memory", "read")]);
        }
        // i64 kl_list_sum(ptr), kl_list_product(ptr), kl_list_max(ptr), kl_list_min(ptr) — readonly
        for name in &["kl_list_sum", "kl_list_product", "kl_list_max", "kl_list_min"] {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern(name, ft, &[("memory", "read")]);
        }
        // void kl_list_reverse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_list_reverse", ft, None);
        }
        // ptr kl_list_slice(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_list_slice", ft, None);
        }
        // void kl_list_extend(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_list_extend", ft, None);
        }
        // ptr kl_dict_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("kl_dict_new", ft, None);
        }
        // void kl_dict_set(ptr, ptr, i64)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_dict_set", ft, None);
        }
        // i64 kl_dict_get(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_dict_get", ft, &[("memory", "read")]);
        }
        // i64 kl_dict_len(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_dict_len", ft, &[("memory", "read")]);
        }
        // void kl_dict_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_dict_free", ft, None);
        }
        // ptr kl_substr(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_substr", ft, None);
        }
        // i32 kl_is_digit(i8), kl_is_alpha(i8), etc. — all readnone (pure)
        {
            let i8_ty = self.context.i8_type();
            let params = [i8_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            for name in &["kl_is_digit", "kl_is_alpha", "kl_is_alnum",
                          "kl_is_whitespace", "kl_is_upper", "kl_is_lower",
                          "kl_ord"] {
                self.add_runtime_extern(name, ft, &[("memory", "none")]);
            }
        }
        // i32 kl_eq_str(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_eq_str", ft, &[("memory", "read")]);
        }
        // ptr kl_init_args(i32, ptr)  — convert C argv to Kyle list
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_init_args", ft, None);
        }
        // i64 kl_spawn_task(ptr, i64)  — spawn async task running extern "C" fn(i64) -> i64
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("kl_spawn_task", ft, None);
        }
        // i64 kl_await_task(i64)  — await task completion, return result
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("kl_await_task", ft, None);
        }
        // void kl_yield()  — cooperative yield hint
        {
            let ft = void_ty.fn_type(&[], false);
            self.module.add_function("kl_yield", ft, None);
        }
        // ptr kl_dict_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("kl_dict_new", ft, None);
        }
        // void kl_dict_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_dict_free", ft, None);
        }
        // i64 kl_dict_get(ptr, ptr)  — get value by key (key is C string ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_dict_get", ft, &[("memory", "read")]);
        }
        // void kl_dict_set(ptr, ptr, i64)  — set key=value
        {
            let params = [ptr_ty.into(), ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_dict_set", ft, None);
        }
        // i64 kl_dict_len(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_dict_len", ft, &[("memory", "read")]);
        }
        // i32 kl_dict_contains(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("kl_dict_contains", ft, &[("memory", "read")]);
        }
        // void kl_dict_remove(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_dict_remove", ft, None);
        }
        // ptr kl_json_parse(ptr) — parse JSON string into dict
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_json_parse", ft, None);
        }
        // ptr kl_json_stringify(ptr) — serialize dict to JSON string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_json_stringify", ft, None);
        }
        // ptr kl_clone_str(ptr) — deep-copy a heap-allocated string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_clone_str", ft, None);
        }
        // ptr kl_clone_list(ptr) — shallow-copy a list
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_clone_list", ft, None);
        }
        // ptr kl_clone_dict(ptr) — shallow-copy a dict
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_clone_dict", ft, None);
        }
        // void kl_assert(i32)
        {
            let params = [i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_assert", ft, None);
        }
        // void kl_assert_eq(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_assert_eq", ft, None);
        }
        // void kl_assert_ne(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_assert_ne", ft, None);
        }
    }

    fn llvm_type(&self, mir_type: &MirType) -> BasicTypeEnum<'ctx> {
        match mir_type {
            MirType::I1 => self.context.bool_type().as_basic_type_enum(),
            MirType::I8 => self.context.i8_type().as_basic_type_enum(),
            MirType::I16 => self.context.i16_type().as_basic_type_enum(),
            MirType::I32 => self.context.i32_type().as_basic_type_enum(),
            MirType::I64 => self.context.i64_type().as_basic_type_enum(),
            MirType::F32 => self.context.f32_type().as_basic_type_enum(),
            MirType::F64 => self.context.f64_type().as_basic_type_enum(),
            MirType::Bool => self.context.bool_type().as_basic_type_enum(),
            MirType::Char => self.context.i8_type().as_basic_type_enum(),
            MirType::Str => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::List(_) | MirType::Dict(_, _) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::Void => self.context.i32_type().as_basic_type_enum(),
            MirType::Ptr(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::Array(inner) => {
                let base = self.llvm_type(inner);
                base.array_type(0).as_basic_type_enum()
            }
            MirType::Struct(name, fields) => {
                // Reuse existing struct type to avoid creating duplicate types
                // with numbered suffixes (e.g. Token.1, Token.2, ...)
                let struct_ty = self.module.get_struct_type(name)
                    .unwrap_or_else(|| {
                        let new_ty = self.context.opaque_struct_type(name);
                        let field_types: Vec<BasicTypeEnum<'ctx>> = fields.iter()
                            .map(|(_, ty)| self.llvm_type(ty))
                            .collect();
                        new_ty.set_body(&field_types, false);
                        new_ty
                    });
                struct_ty.as_basic_type_enum()
            }
        }
    }

    fn declare_function(&mut self, func: &MirFunction) -> Result<(), String> {
        let ret_type = self.llvm_type(&func.return_type);
        let ptr_ty = self.context.ptr_type(Default::default());
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = func.params
            .iter()
            .map(|p| {
                if matches!(p, MirType::Struct(_, _)) {
                    ptr_ty.into()
                } else {
                    self.llvm_type(p).into()
                }
            })
            .collect();

        // If main has a list parameter (e.g. args: [str]), rename to kyle_main
        // and generate a C-compatible main(i32, ptr) wrapper later.
        let fn_name = if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            self.needs_main_wrapper = true;
            "kyle_main"
        } else {
            &func.name
        };

        let fn_type = ret_type.fn_type(&param_types, false);
        let fn_value = self.module.add_function(fn_name, fn_type, None);
        self.fn_value_map.insert(fn_name.to_string(), fn_value);
        Ok(())
    }

    fn compile_function(&mut self, func: &MirFunction) -> Result<(), String> {
        let fn_name = if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            "kyle_main"
        } else {
            &func.name
        };
        let fn_value = self.fn_value_map.get(fn_name)
            .ok_or_else(|| format!("Function {} not declared", fn_name))?;

        self.alloca_types.clear();
        self.ref_param_struct_types.clear();
        self.field_ptr_allocas.clear();
        self.field_ptr_types.clear();
        let ptr_ty = self.context.ptr_type(Default::default());
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Alloca { dest, type_, .. } = inst {
                    let llvm_ty = self.llvm_type(type_);
                    let actual_ty = if let MirType::Ptr(_) = type_ {
                        ptr_ty.as_basic_type_enum()
                    } else {
                        llvm_ty
                    };
                    self.alloca_types.entry(*dest).or_insert(actual_ty);
                }
            }
        }

        // Pre-scan for struct function params: change their alloca type to `ptr`
        // so they receive a pointer to the caller's struct (pass-by-reference ABI).
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Store { dest, value: MirValue::Param(_) } = inst {
                    if let Some(&llvm_type) = self.alloca_types.get(dest) {
                        if matches!(llvm_type, BasicTypeEnum::StructType(_)) {
                            let orig_type = self.alloca_types.insert(*dest, ptr_ty.as_basic_type_enum()).unwrap();
                            self.ref_param_struct_types.insert(*dest, orig_type);
                        }
                    }
                }
            }
        }

        let mut block_map: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();
        for bb in &func.basic_blocks {
            let llvm_bb = self.context.append_basic_block(*fn_value, &bb.label);
            block_map.insert(bb.label.clone(), llvm_bb);
        }

        self.alloca_map.clear();

        if let Some(entry_bb) = func.basic_blocks.first() {
            if let Some(&llvm_entry) = block_map.get(&entry_bb.label) {
                self.builder.position_at_end(llvm_entry);

                for (dest, llvm_type) in &self.alloca_types {
                    while self.alloca_map.len() <= *dest {
                        self.alloca_map.push(None);
                    }
                    let ptr = self.builder.build_alloca(*llvm_type, "")
                        .map_err(|e| format!("alloca {}: {}", dest, e))?;
                    self.alloca_map[*dest] = Some(ptr);
                }

                let ptr_ty = self.context.ptr_type(Default::default());
                for bb in &func.basic_blocks {
                    for inst in &bb.insts {
                        if let MirInst::FieldPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest {
                                self.field_ptr_allocas.push(None);
                            }
                            if self.field_ptr_allocas[*dest].is_none() {
                                let alloca = self.builder.build_alloca(ptr_ty, "_fgep")
                                    .map_err(|e| format!("fgep alloca {}: {}", dest, e))?;
                                self.field_ptr_allocas[*dest] = Some(alloca);
                            }
                        }
                    }
                }

                for (i, param) in fn_value.get_param_iter().enumerate() {
                    self.param_values.insert(i, param);
                }
            }
        }

        let mut last_value_map: HashMap<usize, BasicValueEnum<'ctx>> = HashMap::new();

        for bb in &func.basic_blocks {
            if let Some(&llvm_bb) = block_map.get(&bb.label) {
                self.builder.position_at_end(llvm_bb);

                for inst in &bb.insts {
                    match inst {
                        MirInst::Alloca { .. } => {}
                        MirInst::Store { dest, value } => {
                            let val = self.value_to_llvm(value, &last_value_map)?;
                            // Check if this is a store to a field pointer
                            if *dest < self.field_ptr_allocas.len() && self.field_ptr_allocas[*dest].is_some() {
                                if let Some(field_ptr_alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                    let gep = self.builder.build_load(
                                        self.context.ptr_type(Default::default()),
                                        field_ptr_alloca, "_fgepload"
                                    ).map_err(|e| format!("fptr store load: {}", e))?;
                                    self.builder.build_store(gep.into_pointer_value(), val)
                                        .map_err(|e| format!("fptr store: {}", e))?;
                                }
                            } else if let Some(ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(ptr, val)
                                    .map_err(|e| format!("store: {}", e))?;
                            }
                            last_value_map.insert(*dest, val);
                        }
                        MirInst::Load { dest, src } => {
                            // Check if this is a load from a field pointer
                            if *src < self.field_ptr_allocas.len() && self.field_ptr_allocas[*src].is_some() {
                                if let Some(field_ptr_alloca) = self.field_ptr_allocas.get(*src).and_then(|p| *p) {
                                    let gep = self.builder.build_load(
                                        self.context.ptr_type(Default::default()), 
                                        field_ptr_alloca, "_fgepload"
                                    ).map_err(|e| format!("fptr load: {}", e))?;
                                    let field_type = self.field_ptr_types.get(src).or_else(|| self.alloca_types.get(src));
                                    if let Some(pointee_type) = field_type {
                                        let loaded = self.builder.build_load(*pointee_type, gep.into_pointer_value(), "")
                                            .map_err(|e| format!("field load: {}", e))?;
                                        if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                            self.builder.build_store(dest_ptr, loaded)
                                                .map_err(|e| format!("field load-store: {}", e))?;
                                        }
                                        last_value_map.insert(*dest, loaded);
                                    }
                                }
                            } else if let Some(ptr) = self.alloca_map.get(*src).and_then(|p| *p) {
                                if let Some(pointee_type) = self.alloca_types.get(src) {
                                    let loaded = if self.ref_param_struct_types.contains_key(src) {
                                        // Ref param: alloca stores a pointer to the struct.
                                        // Load the pointer from alloca, then load the struct from that pointer.
                                        let struct_ptr = self.builder.build_load(
                                            *pointee_type, ptr, "_ref_load"
                                        ).map_err(|e| format!("ref load: {}", e))?;
                                        if let Some(&orig_struct_type) = self.ref_param_struct_types.get(src) {
                                            self.builder.build_load(
                                                orig_struct_type, struct_ptr.into_pointer_value(), "_ref_val"
                                            ).map_err(|e| format!("ref load val: {}", e))?
                                        } else {
                                            struct_ptr
                                        }
                                    } else {
                                        self.builder.build_load(*pointee_type, ptr, "")
                                            .map_err(|e| format!("load: {}", e))?
                                    };
                                    // Store to dest alloca for cross-block reads
                                    if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(dest_ptr, loaded)
                                            .map_err(|e| format!("load-store: {}", e))?;
                                    }
                                    last_value_map.insert(*dest, loaded);
                                }
                            }
                        }
                        MirInst::BinaryOp { dest, op, left, right } => {
                            let l = self.value_to_llvm(left, &last_value_map)?;
                            let r = self.value_to_llvm(right, &last_value_map)?;

                            // Check if either operand is float (handles comparison ops whose result is I32)
                            let l_is_float = matches!(l, BasicValueEnum::FloatValue(_));
                            let r_is_float = matches!(r, BasicValueEnum::FloatValue(_));
                            let any_float = l_is_float || r_is_float;
                            // Also check if destination is float type (for arithmetic)
                            let dest_type = self.alloca_types.get(dest).or_else(|| self.field_ptr_types.get(dest));
                            let is_float = dest_type.map_or(false, |t| matches!(t, BasicTypeEnum::FloatType(_)));

                            let result = if any_float || is_float {
                                let lf = self.to_float_value(l);
                                let rf = self.to_float_value(r);
                                match op {
                                    MirBinaryOp::Add => self.builder.build_float_add(lf, rf, "")
                                        .map_err(|e| format!("fadd: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Sub => self.builder.build_float_sub(lf, rf, "")
                                        .map_err(|e| format!("fsub: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Mul => self.builder.build_float_mul(lf, rf, "")
                                        .map_err(|e| format!("fmul: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Div => self.builder.build_float_div(lf, rf, "")
                                        .map_err(|e| format!("fdiv: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Eq => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OEQ, lf, rf, "")
                                            .map_err(|e| format!("feq: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("feq-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Neq => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::ONE, lf, rf, "")
                                            .map_err(|e| format!("fne: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fne-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Lt => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OLT, lf, rf, "")
                                            .map_err(|e| format!("flt: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("flt-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Gt => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OGT, lf, rf, "")
                                            .map_err(|e| format!("fgt: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fgt-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Le => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OLE, lf, rf, "")
                                            .map_err(|e| format!("fle: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fle-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Ge => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OGE, lf, rf, "")
                                            .map_err(|e| format!("fge: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fge-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    _ => {
                                        // Fallback: use int op for bitwise etc.
                                        let li = self.to_int_value(l);
                                        let ri = self.to_int_value(r);
                                        self.builder.build_int_add(li, ri, "")
                                            .map_err(|e| format!("int_add: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                }
                            } else {
                                let li = self.to_int_value(l);
                                let ri = self.to_int_value(r);

                                match op {
                                    MirBinaryOp::Add => self.builder.build_int_add(li, ri, "")
                                        .map_err(|e| format!("add: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Sub => self.builder.build_int_sub(li, ri, "")
                                        .map_err(|e| format!("sub: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Mul => self.builder.build_int_mul(li, ri, "")
                                        .map_err(|e| format!("mul: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Div => self.builder.build_int_signed_div(li, ri, "")
                                        .map_err(|e| format!("div: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Rem => self.builder.build_int_signed_rem(li, ri, "")
                                        .map_err(|e| format!("rem: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::And => self.builder.build_and(li, ri, "")
                                        .map_err(|e| format!("and: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Or => self.builder.build_or(li, ri, "")
                                        .map_err(|e| format!("or: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Xor => self.builder.build_xor(li, ri, "")
                                        .map_err(|e| format!("xor: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Shl => self.builder.build_left_shift(li, ri, "")
                                        .map_err(|e| format!("shl: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Shr => self.builder.build_right_shift(li, ri, true, "")
                                        .map_err(|e| format!("shr: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, li, ri, "")
                                        .map_err(|e| format!("eq: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Neq => self.builder.build_int_compare(IntPredicate::NE, li, ri, "")
                                        .map_err(|e| format!("neq: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, li, ri, "")
                                        .map_err(|e| format!("lt: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, li, ri, "")
                                        .map_err(|e| format!("gt: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Le => self.builder.build_int_compare(IntPredicate::SLE, li, ri, "")
                                        .map_err(|e| format!("le: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, li, ri, "")
                                        .map_err(|e| format!("ge: {}", e))?
                                        .as_basic_value_enum(),
                                }
                            };
                            let result_val = result.as_basic_value_enum();
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                // Auto-extend i1 result to wider int if dest type is wider
                                let extended = match (&result_val, &self.alloca_types.get(dest)) {
                                    (BasicValueEnum::IntValue(iv), Some(BasicTypeEnum::IntType(dt))) => {
                                        let rw = iv.get_type().get_bit_width();
                                        let dw = dt.get_bit_width();
                                        if rw != dw {
                                            if rw == 1 && dw > 1 {
                                                self.builder.build_int_z_extend(*iv, *dt, "")
                                                    .map_err(|e| format!("binop-zext: {}", e))?
                                                    .as_basic_value_enum()
                                            } else {
                                                self.builder.build_int_cast(*iv, *dt, "")
                                                    .map_err(|e| format!("binop-cast: {}", e))?
                                                    .as_basic_value_enum()
                                            }
                                        } else { result_val }
                                    }
                                    _ => result_val,
                                };
                                self.builder.build_store(dest_ptr, extended)
                                    .map_err(|e| format!("binop-store: {}", e))?;
                            }
                            last_value_map.insert(*dest, result_val);
                        }
                        MirInst::UnaryOp { dest, op, operand } => {
                            let val = self.value_to_llvm(operand, &last_value_map)?;
                            let int_val = val.into_int_value();
                            let result = match op {
                                MirUnaryOp::Neg => self.builder.build_int_neg(int_val, "")
                                    .map_err(|e| format!("neg: {}", e))?,
                                MirUnaryOp::Not | MirUnaryOp::BitNot => self.builder.build_not(int_val, "")
                                    .map_err(|e| format!("not: {}", e))?,
                            };
                            let result_val = result.as_basic_value_enum();
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(dest_ptr, result_val)
                                    .map_err(|e| format!("unary-store: {}", e))?;
                            }
                            last_value_map.insert(*dest, result_val);
                        }
                        MirInst::Call { dest, name, args } => {
                            let runtime_name = match name.as_str() {
                                "print" => "kl_print",
                                "println" => "kl_println",
                                "contains" => "kl_str_contains",
                                "to_upper" => "kl_str_to_upper",
                                "to_lower" => "kl_str_to_lower",
                                "trim" => "kl_str_trim",
                                "replace" => "kl_str_replace",
                                "input" => "kl_input",
                                "open" => "kl_open",
                                "read_str" => "kl_read_str",
                                "write_str" => "kl_write_str",
                                "close" => "kl_close",
                                "sleep" => "kl_sleep",
                                "now" => "kl_now",
                                "char_at" => "kl_char_at",
                                "is_digit" => "kl_is_digit",
                                "is_alpha" => "kl_is_alpha",
                                "is_alnum" => "kl_is_alnum",
                                "is_whitespace" => "kl_is_whitespace",
                                "is_upper" => "kl_is_upper",
                                "is_lower" => "kl_is_lower",
                                "ord" => "kl_ord",
                                "substr" => "kl_substr",
                                "list_new" => "kl_list_new",
                                "list_push" => "kl_list_push",
                                "list_get" => "kl_list_get",
                                "list_set" => "kl_list_set",
                                "list_len" => "kl_list_len",
                                "json_parse" => "kl_json_parse",
                                "json_stringify" => "kl_json_stringify",
                                "assert" => "kl_assert",
                                "assert_eq" => "kl_assert_eq",
                                "assert_ne" => "kl_assert_ne",
                                "assert_str" => "kl_assert_eq",
                                _ => name,
                            };
                            let callee = self.module.get_function(runtime_name);
                            if let Some(callee_fn) = callee {
                                let fn_ty = callee_fn.get_type();
                                let param_types = fn_ty.get_param_types();
                                let llvm_args: Vec<BasicValueEnum<'ctx>> = args
                                    .iter()
                                    .enumerate()
                                    .map(|(i, a)| {
                                        // Pass struct locals by pointer (pass-by-reference ABI)
                                        if let MirValue::Local(id) = a {
                                            // Regular struct local: pass alloca pointer
                                            if let Some(&struct_type) = self.alloca_types.get(id) {
                                                if matches!(struct_type, BasicTypeEnum::StructType(_)) {
                                                    if let Some(ptr) = self.alloca_map.get(*id).and_then(|p| *p) {
                                                        return Ok(ptr.as_basic_value_enum());
                                                    }
                                                }
                                            }
                                            // Ref param: alloca stores ptr, load it as-is (already a ptr)
                                            if self.ref_param_struct_types.contains_key(id) {
                                                let val = self.load_value(*id, &last_value_map)?;
                                                return Ok(val);
                                            }
                                        }
                                        let val = self.value_to_llvm(a, &last_value_map)?;
                                        // Auto-cast i64 → ptr when function expects ptr
                                        if i < param_types.len() {
                                            let expected = param_types[i];
                                            if matches!(expected, inkwell::types::BasicMetadataTypeEnum::PointerType(_)) {
                                                if let BasicValueEnum::IntValue(int_val) = val {
                                                    let ptr_ty = self.context.ptr_type(Default::default());
                                                    return Ok(self.builder.build_int_to_ptr(int_val, ptr_ty, "_argptr")
                                                        .map_err(|e| format!("arg inttoptr: {}", e))?
                                                        .as_basic_value_enum());
                                                }
                                            }
                                        }
                                        Ok(val)
                                    })
                                    .collect::<Result<Vec<_>, String>>()?;
                                let llvm_arg_refs: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> =
                                    llvm_args.iter().map(|a| (*a).into()).collect();
                                let call_result = self.builder.build_call(callee_fn, &llvm_arg_refs, "")
                                    .map_err(|e| format!("call {}: {}", name, e))?;
                                if let Some(d) = dest {
                                    if let inkwell::values::ValueKind::Basic(ret_val) = call_result.try_as_basic_value() {
                                        // Store call result to both the last_value_map (for SSA-style use
                                        // within the same basic block) AND the alloca (for cross-block
                                        // references like kl_release in the return block)
                                        if let Some(alloca_ptr) = self.alloca_map.get(*d).and_then(|p| *p) {
                                            self.builder.build_store(alloca_ptr, ret_val)
                                                .map_err(|e| format!("call store {}: {}", name, e))?;
                                        }
                                        last_value_map.insert(*d, ret_val);
                                    }
                                }
                            }
                        }
                        MirInst::PtrOffset { dest, ptr, index } => {
                            if let Some(base_ptr) = self.alloca_map.get(*ptr).and_then(|p| *p) {
                                let idx = self.value_to_llvm(index, &last_value_map)?;
                                let int_idx = idx.into_int_value();
                                if let Some(pointee_type) = self.alloca_types.get(ptr) {
                                    let gep = unsafe {
                                        self.builder.build_gep(*pointee_type, base_ptr, &[int_idx], "")
                                            .map_err(|e| format!("gep: {}", e))?
                                    };
                                    last_value_map.insert(*dest, gep.as_basic_value_enum());
                                }
                            }
                        }
                        MirInst::FieldPtr { dest, ptr, field_index, struct_type } => {
                            if let Some(base_ptr) = self.alloca_map.get(*ptr).and_then(|p| *p) {
                                // Determine field type for later loads from this field pointer
                                if let MirType::Struct(_, fields) = struct_type.as_ref() {
                                    if let Some((_, field_mir_type)) = fields.get(*field_index) {
                                        let field_llvm = self.llvm_type(field_mir_type);
                                        self.field_ptr_types.insert(*dest, field_llvm);
                                    }
                                }
                                // Ref param: alloca stores pointer-to-struct, load it first
                                if let Some(&orig_struct_type) = self.ref_param_struct_types.get(ptr) {
                                    let struct_ptr = self.builder.build_load(
                                        self.context.ptr_type(Default::default()),
                                        base_ptr, "_ref_load"
                                    ).map_err(|e| format!("ref_field_ptr load: {}", e))?;
                                    let zero = self.context.i32_type().const_zero();
                                    let idx_val = self.context.i32_type().const_int(*field_index as u64, false);
                                    let gep = unsafe {
                                        self.builder.build_gep(orig_struct_type, struct_ptr.into_pointer_value(), &[zero, idx_val], "")
                                            .map_err(|e| format!("ref_field_ptr: {}", e))?
                                    };
                                    if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(alloca, gep)
                                            .map_err(|e| format!("ref_fgep store: {}", e))?;
                                    }
                                } else if let Some(struct_type) = self.alloca_types.get(ptr) {
                                    let zero = self.context.i32_type().const_zero();
                                    let idx_val = self.context.i32_type().const_int(*field_index as u64, false);
                                    let gep = unsafe {
                                        self.builder.build_gep(*struct_type, base_ptr, &[zero, idx_val], "")
                                            .map_err(|e| format!("field_ptr: {}", e))?
                                    };
                                    if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(alloca, gep)
                                            .map_err(|e| format!("fgep store: {}", e))?;
                                    }
                                }
                            }
                        }
                        MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                            if let Some(dest_ptr) = last_value_map.get(dest_ptr_local) {
                                if let Some(src_val) = last_value_map.get(src_alloca_local) {
                                    if let BasicValueEnum::StructValue(struct_val) = src_val {
                                        let heap_ptr = dest_ptr.into_pointer_value();
                                        let struct_ptr = self.builder.build_pointer_cast(heap_ptr, self.context.ptr_type(Default::default()), "_mc")
                                            .map_err(|e| format!("memcpy bitcast: {}", e))?;
                                        self.builder.build_store(struct_ptr, *struct_val)
                                            .map_err(|e| format!("memcpy store: {}", e))?;
                                    }
                                }
                            }
                        }
                        MirInst::Cast { dest, value, to_type } => {
                            let val = self.value_to_llvm(value, &last_value_map)?;
                            let target_type = self.llvm_type(to_type);
                            let result = match (&val, &target_type) {
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::IntType(t)) => {
                                    let src_width = int_val.get_type().get_bit_width();
                                    let dst_width = t.get_bit_width();
                                    let result = if src_width == 1 && dst_width > 1 {
                                        self.builder.build_int_z_extend(*int_val, *t, "")
                                            .map_err(|e| format!("zext: {}", e))?
                                    } else {
                                        self.builder.build_int_cast(*int_val, *t, "")
                                            .map_err(|e| format!("cast: {}", e))?
                                    };
                                    result.as_basic_value_enum()
                                }
                                (BasicValueEnum::PointerValue(ptr_val), BasicTypeEnum::IntType(t)) => {
                                    self.builder.build_ptr_to_int(*ptr_val, *t, "")
                                        .map_err(|e| format!("ptrtoint: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::PointerType(t)) => {
                                    self.builder.build_int_to_ptr(*int_val, *t, "")
                                        .map_err(|e| format!("inttoptr: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::StructType(s)) => {
                                    let ptr_ty = self.context.ptr_type(Default::default());
                                    let ptr_val = self.builder.build_int_to_ptr(*int_val, ptr_ty, "_ptr")
                                        .map_err(|e| format!("inttoptr: {}", e))?;
                                    self.builder.build_load(*s, ptr_val, "_struct")
                                        .map_err(|e| format!("load struct: {}", e))?
                                }
                                (BasicValueEnum::StructValue(struct_val), BasicTypeEnum::IntType(i)) => {
                                    let struct_ty = struct_val.get_type();
                                    let temp_alloca = self.builder.build_alloca(struct_ty, "_tmp_struct")
                                        .map_err(|e| format!("alloca: {}", e))?;
                                    self.builder.build_store(temp_alloca, *struct_val)
                                        .map_err(|e| format!("store struct: {}", e))?;
                                    let ptr = temp_alloca.as_basic_value_enum();
                                    self.builder.build_ptr_to_int(ptr.into_pointer_value(), *i, "_ptrint")
                                        .map_err(|e| format!("ptrtoint: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::FloatType(f)) => {
                                    // Integer → Float: sitofp
                                    self.builder.build_signed_int_to_float(*int_val, *f, "_sitofp")
                                        .map_err(|e| format!("sitofp: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::FloatValue(float_val), BasicTypeEnum::IntType(i)) => {
                                    // Float → Integer: fptosi
                                    self.builder.build_float_to_signed_int(*float_val, *i, "_fptosi")
                                        .map_err(|e| format!("fptosi: {}", e))?
                                        .as_basic_value_enum()
                                }
                                _ => self.context.i32_type().const_zero().as_basic_value_enum(),
                            };
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(dest_ptr, result)
                                    .map_err(|e| format!("cast-store: {}", e))?;
                            }
                            last_value_map.insert(*dest, result);
                        }
                        MirInst::FnAddr { dest, name } => {
                            if let Some(fn_val) = self.fn_value_map.get(name) {
                                let global = fn_val.as_global_value();
                                let ptr = global.as_pointer_value();
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, ptr)
                                        .map_err(|e| format!("fnaddr store: {}", e))?;
                                }
                                last_value_map.insert(*dest, ptr.as_basic_value_enum());
                            }
                        }
                        MirInst::CallIndirect { dest, fn_ptr, ret_type, param_types, args } => {
                            let ptr_val = self.load_value(*fn_ptr, &last_value_map)?;
                            let fn_ptr = ptr_val.into_pointer_value();
                            let llvm_ret = self.llvm_type(ret_type);
                            let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types.iter()
                                .map(|p| self.llvm_type(p).into())
                                .collect();
                            let fn_ty = llvm_ret.fn_type(&llvm_params, false);
                            let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum> = args.iter()
                                .map(|a| {
                                    self.value_to_llvm(a, &last_value_map)
                                        .unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum())
                                        .into()
                                })
                                .collect();
                            let call_result = self.builder.build_indirect_call(fn_ty, fn_ptr, &llvm_args, "_icl")
                                .map_err(|e| format!("indirect call: {}", e))?;
                            if let Some(d) = dest {
                                if let inkwell::values::ValueKind::Basic(result) = call_result.try_as_basic_value() {
                                    if let Some(alloca) = self.alloca_map.get(*d).and_then(|p| *p) {
                                        self.builder.build_store(alloca, result)
                                            .map_err(|e| format!("icall store: {}", e))?;
                                    }
                                    last_value_map.insert(*d, result);
                                }
                            }
                        }
                        MirInst::AsyncSpawn { dest, function_name, arg } => {
                            let arg_val = self.value_to_llvm(arg, &last_value_map)?;
                            let spawn_fn = self.module.get_function("kl_spawn_task")
                                .ok_or_else(|| "kl_spawn_task not declared".to_string())?;
                            // Get the function pointer of the async wrapper
                            let fn_val = self.fn_value_map.get(function_name)
                                .ok_or_else(|| format!("async function {} not found", function_name))?;
                            let fn_global = fn_val.as_global_value();
                            let fn_ptr = fn_global.as_pointer_value();
                            let args_meta: Vec<inkwell::values::BasicMetadataValueEnum> = vec![
                                fn_ptr.into(),
                                arg_val.into(),
                            ];
                            let call_result = self.builder.build_call(spawn_fn, &args_meta, "_async_spawn")
                                .map_err(|e| format!("async_spawn: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(ret_val) = call_result.try_as_basic_value() {
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, ret_val)
                                        .map_err(|e| format!("async_spawn store: {}", e))?;
                                }
                                last_value_map.insert(*dest, ret_val);
                            }
                        }
                        MirInst::AsyncAwait { dest, handle } => {
                            let handle_val = self.load_value(*handle, &last_value_map)?;
                            let join_fn = self.module.get_function("kl_await_task")
                                .ok_or_else(|| "kl_await_task not declared".to_string())?;
                            let args_meta: Vec<inkwell::values::BasicMetadataValueEnum> = vec![handle_val.into()];
                            let call_result = self.builder.build_call(join_fn, &args_meta, "_async_join")
                                .map_err(|e| format!("async_join: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(ret_val) = call_result.try_as_basic_value() {
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, ret_val)
                                        .map_err(|e| format!("async_join store: {}", e))?;
                                }
                                last_value_map.insert(*dest, ret_val);
                            }
                        }
                    }
                }

                match &bb.terminator {
                    MirTerminator::Return(value) => {
                        let val = match value {
                            MirValue::Local(id) => {
                                // Ref params: the alloca stores a ptr, dereference to get struct value
                                if let Some(&struct_type) = self.ref_param_struct_types.get(id) {
                                    let ptr_val = self.load_value(*id, &last_value_map)?;
                                    self.builder.build_load(struct_type, ptr_val.into_pointer_value(), "_retderef")
                                        .map_err(|e| format!("ret ref deref: {}", e))?
                                } else {
                                    self.load_value(*id, &last_value_map)?
                                }
                            }
                            _ => self.value_to_llvm(value, &last_value_map)?,
                        };
                        // Auto-cast return value if it doesn't match function return type
                        let fn_ret_type = fn_value.get_type().get_return_type();
                        let val = if let Some(expected_ret_ty) = fn_ret_type {
                            if val.get_type() != expected_ret_ty.as_basic_type_enum() {
                                match (&val, &expected_ret_ty) {
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::PointerType(pt)) =>
                                        self.builder.build_int_to_ptr(*iv, *pt, "_retptr")
                                            .map_err(|e| format!("ret inttoptr: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::PointerValue(pv), BasicTypeEnum::IntType(it)) =>
                                        self.builder.build_ptr_to_int(*pv, *it, "_retint")
                                            .map_err(|e| format!("ret ptrtoint: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(it)) => {
                                        let sw = iv.get_type().get_bit_width();
                                        let dw = it.get_bit_width();
                                        if sw == 1 && dw > 1 {
                                            self.builder.build_int_z_extend(*iv, *it, "")
                                                .map_err(|e| format!("ret zext: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            self.builder.build_int_cast(*iv, *it, "")
                                                .map_err(|e| format!("ret intcast: {}", e))?
                                                .as_basic_value_enum()
                                        }
                                    }
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::StructType(st)) => {
                                        // Heap pointer (i64) → dereference to struct value
                                        let ptr_ty = self.context.ptr_type(Default::default());
                                        let ptr_val = self.builder.build_int_to_ptr(*iv, ptr_ty, "_retptr")
                                            .map_err(|e| format!("ret inttoptr: {}", e))?;
                                        self.builder.build_load(*st, ptr_val, "_retstruct")
                                            .map_err(|e| format!("ret load struct: {}", e))?
                                    }
                                    _ => val,
                                }
                            } else { val }
                        } else { val };
                        self.builder.build_return(Some(&val))
                            .map_err(|e| format!("ret: {}", e))?;
                    }
                    MirTerminator::Br(label) => {
                        if let Some(&target) = block_map.get(label) {
                            let _ = self.builder.build_unconditional_branch(target);
                        }
                    }
                    MirTerminator::CondBr { cond, true_block, false_block } => {
                        let cond_val = match cond {
                            MirValue::Local(id) => self.load_value(*id, &last_value_map)?,
                            _ => self.value_to_llvm(cond, &last_value_map)?,
                        };
                        let cond_int = cond_val.into_int_value();
                        // Truncate to i1 if needed (e.g. string eq returns i32)
                        let i1_cond = if cond_int.get_type().get_bit_width() > 1 {
                            let i1_ty = self.context.bool_type();
                            self.builder.build_int_truncate(cond_int, i1_ty, "")
                                .map_err(|e| format!("cond trunc: {}", e))?
                        } else {
                            cond_int
                        };
                        if let (Some(&t), Some(&f)) = (block_map.get(true_block), block_map.get(false_block)) {
                            let _ = self.builder.build_conditional_branch(i1_cond, t, f);
                        }
                    }
                    MirTerminator::Unreachable => {
                        self.builder.build_unreachable()
                            .map_err(|e| format!("unreachable: {}", e))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate a C-compatible main(i32, ptr) wrapper that:
    /// 1. Calls kl_init_args(argc, argv) to build a Kyle list<str>
    /// 2. Calls kyle_main(list) with the original function's logic
    fn generate_main_wrapper(&mut self) -> Result<(), String> {
        let i32_ty = self.context.i32_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        // Get the kyle_main function that was declared instead of main
        let kyle_main = self.fn_value_map.get("kyle_main")
            .ok_or_else(|| "kyle_main not declared for wrapper".to_string())?;

        // Declare i32 @main(i32, ptr)
        let param_tys = [i32_ty.into(), ptr_ty.into()];
        let main_type = i32_ty.fn_type(&param_tys, false);
        let main_fn = self.module.add_function("main", main_type, None);

        let bb = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(bb);

        // Convert parameters to BasicMetadataValueEnum
        let argc = main_fn.get_nth_param(0).unwrap();
        let argv = main_fn.get_nth_param(1).unwrap();
        let argc_meta: inkwell::values::BasicMetadataValueEnum = argc.into();
        let argv_meta: inkwell::values::BasicMetadataValueEnum = argv.into();

        // Call kl_init_args(argc, argv) -> ptr (list handle)
        let init_args_fn = self.module.get_function("kl_init_args")
            .ok_or_else(|| "kl_init_args not declared".to_string())?;
        let args_call = self.builder.build_call(init_args_fn, &[argc_meta, argv_meta], "args")
            .map_err(|e| format!("call kl_init_args: {}", e))?;
        let args_list = match args_call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => bv,
            _ => return Err("kl_init_args did not return a basic value".to_string()),
        };
        let args_meta: inkwell::values::BasicMetadataValueEnum = args_list.into();

        // Call kyle_main(args_list)
        let result_call = self.builder.build_call(*kyle_main, &[args_meta], "result")
            .map_err(|e| format!("call kyle_main: {}", e))?;
        match result_call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => {
                self.builder.build_return(Some(&bv))
                    .map_err(|e| format!("main_wrapper ret: {}", e))?;
            }
            _ => {
                // kyle_main returns void — return 0
                self.builder.build_return(Some(&i32_ty.const_zero()))
                    .map_err(|e| format!("main_wrapper ret void: {}", e))?;
            }
        }

        Ok(())
    }

    /// Convert a BasicValueEnum to an IntValue, converting pointer to int if needed.
    fn to_int_value(&self, val: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx> {
        match val {
            BasicValueEnum::IntValue(i) => i,
            BasicValueEnum::PointerValue(p) => {
                self.builder.build_ptr_to_int(p, self.context.i64_type(), "")
                    .expect("ptrtoint")
            }
            _ => self.context.i32_type().const_zero(),
        }
    }

    fn to_float_value(&self, val: BasicValueEnum<'ctx>) -> inkwell::values::FloatValue<'ctx> {
        match val {
            BasicValueEnum::FloatValue(f) => f,
            BasicValueEnum::IntValue(i) => {
                self.builder.build_signed_int_to_float(i, self.context.f64_type(), "")
                    .expect("inttofloat")
            }
            _ => self.context.f64_type().const_zero(),
        }
    }

    fn value_to_llvm(
        &self,
        value: &MirValue,
        last_values: &HashMap<usize, BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            MirValue::Local(id) => self.load_value(*id, last_values),
            MirValue::Param(id) => {
                Ok(self.param_values.get(id).copied().unwrap_or_else(|| {
                    self.context.i32_type().const_zero().as_basic_value_enum()
                }))
            }
            MirValue::Constant(c) => Ok(self.constant_to_llvm(c)),
        }
    }

    // SSA Helper: binary operation
    fn ssa_binop(&self, op: MirBinaryOp, l: BasicValueEnum<'ctx>, r: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        let to_float = |v: BasicValueEnum<'ctx>| -> inkwell::values::FloatValue<'ctx> {
            if let BasicValueEnum::FloatValue(f) = v { f }
            else { self.builder.build_signed_int_to_float(self.to_int_value(v), self.context.f64_type(), "").unwrap() }
        };
        let to_int = |v: BasicValueEnum<'ctx>| -> inkwell::values::IntValue<'ctx> {
            if let BasicValueEnum::IntValue(i) = v { i }
            else { self.context.i32_type().const_zero() }
        };
        let l_is_float = matches!(l, BasicValueEnum::FloatValue(_));
        let r_is_float = matches!(r, BasicValueEnum::FloatValue(_));
        if l_is_float || r_is_float {
            let lf = to_float(l); let rf = to_float(r);
            Ok(match op {
                MirBinaryOp::Add => self.builder.build_float_add(lf, rf, "").map_err(|e| format!("fadd: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Sub => self.builder.build_float_sub(lf, rf, "").map_err(|e| format!("fsub: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Mul => self.builder.build_float_mul(lf, rf, "").map_err(|e| format!("fmul: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Div => self.builder.build_float_div(lf, rf, "").map_err(|e| format!("fdiv: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Eq => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lf, rf, "").map_err(|e| format!("feq: {}", e))?;
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("feqe: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Neq => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, lf, rf, "").map_err(|e| format!("fne: {}", e))?;
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fnee: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Lt => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OLT, lf, rf, "").map_err(|e| format!("flt: {}", e))?;
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("flte: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Gt => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OGT, lf, rf, "").map_err(|e| format!("fgt: {}", e))?;
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fgte: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Le => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OLE, lf, rf, "").map_err(|e| format!("fle: {}", e))?;
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("flee: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Ge => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OGE, lf, rf, "").map_err(|e| format!("fge: {}", e))?;
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fgee: {}", e))?.as_basic_value_enum() }
                _ => return Err("ssa: unsupported float op".into()),
            })
        } else {
            let li = to_int(l); let ri = to_int(r);
            Ok(match op {
                MirBinaryOp::Add => self.builder.build_int_add(li, ri, "").map_err(|e| format!("iadd: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Sub => self.builder.build_int_sub(li, ri, "").map_err(|e| format!("isub: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Mul => self.builder.build_int_mul(li, ri, "").map_err(|e| format!("imul: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Div => self.builder.build_int_signed_div(li, ri, "").map_err(|e| format!("idiv: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Rem => self.builder.build_int_signed_rem(li, ri, "").map_err(|e| format!("irem: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::And => self.builder.build_and(li, ri, "").map_err(|e| format!("iand: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Or => self.builder.build_or(li, ri, "").map_err(|e| format!("ior: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Xor => self.builder.build_xor(li, ri, "").map_err(|e| format!("ixor: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Shl => self.builder.build_left_shift(li, ri, "").map_err(|e| format!("ishl: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Shr => self.builder.build_right_shift(li, ri, true, "").map_err(|e| format!("ishr: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Eq => self.builder.build_int_compare(inkwell::IntPredicate::EQ, li, ri, "").map_err(|e| format!("ieq: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Neq => self.builder.build_int_compare(inkwell::IntPredicate::NE, li, ri, "").map_err(|e| format!("ine: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Lt => self.builder.build_int_compare(inkwell::IntPredicate::SLT, li, ri, "").map_err(|e| format!("ilt: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Gt => self.builder.build_int_compare(inkwell::IntPredicate::SGT, li, ri, "").map_err(|e| format!("igt: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Le => self.builder.build_int_compare(inkwell::IntPredicate::SLE, li, ri, "").map_err(|e| format!("ile: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Ge => self.builder.build_int_compare(inkwell::IntPredicate::SGE, li, ri, "").map_err(|e| format!("ige: {}", e))?.as_basic_value_enum(),
            })
        }
    }

    // SSA Helper: type cast
    fn ssa_cast(&self, val: BasicValueEnum<'ctx>, target: &BasicTypeEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        Ok(match (&val, target) {
            (BasicValueEnum::FloatValue(_), BasicTypeEnum::FloatType(t)) =>
                self.builder.build_float_cast(self.to_float_value(val), *t, "").map_err(|e| format!("fcs: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::IntValue(_), BasicTypeEnum::IntType(t)) => {
                let vi = self.to_int_value(val);
                let sw = vi.get_type().get_bit_width();
                let dw = t.get_bit_width();
                (if sw < dw { self.builder.build_int_z_extend(vi, *t, "") }
                 else if sw > dw { self.builder.build_int_truncate(vi, *t, "") }
                 else { self.builder.build_int_cast(vi, *t, "") })
                    .map_err(|e| format!("ics: {}", e))?.as_basic_value_enum()
            }
            (BasicValueEnum::IntValue(_), BasicTypeEnum::FloatType(t)) =>
                self.builder.build_signed_int_to_float(self.to_int_value(val), *t, "").map_err(|e| format!("itof: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::FloatValue(_), BasicTypeEnum::IntType(t)) =>
                self.builder.build_float_to_signed_int(self.to_float_value(val), *t, "").map_err(|e| format!("ftoi: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::PointerValue(_), BasicTypeEnum::IntType(t)) =>
                self.builder.build_ptr_to_int(val.into_pointer_value(), *t, "").map_err(|e| format!("ptoi: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::IntValue(_), BasicTypeEnum::PointerType(t)) =>
                self.builder.build_int_to_ptr(self.to_int_value(val), *t, "").map_err(|e| format!("itop: {}", e))?.as_basic_value_enum(),
            _ => val,
        })
    }

    // SSA Helper: convert MirValue to LLVM value using block_vals
    fn ssa_mir_val(&self, val: &MirValue, bv: &[HashMap<usize, BasicValueEnum<'ctx>>], bi: usize, _func: &SsaFunction) -> Result<BasicValueEnum<'ctx>, String> {
        match val {
            MirValue::Constant(c) => Ok(self.constant_to_llvm(c)),
            MirValue::Local(id) => {
                for m in bv.iter().take(bi + 1).rev() {
                    if let Some(&v) = m.get(id) { return Ok(v); }
                }
                Ok(self.context.i32_type().const_zero().as_basic_value_enum())
            }
            MirValue::Param(id) => self.param_values.get(id).copied()
                .ok_or_else(|| "ssa: param not found".to_string()),
        }
    }

    fn constant_to_llvm(&self, c: &MirConstant) -> BasicValueEnum<'ctx> {
        match c {
            MirConstant::I32(n) => self.context.i32_type().const_int(*n as u64, false).as_basic_value_enum(),
            MirConstant::I64(n) => self.context.i64_type().const_int(*n as u64, false).as_basic_value_enum(),
            MirConstant::F64(n) => self.context.f64_type().const_float(*n).as_basic_value_enum(),
            MirConstant::Bool(b) => self.context.bool_type().const_int(*b as u64, false).as_basic_value_enum(),
            MirConstant::String(s) => {
                self.builder.build_global_string_ptr(s, "")
                    .expect("global string ptr")
                    .as_pointer_value()
                    .as_basic_value_enum()
            }
            MirConstant::Void => self.context.i32_type().const_zero().as_basic_value_enum(),
        }
    }
}
