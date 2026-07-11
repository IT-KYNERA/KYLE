use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::AnyValueEnum;
use inkwell::values::AsValueRef;
use inkwell::values::InstructionValue as InkwellInstructionValue;
use inkwell::values::PointerValue;
use inkwell::IntPredicate;
use llvm_sys::core::{LLVMSetNSW, LLVMSetNUW};
use std::collections::{HashMap, HashSet};

use kyc_mir::mir::*;
use kyc_mir::ssa::{SsaFunction, SsaInst, SsaValueId};

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
    /// TBAA metadata nodes: root, then per-type descriptors.
    tbaa_nodes: HashMap<String, inkwell::values::MetadataValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    /// Add `!range !{i32 0, i32 2}` metadata to an integer comparison result,
    /// indicating the value is always 0 or 1 (bool-like).
    fn add_bool_range(&self, int_val: inkwell::values::IntValue<'ctx>) {
        let range_kind = self.context.get_kind_id("range");
        let lo = self.context.i32_type().const_zero();
        let hi = self.context.i32_type().const_int(2, false);
        let md = self.context.metadata_node(&[lo.into(), hi.into()]);
        if let Ok(iv) = inkwell::values::InstructionValue::try_from(AnyValueEnum::IntValue(int_val)) {
            let _ = iv.set_metadata(md, range_kind);
        }
    }

    /// Add `!tbaa` metadata to a load or store instruction.
    fn add_tbaa(&self, inst: inkwell::values::InstructionValue<'ctx>, tbaa_node: inkwell::values::MetadataValue<'ctx>) {
        let tbaa_kind = self.context.get_kind_id("tbaa");
        let _ = inst.set_metadata(tbaa_node, tbaa_kind);
    }
    /// Load a local's value, always preferring the alloca for cross-block correctness.
    /// Falls back to last_value_map for values that weren't stored to an alloca.
    fn load_value(
        &self,
        id: usize,
        last_values: &HashMap<usize, BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if this is a field pointer: single-load (get the pointer, not the value at it)
        if id < self.field_ptr_allocas.len() && self.field_ptr_allocas[id].is_some() {
            if let Some(field_ptr_alloca) = self.field_ptr_allocas[id] {
                let gep = self.builder.build_load(
                    self.context.ptr_type(Default::default()),
                    field_ptr_alloca, "_fgepload"
                ).map_err(|e| format!("load_value fptr {}: {}", id, e))?;
                return Ok(gep);  // Return the POINTER, not the value at the pointer
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

    /// Collect all MirValue::Local references from an instruction.
    fn collect_local_uses<F: FnMut(usize)>(&self, inst: &MirInst, callback: &mut F) {
        match inst {
            MirInst::Alloca { .. } => {}
            MirInst::Store { value, .. } => {
                if let MirValue::Local(id) = value { callback(*id); }
            }
            MirInst::Load { src, .. } => { callback(*src); }
            MirInst::BinaryOp { left, right, .. } => {
                if let MirValue::Local(id) = left { callback(*id); }
                if let MirValue::Local(id) = right { callback(*id); }
            }
            MirInst::UnaryOp { operand, .. } => {
                if let MirValue::Local(id) = operand { callback(*id); }
            }
            MirInst::Call { args, .. } => {
                for a in args { if let MirValue::Local(id) = a { callback(*id); } }
            }
            MirInst::PtrOffset { index, .. } => {
                if let MirValue::Local(id) = index { callback(*id); }
            }
            MirInst::PtrStore { index, value, .. } => {
                if let MirValue::Local(id) = index { callback(*id); }
                if let MirValue::Local(id) = value { callback(*id); }
            }
            MirInst::FieldPtr { .. } => {}
            MirInst::ArrayElemPtr { .. } => {}
            MirInst::Cast { value, .. } => {
                if let MirValue::Local(id) = value { callback(*id); }
            }
            MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                callback(*dest_ptr_local);
                callback(*src_alloca_local);
            }
            MirInst::FnAddr { .. } => {}
            MirInst::AddressOf { local_id, .. } => { callback(*local_id); }
            MirInst::CallIndirect { fn_ptr, args, .. } => {
                callback(*fn_ptr);
                for a in args { if let MirValue::Local(id) = a { callback(*id); } }
            }
            MirInst::AsyncSpawn { arg, .. } => {
                if let MirValue::Local(id) = arg { callback(*id); }
            }
            MirInst::AsyncAwait { handle, .. } => { callback(*handle); }
            MirInst::SliceMake { ptr, len, .. } => {
                if let MirValue::Local(id) = ptr { callback(*id); }
                if let MirValue::Local(id) = len { callback(*id); }
            }
        }
    }

    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name);
        // Set target triple and data layout for LLVM optimization passes
        let triple = inkwell::targets::TargetMachine::get_default_triple();

        module.set_triple(&triple);
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
            tbaa_nodes: HashMap::new(),
        }
    }

    /// Initialize TBAA metadata nodes for type-based alias analysis.
    fn init_tbaa(&mut self) {
        // Create access tags for struct-path TBAA (LLVM 18+).
        // Type descriptor: {name, parent, i64_offset}
        // Access tag: {type_descriptor, type_descriptor, 0}
        // The !tbaa metadata on loads/stores references the ACCESS TAG.
        let md_str = |s: &str| self.context.metadata_string(s);
        let i64_0 = || self.context.i64_type().const_int(0, false);

        let root = self.context.metadata_node(&[md_str("any level").into()]);
        let int_desc = self.context.metadata_node(&[md_str("int").into(), root.into(), i64_0().into()]);
        let float_desc = self.context.metadata_node(&[md_str("float").into(), root.into(), i64_0().into()]);
        let ptr_desc = self.context.metadata_node(&[md_str("ptr").into(), root.into(), i64_0().into()]);
        let list_desc = self.context.metadata_node(&[md_str("list").into(), root.into(), i64_0().into()]);
        let list_data_desc = self.context.metadata_node(&[md_str("list_data").into(), list_desc.into(), i64_0().into()]);
        let list_len_desc = self.context.metadata_node(&[md_str("list_len").into(), list_desc.into(), i64_0().into()]);

        // Access tags (what !tbaa references)
        let int_tag = self.context.metadata_node(&[int_desc.into(), int_desc.into(), i64_0().into()]);
        let float_tag = self.context.metadata_node(&[float_desc.into(), float_desc.into(), i64_0().into()]);
        let ptr_tag = self.context.metadata_node(&[ptr_desc.into(), ptr_desc.into(), i64_0().into()]);
        let list_data_tag = self.context.metadata_node(&[list_data_desc.into(), list_data_desc.into(), i64_0().into()]);
        let list_len_tag = self.context.metadata_node(&[list_len_desc.into(), list_len_desc.into(), i64_0().into()]);

        self.tbaa_nodes.insert("root".to_string(), root);
        self.tbaa_nodes.insert("int.desc".to_string(), int_desc);
        self.tbaa_nodes.insert("int".to_string(), int_tag);
        self.tbaa_nodes.insert("float".to_string(), float_tag);
        self.tbaa_nodes.insert("ptr".to_string(), ptr_tag);
        self.tbaa_nodes.insert("list_data".to_string(), list_data_tag);
        self.tbaa_nodes.insert("list_len".to_string(), list_len_tag);
    }

    fn tbaa_md(&self, name: &str) -> Option<inkwell::values::MetadataValue<'ctx>> {
        self.tbaa_nodes.get(name).copied()
    }

    /// Add struct-path TBAA metadata to a load instruction.
    fn tbaa_load(&self, val: inkwell::values::BasicValueEnum<'ctx>, node: &str) {
        if let Some(n) = self.tbaa_md(node) {
            if let Ok(inst) = inkwell::values::InstructionValue::try_from(inkwell::values::AnyValueEnum::from(val)) {
                self.add_tbaa(inst, n);
            }
        }
    }

    /// Add struct-path TBAA metadata to a store instruction.
    fn tbaa_store(&self, sv: impl Into<inkwell::values::InstructionValue<'ctx>>, node: &str) {
        if let Some(n) = self.tbaa_md(node) {
            self.add_tbaa(sv.into(), n);
        }
    }

    /// Get the TBAA access tag for a given MirType.
    fn tbaa_for_type(&self, ty: &MirType) -> Option<inkwell::values::MetadataValue<'ctx>> {
        match ty {
            MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64 | MirType::I1
            | MirType::Bool | MirType::Char => self.tbaa_nodes.get("int").copied(),
            MirType::F32 | MirType::F64 => self.tbaa_nodes.get("float").copied(),
            MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) => self.tbaa_nodes.get("ptr").copied(),
            MirType::Str | MirType::Ptr(_) | MirType::Struct(_, _) | MirType::Array(_, _) | MirType::Slice(_) | MirType::Box(_) => self.tbaa_nodes.get("ptr").copied(),
            MirType::Void => None,
        }
    }

    /// Create an integer add with nsw+nuw flags for signed/unsigned overflow UB optimization.
    fn int_nsw_nuw_add(&self, lhs: inkwell::values::IntValue<'ctx>, rhs: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let val = self.builder.build_int_nsw_add(lhs, rhs, "").map_err(|e| format!("add: {}", e))?;
        unsafe { LLVMSetNUW(val.as_value_ref(), true.into()); }
        Ok(val)
    }

    /// Create an integer sub with nsw+nuw flags.
    fn int_nsw_nuw_sub(&self, lhs: inkwell::values::IntValue<'ctx>, rhs: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let val = self.builder.build_int_nsw_sub(lhs, rhs, "").map_err(|e| format!("sub: {}", e))?;
        unsafe { LLVMSetNUW(val.as_value_ref(), true.into()); }
        Ok(val)
    }

    /// Create an integer mul with nsw+nuw flags.
    fn int_nsw_nuw_mul(&self, lhs: inkwell::values::IntValue<'ctx>, rhs: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let val = self.builder.build_int_nsw_mul(lhs, rhs, "").map_err(|e| format!("mul: {}", e))?;
        unsafe { LLVMSetNUW(val.as_value_ref(), true.into()); }
        Ok(val)
    }

    /// Emit `llvm.lifetime.start` for an alloca to help LLVM reuse stack slots.
    fn emit_lifetime_start(&self, ptr: inkwell::values::PointerValue<'ctx>, size: i64) {
        if let Some(func) = self.module.get_function("llvm.lifetime.start.p0") {
            let size_val = self.context.i64_type().const_int(size as u64, false);
            if let Ok(call) = self.builder.build_call(func, &[size_val.into(), ptr.into()], "") {
                let _ = call.try_as_basic_value();
            }
        }
    }

    /// Emit `llvm.lifetime.end` for an alloca.
    fn emit_lifetime_end(&self, ptr: inkwell::values::PointerValue<'ctx>, size: i64) {
        if let Some(func) = self.module.get_function("llvm.lifetime.end.p0") {
            let size_val = self.context.i64_type().const_int(size as u64, false);
            if let Ok(call) = self.builder.build_call(func, &[size_val.into(), ptr.into()], "") {
                let _ = call.try_as_basic_value();
            }
        }
    }

    /// Map an LLVM BasicTypeEnum to a TBAA metadata node.
    fn tbaa_for_llvm_type(&self, t: &BasicTypeEnum<'ctx>) -> Option<inkwell::values::MetadataValue<'ctx>> {
        match t {
            BasicTypeEnum::IntType(_) => self.tbaa_nodes.get("int").copied(),
            BasicTypeEnum::FloatType(_) => self.tbaa_nodes.get("float").copied(),
            BasicTypeEnum::PointerType(_) | BasicTypeEnum::StructType(_) | BasicTypeEnum::ArrayType(_) => self.tbaa_nodes.get("ptr").copied(),
            _ => None,
        }
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn compile(&mut self, mir_module: &MirModule) -> Result<(), String> {
        self.init_tbaa();
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
        self.init_tbaa();
        let result = kyc_mir::ssa::convert_module(mir_module);
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
        // Parameter attributes: noundef on all, noalias on pointer types
        let noundef_kind = Attribute::get_named_enum_kind_id("noundef");
        let noalias_kind = Attribute::get_named_enum_kind_id("noalias");
        for (i, ptype) in func.params.iter().enumerate() {
            let idx = i as u32;
            if noundef_kind > 0 {
                let attr = self.context.create_enum_attribute(noundef_kind, 0);
                fn_value.add_attribute(AttributeLoc::Param(idx), attr);
            }
            if noalias_kind > 0 {
                if matches!(ptype, MirType::Struct(_, _) | MirType::Str | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) | MirType::Ptr(_) | MirType::Box(_)) {
                    let attr = self.context.create_enum_attribute(noalias_kind, 0);
                    fn_value.add_attribute(AttributeLoc::Param(idx), attr);
                }
            }
        }
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

        // Ref params: change struct param allocas from struct type to ptr
        for block in &func.blocks {
            for inst in &block.insts {
                if let SsaInst::Store { dest, value } = inst {
                    if let Some(param_idx) = func.param_value_ids.iter().position(|&p| p == *value) {
                        if matches!(&func.params[param_idx], MirType::Struct(_, _)) {
                            if let Some(&llvm_type) = self.alloca_types.get(dest) {
                                if matches!(llvm_type, BasicTypeEnum::StructType(_)) {
                                    let orig_type = self.alloca_types.insert(*dest, ptr_ty.as_basic_type_enum()).unwrap();
                                    self.ref_param_struct_types.insert(*dest, orig_type);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Create LLVM basic blocks and phi nodes
        let mut block_map: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();
        let mut block_indices: HashMap<String, usize> = HashMap::new();
        let mut phi_map: Vec<(usize, inkwell::values::PhiValue<'ctx>)> = Vec::new();
        for (i, block) in func.blocks.iter().enumerate() {
            let llvm_bb = self.context.append_basic_block(fn_value, &block.label);
            block_map.insert(block.label.clone(), llvm_bb);
            block_indices.insert(block.label.clone(), i);
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
                    if let Ok(iv) = inkwell::values::InstructionValue::try_from(inkwell::values::AnyValueEnum::PointerValue(ptr)) {
                        let _ = iv.set_alignment(8);
                    }
                    // // self.emit_lifetime_start(ptr, -1); // DEBUG: disabled for mem2reg test // DEBUG: disabled for mem2reg test
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
                        if let SsaInst::ArrayElemPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest { self.field_ptr_allocas.push(None); }
                            if self.field_ptr_allocas[*dest].is_none() {
                                self.field_ptr_allocas[*dest] = Some(
                                    self.builder.build_alloca(ptr_ty, "_aelem")
                                        .map_err(|e| format!("ssa aep: {}", e))?
                                );
                            }
                        }
                        if let SsaInst::PtrOffset { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest { self.field_ptr_allocas.push(None); }
                            if self.field_ptr_allocas[*dest].is_none() {
                                self.field_ptr_allocas[*dest] = Some(
                                    self.builder.build_alloca(ptr_ty, "_pgep")
                                        .map_err(|e| format!("ssa pgep: {}", e))?
                                );
                            }
                        }
                        if let SsaInst::Load { src, .. } = inst {
                            while self.field_ptr_allocas.len() <= *src { self.field_ptr_allocas.push(None); }
                        }
                    }
                }
                self.param_values.clear();
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

        // Seed block_vals[0] with param values for ssa_read! in the entry block
        for (&i, &p) in &self.param_values {
            if let Some(&pid) = func.param_value_ids.get(i) {
                block_vals[0].insert(pid, p);
            }
        }

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
                        // 2. Search ALL prior blocks' block_vals (cross-block phis/values)
                        else {
                            let mut found = false;
                            let mut val = self.context.i32_type().const_zero().as_basic_value_enum();
                            for bv in block_vals[..=bi].iter().rev() {
                                if let Some(&v) = bv.get(&id) { val = v; found = true; break; }
                            }
                            if found { val }
                            // 3. Constant values from const_values
                            else if let Some(c) = func.const_values.get(&id) { self.constant_to_llvm(c) }
                            // 4. Fallback to alloca_map (non-promotable allocas like structs, slices)
                            else if let Some(Some(ptr)) = self.alloca_map.get(id) {
                                self.alloca_types.get(&id).and_then(|pointee_type| {
                                    self.builder.build_load(*pointee_type, *ptr, "_ssaload").ok()
                                }).unwrap_or_else(|| self.context.i32_type().const_zero().as_basic_value_enum())
                            }
                            // 5. Default zero
                            else { self.context.i32_type().const_zero().as_basic_value_enum() }
                        }
                    }};
                }

                // Helper: resolve MirValue to LLVM value using block_local_map
                macro_rules! resolve_mir {
                    ($val:expr) => {{
                        let val: &MirValue = $val;
                        match val {
                            MirValue::Constant(c) => self.constant_to_llvm(c),
                            MirValue::Local(mir_id) => {
                                if let Some(ssa_id) = func.block_local_map.get(bi).and_then(|m| m.get(mir_id)).copied() {
                                    ssa_read!(ssa_id)
                                } else if let Some(&v) = alloca_current.get(mir_id) {
                                    v
                                } else if let Some(v) = block_vals.get(bi).and_then(|bv| bv.get(mir_id)).copied() {
                                    v
                                } else if let Some(Some(ptr)) = self.alloca_map.get(*mir_id) {
                                    // Load from actual alloca as fallback (handles non-promotable)
                                    if let Some(pointee_type) = self.alloca_types.get(mir_id) {
                                        self.builder.build_load(*pointee_type, *ptr, "_ssaload")
                                            .map_err(|e| format!("ssa-load {}: {}", mir_id, e))?
                                            .as_basic_value_enum()
                                    } else {
                                        self.context.i32_type().const_zero().as_basic_value_enum()
                                    }
                                } else {
                                    self.context.i32_type().const_zero().as_basic_value_enum()
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
                                let val = if let Some(dest_type) = self.alloca_types.get(dest) {
                                    if val.get_type() != *dest_type {
                                        self.ssa_cast(val, dest_type).unwrap_or(val)
                                    } else { val }
                                } else { val };
                                self.builder.build_store(ptr, val)
                                    .map_err(|e| format!("ssast: {}", e))?;
                                // Also track in block_vals so ssa_read! can find it for Call args
                                block_vals[bi].insert(*value, val);
                                alloca_current.insert(*dest, val);
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
                                    let ft = self.field_ptr_types.get(src).copied();
                                    let lt = ft.or_else(|| {
                                        let t = func.values.get(*dest).map(|sv| {
                                            eprintln!("DBG Load fallback: dest={} sv_type={}", dest, sv.type_);
                                            self.llvm_type(&sv.type_)
                                        });
                                        if t.is_some() { eprintln!("DBG Load fallback FOUND"); }
                                        t
                                    }).or_else(|| {
                                        self.alloca_types.get(src).copied()
                                    }).unwrap_or(self.context.i64_type().as_basic_type_enum());
                                    Some(self.builder.build_load(lt, gep.into_pointer_value(), "")
                                        .map_err(|e| format!("sfld2: {}", e))?)
                                } else { None }
                            } else if let Some(ptr) = self.alloca_map.get(*src).and_then(|p| *p) {
                                if let Some(lt) = self.alloca_types.get(src) {
                                    Some(self.builder.build_load(*lt, ptr, "")
                                        .map_err(|e| format!("ssald: {}", e))?)
                                } else { None }
                            } else {
                                // Promoted alloca: read from block_vals or global map
                                block_vals[bi].get(src).copied()
                                    .or_else(|| alloca_current.get(src).copied())
                            };
                            if let Some(v) = loaded {
                                block_vals[bi].insert(*dest, v);
                                // Also store to alloca if dest is non-promotable (for resolve_mir! fallback)
                                if let Some(dptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    if let Some(dt) = self.alloca_types.get(dest) {
                                        let _ = self.builder.build_store(dptr, v);
                                    }
                                }
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
                            // Inline list operations in SSA path (same as MIR path)
                            match name.as_str() {
                                "ky_list_get" | "ky_list_set" | "ky_list_len" => {
                                    let val = self.emit_ssa_inline_list_op(name, &block_vals[bi], args)?;
                                    if let Some(d) = dest {
                                        if let Some(v) = val {
                                            block_vals[bi].insert(*d, v);
                                        }
                                    }
                                    continue;
                                }
                                _ => {}
                            }
                            let runtime_name = if self.fn_value_map.contains_key(name) {
                                name.clone()
                            } else {
                                match name.as_str() {
                                "print" => "ky_print", "println" => "ky_println",
                                "contains" => "ky_str_contains", "to_upper" => "ky_str_to_upper",
                                "to_lower" => "ky_str_to_lower", "trim" => "ky_str_trim",
                                "replace" => "ky_str_replace", "input" => "ky_input",
                                "open" => "ky_open", "read_str" => "ky_read_str",
                                "write_str" => "ky_write_str", "close" => "ky_close",
                                "sleep" => "ky_sleep", "now" => "ky_now",
                                "char_at" => "ky_char_at", "is_digit" => "ky_is_digit",
                                "is_alpha" => "ky_is_alpha", "is_alnum" => "ky_is_alnum",
                                "is_whitespace" => "ky_is_whitespace", "is_upper" => "ky_is_upper",
                                "is_lower" => "ky_is_lower", "ord" => "ky_ord",
                                "substr" => "ky_substr",
                                // json_parse/stringify handled via prelude wrappers
                                "assert" => "ky_assert", "assert_eq" => "ky_assert_eq",
                                "assert_ne" => "ky_assert_ne", "assert_str" => "ky_assert_str_eq",
                                "list_new" => "ky_list_new", "list_push" => "ky_list_push",
                                "list_get" => "ky_list_get", "list_set" => "ky_list_set",
                                "list_len" => "ky_list_len", "list_pop" => "ky_list_pop", "reserve" => "ky_list_reserve",
                                "ky_str_builder_new" => "ky_str_builder_new",
                                "ky_str_builder_append" => "ky_str_builder_append",
                                "ky_str_builder_to_str" => "ky_str_builder_to_str",
                                "ky_str_builder_free" => "ky_str_builder_free",
                                _ => name.as_str(),
                                }.to_string()
                            };
                             if self.module.get_function(&runtime_name).is_none() {
                                 let ret_type = if let Some(d) = dest {
                                     let mir_type = func.values.get(*d).map(|v| &v.type_).unwrap_or(&MirType::I64);
                                     self.llvm_type(mir_type).as_basic_type_enum()
                                 } else {
                                     self.context.i64_type().as_basic_type_enum()
                                 };
                                 let param_tys: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = args.iter()
                                     .filter_map(|a| func.values.get(*a))
                                     .map(|v| self.llvm_type(&v.type_).into())
                                     .collect();
                                 let fn_type = ret_type.fn_type(&param_tys, false);
                                 self.module.add_function(&runtime_name, fn_type, None);
                             }
                            if let Some(callee) = self.module.get_function(&runtime_name) {
                                let fn_ty = callee.get_type();
                                let param_tys = fn_ty.get_param_types();
                                let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = args.iter()
                                    .enumerate().map(|(i, a)| {
                                        let v = ssa_read!(*a);
                                        if i < param_tys.len() {
                                            match param_tys[i] {
                                                inkwell::types::BasicMetadataTypeEnum::PointerType(_) => {
                                                    if let BasicValueEnum::IntValue(iv) = v {
                                                        let ptr_ty = self.context.ptr_type(Default::default());
                                                        if let Ok(pv) = self.builder.build_int_to_ptr(iv, ptr_ty, "_aptr") {
                                                            return pv.into();
                                                        }
                                                    }
                                                    // Struct value → pass alloca pointer if available
                                                    if let BasicValueEnum::StructValue(sv) = v {
                                                        // Check if the arg has a non-promotable alloca to pass by reference
                                                        if let Some(Some(ptr)) = self.alloca_map.get(*a) {
                                                            return ptr.as_basic_value_enum().into();
                                                        }
                                                        // Otherwise create temp alloca (pass-by-ref ABI)
                                                        let st = sv.get_type();
                                                        if let Ok(temp) = self.builder.build_alloca(st, "_stmp") {
                                                            let _ = self.builder.build_store(temp, sv);
                                                            return temp.as_basic_value_enum().into();
                                                        }
                                                    }
                                                }
                                                inkwell::types::BasicMetadataTypeEnum::IntType(it) if it.get_bit_width() > 32 => {
                                                    if let BasicValueEnum::IntValue(iv) = v {
                                                        let vw = iv.get_type().get_bit_width();
                                                        if vw < it.get_bit_width() {
                                                            if let Ok(ext) = self.builder.build_int_s_extend(iv, it, "_ssac") {
                                                                return ext.into();
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
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
                        SsaInst::AddressOf { dest, local_id } => {
                            if let Some(Some(ptr)) = self.alloca_map.get(*local_id) {
                                block_vals[bi].insert(*dest, ptr.as_basic_value_enum());
                            }
                        }
                        SsaInst::CallIndirect { dest, fn_ptr, ret_type, param_types, args } => {
                            let raw_val = ssa_read!(*fn_ptr);
                            let fn_ptr = match raw_val {
                                BasicValueEnum::IntValue(iv) => {
                                    let ptr_ty = self.context.ptr_type(Default::default());
                                    self.builder.build_int_to_ptr(iv, ptr_ty, "_ssa_fnptr")
                                        .map_err(|e| format!("ssaic inttoptr: {}", e))?
                                }
                                _ => raw_val.into_pointer_value(),
                            };
                            let llvm_ret = self.llvm_type(ret_type);
                            let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types.iter()
                                .map(|p| self.llvm_type(p).into()).collect();
                            let fn_ty = llvm_ret.fn_type(&llvm_params, false);
                            let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum> = args.iter()
                                .map(|a| { let v = ssa_read!(*a); v.into() }).collect();
                            let call_res = unsafe {
                                self.builder.build_indirect_call(fn_ty, fn_ptr, &llvm_args, "_sicl")
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
                            let spawn_fn = self.module.get_function("ky_spawn_task")
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
                            let join_fn = self.module.get_function("ky_await_task")
                                .ok_or_else(|| "no kl_await_task".to_string())?;
                            let args_m: Vec<inkwell::values::BasicMetadataValueEnum> = vec![handle_val.into()];
                            let call_res = self.builder.build_call(join_fn, &args_m, "_ssaaw")
                                .map_err(|e| format!("ssaaw: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                block_vals[bi].insert(*dest, rv);
                            }
                        }
                        SsaInst::PtrOffset { dest, ptr, index, elem_type } => {
                            let base_val = ssa_read!(*ptr);
                            let idx_val = ssa_read!(*index);
                            let ptr_val = match base_val {
                                BasicValueEnum::IntValue(iv) => self.builder.build_int_to_ptr(iv, self.context.ptr_type(Default::default()), "_ptttr")
                                    .map_err(|e| format!("ptroff inttoptr: {}", e))?,
                                BasicValueEnum::PointerValue(pv) => pv,
                                _ => return Err(format!("ptroff: unexpected base for ptr={}", ptr)),
                            };
                            let gep = unsafe {
                                self.builder.build_in_bounds_gep(self.context.i8_type(), ptr_val, &[self.to_int_value(idx_val)], "_ssgep")
                                    .map_err(|e| format!("ssgep: {}", e))?
                            };
                            block_vals[bi].insert(*dest, gep.as_basic_value_enum());
                            if *dest < self.field_ptr_allocas.len() {
                                if let Some(fpa) = self.field_ptr_allocas[*dest] {
                                    let _ = self.builder.build_store(fpa, gep.as_basic_value_enum());
                                }
                            }
                            let elem_llvm = self.llvm_type(elem_type);
                            self.field_ptr_types.insert(*dest, elem_llvm);
                        }
                        SsaInst::PtrStore { ptr, index, value } => {
                            if let Some(base) = self.alloca_map.get(*ptr).and_then(|p| *p) {
                                let idx = ssa_read!(*index);
                                let val = ssa_read!(*value);
                                let gep = unsafe {
                                    self.builder.build_in_bounds_gep(self.context.i8_type(), base, &[self.to_int_value(idx)], "_ssps")
                                        .map_err(|e| format!("ssps gep: {}", e))?
                                };
                                self.builder.build_store(gep, val)
                                    .map_err(|e| format!("ssps store: {}", e))?;
                            }
                        }
                        SsaInst::FieldPtr { dest, ptr, field_index, struct_type } => {
                            let base_ptr = self.alloca_map.get(*ptr).and_then(|p| *p)
                                .or_else(|| {
                                    let val = block_vals.get(bi).and_then(|m| {
                                        func.block_local_map.get(bi)
                                            .and_then(|m2| m2.get(ptr))
                                            .and_then(|sid| m.get(sid))
                                    }).or_else(|| alloca_current.get(ptr));
                                    match val.copied() {
                                        Some(BasicValueEnum::StructValue(sv)) => {
                                            let st = sv.get_type();
                                            self.builder.build_alloca(st, "_stmp").ok()
                                        }
                                        _ => None,
                                    }
                                });
                            if let Some(base) = base_ptr {
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
                                    // Check if the alloca stores a pointer to the struct (Ptr(Struct) closure param)
                                    let gep = if self.ref_param_struct_types.contains_key(ptr) {
                                        // Ref param: load pointer from alloca, GEP on struct
                                        let struct_ptr = self.builder.build_load(
                                            self.context.ptr_type(Default::default()), base, "_ssref_load"
                                        ).map_err(|e| format!("ssrefld: {}", e))?;
                                        unsafe {
                                            self.builder.build_in_bounds_gep(st, struct_ptr.into_pointer_value(), &[zero, idx_v], "_ssrfptr")
                                                .map_err(|e| format!("ssrfptr: {}", e))?
                                        }
                                    } else if let Some(&alloca_ty) = self.alloca_types.get(ptr) {
                                        if alloca_ty.is_pointer_type() {
                                            // Ptr(Struct): load pointer from alloca, GEP on struct
                                            if let MirType::Struct(sname, fields) = struct_type.as_ref() {
                                                if !fields.is_empty() {
                                                    let struct_llvm = self.llvm_type(&MirType::Struct(sname.clone(), fields.clone()));
                                                    let struct_ptr = self.builder.build_load(
                                                        self.context.ptr_type(Default::default()), base, "_ssptrld"
                                                    ).map_err(|e| format!("ssptrld: {}", e))?;
                                                    unsafe {
                                                        self.builder.build_in_bounds_gep(struct_llvm, struct_ptr.into_pointer_value(), &[zero, idx_v], "_sspptr")
                                                            .map_err(|e| format!("sspptr: {}", e))?
                                                    }
                                                } else {
                                                    unsafe {
                                                        self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                            .map_err(|e| format!("ssfptr: {}", e))?
                                                    }
                                                }
                                            } else {
                                                unsafe {
                                                    self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                        .map_err(|e| format!("ssfptr: {}", e))?
                                                }
                                            }
                                        } else {
                                            unsafe {
                                                self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                    .map_err(|e| format!("ssfptr: {}", e))?
                                            }
                                        }
                                    } else {
                                        unsafe {
                                            self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                .map_err(|e| format!("ssfptr: {}", e))?
                                        }
                                    };
                                    self.builder.build_store(fpa, gep)
                                        .map_err(|e| format!("ssfptr2: {}", e))?;
                                }
                            }
                        }
                        SsaInst::ArrayElemPtr { dest, ptr, index, arr_type, elem_type } => {
                            let base_ptr = if *ptr < self.field_ptr_allocas.len() && self.field_ptr_allocas[*ptr].is_some() {
                                let p = self.field_ptr_allocas[*ptr].unwrap();
                                Some(self.builder.build_load(
                                    self.context.ptr_type(Default::default()), p, "_ssaebase"
                                ).map_err(|e| format!("ssaebase: {}", e))?.as_basic_value_enum())
                            } else {
                                self.alloca_map.get(*ptr).and_then(|p| *p).map(|p| p.as_basic_value_enum())
                            };
                            if let Some(base) = base_ptr {
                                if let BasicValueEnum::PointerValue(base_pv) = base {
                                    let arr_llvm = self.llvm_type(arr_type);
                                    let zero = self.context.i32_type().const_zero();
                                    let idx_val = block_vals[bi].get(index).copied()
                                        .or_else(|| func.const_values.get(index).map(|c| self.constant_to_llvm(c)))
                                        .or_else(|| self.alloca_map.get(*index).and_then(|p| *p).map(|p| p.as_basic_value_enum()))
                                        .unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum());
                                    let idx_i32 = if let BasicValueEnum::IntValue(iv) = idx_val {
                                        if iv.get_type().get_bit_width() != 32 {
                                            self.builder.build_int_truncate(iv, self.context.i32_type(), "_ssaetrunc")
                                                .map_err(|e| format!("ssaetrunc: {}", e))?
                                        } else { iv }
                                    } else {
                                        self.context.i32_type().const_zero()
                                    };
                                    let gep = unsafe {
                                        self.builder.build_in_bounds_gep(arr_llvm, base_pv, &[zero, idx_i32], "_ssaelem")
                                            .map_err(|e| format!("ssaelem: {}", e))?
                                    };
                                    block_vals[bi].insert(*dest, gep.as_basic_value_enum());
                                    if *dest < self.field_ptr_allocas.len() {
                                        if let Some(fpa) = self.field_ptr_allocas[*dest] {
                                            self.builder.build_store(fpa, gep)
                                                .map_err(|e| format!("aelem fpa store: {}", e))?;
                                        }
                                    }
                                    // Also store to alloca_map for ssa_read! fallback
                                    if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                        let _ = self.builder.build_store(dest_ptr, gep.as_basic_value_enum());
                                    }
                                    let elem_llvm = self.llvm_type(elem_type);
                                    self.field_ptr_types.insert(*dest, elem_llvm);
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
                        SsaInst::SliceMake { dest, ptr, len, elem_type } => {
                            let ptr_val = ssa_read!(*ptr);
                            let len_val = ssa_read!(*len);
                            let slice_type = self.llvm_type(&MirType::Slice(elem_type.clone()));
                            if let BasicTypeEnum::StructType(st) = slice_type {
                                let undef = st.get_undef();
                                let sv = unsafe {
                                    self.builder.build_insert_value(undef, ptr_val, 0, "_smi0")
                                        .map_err(|e| format!("smi0: {}", e))?
                                };
                                let sv = unsafe {
                                    self.builder.build_insert_value(sv, len_val, 1, "_smi1")
                                        .map_err(|e| format!("smi1: {}", e))?
                                };
                                block_vals[bi].insert(*dest, sv.as_basic_value_enum());
                            }
                        }
                    }
                }

                // Terminator
                match &block.terminator {
                    MirTerminator::Return(val) => {
                        let ret = resolve_mir!(val);
                        // Auto-cast return value if it doesn't match function return type
                        let ret_ty = fn_value.get_type().get_return_type();
                        let ret = if let Some(expected_ret_ty) = ret_ty {
                            if ret.get_type() != expected_ret_ty.as_basic_type_enum() {
                                match (&ret, &expected_ret_ty) {
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::PointerType(pt)) =>
                                        self.builder.build_int_to_ptr(*iv, *pt, "_retptr")
                                            .map_err(|e| format!("ssa ret inttoptr: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::PointerValue(pv), BasicTypeEnum::IntType(it)) =>
                                        self.builder.build_ptr_to_int(*pv, *it, "_retint")
                                            .map_err(|e| format!("ssa ret ptrtoint: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(it)) => {
                                        let sw = iv.get_type().get_bit_width();
                                        let dw = it.get_bit_width();
                                        if sw == 1 && dw > 1 {
                                            self.builder.build_int_z_extend(*iv, *it, "_retzext")
                                                .map_err(|e| format!("ssa ret zext: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            self.builder.build_int_cast(*iv, *it, "_retcast")
                                                .map_err(|e| format!("ssa ret cast: {}", e))?
                                                .as_basic_value_enum()
                                        }
                                    }
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::StructType(st)) => {
                                        let ptr_ty = self.context.ptr_type(Default::default());
                                        let ptr_val = self.builder.build_int_to_ptr(*iv, ptr_ty, "_retptr")
                                            .map_err(|e| format!("ssa ret inttoptr: {}", e))?;
                                        self.builder.build_load(*st, ptr_val, "_retstruct")
                                            .map_err(|e| format!("ssa ret load struct: {}", e))?
                                    }
                                    _ => ret,
                                }
                            } else { ret }
                        } else { ret };
                        self.builder.build_return(Some(&ret))
                            .map_err(|e| format!("ssaret: {}", e))?;
                    }
                    MirTerminator::Br(label) => {
                        if let Some(&tb) = block_map.get(label) {
                            let br = self.builder.build_unconditional_branch(tb)
                                .map_err(|e| format!("ssabr: {}", e))?;
                            // Mark loop back-edges for LLVM optimizations (vectorize, unroll)
                            if let Some(target_idx) = block_indices.get(label) {
                                if *target_idx < bi {
                                    let loop_md = self.context.metadata_node(&[
                                        self.context.metadata_string("llvm.loop.vectorize.enable").into(),
                                    ]);
                                    let kind = self.context.get_kind_id("llvm.loop");
                                    let _ = br.set_metadata(loop_md, kind);
                                }
                            }
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
                            let br = self.builder.build_conditional_branch(i1_cond, tb, fb)
                                .map_err(|e| format!("ssacbr: {}", e))?;
                            // Mark loop back-edges (conditional branch to previous block)
                            if let Some(target_idx) = block_indices.get(true_block) {
                                if *target_idx < bi {
                                    let loop_md = self.context.metadata_node(&[
                                        self.context.metadata_string("llvm.loop.vectorize.enable").into(),
                                    ]);
                                    let kind = self.context.get_kind_id("llvm.loop");
                                    let _ = br.set_metadata(loop_md, kind);
                                }
                            }
                            if let Some(target_idx) = block_indices.get(false_block) {
                                if *target_idx < bi {
                                    let loop_md = self.context.metadata_node(&[
                                        self.context.metadata_string("llvm.loop.vectorize.enable").into(),
                                    ]);
                                    let kind = self.context.get_kind_id("llvm.loop");
                                    let _ = br.set_metadata(loop_md, kind);
                                }
                            }
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
                    let mut incomings: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
                    for &(val_id, ref pred_label) in &phi.incomings {
                        if let Some(pred_bb) = block_map.get(pred_label).copied() {
                            if let Some(pred_bi) = func.blocks.iter().position(|b| &b.label == pred_label) {
                                 let mut val = block_vals[pred_bi].get(&val_id).copied()
                                     .or_else(|| {
                                         func.const_values.get(&val_id)
                                             .map(|c| self.constant_to_llvm(c))
                                     })
                                     .unwrap_or_else(|| {
                                         let phi_type = self.llvm_type(&phi.type_);
                                         match phi_type {
                                             BasicTypeEnum::IntType(it) => it.const_zero().as_basic_value_enum(),
                                             BasicTypeEnum::FloatType(ft) => ft.const_zero().as_basic_value_enum(),
                                             _ => self.context.i32_type().const_zero().as_basic_value_enum(),
                                         }
                                     });
                                 let phi_type = self.llvm_type(&phi.type_);
                                 if val.get_type() != phi_type {
                                     if let Some(term) = pred_bb.get_terminator() {
                                         self.builder.position_before(&term);
                                     }
                                     val = self.ssa_cast(val, &phi_type).unwrap_or(val);
                                 }
                                 incomings.push((val, pred_bb));
                            }
                        }
                    }
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
            self.module.add_function("ky_print", ft, None);
        }
        // void kl_println(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_println", ft, None);
        }
        // ptr kl_alloc(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_alloc", ft, None);
        }
        // void kl_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_free", ft, None);
        }
        // ptr kl_i64_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_i64_to_str", ft, None);
        }
        // ptr kl_f64_to_str(f64)
        {
            let params = [f64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_f64_to_str", ft, None);
        }
        // i64 kl_str_to_i64(ptr) — parse string to i64
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_i64", ft, None);
        }
        // i32 kl_strlen(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_strlen", ft, &[("memory", "read")]);
        }
        // ptr kl_concat(ptr, i32, ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_concat", ft, None);
        }
        // ptr kl_input()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_input", ft, None);
        }
        // ptr kl_input_with_prompt(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_input_with_prompt", ft, None);
        }
        // i32 kl_str_contains(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_str_contains", ft, &[("memory", "read")]);
        }
        // ptr kl_str_to_upper(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_upper", ft, None);
        }
        // ptr kl_str_to_lower(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_lower", ft, None);
        }
        // ptr kl_str_trim(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_trim", ft, None);
        }
        // ptr kl_str_replace(ptr, ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_replace", ft, None);
        }
        // i32 kl_open(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_open", ft, None);
        }
        // ptr kl_read_str(i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_read_str", ft, None);
        }
        // i32 kl_write_str(i32, ptr)
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_write_str", ft, None);
        }
        // i32 kl_close(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_close", ft, None);
        }
        // ptr kl_from_cstr(ptr) — convert C string to Kyle string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_from_cstr", ft, None);
        }
        // ptr kl_getenv(ptr) — read environment variable
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_getenv", ft, None);
        }
        // i32 kl_setenv(ptr, ptr, i32) — set environment variable
        {
            let params = [ptr_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_setenv", ft, None);
        }
        // void kl_sleep(i32)
        {
            let params = [i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_sleep", ft, None);
        }
        // i64 kl_now()
        {
            let ft = i64_ty.fn_type(&[], false);
            self.module.add_function("ky_now", ft, None);
        }
        // i32 ky_tcp_listen(i32) — create TCP listener
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_listen", ft, None);
        }
        // i32 ky_tcp_accept(i32) — accept connection
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_accept", ft, None);
        }
        // ptr ky_tcp_read(i32, i32) — read from socket
        {
            let params = [i32_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_read", ft, None);
        }
        // i32 ky_tcp_write(i32, ptr, i32) — write to socket
        {
            let params = [i32_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_write", ft, None);
        }
        // i32 ky_ptr_read_i32(ptr) — read i32 from memory
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_ptr_read_i32", ft, None);
        }
        // ptr ky_ptr_read_ptr(ptr) — read ptr from memory
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_ptr_read_ptr", ft, None);
        }
        // void ky_ptr_write_i32(ptr, i32) — write i32 to memory
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = self.context.void_type().fn_type(&params, false);
            self.module.add_function("ky_ptr_write_i32", ft, None);
        }
        // i32 ky_tcp_close(i32) — close socket
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_close", ft, None);
        }
        // i32 ky_sha1(ptr, i32, ptr) — SHA-1 hash
        {
            let params = [ptr_ty.into(), i32_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_sha1", ft, None);
        }
        // ptr ky_base64_encode(ptr, i32) — Base64 encode
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_base64_encode", ft, None);
        }
        // ptr ky_ws_accept(ptr) — WebSocket handshake accept key
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_ws_accept", ft, None);
        }
        // ptr ky_ws_read_frame(i32) — read WebSocket frame
        {
            let params = [i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_ws_read_frame", ft, None);
        }
        // i32 ky_ws_send_frame(i32, i32, ptr, i32) — send WebSocket frame
        {
            let params = [i32_ty.into(), i32_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_ws_send_frame", ft, None);
        }
        // i64 ky_pow(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_pow", ft, None);
        }
        // i64 ky_add_pct(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_add_pct", ft, None);
        }
        // i64 ky_sub_pct(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_sub_pct", ft, None);
        }
        // i64 ky_mul_pct(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_mul_pct", ft, None);
        }
        // i8 kl_char_at(ptr, i32) — readonly
        {
            let i8_ty = self.context.i8_type();
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = i8_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_char_at", ft, &[("memory", "read")]);
        }
        // ptr kl_list_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_list_new", ft, None);
        }
        // void kl_list_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_free", ft, None);
        }
        // ptr kl_range(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_range", ft, None);
        }
        // ptr kl_range_two(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_range_two", ft, None);
        }
        // void kl_list_push(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_push", ft, None);
        }
        // i64 kl_list_pop(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_pop", ft, None);
        }
        // i64 kl_list_get(ptr, i64) — readonly
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_list_get", ft, &[("memory", "read")]);
        }
        // void kl_list_set(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_set", ft, None);
        }
        // i64 kl_list_len(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_list_len", ft, &[("memory", "read")]);
        }
        // i64 kl_list_sum(ptr), kl_list_product(ptr), kl_list_max(ptr), kl_list_min(ptr) — readonly
        for name in &["ky_list_sum", "ky_list_product", "ky_list_max", "ky_list_min"] {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern(name, ft, &[("memory", "read")]);
        }
        // void kl_list_reverse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_reverse", ft, None);
        }
        // ptr kl_list_slice(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_list_slice", ft, None);
        }
        // void kl_list_extend(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_extend", ft, None);
        }
        // i64 kl_list_pop_first(ptr) — remove and return first element
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_pop_first", ft, None);
        }
        // void kl_list_clear(ptr) — remove all elements
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_clear", ft, None);
        }
        // i32 kl_list_contains(ptr, i64) — readonly
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_list_contains", ft, &[("memory", "read")]);
        }
        // void kl_list_insert(ptr, i64, i64) — insert at index
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_insert", ft, None);
        }
        // i64 kl_list_remove_at(ptr, i64) — remove element at index
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_remove_at", ft, None);
        }
        // i32 ky_list_remove_value(ptr, i64) — remove first occurrence by value
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_list_remove_value", ft, None);
        }
        // ptr kl_list_map(ptr, ptr) — map with fn pointer
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_list_map", ft, None);
        }
        // ptr kl_list_filter(ptr, ptr) — filter with fn pointer
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_list_filter", ft, None);
        }
        // i64 kl_list_fold(ptr, i64, ptr) — fold with init + fn pointer
        {
            let params = [ptr_ty.into(), i64_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_fold", ft, None);
        }
        // i64 kl_list_reduce(ptr, ptr) — reduce with fn pointer
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_reduce", ft, None);
        }
        // i64 kl_iter_new(i64) — create iterator from list (i64 cast pointer)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_new", ft, None);
        }
        // i64 kl_iter_next(i64) — get next element
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_next", ft, None);
        }
        // i64 kl_iter_map(i64, i64) — lazy map
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_map", ft, None);
        }
        // i64 kl_iter_filter(i64, i64) — lazy filter
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_filter", ft, None);
        }
        // ptr kl_iter_collect(i64) — collect iterator to list
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_collect", ft, None);
        }
        // ptr kl_str_builder_new(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_new", ft, None);
        }
        // void kl_str_builder_append(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_append", ft, None);
        }
        // ptr kl_str_builder_to_str(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_to_str", ft, None);
        }
        // void kl_str_builder_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_free", ft, None);
        }
        // ptr kl_dict_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_dict_new", ft, None);
        }
        // void kl_dict_set(ptr, ptr, i64)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_dict_set", ft, None);
        }
        // i64 kl_dict_get(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_dict_get", ft, &[("memory", "read")]);
        }
        // i64 kl_dict_len(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_dict_len", ft, &[("memory", "read")]);
        }
        // void kl_dict_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_dict_free", ft, None);
        }
        // ptr kl_substr(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_substr", ft, None);
        }
        // i32 kl_is_digit(i8), kl_is_alpha(i8), etc. — all readnone (pure)
        {
            let i8_ty = self.context.i8_type();
            let params = [i8_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            for name in &["ky_is_digit", "ky_is_alpha", "ky_is_alnum",
                          "ky_is_whitespace", "ky_is_upper", "ky_is_lower",
                          "ky_ord"] {
                self.add_runtime_extern(name, ft, &[("memory", "none")]);
            }
        }
        // i32 kl_eq_str(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_eq_str", ft, &[("memory", "read")]);
        }
        // ptr kl_init_args(i32, ptr)  — convert C argv to Kyle list
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_init_args", ft, None);
        }
        // i64 kl_spawn_task(ptr, i64)  — spawn async task running extern "C" fn(i64) -> i64
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_spawn_task", ft, None);
        }
        // i64 kl_await_task(i64)  — await task completion, return result
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_await_task", ft, None);
        }
        // void kl_yield()  — cooperative yield hint
        {
            let ft = void_ty.fn_type(&[], false);
            self.module.add_function("ky_yield", ft, None);
        }
        // i64 kl_spawn_thread(ptr, i64)  — spawn dedicated OS thread
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_spawn_thread", ft, None);
        }
        // i64 kl_join_thread(i64)  — join thread, return result
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_join_thread", ft, None);
        }
        // i64 kl_parallel_for(ptr, i64, i64)  — parallel for loop
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_parallel_for", ft, None);
        }
        // i64 kl_channel_new(i64)  — create channel with capacity
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_new", ft, None);
        }
        // i64 kl_channel_send(i64, i64)  — send value to channel
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_send", ft, None);
        }
        // i64 kl_channel_recv(i64)  — receive from channel
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_recv", ft, None);
        }
        // void kl_channel_close(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_close", ft, None);
        }
        // i64 kl_channel_len(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_len", ft, None);
        }
        // void kl_channel_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_free", ft, None);
        }
        // i32 kl_dict_contains(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_dict_contains", ft, &[("memory", "read")]);
        }
        // i64 kl_dict_remove(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = self.context.i64_type().fn_type(&params, false);
            self.module.add_function("ky_dict_remove", ft, None);
        }
        // ptr kl_set_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_set_new", ft, None);
        }
        // void kl_set_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_set_free", ft, None);
        }
        // void kl_set_add(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_set_add", ft, None);
        }
        // i32 kl_set_contains(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_set_contains", ft, &[("memory", "read")]);
        }
        // i32 kl_set_remove(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_set_remove", ft, None);
        }
        // i64 kl_set_len(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_set_len", ft, &[("memory", "read")]);
        }
        // ptr kl_json_parse(ptr) — parse JSON string into dict
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_json_parse", ft, None);
        }
        // ptr kl_json_stringify(ptr) — serialize dict to JSON string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_json_stringify", ft, None);
        }
        // ptr kl_json_stringify_str(ptr) — serialize {str:str} dict to JSON
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_json_stringify_str", ft, None);
        }
        // ptr kl_clone_str(ptr) — deep-copy a heap-allocated string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_clone_str", ft, None);
        }
        // ptr kl_clone_list(ptr) — shallow-copy a list
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_clone_list", ft, None);
        }
        // ptr kl_clone_dict(ptr) — shallow-copy a dict
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_clone_dict", ft, None);
        }
        // ptr kl_struct_to_json(ptr, ptr) — serialize struct to JSON
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_struct_to_json", ft, None);
        }
        // i32 kl_json_to_struct(ptr, ptr, ptr) — deserialize JSON to struct
        {
            let params = [ptr_ty.into(), ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_json_to_struct", ft, None);
        }
        // void kl_assert(i32)
        {
            let params = [i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_assert", ft, None);
        }
        // void kl_assert_eq(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_assert_eq", ft, None);
        }
        // void kl_assert_ne(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_assert_ne", ft, None);
        }
        // Declare LLVM lifetime.start/end intrinsics for stack slot reuse.
        {
            let ptr_ty = self.context.ptr_type(Default::default());
            let params = [i64_ty.into(), ptr_ty.into()];
            let lifetime_ft = void_ty.fn_type(&params, false);
            // LLVM lifetime intrinsics need mangled names for pointer types (p0 = addrspace 0)
            self.module.add_function("llvm.lifetime.start.p0", lifetime_ft, None);
            self.module.add_function("llvm.lifetime.end.p0", lifetime_ft, None);
        }

        // === Prelude types: bytes ===
        // ptr ky_bytes_new(i32)
        {
            let params = [i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_new", ft, None);
        }
        // i32 ky_bytes_get(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_get", ft, None);
        }
        // void ky_bytes_set(ptr, i32, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_set", ft, None);
        }
        // ptr ky_bytes_to_hex(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_to_hex", ft, None);
        }
        // void ky_bytes_free(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_free", ft, None);
        }
        // ptr ky_bytes_from_hex(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_from_hex", ft, None);
        }
        // ptr ky_bytes_to_base64(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_to_base64", ft, None);
        }

        // === Prelude types: decimal ===
        // i64 ky_decimal_from_str(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_from_str", ft, None);
        }
        // ptr ky_decimal_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_to_str", ft, None);
        }
        // i64 ky_decimal_round(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_round", ft, None);
        }
        // i64 ky_decimal_truncate(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_truncate", ft, None);
        }

        // === Prelude types: regex ===
        // ptr ky_regex_new(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_new", ft, None);
        }
        // i32 ky_regex_is_match(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_is_match", ft, None);
        }
        // ptr ky_regex_find(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_find", ft, None);
        }
        // ptr ky_regex_replace(ptr, ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_replace", ft, None);
        }
        // void ky_regex_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_free", ft, None);
        }

        // === Prelude types: uuid ===
        // ptr ky_uuid_v4()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_uuid_v4", ft, None);
        }
        // ptr ky_uuid_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_uuid_parse", ft, None);
        }

        // === Prelude types: url ===
        // ptr ky_url_scheme(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_scheme", ft, None);
        }
        // ptr ky_url_host(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_host", ft, None);
        }
        // i32 ky_url_port(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_url_port", ft, None);
        }
        // ptr ky_url_path(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_path", ft, None);
        }
        // ptr ky_url_query(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_query", ft, None);
        }

        // === Crypto ===
        // ptr ky_sha256(ptr, i32, ptr)
        {
            let params = [ptr_ty.into(), i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_sha256", ft, None);
        }
        // i32 ky_random_bytes(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_random_bytes", ft, None);
        }

        // === Prelude types: datetime ===
        // i64 ky_datetime_now()
        {
            let ft = i64_ty.fn_type(&[], false);
            self.module.add_function("ky_datetime_now", ft, None);
        }
        // i64 ky_datetime_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_parse", ft, None);
        }
        // ptr ky_datetime_format(i64, ptr)
        {
            let params = [i64_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_format", ft, None);
        }
        // i32 ky_datetime_year(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_year", ft, None);
        }
        // i32 ky_datetime_month(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_month", ft, None);
        }
        // i32 ky_datetime_day(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_day", ft, None);
        }
        // i32 ky_datetime_hour(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_hour", ft, None);
        }
        // i32 ky_datetime_minute(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_minute", ft, None);
        }
        // i32 ky_datetime_second(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_second", ft, None);
        }
        // i64 ky_datetime_add_days(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_add_days", ft, None);
        }
        // i64 ky_datetime_add_hours(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_add_hours", ft, None);
        }
        // i64 ky_datetime_diff(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_diff", ft, None);
        }
        // i64 ky_datetime_from_ymdhms(i32, i32, i32, i32, i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into(), i32_ty.into(), i32_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_from_ymdhms", ft, None);
        }

        // === Prelude types: date ===
        // i32 ky_date_today()
        {
            let ft = i32_ty.fn_type(&[], false);
            self.module.add_function("ky_date_today", ft, None);
        }
        // i32 ky_date_from_ymd(i32, i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_from_ymd", ft, None);
        }
        // i32 ky_date_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_parse", ft, None);
        }
        // i32 ky_date_year(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_year", ft, None);
        }
        // i32 ky_date_month(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_month", ft, None);
        }
        // i32 ky_date_day(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_day", ft, None);
        }
        // i32 ky_date_weekday(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_weekday", ft, None);
        }
        // i32 ky_date_add_days(i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_add_days", ft, None);
        }
        // ptr ky_date_format(i32, ptr)
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_date_format", ft, None);
        }

        // === Prelude types: time ===
        // i32 ky_time_now()
        {
            let ft = i32_ty.fn_type(&[], false);
            self.module.add_function("ky_time_now", ft, None);
        }
        // i32 ky_time_from_hms(i32, i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_from_hms", ft, None);
        }
        // i32 ky_time_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_parse", ft, None);
        }
        // i32 ky_time_hour(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_hour", ft, None);
        }
        // i32 ky_time_minute(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_minute", ft, None);
        }
        // i32 ky_time_second(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_second", ft, None);
        }

        // === FS / Path operations ===
        // i32 ky_fs_exists(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_exists", ft, None);
        }
        // i32 ky_fs_is_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_is_dir", ft, None);
        }
        // i32 ky_fs_is_file(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_is_file", ft, None);
        }
        // i64 ky_fs_size(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_size", ft, None);
        }
        // i32 ky_fs_copy(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_copy", ft, None);
        }
        // i32 ky_fs_remove(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_remove", ft, None);
        }
        // i32 ky_fs_create_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_create_dir", ft, None);
        }
        // i32 ky_fs_remove_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_remove_dir", ft, None);
        }
        // i32 ky_fs_rename(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_rename", ft, None);
        }
        // ptr ky_fs_read_to_string(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_read_to_string", ft, None);
        }
        // i32 ky_fs_write_string(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_write_string", ft, None);
        }
        // i64 ky_fs_list_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_list_dir", ft, None);
        }

        // === Duration ===
        // i64 ky_duration_from_secs(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_secs", ft, None);
        }
        // i64 ky_duration_from_millis(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_millis", ft, None);
        }
        // i64 ky_duration_from_hours(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_hours", ft, None);
        }
        // i64 ky_duration_from_days(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_days", ft, None);
        }
        // ptr ky_duration_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_to_str", ft, None);
        }
        // void ky_duration_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_free", ft, None);
        }

        // === Path ===
        // i64 ky_path_new(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_path_new", ft, None);
        }
        // ptr ky_path_dirname(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_dirname", ft, None);
        }
        // ptr ky_path_basename(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_basename", ft, None);
        }
        // ptr ky_path_extension(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_extension", ft, None);
        }
        // i64 ky_path_join(i64, ptr)
        {
            let params = [i64_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_path_join", ft, None);
        }
        // ptr ky_path_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_to_str", ft, None);
        }
        // void ky_path_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_path_free", ft, None);
        }

        // === BigInt ===
        // i64 ky_big_int_from_str(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_from_str", ft, None);
        }
        // i64 ky_big_int_from_i64(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_from_i64", ft, None);
        }
        // i64 ky_big_int_add(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_add", ft, None);
        }
        // i64 ky_big_int_sub(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_sub", ft, None);
        }
        // i64 ky_big_int_mul(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_mul", ft, None);
        }
        // ptr ky_big_int_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_to_str", ft, None);
        }
        // void ky_big_int_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_free", ft, None);
        }

        // === RC / ARC (reference counting) ===
        // void ky_retain(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_retain", ft, None);
        }
        // void ky_release(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_release", ft, None);
        }

        // === Mutex ===
        // i64 ky_mutex_new(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_new", ft, None);
        }
        // i64 ky_mutex_lock(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_lock", ft, None);
        }
        // void ky_mutex_store(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_store", ft, None);
        }
        // void ky_mutex_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_free", ft, None);
        }

        // === AtomicI64 ===
        // i64 ky_atomic_i64_new(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_new", ft, None);
        }
        // i64 ky_atomic_i64_load(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_load", ft, None);
        }
        // void ky_atomic_i64_store(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_store", ft, None);
        }
        // i64 ky_atomic_i64_add(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_add", ft, None);
        }
        // void ky_atomic_i64_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_free", ft, None);
        }

        // === AtomicBool ===
        // i64 ky_atomic_bool_new(i32)
        {
            let params = [i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_new", ft, None);
        }
        // i32 ky_atomic_bool_load(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_load", ft, None);
        }
        // void ky_atomic_bool_store(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_store", ft, None);
        }
        // void ky_atomic_bool_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_free", ft, None);
        }
    }

    /// Emit inline LLVM IR for list operations instead of calling FFI functions.
    /// KlList layout (24 bytes): [ptr: data, i64: len, i64: cap] at offsets 0, 8, 16.
    fn emit_inline_list_op(
        &self,
        name: &str,
        dest: &Option<usize>,
        args: &[MirValue],
        last_value_map: &mut HashMap<usize, BasicValueEnum<'ctx>>,
    ) -> Result<(), String> {
        let i64_ty = self.context.i64_type();
        let i8_ty = self.context.i8_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        let list_arg = self.value_to_llvm(&args[0], last_value_map)?;
        let list_i64 = match list_arg {
            BasicValueEnum::IntValue(iv) => iv,
            _ => return Err("inline list: expected i64 list pointer".to_string()),
        };
        let list_ptr = self.builder.build_int_to_ptr(list_i64, ptr_ty, "_lptr")
            .map_err(|e| format!("inline list inttoptr: {}", e))?;

        match name {
            "ky_list_len" => {
                let off8 = i64_ty.const_int(8, false);
                let len_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[off8], "_lenp")
                        .map_err(|e| format!("list_len gep: {}", e))?
                };
                let len_val = self.builder.build_load(i64_ty, len_ptr, "_len")
                    .map_err(|e| format!("list_len load: {}", e))?;
                self.tbaa_load(len_val, "list_len");
                if let Some(d) = dest {
                    if let Some(alloca_ptr) = self.alloca_map.get(*d).and_then(|p| *p) {
                        let sv = self.builder.build_store(alloca_ptr, len_val)
                            .map_err(|e| format!("list_len store: {}", e))?;
                        self.tbaa_store(sv, "int");
                    }
                    last_value_map.insert(*d, len_val);
                }
            }
            "ky_list_get" if args.len() >= 2 => {
                let idx_arg = self.value_to_llvm(&args[1], last_value_map)?;
                let idx_i64 = match idx_arg {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_get: expected i64 index".to_string()),
                };
                // Load data pointer (TBAA list_data — doesn't alias with array elements)
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("list_get gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("list_get load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                // GEP into data array
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("list_get gep elem: {}", e))?
                };
                let val = self.builder.build_load(i64_ty, elem_ptr, "_val")
                    .map_err(|e| format!("list_get load val: {}", e))?;
                self.tbaa_load(val, "int");
                if let Some(d) = dest {
                    if let Some(alloca_ptr) = self.alloca_map.get(*d).and_then(|p| *p) {
                        let sv = self.builder.build_store(alloca_ptr, val)
                            .map_err(|e| format!("list_get store: {}", e))?;
                        self.tbaa_store(sv, "int");
                    }
                    last_value_map.insert(*d, val);
                }
            }
            "ky_list_set" if args.len() >= 3 => {
                let idx_arg = self.value_to_llvm(&args[1], last_value_map)?;
                let val_arg = self.value_to_llvm(&args[2], last_value_map)?;
                let idx_i64 = match idx_arg {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 index".to_string()),
                };
                let val_i64 = match val_arg {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 value".to_string()),
                };
                // Load data pointer (TBAA list_data — doesn't alias with element stores)
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("list_set gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("list_set load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("list_set gep elem: {}", e))?
                };
                let sv = self.builder.build_store(elem_ptr, val_i64)
                    .map_err(|e| format!("list_set store: {}", e))?;
                self.tbaa_store(sv, "int");
            }
            _ => return Err(format!("unknown inline list op: {}", name)),
        }
        Ok(())
    }

    /// SSA version of inline list operations. Uses block_vals directly instead of last_value_map.
    /// Reads SsaValueId from block_vals (same as ssa_read! macro).
    fn emit_ssa_inline_list_op(
        &self,
        name: &str,
        block_vals: &HashMap<usize, BasicValueEnum<'ctx>>,
        args: &[SsaValueId],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let i64_ty = self.context.i64_type();
        let i8_ty = self.context.i8_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        let read_val = |id: SsaValueId| -> BasicValueEnum<'ctx> {
            block_vals.get(&id).copied().unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum())
        };
        let list_val = read_val(args[0]);
        let list_ptr = match &list_val {
            BasicValueEnum::IntValue(iv) => self.builder.build_int_to_ptr(*iv, ptr_ty, "_lptr")
                .map_err(|e| format!("ssa inline list inttoptr: {}", e))?,
            BasicValueEnum::PointerValue(pv) => *pv,
            _ => return Err("ssa inline list: expected ptr or i64".to_string()),
        };

        match name {
            "ky_list_len" => {
                let off8 = i64_ty.const_int(8, false);
                let len_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[off8], "_lenp")
                        .map_err(|e| format!("ssa list_len gep: {}", e))?
                };
                let len_val = self.builder.build_load(i64_ty, len_ptr, "_len")
                    .map_err(|e| format!("ssa list_len load: {}", e))?;
                self.tbaa_load(len_val, "list_len");
                Ok(Some(len_val))
            }
            "ky_list_get" if args.len() >= 2 => {
                let idx_val = read_val(args[1]);
                let idx_i64 = match idx_val {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_get: expected i64 index".to_string()),
                };
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("ssa list_get gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("ssa list_get load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("ssa list_get gep elem: {}", e))?
                };
                let val = self.builder.build_load(i64_ty, elem_ptr, "_val")
                    .map_err(|e| format!("ssa list_get load val: {}", e))?;
                self.tbaa_load(val, "int");
                Ok(Some(val))
            }
            "ky_list_set" if args.len() >= 3 => {
                let idx_val = read_val(args[1]);
                let val_val = read_val(args[2]);
                let idx_i64 = match idx_val {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 index".to_string()),
                };
                let val_i64 = match val_val {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 value".to_string()),
                };
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("ssa list_set gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("ssa list_set load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("ssa list_set gep elem: {}", e))?
                };
                let sv = self.builder.build_store(elem_ptr, val_i64)
                    .map_err(|e| format!("ssa list_set store: {}", e))?;
                self.tbaa_store(sv, "int");
                Ok(None)
            }
            _ => Err(format!("unknown ssa inline list op: {}", name)),
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
            MirType::Char => self.context.i32_type().as_basic_type_enum(),
            MirType::Str | MirType::Box(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::Void => self.context.i32_type().as_basic_type_enum(),
            MirType::Ptr(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::Array(inner, size) => {
                let base = self.llvm_type(inner);
                base.array_type(*size as u32).as_basic_type_enum()
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
            MirType::Slice(inner) => {
                let inner_str = format!("{}", inner);
                let name = format!("__slice_{}", inner_str);
                let struct_ty = self.module.get_struct_type(&name)
                    .unwrap_or_else(|| {
                        let ty = self.context.opaque_struct_type(&name);
                        let ptr_ty = self.llvm_type(inner).ptr_type(Default::default());
                        let len_ty = self.context.i64_type();
                        ty.set_body(&[ptr_ty.into(), len_ty.into()], false);
                        ty
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
        // Parameter attributes: noundef on all, noalias on pointer types
        let noundef_kind = Attribute::get_named_enum_kind_id("noundef");
        let noalias_kind = Attribute::get_named_enum_kind_id("noalias");
        for (i, ptype) in func.params.iter().enumerate() {
            let idx = i as u32;
            if noundef_kind > 0 {
                let attr = self.context.create_enum_attribute(noundef_kind, 0);
                fn_value.add_attribute(AttributeLoc::Param(idx), attr);
            }
            if noalias_kind > 0 {
                if matches!(ptype, MirType::Struct(_, _) | MirType::Str | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) | MirType::Ptr(_) | MirType::Box(_)) {
                    let attr = self.context.create_enum_attribute(noalias_kind, 0);
                    fn_value.add_attribute(AttributeLoc::Param(idx), attr);
                }
            }
        }
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

        // Phase 17.5: identify single-block locals to skip allocas for
        // A local is eligble if:
        //   1. Simple type (i32/i64/f32/f64/bool — not struct/string/list)
        //   2. Only used in FieldPtr/Memcpy/PtrOffset → needs alloca (always keep)
        //   3. Defined and used within a single basic block (no cross-block flow)
        let simple_types: [MirType; 5] = [MirType::I32, MirType::I64, MirType::F32, MirType::F64, MirType::Bool];
        let mut escaping: HashSet<usize> = HashSet::new();
        let mut def_blocks: HashMap<usize, usize> = HashMap::new();
        let mut use_blocks: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (bi, bb) in func.basic_blocks.iter().enumerate() {
            for inst in &bb.insts {
                match inst {
                    MirInst::Alloca { dest, type_, .. } => {
                        if !simple_types.contains(type_) {
                            escaping.insert(*dest);
                        }
                    }
                    MirInst::FieldPtr { ptr, .. } => { escaping.insert(*ptr); }
                    MirInst::ArrayElemPtr { ptr, .. } => { escaping.insert(*ptr); }
                    MirInst::PtrOffset { dest, ptr, .. } => { 
                        escaping.insert(*ptr);
                        escaping.insert(*dest);
                    }
                    MirInst::PtrStore { ptr, .. } => { escaping.insert(*ptr); }
                    MirInst::Memcpy { dest_ptr_local, .. } => { escaping.insert(*dest_ptr_local); }
                    MirInst::SliceMake { dest, .. } => { escaping.insert(*dest); }
                    _ => {}
                }
                // Track definitions (any instruction writing to dest)
                let dest_opt = match inst {
                    MirInst::Store { dest, .. } => Some(*dest),
                    MirInst::Load { dest, .. } => Some(*dest),
                    MirInst::BinaryOp { dest, .. } => Some(*dest),
                    MirInst::UnaryOp { dest, .. } => Some(*dest),
                    MirInst::Cast { dest, .. } => Some(*dest),
                    MirInst::Call { dest, .. } => dest.map(|d| d),
                    MirInst::CallIndirect { dest, .. } => dest.map(|d| d),
                    MirInst::PtrOffset { dest, .. } => Some(*dest),
                    MirInst::PtrStore { dest, .. } => Some(*dest),
                    MirInst::FieldPtr { dest, .. } => Some(*dest),
                    MirInst::ArrayElemPtr { dest, .. } => Some(*dest),
                    MirInst::FnAddr { dest, .. } => Some(*dest),
                    MirInst::AddressOf { dest, .. } => Some(*dest),
                    MirInst::AsyncSpawn { dest, .. } => Some(*dest),
                    MirInst::AsyncAwait { dest, .. } => Some(*dest),
                    MirInst::SliceMake { dest, .. } => Some(*dest),
                    _ => None,
                };
                if let Some(d) = dest_opt {
                    def_blocks.entry(d).or_insert(bi);
                }
                // Track uses (MirValue::Local references)
                self.collect_local_uses(inst, &mut |local_id| {
                    use_blocks.entry(local_id).or_default().insert(bi);
                });
            }
            // Track uses in terminator
            match &bb.terminator {
                MirTerminator::Return(val) => {
                    if let MirValue::Local(lid) = val {
                        use_blocks.entry(*lid).or_default().insert(bi);
                    }
                }
                MirTerminator::CondBr { cond, .. } => {
                    if let MirValue::Local(lid) = cond {
                        use_blocks.entry(*lid).or_default().insert(bi);
                    }
                }
                _ => {}
            }
        }
        let mut skip_allocas: HashSet<usize> = HashSet::new();
        for (local, db) in &def_blocks {
            if escaping.contains(local) { continue; }
            let ub = use_blocks.get(local);
            // Single-block: defined and all uses are in the same block
            let is_single_block = ub.map_or(true, |blocks| blocks.len() == 1 && blocks.contains(db));
            // Also skip locals with no uses (defined but never read — dead code)
            let has_no_uses = ub.map_or(true, |blocks| blocks.is_empty());
            if is_single_block || has_no_uses {
                skip_allocas.insert(*local);
            }
        }
        // Params always need allocas (value flows from entry to other blocks)
        for (bi, bb) in func.basic_blocks.iter().enumerate() {
            for inst in &bb.insts {
                if let MirInst::Store { dest, value: MirValue::Param(_) } = inst {
                    skip_allocas.remove(dest);
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
                    if skip_allocas.contains(dest) {
                        // Ensure the vec is sized correctly even without an alloca
                        while self.alloca_map.len() <= *dest {
                            self.alloca_map.push(None);
                        }
                        continue;
                    }
                    while self.alloca_map.len() <= *dest {
                        self.alloca_map.push(None);
                    }
                    let ptr = self.builder.build_alloca(*llvm_type, "")
                        .map_err(|e| format!("alloca {}: {}", dest, e))?;
                    if let Ok(iv) = inkwell::values::InstructionValue::try_from(AnyValueEnum::PointerValue(ptr)) {
                        let _ = iv.set_alignment(8);
                    }
                    // // self.emit_lifetime_start(ptr, -1); // DEBUG: disabled for mem2reg test // DEBUG: disabled for mem2reg test
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
                        if let MirInst::ArrayElemPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest {
                                self.field_ptr_allocas.push(None);
                            }
                            if self.field_ptr_allocas[*dest].is_none() {
                                let alloca = self.builder.build_alloca(ptr_ty, "_aelem")
                                    .map_err(|e| format!("aelem alloca {}: {}", dest, e))?;
                                self.field_ptr_allocas[*dest] = Some(alloca);
                            }
                        }
                        if let MirInst::PtrOffset { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest {
                                self.field_ptr_allocas.push(None);
                            }
                            if self.field_ptr_allocas[*dest].is_none() {
                                let alloca = self.builder.build_alloca(ptr_ty, "_pgep")
                                    .map_err(|e| format!("pgep alloca {}: {}", dest, e))?;
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
                                    let siv = self.builder.build_store(gep.into_pointer_value(), val)
                                        .map_err(|e| format!("fptr store: {}", e))?;
                                    if let Some(tbaa_node) = self.alloca_types.get(dest).and_then(|t| self.tbaa_for_llvm_type(t)) {
                                        self.add_tbaa(siv, tbaa_node);
                                    }
                                }
                            } else if let Some(ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                let siv = self.builder.build_store(ptr, val)
                                    .map_err(|e| format!("store: {}", e))?;
                                if let Some(tbaa_node) = self.alloca_types.get(dest).and_then(|t| self.tbaa_for_llvm_type(t)) {
                                    self.add_tbaa(siv, tbaa_node);
                                }
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
                                    // Add TBAA metadata to load instruction
                                    if let Some(src_ty) = self.alloca_types.get(src) {
                                        if let Some(tbaa_node) = self.tbaa_for_llvm_type(src_ty) {
                                            if let Ok(liv) = inkwell::values::InstructionValue::try_from(AnyValueEnum::from(loaded.clone())) {
                                                self.add_tbaa(liv, tbaa_node);
                                            }
                                        }
                                    }
                                    // Store to dest alloca for cross-block reads
                                    if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(dest_ptr, loaded)
                                            .map_err(|e| format!("load-store: {}", e))?;
                                    }
                                    last_value_map.insert(*dest, loaded);
                                }
                            } else {
                                // Promoted temp (no alloca): read from last_value_map
                                if let Some(&val) = last_value_map.get(src) {
                                    last_value_map.insert(*dest, val);
                                    if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(dest_ptr, val)
                                            .map_err(|e| format!("promoted-load-store: {}", e))?;
                                    }
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
                                    MirBinaryOp::Add => self.int_nsw_nuw_add(li, ri).map_err(|e| format!("iadd: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Sub => self.int_nsw_nuw_sub(li, ri).map_err(|e| format!("isub: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Mul => self.int_nsw_nuw_mul(li, ri).map_err(|e| format!("imul: {}", e))?
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
                                    MirBinaryOp::Eq => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::EQ, li, ri, "")
                                            .map_err(|e| format!("eq: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("eqz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Neq => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::NE, li, ri, "")
                                            .map_err(|e| format!("neq: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("nqz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Lt => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SLT, li, ri, "")
                                            .map_err(|e| format!("lt: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("ltz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Gt => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SGT, li, ri, "")
                                            .map_err(|e| format!("gt: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("gtz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Le => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SLE, li, ri, "")
                                            .map_err(|e| format!("le: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("lez: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Ge => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SGE, li, ri, "")
                                            .map_err(|e| format!("ge: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("gez: {}", e))?
                                            .as_basic_value_enum()
                                    }
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
                            // Inline list operations (avoid FFI overhead for tight loops)
                            match name.as_str() {
                                "ky_list_get" | "ky_list_set" | "ky_list_len" => {
                                    self.emit_inline_list_op(name, dest, args, &mut last_value_map)?;
                                    continue;
                                }
                                _ => {}
                            }
                            // Don't apply runtime name mapping for user-defined functions
                            let runtime_name = if self.fn_value_map.contains_key(name) {
                                name.clone()
                            } else {
                                match name.as_str() {
                                "print" => "ky_print",
                                "println" => "ky_println",
                                "contains" => "ky_str_contains",
                                "to_upper" => "ky_str_to_upper",
                                "to_lower" => "ky_str_to_lower",
                                "trim" => "ky_str_trim",
                                "replace" => "ky_str_replace",
                                "input" => "ky_input",
                                "open" => "ky_open",
                                "read_str" => "ky_read_str",
                                "write_str" => "ky_write_str",
                                "close" => "ky_close",
                                "sleep" => "ky_sleep",
                                "now" => "ky_now",
                                "char_at" => "ky_char_at",
                                "is_digit" => "ky_is_digit",
                                "is_alpha" => "ky_is_alpha",
                                "is_alnum" => "ky_is_alnum",
                                "is_whitespace" => "ky_is_whitespace",
                                "is_upper" => "ky_is_upper",
                                "is_lower" => "ky_is_lower",
                                "ord" => "ky_ord",
                                "substr" => "ky_substr",
                                "list_new" => "ky_list_new",
                                "list_push" => "ky_list_push",
                                "list_get" => "ky_list_get",
                                "list_set" => "ky_list_set",
                                "list_len" => "ky_list_len",
                                "list_pop" => "ky_list_pop", "reserve" => "ky_list_reserve",
                                "ky_str_builder_new" => "ky_str_builder_new",
                                "ky_str_builder_append" => "ky_str_builder_append",
                                "ky_str_builder_to_str" => "ky_str_builder_to_str",
                                "ky_str_builder_free" => "ky_str_builder_free",
                                _ => name.as_str(),
                                }.to_string()
                            };
                             if self.module.get_function(&runtime_name).is_none() {
                                // Auto-declare extern function on first use with inferred types
                                let ret_type: BasicTypeEnum = if let Some(d) = dest {
                                    let raw = self.alloca_types.get(&d).copied()
                                        .unwrap_or(self.context.i64_type().as_basic_type_enum());
                                    // Struct return types are actually i32 (runtime returns status code)
                                    if matches!(raw, BasicTypeEnum::StructType(_)) {
                                        self.context.i32_type().as_basic_type_enum()
                                    } else {
                                        raw
                                    }
                                } else {
                                    self.context.i64_type().as_basic_type_enum()
                                };
                                let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = args.iter()
                                    .map(|a| {
                                        let t = match a {
                                            MirValue::Local(id) => {
                                                let raw = self.alloca_types.get(id).copied()
                                                    .unwrap_or(self.context.i32_type().as_basic_type_enum());
                                                // Struct allocas are passed by pointer, so use ptr type
                                                if matches!(raw, BasicTypeEnum::StructType(_)) {
                                                    self.context.ptr_type(Default::default()).as_basic_type_enum()
                                                } else {
                                                    raw
                                                }
                                            }
                                            MirValue::Constant(c) => {
                                                match c {
                                                    MirConstant::String(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
                                                    MirConstant::I32(_) => self.context.i32_type().as_basic_type_enum(),
                                                    MirConstant::I64(_) => self.context.i64_type().as_basic_type_enum(),
                                                    MirConstant::Bool(_) => self.context.i8_type().as_basic_type_enum(),
                                                    _ => self.context.i32_type().as_basic_type_enum(),
                                                }
                                            }
                                            _ => self.context.i32_type().as_basic_type_enum(),
                                        };
                                        t.into()
                                    }).collect();
                                let fn_type = ret_type.fn_type(&param_types, false);
                                self.module.add_function(&runtime_name, fn_type, None);
                            }
                            if let Some(callee_fn) = self.module.get_function(&runtime_name) {
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
                        MirInst::PtrOffset { dest, ptr, index, elem_type } => {
                            let base_val = self.load_value(*ptr, &last_value_map)?;
                            let idx = self.value_to_llvm(index, &last_value_map)?;
                            let int_idx = idx.into_int_value();
                            let pointee_type = self.context.i8_type().as_basic_type_enum();
                            let gep = unsafe {
                                let ptr_val = if let BasicValueEnum::IntValue(iv) = base_val {
                                    self.builder.build_int_to_ptr(iv, self.context.ptr_type(Default::default()), "_inttoptr")
                                        .map_err(|e| format!("ptroff inttoptr: {}", e))?
                                } else {
                                    base_val.into_pointer_value()
                                };
                                self.builder.build_in_bounds_gep(pointee_type, ptr_val, &[int_idx], "")
                                    .map_err(|e| format!("ptroff gep: {}", e))?
                            };
                            // Store GEP result in last_value_map, dest alloca, AND field_ptr_allocas
                            last_value_map.insert(*dest, gep.as_basic_value_enum());
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(dest_ptr, gep.as_basic_value_enum())
                                    .map_err(|e| format!("ptroff-store: {}", e))?;
                            }
                            if let Some(fpa) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(fpa, gep.as_basic_value_enum())
                                    .map_err(|e| format!("ptroff-fpa: {}", e))?;
                            }
                            let elem_llvm = self.llvm_type(elem_type);
                            self.field_ptr_types.insert(*dest, elem_llvm);
                        }
                        MirInst::PtrStore { dest, ptr, index, value } => {
                            let base_val = self.load_value(*ptr, &last_value_map)?;
                            let idx = self.value_to_llvm(index, &last_value_map)?;
                            let int_idx = idx.into_int_value();
                            let pointee_type = self.context.i8_type().as_basic_type_enum();
                            let gep = unsafe {
                                let ptr_val = if let BasicValueEnum::IntValue(iv) = base_val {
                                    self.builder.build_int_to_ptr(iv, self.context.ptr_type(Default::default()), "_psint")
                                        .map_err(|e| format!("ps inttoptr: {}", e))?
                                } else {
                                    base_val.into_pointer_value()
                                };
                                self.builder.build_in_bounds_gep(pointee_type, ptr_val, &[int_idx], "")
                                    .map_err(|e| format!("ps gep: {}", e))?
                            };
                            let val = self.value_to_llvm(value, &last_value_map)?;
                            let siv = self.builder.build_store(gep, val)
                                .map_err(|e| format!("ps store: {}", e))?;
                            last_value_map.insert(*dest, val);
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
                                let zero = self.context.i32_type().const_zero();
                                let idx_val = self.context.i32_type().const_int(*field_index as u64, false);
                                // Ref param: alloca stores pointer-to-struct, load it first
                                if let Some(&orig_struct_type) = self.ref_param_struct_types.get(ptr) {
                                    let struct_ptr = self.builder.build_load(
                                        self.context.ptr_type(Default::default()),
                                        base_ptr, "_ref_load"
                                    ).map_err(|e| format!("ref_field_ptr load: {}", e))?;
                                    let gep = unsafe {
                                        self.builder.build_in_bounds_gep(orig_struct_type, struct_ptr.into_pointer_value(), &[zero, idx_val], "")
                                            .map_err(|e| format!("ref_field_ptr: {}", e))?
                                    };
                                    if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(alloca, gep)
                                            .map_err(|e| format!("ref_fgep store: {}", e))?;
                                    }
                                } else if let Some(ptr_type) = self.alloca_types.get(ptr) {
                                    if ptr_type.is_pointer_type() {
                                        // Ptr(Struct) closure param: alloca stores a pointer to the struct.
                                        // Load the pointer, then GEP on the pointed-to struct.
                                        if let MirType::Struct(sname, fields) = struct_type.as_ref() {
                                            if !fields.is_empty() {
                                                let struct_llvm = self.llvm_type(&MirType::Struct(sname.clone(), fields.clone()));
                                                let struct_ptr = self.builder.build_load(
                                                    self.context.ptr_type(Default::default()),
                                                    base_ptr, "_ptr_param_load"
                                                ).map_err(|e| format!("ptr_param_field_ptr load: {}", e))?;
                                                let gep = unsafe {
                                                    self.builder.build_in_bounds_gep(struct_llvm, struct_ptr.into_pointer_value(), &[zero, idx_val], "")
                                                        .map_err(|e| format!("ptr_param_field_ptr: {}", e))?
                                                };
                                                if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                                    self.builder.build_store(alloca, gep)
                                                        .map_err(|e| format!("ptr_param_fgep store: {}", e))?;
                                                }
                                            }
                                        }
                                    } else {
                                        let gep = unsafe {
                                            self.builder.build_in_bounds_gep(*ptr_type, base_ptr, &[zero, idx_val], "")
                                                .map_err(|e| format!("field_ptr: {}", e))?
                                        };
                                        if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                            self.builder.build_store(alloca, gep)
                                                .map_err(|e| format!("fgep store: {}", e))?;
                                        }
                                    }
                                }
                            }
                        }
                        MirInst::ArrayElemPtr { dest, ptr, index, arr_type, elem_type } => {
                            // Check if ptr is a previously-computed GEP pointer (chained GEP)
                            let fpa_found = *ptr < self.field_ptr_allocas.len() && self.field_ptr_allocas[*ptr].is_some();
                            let alloca_found = self.alloca_map.get(*ptr).and_then(|p| *p);
                            let base_ptr = if fpa_found {
                                let p = self.field_ptr_allocas[*ptr].unwrap();
                                let loaded = self.builder.build_load(
                                    self.context.ptr_type(Default::default()), p, "_aebase"
                                ).map_err(|e| format!("aebase load: {}", e))?;
                                loaded.as_basic_value_enum()
                            } else if let Some(p) = alloca_found {
                                p.as_basic_value_enum()
                            } else { continue; };
                            if let BasicValueEnum::PointerValue(base_ptr) = base_ptr {
                                let arr_llvm = self.llvm_type(arr_type);
                                let zero = self.context.i32_type().const_zero();
                                let idx_val = self.value_to_llvm(index, &last_value_map)
                                    .unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum());
                                let idx_i32 = if let BasicValueEnum::IntValue(iv) = idx_val {
                                    if iv.get_type().get_bit_width() != 32 {
                                        self.builder.build_int_truncate(iv, self.context.i32_type(), "_aeptrunc")
                                            .map_err(|e| format!("aeptrunc: {}", e))?
                                    } else { iv }
                                } else {
                                    self.context.i32_type().const_zero()
                                };
                                let gep = unsafe {
                                    self.builder.build_in_bounds_gep(arr_llvm, base_ptr, &[zero, idx_i32], "_aelem")
                                        .map_err(|e| format!("aelem: {}", e))?
                                };
                                if let Some(fpa) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(fpa, gep)
                                        .map_err(|e| format!("aelem store gep: {}", e))?;
                                }
                                let elem_llvm = self.llvm_type(elem_type);
                                self.field_ptr_types.insert(*dest, elem_llvm);
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
                                    let result = if src_width < dst_width && src_width > 1 {
                                        self.builder.build_int_s_extend(*int_val, *t, "")
                                            .map_err(|e| format!("sext: {}", e))?
                                    } else if src_width == 1 && dst_width > 1 {
                                        self.builder.build_int_z_extend(*int_val, *t, "")
                                            .map_err(|e| format!("zext: {}", e))?
                                    } else if dst_width == 1 && src_width > 1 {
                                        // Int → Bool: compare with zero, not truncate
                                        let zero = self.context.i32_type().const_zero();
                                        let widened = if src_width < 32 {
                                            self.builder.build_int_z_extend(*int_val, self.context.i32_type(), "_widen")
                                                .map_err(|e| format!("widen: {}", e))?
                                        } else { *int_val };
                                        self.builder.build_int_compare(inkwell::IntPredicate::NE, widened, zero, "_tobool")
                                            .map_err(|e| format!("tobool: {}", e))?
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
                                (BasicValueEnum::FloatValue(float_val), BasicTypeEnum::FloatType(t)) => {
                                    // Float → Float: fpext/fptrunc
                                    self.builder.build_float_cast(*float_val, *t, "_ffcast")
                                        .map_err(|e| format!("ffcast: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::FloatValue(float_val), BasicTypeEnum::IntType(i)) => {
                                    // Float → Integer: fptosi
                                    self.builder.build_float_to_signed_int(*float_val, *i, "_fptosi")
                                        .map_err(|e| format!("fptosi: {}", e))?
                                        .as_basic_value_enum()
                                }
                                // Pointer → Pointer: identity (no-op cast)
                                (BasicValueEnum::PointerValue(ptr_val), BasicTypeEnum::PointerType(_)) => {
                                    ptr_val.as_basic_value_enum()
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
                        MirInst::AddressOf { dest, local_id } => {
                            if let Some(ptr) = self.alloca_map.get(*local_id).and_then(|p| *p) {
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, ptr)
                                        .map_err(|e| format!("addr store: {}", e))?;
                                }
                                last_value_map.insert(*dest, ptr.as_basic_value_enum());
                            }
                        }
                        MirInst::CallIndirect { dest, fn_ptr, ret_type, param_types, args } => {
                            let ptr_val = self.load_value(*fn_ptr, &last_value_map)?;
                            let fn_ptr = match ptr_val {
                                BasicValueEnum::IntValue(iv) => {
                                    let ptr_ty = self.context.ptr_type(Default::default());
                                    self.builder.build_int_to_ptr(iv, ptr_ty, "_fnptr")
                                        .map_err(|e| format!("callindirect inttoptr: {}", e))?
                                }
                                _ => ptr_val.into_pointer_value(),
                            };
                            let llvm_ret = self.llvm_type(ret_type);
                            let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types.iter()
                                .map(|p| self.llvm_type(p).into())
                                .collect();
                            let fn_ty = llvm_ret.fn_type(&llvm_params, false);
                            let fn_param_types = fn_ty.get_param_types();
                            let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum> = args.iter()
                                .enumerate()
                                .map(|(i, a)| {
                                    // If the function expects ptr but the MIR arg is a struct alloca,
                                    // pass the alloca pointer instead of loading the struct value.
                                    if i < fn_param_types.len() {
                                        if let inkwell::types::BasicMetadataTypeEnum::PointerType(_) = fn_param_types[i] {
                                            if let MirValue::Local(id) = a {
                                                if let Some(Some(alloca)) = self.alloca_map.get(*id) {
                                                    if let Some(pointee_type) = self.alloca_types.get(id) {
                                                        if matches!(pointee_type, BasicTypeEnum::StructType(_)) {
                                                            return alloca.as_basic_value_enum().into();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let val: BasicValueEnum = self.value_to_llvm(a, &last_value_map)
                                        .unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum());
                                    // Auto-truncate i64 args to i32 if needed (for closure calls)
                                    if i < fn_param_types.len() {
                                        if let BasicValueEnum::IntValue(iv) = val {
                                            let expected_ty = fn_param_types[i];
                                            let actual_w = iv.get_type().get_bit_width();
                                            if let inkwell::types::BasicMetadataTypeEnum::IntType(eit) = expected_ty {
                                                let expected_w = eit.get_bit_width();
                                                if actual_w > expected_w {
                                                    if let Ok(trunc) = self.builder.build_int_truncate(iv, eit, "") {
                                                        return trunc.as_basic_value_enum().into();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    val.into()
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
                            let spawn_fn = self.module.get_function("ky_spawn_task")
                                .ok_or_else(|| "ky_spawn_task not declared".to_string())?;
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
                        MirInst::SliceMake { dest, ptr, len, elem_type } => {
                            let ptr_val = self.value_to_llvm(ptr, &last_value_map)?;
                            let len_val = self.value_to_llvm(len, &last_value_map)?;
                            let slice_type = self.llvm_type(&MirType::Slice(elem_type.clone()));
                            if let BasicTypeEnum::StructType(st) = slice_type {
                                let undef = st.get_undef();
                                let sv = unsafe {
                                    self.builder.build_insert_value(undef, ptr_val, 0, "_msmi0")
                                        .map_err(|e| format!("msmi0: {}", e))?
                                };
                                let sv = unsafe {
                                    self.builder.build_insert_value(sv, len_val, 1, "_msmi1")
                                        .map_err(|e| format!("msmi1: {}", e))?
                                };
                                let bv = sv.as_basic_value_enum();
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, bv)
                                        .map_err(|e| format!("msmst: {}", e))?;
                                }
                                last_value_map.insert(*dest, bv);
                            }
                        }
                        MirInst::AsyncAwait { dest, handle } => {
                            let handle_val = self.load_value(*handle, &last_value_map)?;
                            let join_fn = self.module.get_function("ky_await_task")
                                .ok_or_else(|| "ky_await_task not declared".to_string())?;
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
        let init_args_fn = self.module.get_function("ky_init_args")
            .ok_or_else(|| "ky_init_args not declared".to_string())?;
        let args_call = self.builder.build_call(init_args_fn, &[argc_meta, argv_meta], "args")
            .map_err(|e| format!("call kl_init_args: {}", e))?;
        let args_list = match args_call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => bv,
            _ => return Err("ky_init_args did not return a basic value".to_string()),
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
            self.to_int_value(v)
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
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("feqe: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Neq => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, lf, rf, "").map_err(|e| format!("fne: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fnee: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Lt => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OLT, lf, rf, "").map_err(|e| format!("flt: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("flte: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Gt => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OGT, lf, rf, "").map_err(|e| format!("fgt: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fgte: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Le => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OLE, lf, rf, "").map_err(|e| format!("fle: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("flee: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Ge => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OGE, lf, rf, "").map_err(|e| format!("fge: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fgee: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                _ => return Err("ssa: unsupported float op".into()),
            })
        } else {
            let li = to_int(l); let ri = to_int(r);
            // Auto-widen mismatched integer widths
            let lw = li.get_type().get_bit_width();
            let rw = ri.get_type().get_bit_width();
            let widen = |val: inkwell::values::IntValue<'ctx>, target_w: u32| -> Option<inkwell::values::IntValue<'ctx>> {
                let ty = match target_w { 8 => self.context.i8_type(), 16 => self.context.i16_type(), 32 => self.context.i32_type(), 64 => self.context.i64_type(), _ => return None };
                self.builder.build_int_s_extend(val, ty, "_ssaw").ok()
            };
            let (li, ri) = if lw < rw {
                (widen(li, rw).unwrap_or(li), ri)
            } else if rw < lw {
                (li, widen(ri, lw).unwrap_or(ri))
            } else { (li, ri) };
            Ok(match op {
                MirBinaryOp::Add => self.int_nsw_nuw_add(li, ri).map_err(|e| format!("iadd: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Sub => self.int_nsw_nuw_sub(li, ri).map_err(|e| format!("isub: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Mul => self.int_nsw_nuw_mul(li, ri).map_err(|e| format!("imul: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Div => self.builder.build_int_signed_div(li, ri, "").map_err(|e| format!("idiv: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Rem => self.builder.build_int_signed_rem(li, ri, "").map_err(|e| format!("irem: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::And => self.builder.build_and(li, ri, "").map_err(|e| format!("iand: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Or => self.builder.build_or(li, ri, "").map_err(|e| format!("ior: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Xor => self.builder.build_xor(li, ri, "").map_err(|e| format!("ixor: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Shl => self.builder.build_left_shift(li, ri, "").map_err(|e| format!("ishl: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Shr => self.builder.build_right_shift(li, ri, true, "").map_err(|e| format!("ishr: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Eq => { let c = self.builder.build_int_compare(inkwell::IntPredicate::EQ, li, ri, "").map_err(|e| format!("ieq: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("iez: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Neq => { let c = self.builder.build_int_compare(inkwell::IntPredicate::NE, li, ri, "").map_err(|e| format!("ine: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("inz: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Lt => { let c = self.builder.build_int_compare(inkwell::IntPredicate::SLT, li, ri, "").map_err(|e| format!("ilt: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("ilz: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Gt => { let c = self.builder.build_int_compare(inkwell::IntPredicate::SGT, li, ri, "").map_err(|e| format!("igt: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("igz: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Le => { let c = self.builder.build_int_compare(inkwell::IntPredicate::SLE, li, ri, "").map_err(|e| format!("ile: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("ilz2: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Ge => { let c = self.builder.build_int_compare(inkwell::IntPredicate::SGE, li, ri, "").map_err(|e| format!("ige: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("igz2: {}", e))?.as_basic_value_enum() }
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
            MirConstant::Null => self.context.ptr_type(Default::default()).const_null().as_basic_value_enum(),
        }
    }
}
