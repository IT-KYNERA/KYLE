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
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name);
        Self {
            context,
            builder,
            module,
            fn_value_map: HashMap::new(),
            param_values: HashMap::new(),
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
                let struct_ty = self.context.opaque_struct_type(name);
                if struct_ty.is_opaque() {
                    let field_types: Vec<BasicTypeEnum<'ctx>> = fields.iter()
                        .map(|(_, ty)| self.llvm_type(ty))
                        .collect();
                    struct_ty.set_body(&field_types, false);
                }
                struct_ty.as_basic_type_enum()
            }
        }
    }

    fn declare_function(&mut self, func: &MirFunction) -> Result<(), String> {
        let ret_type = self.llvm_type(&func.return_type);
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = func.params
            .iter()
            .map(|p| self.llvm_type(p).into())
            .collect();
        let fn_type = ret_type.fn_type(&param_types, false);
        let fn_value = self.module.add_function(&func.name, fn_type, None);
        self.fn_value_map.insert(func.name.clone(), fn_value);
        Ok(())
    }

    fn compile_function(&mut self, func: &MirFunction) -> Result<(), String> {
        let fn_value = self.fn_value_map.get(&func.name)
            .ok_or_else(|| format!("Function {} not declared", func.name))?;

        let mut alloca_types: HashMap<usize, BasicTypeEnum<'ctx>> = HashMap::new();
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Alloca { dest, type_, .. } = inst {
                    alloca_types.entry(*dest).or_insert_with(|| self.llvm_type(type_));
                }
            }
        }

        let mut block_map: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();
        for bb in &func.basic_blocks {
            let llvm_bb = self.context.append_basic_block(*fn_value, &bb.label);
            block_map.insert(bb.label.clone(), llvm_bb);
        }

        let mut alloca_map: Vec<Option<PointerValue<'ctx>>> = Vec::new();

        if let Some(entry_bb) = func.basic_blocks.first() {
            if let Some(&llvm_entry) = block_map.get(&entry_bb.label) {
                self.builder.position_at_end(llvm_entry);

                for (dest, llvm_type) in &alloca_types {
                    while alloca_map.len() <= *dest {
                        alloca_map.push(None);
                    }
                    let ptr = self.builder.build_alloca(*llvm_type, "")
                        .map_err(|e| format!("alloca {}: {}", dest, e))?;
                    alloca_map[*dest] = Some(ptr);
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
                            if let Some(ptr) = alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(ptr, val)
                                    .map_err(|e| format!("store: {}", e))?;
                            }
                            last_value_map.insert(*dest, val);
                        }
                        MirInst::Load { dest, src } => {
                            if let Some(ptr) = alloca_map.get(*src).and_then(|p| *p) {
                                if let Some(pointee_type) = alloca_types.get(src) {
                                    let loaded = self.builder.build_load(*pointee_type, ptr, "")
                                        .map_err(|e| format!("load: {}", e))?;
                                    last_value_map.insert(*dest, loaded);
                                }
                            }
                        }
                        MirInst::BinaryOp { dest, op, left, right } => {
                            let l = self.value_to_llvm(left, &last_value_map)?;
                            let r = self.value_to_llvm(right, &last_value_map)?;
                            let li = self.to_int_value(l);
                            let ri = self.to_int_value(r);
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
                                MirBinaryOp::And => self.builder.build_and(li, ri, "")
                                    .map_err(|e| format!("and: {}", e))?,
                                MirBinaryOp::Or => self.builder.build_or(li, ri, "")
                                    .map_err(|e| format!("or: {}", e))?,
                                MirBinaryOp::Xor => self.builder.build_xor(li, ri, "")
                                    .map_err(|e| format!("xor: {}", e))?,
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
                            last_value_map.insert(*dest, result.as_basic_value_enum());
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
                            last_value_map.insert(*dest, result.as_basic_value_enum());
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
                                let llvm_args: Vec<BasicValueEnum<'ctx>> = args
                                    .iter()
                                    .map(|a| self.value_to_llvm(a, &last_value_map))
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
                                        if let Some(alloca_ptr) = alloca_map.get(*d).and_then(|p| *p) {
                                            self.builder.build_store(alloca_ptr, ret_val)
                                                .map_err(|e| format!("call store {}: {}", name, e))?;
                                        }
                                        last_value_map.insert(*d, ret_val);
                                    }
                                }
                            }
                        }
                        MirInst::PtrOffset { dest, ptr, index } => {
                            if let Some(base_ptr) = alloca_map.get(*ptr).and_then(|p| *p) {
                                let idx = self.value_to_llvm(index, &last_value_map)?;
                                let int_idx = idx.into_int_value();
                                if let Some(pointee_type) = alloca_types.get(ptr) {
                                    let gep = unsafe {
                                        self.builder.build_gep(*pointee_type, base_ptr, &[int_idx], "")
                                            .map_err(|e| format!("gep: {}", e))?
                                    };
                                    last_value_map.insert(*dest, gep.as_basic_value_enum());
                                }
                            }
                        }
                        MirInst::FieldPtr { dest, ptr, field_index, .. } => {
                            if let Some(base_ptr) = alloca_map.get(*ptr).and_then(|p| *p) {
                                if let Some(struct_type) = alloca_types.get(ptr) {
                                    let zero = self.context.i32_type().const_zero();
                                    let idx_val = self.context.i32_type().const_int(*field_index as u64, false);
                                    let gep = unsafe {
                                        self.builder.build_gep(*struct_type, base_ptr, &[zero, idx_val], "")
                                            .map_err(|e| format!("field_ptr: {}", e))?
                                    };
                                    alloca_map[*dest] = Some(gep);
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
                            match (&val, &target_type) {
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::IntType(t)) => {
                                    let result = self.builder.build_int_cast(*int_val, *t, "")
                                        .map_err(|e| format!("cast: {}", e))?;
                                    last_value_map.insert(*dest, result.as_basic_value_enum());
                                }
                                (BasicValueEnum::PointerValue(ptr_val), BasicTypeEnum::IntType(t)) => {
                                    let result = self.builder.build_ptr_to_int(*ptr_val, *t, "")
                                        .map_err(|e| format!("ptrtoint: {}", e))?;
                                    last_value_map.insert(*dest, result.as_basic_value_enum());
                                }
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::PointerType(t)) => {
                                    let result = self.builder.build_int_to_ptr(*int_val, *t, "")
                                        .map_err(|e| format!("inttoptr: {}", e))?;
                                    last_value_map.insert(*dest, result.as_basic_value_enum());
                                }
                                        (BasicValueEnum::IntValue(int_val), BasicTypeEnum::StructType(s)) => {
                                    let ptr_ty = self.context.ptr_type(Default::default());
                                    let ptr_val = self.builder.build_int_to_ptr(*int_val, ptr_ty, "_ptr")
                                        .map_err(|e| format!("inttoptr: {}", e))?;
                                    let loaded = self.builder.build_load(*s, ptr_val, "_struct")
                                        .map_err(|e| format!("load struct: {}", e))?;
                                    last_value_map.insert(*dest, loaded);
                                }
                                (BasicValueEnum::StructValue(struct_val), BasicTypeEnum::IntType(i)) => {
                                    // Allocate a temp, store the struct, ptrtoint the pointer
                                    let struct_ty = struct_val.get_type();
                                    let temp_alloca = self.builder.build_alloca(struct_ty, "_tmp_struct")
                                        .map_err(|e| format!("alloca: {}", e))?;
                                    self.builder.build_store(temp_alloca, *struct_val)
                                        .map_err(|e| format!("store struct: {}", e))?;
                                    let ptr = temp_alloca.as_basic_value_enum();
                                    let ptr_val = self.builder.build_ptr_to_int(ptr.into_pointer_value(), *i, "_ptrint")
                                        .map_err(|e| format!("ptrtoint: {}", e))?;
                                    last_value_map.insert(*dest, ptr_val.as_basic_value_enum());
                                }
                                _ => {} // type pair not supported
                            }
                        }
                    }
                }

                match &bb.terminator {
                    MirTerminator::Return(value) => {
                        let val = self.value_to_llvm(value, &last_value_map)?;
                        self.builder.build_return(Some(&val))
                            .map_err(|e| format!("ret: {}", e))?;
                    }
                    MirTerminator::Br(label) => {
                        if let Some(&target) = block_map.get(label) {
                            let _ = self.builder.build_unconditional_branch(target);
                        }
                    }
                    MirTerminator::CondBr { cond, true_block, false_block } => {
                        let cond_val = self.value_to_llvm(cond, &last_value_map)?;
                        let cond_int = cond_val.into_int_value();
                        if let (Some(&t), Some(&f)) = (block_map.get(true_block), block_map.get(false_block)) {
                            let _ = self.builder.build_conditional_branch(cond_int, t, f);
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
            MirValue::Local(id) => {
                Ok(last_values.get(id).copied().unwrap_or_else(|| {
                    self.context.i32_type().const_zero().as_basic_value_enum()
                }))
            }
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
