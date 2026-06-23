use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::PointerValue;
use inkwell::IntPredicate;
use std::collections::HashMap;

use klc_mir::mir::*;

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    fn_value_map: HashMap<String, inkwell::values::FunctionValue<'ctx>>,
    param_values: HashMap<usize, BasicValueEnum<'ctx>>,
    alloca_map: Vec<Option<PointerValue<'ctx>>>,
    alloca_types: HashMap<usize, BasicTypeEnum<'ctx>>,
    field_ptr_allocas: Vec<Option<PointerValue<'ctx>>>,
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
                if let Some(pointee_type) = self.alloca_types.get(&id) {
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

    /// Declare external runtime functions that generated code can call.
    fn declare_runtime_externs(&mut self) {
        let void_ty = self.context.void_type();
        let i64_ty = self.context.i64_type();
        let i32_ty = self.context.i32_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        // void kl_print_int(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_print_int", ft, None);
        }
        // void kl_println_int(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_println_int", ft, None);
        }
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
        // void kl_release(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_release", ft, None);
        }
        // ptr kl_i64_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_i64_to_str", ft, None);
        }
        // i32 kl_strlen(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("kl_strlen", ft, None);
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
        // i32 kl_str_contains(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("kl_str_contains", ft, None);
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
        // i8 kl_char_at(ptr, i32)
        {
            let i8_ty = self.context.i8_type();
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = i8_ty.fn_type(&params, false);
            self.module.add_function("kl_char_at", ft, None);
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
        // i64 kl_list_get(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("kl_list_get", ft, None);
        }
        // void kl_list_set(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("kl_list_set", ft, None);
        }
        // i64 kl_list_len(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("kl_list_len", ft, None);
        }
        // ptr kl_substr(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_substr", ft, None);
        }
        // i32 kl_is_digit(i8), kl_is_alpha(i8), etc.
        {
            let i8_ty = self.context.i8_type();
            let params = [i8_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            for name in &["kl_is_digit", "kl_is_alpha", "kl_is_alnum",
                          "kl_is_whitespace", "kl_is_upper", "kl_is_lower",
                          "kl_ord"] {
                self.module.add_function(name, ft, None);
            }
        }
        // i32 kl_eq_str(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("kl_eq_str", ft, None);
        }
        // ptr kl_init_args(i32, ptr)  — convert C argv to Kyle list
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("kl_init_args", ft, None);
        }
        // i64 kl_spawn_thread(ptr, i64)  — spawn a thread running extern "C" fn(i64) -> i64
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("kl_spawn_thread", ft, None);
        }
        // i64 kl_join_thread(i64)  — join a thread, return result
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("kl_join_thread", ft, None);
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
            MirType::List(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
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
        let ptr_ty = self.context.ptr_type(Default::default());
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Alloca { dest, type_, .. } = inst {
                    let llvm_ty = self.llvm_type(type_);
                    self.alloca_types.entry(*dest).or_insert(llvm_ty);
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
                                    if let Some(pointee_type) = self.alloca_types.get(src) {
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
                                    let loaded = self.builder.build_load(*pointee_type, ptr, "")
                                        .map_err(|e| format!("load: {}", e))?;
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
                            let li = self.to_int_value(l);
                            let ri = self.to_int_value(r);

                            // For logical ops, truncate wider operands to i1 first
                            let bool_ty = self.context.bool_type();
                            let to_i1 = |val: inkwell::values::IntValue<'ctx>| {
                                if val.get_type().get_bit_width() > 1 {
                                    self.builder.build_int_truncate(val, bool_ty, "")
                                } else {
                                    Ok(val)
                                }
                            };

                            let result = match op {
                                MirBinaryOp::Add => self.builder.build_int_add(li, ri, "")
                                    .map_err(|e| format!("add: {}", e))?,
                                MirBinaryOp::Sub => self.builder.build_int_sub(li, ri, "")
                                    .map_err(|e| format!("sub: {}", e))?,
                                MirBinaryOp::Mul => self.builder.build_int_mul(li, ri, "")
                                    .map_err(|e| format!("mul: {}", e))?,
                                MirBinaryOp::Div => self.builder.build_int_signed_div(li, ri, "")
                                    .map_err(|e| format!("div: {}", e))?,
                                MirBinaryOp::Rem => self.builder.build_int_signed_rem(li, ri, "")
                                    .map_err(|e| format!("rem: {}", e))?,
                                MirBinaryOp::And => {
                                    let l1 = to_i1(li)
                                        .map_err(|e| format!("and-trunc: {}", e))?;
                                    let r1 = to_i1(ri)
                                        .map_err(|e| format!("and-trunc: {}", e))?;
                                    self.builder.build_and(l1, r1, "")
                                        .map_err(|e| format!("and: {}", e))?
                                },
                                MirBinaryOp::Or => {
                                    let l1 = to_i1(li)
                                        .map_err(|e| format!("or-trunc: {}", e))?;
                                    let r1 = to_i1(ri)
                                        .map_err(|e| format!("or-trunc: {}", e))?;
                                    self.builder.build_or(l1, r1, "")
                                        .map_err(|e| format!("or: {}", e))?
                                },
                                MirBinaryOp::Xor => {
                                    let l1 = to_i1(li)
                                        .map_err(|e| format!("xor-trunc: {}", e))?;
                                    let r1 = to_i1(ri)
                                        .map_err(|e| format!("xor-trunc: {}", e))?;
                                    self.builder.build_xor(l1, r1, "")
                                        .map_err(|e| format!("xor: {}", e))?
                                },
                                MirBinaryOp::Shl => self.builder.build_left_shift(li, ri, "")
                                    .map_err(|e| format!("shl: {}", e))?,
                                MirBinaryOp::Shr => self.builder.build_right_shift(li, ri, true, "")
                                    .map_err(|e| format!("shr: {}", e))?,
                                MirBinaryOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, li, ri, "")
                                    .map_err(|e| format!("eq: {}", e))?,
                                MirBinaryOp::Neq => self.builder.build_int_compare(IntPredicate::NE, li, ri, "")
                                    .map_err(|e| format!("neq: {}", e))?,
                                MirBinaryOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, li, ri, "")
                                    .map_err(|e| format!("lt: {}", e))?,
                                MirBinaryOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, li, ri, "")
                                    .map_err(|e| format!("gt: {}", e))?,
                                MirBinaryOp::Le => self.builder.build_int_compare(IntPredicate::SLE, li, ri, "")
                                    .map_err(|e| format!("le: {}", e))?,
                                MirBinaryOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, li, ri, "")
                                    .map_err(|e| format!("ge: {}", e))?,
                            };
                            let result_val = result.as_basic_value_enum();
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(dest_ptr, result_val)
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
                                "print" => "kl_print_int",
                                "println" => "kl_println_int",
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
                        MirInst::FieldPtr { dest, ptr, field_index, .. } => {
                            if let Some(base_ptr) = self.alloca_map.get(*ptr).and_then(|p| *p) {
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
                                    self.builder.build_int_cast(*int_val, *t, "")
                                        .map_err(|e| format!("cast: {}", e))?
                                        .as_basic_value_enum()
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
                            let spawn_fn = self.module.get_function("kl_spawn_thread")
                                .ok_or_else(|| "kl_spawn_thread not declared".to_string())?;
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
                            let join_fn = self.module.get_function("kl_join_thread")
                                .ok_or_else(|| "kl_join_thread not declared".to_string())?;
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
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(it)) =>
                                        self.builder.build_int_cast(*iv, *it, "")
                                            .map_err(|e| format!("ret intcast: {}", e))?
                                            .as_basic_value_enum(),
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
