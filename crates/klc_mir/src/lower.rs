use std::cell::RefCell;
use klc_core::ast::*;
use crate::mir::*;

/// Convert a Literal to (MirType, MirConstant).
fn literal_to_mir(value: &Literal) -> (MirType, MirConstant) {
    match value {
        Literal::Integer(n) => (MirType::I32, MirConstant::I32(*n as i32)),
        Literal::Float(n) => (MirType::F64, MirConstant::F64(*n)),
        Literal::String(s) => (MirType::Str, MirConstant::String(s.clone())),
        Literal::Boolean(b) => (MirType::Bool, MirConstant::Bool(*b)),
        Literal::None => (MirType::I32, MirConstant::Void),
    }
}

/// Return true if the MIR type is an integer type (i1, i8, i16, i32, i64, char, bool).
fn is_int_type(t: &MirType) -> bool {
    matches!(t, MirType::I1 | MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64 | MirType::Char | MirType::Bool)
}

/// Return the wider of two integer types. If either is not an integer, returns I32.
fn wider_int_type(a: &MirType, b: &MirType) -> MirType {
    use MirType::*;
    if a == b { return a.clone(); }
    let bit_width = |t: &MirType| -> u32 {
        match t {
            I1 | Bool => 1,
            I8 | Char => 8,
            I16 => 16,
            I32 => 32,
            I64 => 64,
            _ => 32,
        }
    };
    let wa = bit_width(a);
    let wb = bit_width(b);
    if wa >= wb { a.clone() } else { b.clone() }
}

/// Context for lowering a single function.
struct LowerCtx {
    /// Next available local index.
    next_local: usize,
    /// Map from variable name to local index.
    locals: std::collections::HashMap<String, usize>,
    /// Current basic block being built.
    current_block: MirBasicBlock,
    /// All completed basic blocks.
    blocks: Vec<MirBasicBlock>,
    /// Block counter for unique labels.
    block_counter: usize,
    /// Set of local IDs that hold string pointers (from str() or string literals).
    string_locals: Vec<usize>,
    /// Map from local ID to MIR type for every alloca created.
    local_types: std::collections::HashMap<usize, MirType>,
    /// Stack of break target labels for loops.
    break_targets: Vec<String>,
    /// Stack of continue target labels (loop header/condition) for loops.
    continue_targets: Vec<String>,
    /// Struct definitions: struct_name → Vec<(field_name, field_type)>
    struct_defs: std::collections::HashMap<String, Vec<(String, MirType)>>,
    /// When true, the next Stmt::If should treat each branch as a return expression.
    tail_if_as_return: bool,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            next_local: 0,
            locals: std::collections::HashMap::new(),
            current_block: MirBasicBlock::new("entry"),
            blocks: Vec::new(),
            block_counter: 0,
            string_locals: Vec::new(),
            local_types: std::collections::HashMap::new(),
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            struct_defs: std::collections::HashMap::new(),
            tail_if_as_return: false,
        }
    }

    fn alloc_local(&mut self, name: &str, type_: MirType) -> usize {
        let id = self.next_local;
        self.next_local += 1;
        self.current_block.insts.push(MirInst::Alloca {
            dest: id,
            type_: type_.clone(),
            name: name.to_string(),
        });
        self.local_types.insert(id, type_);
        id
    }

    fn fresh_block(&mut self) -> String {
        let label = format!("bb{}", self.block_counter);
        self.block_counter += 1;
        label
    }

    fn finish_block(&mut self, terminator: MirTerminator) {
        self.current_block.terminator = terminator;
        let label = self.fresh_block();
        let block = std::mem::replace(
            &mut self.current_block,
            MirBasicBlock::new(&label),
        );
        self.blocks.push(block);
    }
}

/// Lower a type-checked KL program to MIR.
pub struct Lowerer {
    fn_returns: RefCell<std::collections::HashMap<String, MirType>>,
    struct_defs: RefCell<std::collections::HashMap<String, Vec<(String, MirType)>>>,
    class_constructor_map: RefCell<std::collections::HashMap<String, String>>,
    const_values: RefCell<std::collections::HashMap<String, Expr>>,
    /// Method dispatch table: class_name -> (method_name -> mangled_function_name).
    /// Built during lower_program by scanning ClassMember::Method entries.
    /// Used to lower `obj.method(args)` into `Call ClassName::method(obj, args...)`.
    method_table: RefCell<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            fn_returns: RefCell::new(std::collections::HashMap::new()),
            struct_defs: RefCell::new(std::collections::HashMap::new()),
            class_constructor_map: RefCell::new(std::collections::HashMap::new()),
            const_values: RefCell::new(std::collections::HashMap::new()),
            method_table: RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// Lower a program to a MirModule.
    pub fn lower_program(&self, program: &Program) -> MirModule {
        // Pre-scan struct declarations to build struct definition map
        {
            let mut struct_defs = self.struct_defs.borrow_mut();
            struct_defs.clear();
            for decl in &program.declarations {
                if let Decl::Struct(s) = decl {
                    let fields: Vec<(String, MirType)> = s.fields.iter()
                        .map(|f| (f.name.clone(), ast_type_to_mir(&f.type_, None)))
                        .collect();
                    struct_defs.insert(s.name.clone(), fields);
                }
                if let Decl::Class(c) = decl {
                    let fields: Vec<(String, MirType)> = c.members.iter()
                        .filter_map(|m| {
                            if let ClassMember::Field(f) = m {
                                Some((f.name.clone(), ast_type_to_mir(&f.type_, None)))
                            } else {
                                None
                            }
                        })
                        .collect();
                    struct_defs.insert(c.name.clone(), fields);
                }
            }
        }

        // Pre-scan module-level constants
        {
            let mut cv = self.const_values.borrow_mut();
            cv.clear();
            for decl in &program.declarations {
                if let Decl::Constant(c) = decl {
                    cv.insert(c.name.clone(), *c.value.clone());
                }
            }
        }

        // Pre-scan function declarations and class constructors to build a return-type map.
        {
            let mut fn_returns = self.fn_returns.borrow_mut();
            let mut cc_map = self.class_constructor_map.borrow_mut();
            let mut method_table = self.method_table.borrow_mut();
            fn_returns.clear();
            cc_map.clear();
            method_table.clear();
            for decl in &program.declarations {
                if let Decl::Function(f) = decl {
                    let struct_defs = self.struct_defs.borrow().clone();
                    let ret_type = f.return_type.as_ref()
                        .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
                        .unwrap_or(MirType::Void);
                    fn_returns.insert(f.name.clone(), ret_type);
                }
                if let Decl::Class(c) = decl {
                    if c.members.iter().any(|m| matches!(m, ClassMember::Constructor(_))) {
                        cc_map.insert(c.name.clone(), format!("{}::new", c.name));
                        for member in &c.members {
                            if let ClassMember::Constructor(_ctor) = member {
                                let defs = self.struct_defs.borrow();
                                let fields = defs.get(&c.name).cloned().unwrap_or_default();
                                fn_returns.insert(format!("{}::new", c.name), MirType::Struct(c.name.clone(), fields));
                            }
                        }
                    }
                    // Build method dispatch table for this class.
                    // Each method `fn foo()` inside `class C` becomes a free function
                    // named `C::foo` that takes `this: C` as its first parameter.
                    let mut methods: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                    for member in &c.members {
                        if let ClassMember::Method(m) = member {
                            let mangled = format!("{}::{}", c.name, m.name);
                            methods.insert(m.name.clone(), mangled.clone());
                            // Record the method's return type for call resolution.
                            let defs = self.struct_defs.borrow();
                            let ret_type = m.return_type.as_ref()
                                .map(|rt| ast_type_to_mir(rt, Some(&defs)))
                                .unwrap_or(MirType::Void);
                            fn_returns.insert(mangled, ret_type);
                        }
                    }
                    if !methods.is_empty() {
                        method_table.insert(c.name.clone(), methods);
                    }
                }
            }
        }

        let mut module = MirModule::new();

        for decl in &program.declarations {
            match decl {
                Decl::Function(f) => {
                    if let Some(func) = self.lower_function(f) {
                        module.functions.push(func);
                    }
                }
                Decl::Class(c) => {
                    for member in &c.members {
                        if let ClassMember::Method(m) = member {
                            if let Some(func) = self.lower_method(m, &c.name) {
                                module.functions.push(func);
                            }
                        }
                        if let ClassMember::Constructor(ctor) = member {
                            let mut mir_func = MirFunction::new(format!("{}::new", c.name));
                            {
                                let defs = self.struct_defs.borrow();
                                mir_func.params = ctor.params.iter()
                                    .map(|p| ast_type_to_mir(&p.type_, Some(&defs)))
                                    .collect();
                                mir_func.return_type = if let Some(fields) = defs.get(&c.name) {
                                    MirType::Struct(c.name.clone(), fields.clone())
                                } else {
                                    MirType::Struct(c.name.clone(), vec![])
                                };
                            }
                            mir_func.local_count = 1;
                            let mut ctx = LowerCtx::new();
                            ctx.struct_defs = self.struct_defs.borrow().clone();
                            // Bind constructor params to locals
                            for (i, param) in ctor.params.iter().enumerate() {
                                let local = ctx.alloc_local(&param.name, ast_type_to_mir(&param.type_, Some(&ctx.struct_defs)));
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: local,
                                    value: MirValue::Param(i),
                                });
                                ctx.locals.insert(param.name.clone(), local);
                            }
                            let last_is_tail = matches!(ctor.body.statements.last(), Some(Stmt::Expression(_)));
                            for stmt in &ctor.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if ctx.current_block.terminator == MirTerminator::Unreachable {
                                if last_is_tail {
                                    let val_local = ctx.next_local - 1;
                                    ctx.finish_block(MirTerminator::Return(MirValue::Local(val_local)));
                                } else {
                                    ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
                                }
                            }
                            mir_func.local_count = ctx.next_local;
                            mir_func.basic_blocks = ctx.blocks;
                            module.functions.push(mir_func);
                        }
                    }
                }
                _ => {}
            }
        }

        module
    }

    fn lower_function(&self, f: &FunctionDecl) -> Option<MirFunction> {
        let body = f.body.as_ref()?;
        let struct_defs = self.struct_defs.borrow().clone();
        let mut mir_func = MirFunction::new(&f.name);
        mir_func.params = f.params.iter()
            .map(|p| ast_type_to_mir(&p.type_, Some(&struct_defs)))
            .collect();
        mir_func.return_type = f.return_type.as_ref()
            .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
            .unwrap_or(MirType::Void);

        let mut ctx = LowerCtx::new();
        ctx.struct_defs = self.struct_defs.borrow().clone();

        // Allocate and store params
        for (i, param) in f.params.iter().enumerate() {
            let local = ctx.alloc_local(&param.name, ast_type_to_mir(&param.type_, Some(&ctx.struct_defs)));
            ctx.current_block.insts.push(MirInst::Store {
                dest: local,
                value: MirValue::Param(i),
            });
            ctx.locals.insert(param.name.clone(), local);
        }

        // Lower body statements
        let last_is_tail = matches!(body.statements.last(), Some(Stmt::Expression(_)));
        let last_is_if_tail = matches!(body.statements.last(), Some(Stmt::If(_)));
        let stmt_count = body.statements.len();
        for (i, stmt) in body.statements.iter().enumerate() {
            if i + 1 == stmt_count && last_is_if_tail {
                ctx.tail_if_as_return = true;
            }
            ctx = self.lower_stmt(ctx, stmt);
        }

        // Default terminator: return void or tail expression value
        if ctx.current_block.terminator == MirTerminator::Unreachable {
            if last_is_tail {
                let val_local = ctx.next_local - 1;
                ctx.finish_block(MirTerminator::Return(MirValue::Local(val_local)));
            } else if !last_is_if_tail {
                ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
            }
        }

        mir_func.basic_blocks = ctx.blocks;
        mir_func.local_count = ctx.next_local;
        Some(mir_func)
    }

    /// Lower a class method. Like `lower_function`, but the method's MIR
    /// signature prepends an implicit `this` parameter of type
    /// `MirType::Struct(class_name, fields)` so the body can reference `this`
    /// and the method can be called as `ClassName::method(obj, args...)`.
    fn lower_method(&self, m: &FunctionDecl, class_name: &str) -> Option<MirFunction> {
        let body = m.body.as_ref()?;
        let struct_defs = self.struct_defs.borrow().clone();
        let mut mir_func = MirFunction::new(&format!("{}::{}", class_name, m.name));
        let this_type = struct_defs.get(class_name)
            .map(|fields| MirType::Struct(class_name.to_string(), fields.clone()))
            .unwrap_or(MirType::Struct(class_name.to_string(), vec![]));
        // First param is `this` (the receiver), then the explicit params
        // (skipping the first explicit param if it's named "this").
        let mut params: Vec<MirType> = vec![this_type.clone()];
        for (i, p) in m.params.iter().enumerate() {
            if i == 0 && p.name == "this" {
                continue;
            }
            params.push(ast_type_to_mir(&p.type_, Some(&struct_defs)));
        }
        mir_func.params = params;
        mir_func.return_type = m.return_type.as_ref()
            .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
            .unwrap_or(MirType::Void);

        let mut ctx = LowerCtx::new();
        ctx.struct_defs = struct_defs;

        // Bind `this` (param 0) into a local so the body's `Expr::PropertyAccess`
        // on `this` resolves to a struct field.
        let this_local = ctx.alloc_local("this", this_type);
        ctx.current_block.insts.push(MirInst::Store {
            dest: this_local,
            value: MirValue::Param(0),
        });
        ctx.locals.insert("this".to_string(), this_local);

        // Bind the explicit params (offset by 1 because of implicit `this`).
        // Skip the first explicit param if it's named "this" (it IS the receiver).
        let mut param_offset = 1;
        for (i, param) in m.params.iter().enumerate() {
            if i == 0 && param.name == "this" {
                // This param IS the receiver (this) — already bound above
                continue;
            }
            let local = ctx.alloc_local(&param.name, ast_type_to_mir(&param.type_, Some(&ctx.struct_defs)));
            ctx.current_block.insts.push(MirInst::Store {
                dest: local,
                value: MirValue::Param(param_offset),
            });
            ctx.locals.insert(param.name.clone(), local);
            param_offset += 1;
        }

        // Lower body statements
        let last_is_tail = matches!(body.statements.last(), Some(Stmt::Expression(_)));
        let last_is_if_tail = matches!(body.statements.last(), Some(Stmt::If(_)));
        let stmt_count = body.statements.len();
        for (i, stmt) in body.statements.iter().enumerate() {
            if i + 1 == stmt_count && last_is_if_tail {
                ctx.tail_if_as_return = true;
            }
            ctx = self.lower_stmt(ctx, stmt);
        }

        if ctx.current_block.terminator == MirTerminator::Unreachable {
            if last_is_tail {
                let val_local = ctx.next_local - 1;
                ctx.finish_block(MirTerminator::Return(MirValue::Local(val_local)));
            } else if !last_is_if_tail {
                ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
            }
        }

        mir_func.basic_blocks = ctx.blocks;
        mir_func.local_count = ctx.next_local;
        Some(mir_func)
    }

    fn lower_stmt(&self, mut ctx: LowerCtx, stmt: &Stmt) -> LowerCtx {
        match stmt {
            Stmt::Expression(expr) => {
                ctx = self.lower_expr(ctx, expr);
                ctx
            }
            Stmt::Variable(v) => {
                let has_init = !matches!(v.value.as_ref(), Expr::Literal { value: Literal::None, .. });
                let mut is_list = false;
                if !has_init {
                    if let Some(AstType::Generic { name, .. }) = &v.type_ {
                        if name == "list" {
                            is_list = true;
                        }
                    }
                }
                let val_local = if has_init {
                    ctx = self.lower_expr(ctx, &v.value);
                    Some(ctx.next_local - 1)
                } else if is_list {
                    // Auto-initialize list<T> variables with kl_list_new()
                    let list_ptr = ctx.alloc_local("_listv", ast_type_to_mir(v.type_.as_ref().unwrap(), Some(&ctx.struct_defs)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(list_ptr),
                        name: "kl_list_new".to_string(),
                        args: vec![],
                    });
                    Some(list_ptr)
                } else {
                    None
                };
                let var_type = v.type_.as_ref()
                    .map(|t| ast_type_to_mir(t, Some(&ctx.struct_defs)))
                    .unwrap_or_else(|| {
                        if let Some(vl) = val_local {
                            let inferred = ctx.local_types.get(&vl).cloned();
                            if let Some(t) = inferred {
                                if matches!(t, MirType::List(_) | MirType::Struct(_, _)) {
                                    t
                                } else if matches!(t, MirType::Str) {
                                    MirType::Str
                                } else {
                                    MirType::I32
                                }
                            } else {
                                MirType::I32
                            }
                        } else {
                            MirType::I32
                        }
                    });
                let local = ctx.alloc_local(&v.name, var_type);
                if let Some(vl) = val_local {
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: local,
                        value: MirValue::Local(vl),
                    });
                }
                ctx.locals.insert(v.name.clone(), local);
                ctx
            }
            Stmt::Return(ret_val) => {
                if let Some(expr) = ret_val {
                    let mut val_ctx = self.lower_expr(ctx, expr);
                    let val = if let Some(last) = val_ctx.current_block.insts.last() {
                        match last {
                            MirInst::Call { dest: Some(d), .. } => MirValue::Local(*d),
                            _ => MirValue::Local(val_ctx.next_local - 1),
                        }
                    } else {
                        MirValue::Constant(MirConstant::Void)
                    };
                    val_ctx.finish_block(MirTerminator::Return(val));
                    val_ctx
                } else {
                    ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
                    ctx
                }
            }
            Stmt::If(s) => {
                let is_tail = ctx.tail_if_as_return;
                ctx.tail_if_as_return = false;
                let else_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                let then_label = ctx.fresh_block();
                let cond_ctx = self.lower_expr(ctx, &s.condition);
                ctx = cond_ctx;
                let cond_val = if let Some(last) = ctx.current_block.insts.last() {
                    match last {
                        MirInst::Call { dest: Some(d), .. } => MirValue::Local(*d),
                        MirInst::BinaryOp { dest, .. } => MirValue::Local(*dest),
                        MirInst::UnaryOp { dest, .. } => MirValue::Local(*dest),
                        MirInst::Load { dest, .. } => MirValue::Local(*dest),
                        _ => MirValue::Local(ctx.next_local - 1),
                    }
                } else {
                    MirValue::Constant(MirConstant::Bool(false))
                };
                let elif_cond_labels: Vec<String> = (0..s.elif_branches.len())
                    .map(|_| ctx.fresh_block())
                    .collect();
                if !s.elif_branches.is_empty() {
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: cond_val,
                        true_block: then_label.clone(),
                        false_block: elif_cond_labels[0].clone(),
                    });
                } else if s.else_branch.is_some() {
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: cond_val,
                        true_block: then_label.clone(),
                        false_block: else_label.clone(),
                    });
                } else {
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: cond_val,
                        true_block: then_label.clone(),
                        false_block: end_label.clone(),
                    });
                }
                // Then block
                ctx.current_block = MirBasicBlock::new(then_label);
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                if is_tail {
                    ctx.finish_block(MirTerminator::Return(MirValue::Local(ctx.next_local - 1)));
                } else {
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
                }

                // Handle elif branches
                for (i, elif) in s.elif_branches.iter().enumerate() {
                    ctx.current_block = MirBasicBlock::new(elif_cond_labels[i].clone());
                    let elif_cond_ctx = self.lower_expr(ctx, &elif.condition);
                    ctx = elif_cond_ctx;
                    let elif_cond = MirValue::Local(ctx.next_local - 1);
                    let elif_then_label = ctx.fresh_block();
                    let elif_false_target = if i + 1 < s.elif_branches.len() {
                        elif_cond_labels[i + 1].clone()
                    } else if s.else_branch.is_some() {
                        else_label.clone()
                    } else {
                        end_label.clone()
                    };
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: elif_cond,
                        true_block: elif_then_label.clone(),
                        false_block: elif_false_target,
                    });
                    // elif then
                    ctx.current_block = MirBasicBlock::new(elif_then_label);
                    for stmt in &elif.body.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    if is_tail {
                        ctx.finish_block(MirTerminator::Return(MirValue::Local(ctx.next_local - 1)));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                }

                // Else block
                if let Some(el) = &s.else_branch {
                    ctx.current_block = MirBasicBlock::new(else_label);
                    for stmt in &el.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    if is_tail {
                        ctx.finish_block(MirTerminator::Return(MirValue::Local(ctx.next_local - 1)));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                } else if !s.elif_branches.is_empty() {
                    ctx.current_block = MirBasicBlock::new(else_label);
                    if is_tail {
                        ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                }

                if is_tail {
                    // All branches return directly; merge block returns void
                    ctx.current_block = MirBasicBlock::new(end_label);
                    ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
                } else {
                    ctx.current_block = MirBasicBlock::new(end_label);
                }
                ctx
            }
            Stmt::While(s) => {
                let cond_label = ctx.fresh_block();
                let body_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                let cond_label2 = cond_label.clone();

                ctx.finish_block(MirTerminator::Br(cond_label2.clone()));
                ctx.current_block = MirBasicBlock::new(cond_label);
                let cond_ctx = self.lower_expr(ctx, &s.condition);
                ctx = cond_ctx;
                let cond_val = MirValue::Local(ctx.next_local - 1);
                ctx.finish_block(MirTerminator::CondBr {
                    cond: cond_val,
                    true_block: body_label.clone(),
                    false_block: end_label.clone(),
                });

                ctx.current_block = MirBasicBlock::new(body_label);
                ctx.break_targets.push(end_label.clone());
                ctx.continue_targets.push(cond_label2.clone());
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.break_targets.pop();
                ctx.continue_targets.pop();
                ctx.finish_block(MirTerminator::Br(cond_label2.clone()));

                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
            Stmt::For(s) => {
                let body_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                let cond_label = ctx.fresh_block();
                let cond_label2 = cond_label.clone();

                let iter_local = ctx.alloc_local(&s.variable, MirType::I32);
                ctx.locals.insert(s.variable.clone(), iter_local);

                ctx.finish_block(MirTerminator::Br(cond_label2.clone()));
                ctx.current_block = MirBasicBlock::new(cond_label);
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Constant(MirConstant::Bool(true)),
                    true_block: body_label.clone(),
                    false_block: end_label.clone(),
                });
                ctx.current_block = MirBasicBlock::new(body_label);
                ctx.break_targets.push(end_label.clone());
                ctx.continue_targets.push(cond_label2.clone());
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.break_targets.pop();
                ctx.continue_targets.pop();
                ctx.finish_block(MirTerminator::Br(cond_label2.clone()));
                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
            Stmt::Match(s) => {
                let end_label = ctx.fresh_block();
                let has_lit = s.arms.iter().any(|a| matches!(a.pattern, Pattern::Literal { .. }));
                if has_lit {
                    ctx = self.lower_expr(ctx, &s.expression);
                }
                let match_val = if has_lit { Some(ctx.next_local - 1) } else { None };

                // Lower each arm with condition checks (forward order).
                // Each literal arm: check cond → arm_body, fallthrough → next check.
                // Wildcard/identifier: always execute (no check).
                let arm_count = s.arms.len();
                for (i, arm) in s.arms.iter().enumerate() {
                    let arm_label = ctx.fresh_block();
                    let is_last = i == arm_count - 1;
                    match &arm.pattern {
                        Pattern::Literal { value, .. } => {
                            let (vt, lc) = literal_to_mir(value);
                            let lit = ctx.alloc_local("_lit", vt);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: lit, value: MirValue::Constant(lc),
                            });
                            let eq = ctx.alloc_local("_eq", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: eq, op: MirBinaryOp::Eq,
                                left: MirValue::Local(match_val.unwrap()),
                                right: MirValue::Local(lit),
                            });
                            // Check: if eq → arm_body, else → next or end
                            let false_target = if is_last {
                                end_label.clone()
                            } else {
                                ctx.fresh_block()
                            };
                            ctx.finish_block(MirTerminator::CondBr {
                                cond: MirValue::Local(eq),
                                true_block: arm_label.clone(),
                                false_block: false_target.clone(),
                            });
                            // Arm body block
                            ctx.current_block = MirBasicBlock::new(arm_label);
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            // Switch to the false_target (next check or end)
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(false_target);
                            }
                        }
                        Pattern::Wildcard { .. } | Pattern::Identifier { .. } => {
                            // Always matches: branch from current block to arm_body
                            ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                            ctx.current_block = MirBasicBlock::new(arm_label);
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            // After Wildcard/Identifier, no more checks needed
                            ctx.current_block = MirBasicBlock::new(end_label);
                            return ctx;
                        }
                        _ => {
                            // Unknown pattern: skip
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            ctx.current_block = MirBasicBlock::new(end_label);
                            return ctx;
                        }
                    }
                }
                // If all arms were literal (no wildcard), the current block is already end_label
                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
            Stmt::Guard(g) => {
                for stmt in &g.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx
            }
            Stmt::Unsafe(u) => {
                for stmt in &u.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx
            }
            Stmt::Defer(d) => {
                ctx = self.lower_expr(ctx, &d.call);
                ctx
            }
            Stmt::BindingIf(b) => {
                for stmt in &b.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                if let Some(el) = &b.else_branch {
                    for stmt in &el.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                }
                ctx
            }
            Stmt::Break(_) => {
                if let Some(target) = ctx.break_targets.last().cloned() {
                    ctx.finish_block(MirTerminator::Br(target));
                } else {
                    ctx.finish_block(MirTerminator::Unreachable);
                }
                ctx
            }
            Stmt::Continue => {
                if let Some(target) = ctx.continue_targets.last().cloned() {
                    ctx.finish_block(MirTerminator::Br(target));
                } else {
                    ctx.finish_block(MirTerminator::Unreachable);
                }
                ctx
            }
            Stmt::WhileBind(w) => {
                for stmt in &w.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx
            }
            Stmt::Constant(c) => {
                ctx = self.lower_expr(ctx, &c.value);
                ctx
            }
            Stmt::TypedVariable(v) => {
                let mir_default = MirType::I32;
                ctx = self.lower_expr(ctx, &v.value);
                let val_local = ctx.next_local - 1;
                let mut is_str = false;
                for &id in &ctx.string_locals {
                    if id == val_local {
                        is_str = true;
                        break;
                    }
                }
                let var_type = v.type_.as_ref()
                    .map(|t| ast_type_to_mir(t, Some(&ctx.struct_defs)))
                    .unwrap_or(if is_str { MirType::Str } else { mir_default });
                let local = ctx.alloc_local(&v.name, var_type);
                ctx.locals.insert(v.name.clone(), local);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: local,
                    value: MirValue::Local(val_local),
                });
                ctx
            }
        }
    }

    fn lower_expr(&self, mut ctx: LowerCtx, expr: &Expr) -> LowerCtx {
        match expr {
            Expr::Literal { value, .. } => {
                let (mir_type, constant) = match value {
                    Literal::Integer(n) => (MirType::I32, MirConstant::I32(*n as i32)),
                    Literal::Float(n) => (MirType::F64, MirConstant::F64(*n)),
                    Literal::String(s) => (MirType::Str, MirConstant::String(s.clone())),
                    Literal::Boolean(b) => (MirType::Bool, MirConstant::Bool(*b)),
                    Literal::None => (MirType::I32, MirConstant::Void),
                };
                let local = ctx.alloc_local("_lit", mir_type);
                let is_str = matches!(constant, MirConstant::String(_));
                ctx.current_block.insts.push(MirInst::Store {
                    dest: local,
                    value: MirValue::Constant(constant),
                });
                if is_str {
                    ctx.string_locals.push(local);
                }
                ctx
            }
            Expr::Identifier { name, .. } => {
                if let Some(local) = ctx.locals.get(name) {
                    let src = *local;
                    let load_type = ctx.local_types.get(&src).cloned().unwrap_or(MirType::I32);
                    let is_str_load = load_type == MirType::Str;
                    let dest = ctx.alloc_local("_load", load_type);
                    if is_str_load {
                        ctx.string_locals.push(dest);
                    }
                    ctx.current_block.insts.push(MirInst::Load {
                        dest,
                        src,
                    });
                } else if let Some(const_expr) = self.const_values.borrow().get(name) {
                    // Inline module-level constant value
                    ctx = self.lower_expr(ctx, const_expr);
                }
                ctx
            }
            Expr::Binary { left, operator, right, .. } => {
                // Handle assignment: target = value
                if matches!(operator, BinaryOp::Assign) {
                    if let Expr::Index { target: list_expr, index, .. } = left.as_ref() {
                        ctx = self.lower_expr(ctx, list_expr);
                        let list_val = ctx.next_local - 1;
                        ctx = self.lower_expr(ctx, index);
                        let idx_val = ctx.next_local - 1;
                        let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: idx_i64,
                            value: MirValue::Local(idx_val),
                            to_type: MirType::I64,
                        });
                        ctx = self.lower_expr(ctx, right);
                        let val_local = ctx.next_local - 1;
                        let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: val_i64,
                            value: MirValue::Local(val_local),
                            to_type: MirType::I64,
                        });
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "kl_list_set".to_string(),
                            args: vec![
                                MirValue::Local(list_val),
                                MirValue::Local(idx_i64),
                                MirValue::Local(val_i64),
                            ],
                        });
                        return ctx;
                    }
                    if let Expr::PropertyAccess { object, property, .. } = left.as_ref() {
                        ctx = self.lower_expr(ctx, right);
                        let val_local = ctx.next_local - 1;
                        let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                            ctx.locals.get(name).copied()
                        } else {
                            None
                        };
                        if let Some(obj_ptr) = obj_ptr {
                            if let Some(MirType::Struct(_, fields)) = ctx.local_types.get(&obj_ptr).cloned() {
                                if let Some(field_idx) = fields.iter().position(|(fname, _)| fname == property) {
                                    let field_type = fields[field_idx].1.clone();
                                    let ft = ctx.alloc_local("_fptr", field_type.clone());
                                    ctx.current_block.insts.push(MirInst::FieldPtr {
                                        dest: ft,
                                        ptr: obj_ptr,
                                        field_index: field_idx,
                                        struct_type: Box::new(MirType::Struct("_".to_string(), fields)),
                                    });
                                    ctx.current_block.insts.push(MirInst::Store {
                                        dest: ft,
                                        value: MirValue::Local(val_local),
                                    });
                                }
                            }
                        }
                        return ctx;
                    }
                }

                ctx = self.lower_expr(ctx, left);
                let left_local = ctx.next_local - 1;
                let left_is_str = ctx.string_locals.contains(&left_local);
                ctx = self.lower_expr(ctx, right);
                let right_local = ctx.next_local - 1;
                let right_is_str = ctx.string_locals.contains(&right_local);

                // String concatenation if either operand is a string
                if matches!(operator, BinaryOp::Add) && (left_is_str || right_is_str) {
                    // Get ptr and len for each operand
                    // Left: use the existing local (pointer)
                    // Need length for left if it's a string
                    let left_len = if left_is_str {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(d),
                            name: "kl_strlen".to_string(),
                            args: vec![MirValue::Local(left_local)],
                        });
                        d
                    } else {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: d,
                            value: MirValue::Constant(MirConstant::I32(0)),
                        });
                        d
                    };
                    let right_len = if right_is_str {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(d),
                            name: "kl_strlen".to_string(),
                            args: vec![MirValue::Local(right_local)],
                        });
                        d
                    } else {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: d,
                            value: MirValue::Constant(MirConstant::I32(0)),
                        });
                        d
                    };
                    let result = ctx.alloc_local("_bin", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "kl_concat".to_string(),
                        args: vec![
                            MirValue::Local(left_local),
                            MirValue::Local(left_len),
                            MirValue::Local(right_local),
                            MirValue::Local(right_len),
                        ],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }

                // String equality comparison: use kl_eq_str instead of pointer comparison
                if (matches!(operator, BinaryOp::Eq | BinaryOp::Neq)) && (left_is_str || right_is_str) {
                    let result = ctx.alloc_local("_bin", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "kl_eq_str".to_string(),
                        args: vec![MirValue::Local(left_local), MirValue::Local(right_local)],
                    });
                    if matches!(operator, BinaryOp::Neq) {
                        let neq = ctx.alloc_local("_bin", MirType::I32);
                        ctx.current_block.insts.push(MirInst::UnaryOp {
                            dest: neq,
                            op: MirUnaryOp::Not,
                            operand: MirValue::Local(result),
                        });
                        return ctx;
                    }
                    return ctx;
                }

                // Coerce operands to the same type for binary operations.
                // Get the actual MIR types of each operand.
                let left_type = ctx.local_types.get(&left_local).cloned().unwrap_or(MirType::I32);
                let right_type = ctx.local_types.get(&right_local).cloned().unwrap_or(MirType::I32);
                let wider = wider_int_type(&left_type, &right_type);
                let left_operand = if left_type != wider && is_int_type(&left_type) {
                    let cast = ctx.alloc_local("_widen", wider.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(left_local),
                        to_type: wider.clone(),
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Local(left_local)
                };
                let right_operand = if right_type != wider && is_int_type(&right_type) {
                    let cast = ctx.alloc_local("_widen", wider.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(right_local),
                        to_type: wider.clone(),
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Local(right_local)
                };

                let dest = ctx.alloc_local("_bin", MirType::I32);

                let op = match operator {
                    BinaryOp::Add => MirBinaryOp::Add,
                    BinaryOp::Sub => MirBinaryOp::Sub,
                    BinaryOp::Mul => MirBinaryOp::Mul,
                    BinaryOp::Div => MirBinaryOp::Div,
                    BinaryOp::Rem => MirBinaryOp::Rem,
                    BinaryOp::And => MirBinaryOp::And,
                    BinaryOp::Or => MirBinaryOp::Or,
                    BinaryOp::BitAnd => MirBinaryOp::And,
                    BinaryOp::BitOr => MirBinaryOp::Or,
                    BinaryOp::BitXor => MirBinaryOp::Xor,
                    BinaryOp::Shl => MirBinaryOp::Shl,
                    BinaryOp::Shr => MirBinaryOp::Shr,
                    BinaryOp::Eq => MirBinaryOp::Eq,
                    BinaryOp::Neq => MirBinaryOp::Neq,
                    BinaryOp::Lt => MirBinaryOp::Lt,
                    BinaryOp::Gt => MirBinaryOp::Gt,
                    BinaryOp::Le => MirBinaryOp::Le,
                    BinaryOp::Ge => MirBinaryOp::Ge,
                    BinaryOp::Pow => MirBinaryOp::Mul, // simplified
                    _ => MirBinaryOp::Add,
                };
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest,
                    op,
                    left: left_operand,
                    right: right_operand,
                });
                ctx
            }
            Expr::Unary { operator, operand, .. } => {
                ctx = self.lower_expr(ctx, operand);
                let dest = ctx.alloc_local("_un", MirType::I32);
                let op = match operator {
                    UnaryOp::Neg => MirUnaryOp::Neg,
                    UnaryOp::Not => MirUnaryOp::Not,
                    UnaryOp::BitNot => MirUnaryOp::BitNot,
                };
                ctx.current_block.insts.push(MirInst::UnaryOp {
                    dest,
                    op,
                    operand: MirValue::Local(ctx.next_local - 2),
                });
                ctx
            }
            Expr::FunctionCall { target, arguments, .. } => {
                // Method dispatch: obj.method(args) → Call ClassName::method(obj, args...)
                // Checked BEFORE the list-specific shortcuts so real class methods win
                // over the generic list.add/list.pop fallbacks.
                if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    // Lower the receiver first to learn its type.
                    ctx = self.lower_expr(ctx, object);
                    let obj_local = ctx.next_local - 1;
                    let obj_type = ctx.local_types.get(&obj_local).cloned();

                    // If the receiver is a class instance (MirType::Struct) and the class
                    // declares a method named `property`, emit a real method call.
                    if let Some(MirType::Struct(class_name, _)) = &obj_type {
                        let method_table = self.method_table.borrow();
                        if let Some(methods) = method_table.get(class_name) {
                            if let Some(mangled) = methods.get(property) {
                                // First argument is the receiver (this).
                                let mut call_args = vec![MirValue::Local(obj_local)];
                                for arg in arguments {
                                    ctx = self.lower_expr(ctx, arg);
                                    call_args.push(MirValue::Local(ctx.next_local - 1));
                                }
                                let call_type = self.fn_returns.borrow()
                                    .get(mangled).cloned().unwrap_or(MirType::Void);
                                let dest = ctx.alloc_local("_mcall", call_type.clone());
                                if call_type == MirType::Str {
                                    ctx.string_locals.push(dest);
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(dest),
                                    name: mangled.clone(),
                                    args: call_args,
                                });
                                return ctx;
                            }
                        }
                    }

                    // Otherwise, fall back to list method shortcuts (pop/add) when the
                    // receiver is a list, not a class instance.
                    let is_list = obj_type.as_ref().map(|t| matches!(t, MirType::List(_))).unwrap_or(false);
                    if is_list {
                        if property == "pop" {
                            let result = ctx.alloc_local("_pop", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "kl_list_pop".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "add" {
                            for arg in arguments {
                                ctx = self.lower_expr(ctx, arg);
                                let arg_val = ctx.next_local - 1;
                                let arg_type = ctx.local_types.get(&arg_val).cloned().unwrap_or(MirType::I32);
                                let arg_i64 = ctx.alloc_local("_arg64", MirType::I64);
                                if matches!(arg_type, MirType::Struct(_, _)) {
                                    // When adding a struct to a list whose elem type is I64 (inferred),
                                    // update the list's ELEMENT type so that list[i] returns the struct.
                                    // We need to update the alloca's type, not the loaded value's type.
                                    if let Expr::Identifier { name, .. } = object.as_ref() {
                                        if let Some(var_local) = ctx.locals.get(name) {
                                            if let Some(MirType::List(elem)) = ctx.local_types.get(var_local) {
                                                if matches!(elem.as_ref(), MirType::I64) {
                                                    let struct_type = arg_type.clone();
                                                    ctx.local_types.insert(*var_local, MirType::List(Box::new(struct_type)));
                                                }
                                            }
                                        }
                                    }
                                    // Allocate heap memory for struct, store value there, push pointer
                                    let heap_ptr = ctx.alloc_local("_heapp", MirType::I64);
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(heap_ptr),
                                        name: "kl_alloc".to_string(),
                                        args: vec![MirValue::Constant(MirConstant::I64(64))],
                                    });
                                    // Copy struct value to heap memory
                                    ctx.current_block.insts.push(MirInst::Memcpy {
                                        dest_ptr_local: heap_ptr,
                                        src_alloca_local: arg_val,
                                        struct_type: Box::new(arg_type.clone()),
                                    });
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: arg_i64,
                                        value: MirValue::Local(heap_ptr),
                                        to_type: MirType::I64,
                                    });
                                } else {
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: arg_i64,
                                        value: MirValue::Local(arg_val),
                                        to_type: MirType::I64,
                                    });
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: None,
                                    name: "kl_list_push".to_string(),
                                    args: vec![MirValue::Local(obj_local), MirValue::Local(arg_i64)],
                                });
                            }
                            return ctx;
                        }
                    }
                }

                let name = if let Expr::Identifier { name, .. } = target.as_ref() {
                    name.clone()
                } else {
                    "_call".to_string()
                };

                // Special case: len() built-in — return string or list length
                if name == "len" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let arg_local = ctx.next_local - 1;
                    let is_list = ctx.local_types.get(&arg_local)
                        .map(|t| matches!(t, MirType::List(_)))
                        .unwrap_or(false);
                    if is_list {
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "kl_list_len".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    } else {
                        let result = ctx.alloc_local("_len", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "kl_strlen".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    }
                    return ctx;
                }

                // Special case: input() built-in — read line from stdin
                if name == "input" && arguments.is_empty() {
                    let result = ctx.alloc_local("_inp", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "kl_input".to_string(),
                        args: vec![],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }

                // Special case: str() built-in — convert integer to string
                if name == "str" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let arg_local = ctx.next_local - 1;
                    // Cast the argument from i32 to i64 (kl_i64_to_str expects i64)
                    let cast_local = ctx.alloc_local("_cast64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast_local,
                        value: MirValue::Local(arg_local),
                        to_type: MirType::I64,
                    });
                    // Call kl_i64_to_str(cast_arg) → returns string pointer
                    let ptr_dest = ctx.alloc_local("_strptr", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(ptr_dest),
                        name: "kl_i64_to_str".to_string(),
                        args: vec![MirValue::Local(cast_local)],
                    });
                    ctx.string_locals.push(ptr_dest);
                    return ctx;
                }

                // Special case: substr(str, start, count) — substring extraction
                if name == "substr" && arguments.len() == 3 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let str_local = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, &arguments[1]);
                    let start_local = ctx.next_local - 1;
                    let start_i64 = ctx.alloc_local("_s64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: start_i64,
                        value: MirValue::Local(start_local),
                        to_type: MirType::I64,
                    });
                    ctx = self.lower_expr(ctx, &arguments[2]);
                    let count_local = ctx.next_local - 1;
                    let count_i64 = ctx.alloc_local("_c64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: count_i64,
                        value: MirValue::Local(count_local),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_substr", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "kl_substr".to_string(),
                        args: vec![
                            MirValue::Local(str_local),
                            MirValue::Local(start_i64),
                            MirValue::Local(count_i64),
                        ],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }

                // Special case: print/println with string literal argument
                if (name == "print" || name == "println") && arguments.len() == 1 {
                    if let Expr::Literal { value: Literal::String(s), .. } = &arguments[0] {
                        let dest = ctx.alloc_local("_call", MirType::I32);
                        let call_name = if name == "println" { "kl_println" } else { "kl_print" };
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(dest),
                            name: call_name.to_string(),
                            args: vec![
                                MirValue::Constant(MirConstant::String(s.clone())),
                                MirValue::Constant(MirConstant::I32(s.len() as i32)),
                            ],
                        });
                        return ctx;
                    }
                }

                // Remap class constructor calls: Point(10, 20) → Point::new
                let resolved_name = self.class_constructor_map.borrow()
                    .get(&name).cloned().unwrap_or_else(|| name.clone());

                let mut args = Vec::new();
                for arg in arguments {
                    ctx = self.lower_expr(ctx, arg);
                    args.push(MirValue::Local(ctx.next_local - 1));
                }
                let call_type = builtin_return_type(&resolved_name).unwrap_or_else(|| {
                    self.fn_returns.borrow().get(&resolved_name).cloned().unwrap_or(MirType::I32)
                });
                let dest = ctx.alloc_local("_call", call_type.clone());
                if call_type == MirType::Str {
                    ctx.string_locals.push(dest);
                }

                // Special case: println() with no arguments — print a newline
                if name == "println" && args.is_empty() {
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(dest),
                        name: "kl_println".to_string(),
                        args: vec![
                            MirValue::Constant(MirConstant::String("\n".to_string())),
                            MirValue::Constant(MirConstant::I32(1)),
                        ],
                    });
                    return ctx;
                }

                // For print/println with dynamic arguments
                if (name == "print" || name == "println") && !args.is_empty() {
                    let first_arg = &args[0];
                    if let MirValue::Local(id) = first_arg {
                        if ctx.string_locals.contains(id) {
                            // This local holds a string pointer — need to get its length
                            let len_dest = ctx.alloc_local("_strlen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(len_dest),
                                name: "kl_strlen".to_string(),
                                args: vec![MirValue::Local(*id)],
                            });
                            let call_name = if name == "println" { "kl_println" } else { "kl_print" };
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(dest),
                                name: call_name.to_string(),
                                args: vec![
                                    MirValue::Local(*id),
                                    MirValue::Local(len_dest),
                                ],
                            });
                            return ctx;
                        } else {
                            // Non-string argument — call kl_print_int / kl_println_int
                            let call_name = if name == "println" { "kl_println_int" } else { "kl_print_int" };
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(dest),
                                name: call_name.to_string(),
                                args: vec![MirValue::Local(*id)],
                            });
                            return ctx;
                        }
                    }
                }

                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(dest),
                    name: resolved_name.clone(),
                    args,
                });
                if matches!(resolved_name.as_str(), "to_upper" | "to_lower" | "trim" | "replace" | "input" | "read_str" | "substr") {
                    ctx.string_locals.push(dest);
                }
                ctx
            }
            Expr::Assignment { target, value, .. } => {
                // Handle list[index] = value → kl_list_set
                if let Expr::Index { target: list_expr, index, .. } = target.as_ref() {
                    ctx = self.lower_expr(ctx, list_expr);
                    let list_val = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, index);
                    let idx_val = ctx.next_local - 1;
                    let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: idx_i64,
                        value: MirValue::Local(idx_val),
                        to_type: MirType::I64,
                    });
                    ctx = self.lower_expr(ctx, value);
                    let val_local = ctx.next_local - 1;
                    let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: None,
                        name: "kl_list_set".to_string(),
                        args: vec![
                            MirValue::Local(list_val),
                            MirValue::Local(idx_i64),
                            MirValue::Local(val_i64),
                        ],
                    });
                    return ctx;
                }

                ctx = self.lower_expr(ctx, value);
                let val_local = ctx.next_local - 1;
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    let local = if let Some(existing) = ctx.locals.get(name) {
                        *existing
                    } else {
                        // Implicit variable declaration: infer type from value
                        let var_type = ctx.local_types.get(&val_local).cloned().unwrap_or_else(|| {
                            if ctx.string_locals.contains(&val_local) { MirType::Str } else { MirType::I32 }
                        });
                        let new_local = ctx.alloc_local(name, var_type);
                        ctx.locals.insert(name.clone(), new_local);
                        new_local
                    };
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: local,
                        value: MirValue::Local(val_local),
                    });
                } else if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    // Struct field assignment: p.x = val
                    // Use the variable's alloca pointer directly, not a loaded value
                    let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                        ctx.locals.get(name).copied()
                    } else {
                        None
                    };
                    if let Some(obj_ptr) = obj_ptr {
                        let obj_type = ctx.local_types.get(&obj_ptr).cloned();
                        if let Some(MirType::Struct(_, fields)) = &obj_type {
                            if let Some(field_idx) = fields.iter().position(|(fname, _)| fname == property) {
                                let field_type = fields[field_idx].1.clone();
                                let field_ptr = ctx.alloc_local("_fieldptr", field_type.clone());
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: field_ptr,
                                    ptr: obj_ptr,
                                    field_index: field_idx,
                                    struct_type: Box::new(obj_type.unwrap()),
                                });
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: field_ptr,
                                    value: MirValue::Local(val_local),
                                });
                            }
                        }
                    }
                }
                ctx
            }
            Expr::PropertyAccess { object, property, .. } => {
                // If accessing `len` on a list-typed variable, call kl_list_len.
                // Must check type before lowering to avoid treating struct fields
                // named `len` (e.g., Parser.len) as list length calls.
                if property == "len" {
                    let is_list = match object.as_ref() {
                        Expr::Identifier { name, .. } => {
                            ctx.locals.get(name)
                                .and_then(|l| ctx.local_types.get(l))
                                .map(|t| matches!(t, MirType::List(_)))
                                .unwrap_or(false)
                        }
                        _ => false,
                    };
                    if is_list {
                        ctx = self.lower_expr(ctx, object);
                        let obj_val = ctx.next_local - 1;
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "kl_list_len".to_string(),
                            args: vec![MirValue::Local(obj_val)],
                        });
                        return ctx;
                    }
                }
                // Struct field access: use the variable's alloca pointer directly
                let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                    ctx.locals.get(name).copied()
                } else {
                    ctx = self.lower_expr(ctx, object);
                    let obj_val = ctx.next_local - 1;
                    // Fallback: assume the loaded value is a struct pointer
                    Some(obj_val)
                };
                if let Some(obj_ptr) = obj_ptr {
                    let obj_type = ctx.local_types.get(&obj_ptr).cloned();
                    if let Some(MirType::Struct(_, fields)) = &obj_type {
                        if let Some(field_idx) = fields.iter().position(|(fname, _)| fname == property) {
                            let field_type = fields[field_idx].1.clone();
                            let field_ptr = ctx.alloc_local("_fieldptr", field_type.clone());
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: field_ptr,
                                ptr: obj_ptr,
                                field_index: field_idx,
                                struct_type: Box::new(obj_type.unwrap()),
                            });
                            let result = ctx.alloc_local("_field", field_type.clone());
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: result,
                                src: field_ptr,
                            });
                            if field_type == MirType::Str {
                                ctx.string_locals.push(result);
                            }
                            return ctx;
                        }
                    }
                }
                ctx
            }
            Expr::OptionalChain { target, property, .. } => {
                ctx = self.lower_expr(ctx, target);
                let _ = property;
                ctx
            }
            Expr::ErrorProp { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                ctx
            }
            Expr::List { elements, .. } => {
                // Determine element type: first non-empty string element → Str, else I64 (runtime stores i64)
                let elem_type = elements.iter().find_map(|e| {
                    if let Expr::Literal { value: Literal::String(_), .. } = e { Some(MirType::Str) }
                    else { None }
                }).unwrap_or(MirType::I64);
                let handle = ctx.alloc_local("_list", MirType::List(Box::new(elem_type.clone())));
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(handle),
                    name: "kl_list_new".to_string(),
                    args: vec![],
                });
                for elem in elements {
                    ctx = self.lower_expr(ctx, elem);
                    let val = ctx.next_local - 1;
                    let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: MirValue::Local(val),
                        to_type: MirType::I64,
                    });
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: None,
                        name: "kl_list_push".to_string(),
                        args: vec![MirValue::Local(handle), MirValue::Local(val_i64)],
                    });
                }
                let result = ctx.alloc_local("_listv", MirType::List(Box::new(elem_type)));
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result,
                    value: MirValue::Local(handle),
                });
                ctx
            }
            Expr::Index { target, index, .. } => {
                ctx = self.lower_expr(ctx, target);
                let target_val = ctx.next_local - 1;
                ctx = self.lower_expr(ctx, index);
                let index_val = ctx.next_local - 1;
                let target_type = ctx.local_types.get(&target_val).cloned().unwrap_or(MirType::I32);
                if target_type == MirType::Str {
                    // String indexing: source[i] -> char_at(source, i) -> returns i8
                    let idx_i32 = ctx.alloc_local("_idx32", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: idx_i32,
                        value: MirValue::Local(index_val),
                        to_type: MirType::I32,
                    });
                    let result = ctx.alloc_local("_char", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "kl_char_at".to_string(),
                        args: vec![MirValue::Local(target_val), MirValue::Local(idx_i32)],
                    });
                    return ctx;
                }
                let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                ctx.current_block.insts.push(MirInst::Cast {
                    dest: idx_i64,
                    value: MirValue::Local(index_val),
                    to_type: MirType::I64,
                });
                let list_elem_type = match &target_type {
                    MirType::List(inner) => inner.as_ref().clone(),
                    _ => MirType::I64,
                };
                // kl_list_get always returns i64 (pointers for strings, values for ints)
                let result = ctx.alloc_local("_idx", MirType::I64);
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(result),
                    name: "kl_list_get".to_string(),
                    args: vec![MirValue::Local(target_val), MirValue::Local(idx_i64)],
                });
                if list_elem_type == MirType::Str {
                    let str_result = ctx.alloc_local("_idxstr", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: str_result,
                        value: MirValue::Local(result),
                        to_type: MirType::Str,
                    });
                    ctx.string_locals.push(str_result);
                } else if matches!(list_elem_type, MirType::Struct(_, _)) {
                    // Struct pointer from list: inttoptr + load
                    let struct_result = ctx.alloc_local("_struct", list_elem_type.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: struct_result,
                        value: MirValue::Local(result),
                        to_type: list_elem_type,
                    });
                }
                ctx
            }
            Expr::Dictionary { entries, .. } => {
                for (_, val) in entries {
                    ctx = self.lower_expr(ctx, val);
                }
                ctx
            }
            Expr::StructLiteral { struct_name, fields, .. } => {
                let struct_defs = ctx.struct_defs.clone();
                if let Some(def_fields) = struct_defs.get(struct_name.as_str()) {
                    let def_fields = def_fields.clone();
                    let struct_type = MirType::Struct(struct_name.clone(), def_fields.clone());
                    let struct_local = ctx.alloc_local("_slit", struct_type.clone());
                    for (field_name, field_expr) in fields {
                        ctx = self.lower_expr(ctx, field_expr);
                        let val_local = ctx.next_local - 1;
                        if let Some(field_idx) = def_fields.iter()
                            .position(|(fname, _)| fname == field_name)
                        {
                            let fptr = ctx.alloc_local("_sfptr", MirType::Void);
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: fptr,
                                ptr: struct_local,
                                field_index: field_idx,
                                struct_type: Box::new(MirType::Struct(struct_name.clone(), def_fields.clone())),
                            });
                            let field_type = def_fields[field_idx].1.clone();
                            let cast_val = if field_type != ctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I64) {
                                let cast_local = ctx.alloc_local("_scast", field_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: cast_local,
                                    value: MirValue::Local(val_local),
                                    to_type: field_type,
                                });
                                MirValue::Local(cast_local)
                            } else {
                                MirValue::Local(val_local)
                            };
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: fptr,
                                value: cast_val,
                            });
                        }
                    }
                    let result = ctx.alloc_local("_sval", struct_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result,
                        src: struct_local,
                    });
                    result
                } else {
                    ctx.alloc_local("_slit_err", MirType::Void)
                };
                ctx
            }
            Expr::Tuple { elements, .. } => {
                for elem in elements {
                    ctx = self.lower_expr(ctx, elem);
                }
                ctx
            }
            Expr::Closure { body, .. } => {
                ctx = self.lower_expr(ctx, body);
                ctx
            }
            Expr::Await { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                ctx
            }
            Expr::Async { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                ctx
            }
            Expr::Spread { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                ctx
            }
            Expr::RangeSlice { start, end, .. } => {
                if let Some(s) = start { ctx = self.lower_expr(ctx, s); }
                if let Some(e) = end { ctx = self.lower_expr(ctx, e); }
                ctx
            }
            Expr::Loop { body, .. } => {
                let loop_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                let loop_label2 = loop_label.clone();
                ctx.finish_block(MirTerminator::Br(loop_label2.clone()));
                ctx.current_block = MirBasicBlock::new(loop_label);
                ctx.break_targets.push(end_label.clone());
                ctx.continue_targets.push(loop_label2.clone());
                for stmt in &body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.break_targets.pop();
                ctx.continue_targets.pop();
                ctx.finish_block(MirTerminator::Br(loop_label2.clone()));
                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
        }
    }
}

/// Check if a call name refers to a builtin that returns a string.
#[allow(dead_code)]
fn is_string_builtin_name(name: &str) -> bool {
    matches!(name, "kl_strlen" | "kl_i64_to_str" | "kl_input" | "kl_concat"
        | "kl_str_to_upper" | "kl_str_to_lower" | "kl_str_trim" | "kl_str_replace"
        | "kl_read_str"
        | "to_upper" | "to_lower" | "trim" | "replace" | "str" | "input" | "read_str")
}

/// Return the MIR type for known builtin functions, or None for generic functions.
fn builtin_return_type(name: &str) -> Option<MirType> {
    match name {
        "contains" => Some(MirType::I32),
        "to_upper" | "to_lower" | "trim" | "replace" | "input" => Some(MirType::Str),
        "open" | "close" | "write_str" => Some(MirType::I32),
        "read_str" => Some(MirType::Str),
        "char_at" => Some(MirType::Char),
        "ord" => Some(MirType::I32),
        "is_digit" | "is_alpha" | "is_alnum" | "is_whitespace" | "is_upper" | "is_lower" => Some(MirType::I32),
        "now" => Some(MirType::I64),
        "sleep" | "list_push" | "list_set" => Some(MirType::Void),
        "list_new" => Some(MirType::List(Box::new(MirType::I32))),
        "list_get" => Some(MirType::I64),
        "list_len" => Some(MirType::I64),
        "substr" => Some(MirType::Str),
        "eq_str" => Some(MirType::I32),
        _ => None,
    }
}

/// Convert an AST type to an MIR type.
fn ast_type_to_mir(ast: &AstType, struct_defs: Option<&std::collections::HashMap<String, Vec<(String, MirType)>>>) -> MirType {
    match ast {
        AstType::Primitive { name, .. } => match name.as_str() {
            "i8" => MirType::I8,
            "i16" => MirType::I16,
            "i32" => MirType::I32,
            "i64" => MirType::I64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            "bool" => MirType::Bool,
            "char" => MirType::Char,
            "str" => MirType::Str,
            "void" => MirType::Void,
            _ => MirType::I32,
        },
        AstType::User { name, .. } => match name.as_str() {
            "i8" => MirType::I8,
            "i16" => MirType::I16,
            "i32" => MirType::I32,
            "i64" => MirType::I64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            "bool" => MirType::Bool,
            "char" => MirType::Char,
            "str" => MirType::Str,
            name => {
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        return MirType::Struct(name.to_string(), fields.clone());
                    }
                }
                MirType::Struct(name.to_string(), vec![])
            }
        },
        AstType::Generic { name, args, .. } => {
            if name == "list" {
                if args.is_empty() { MirType::List(Box::new(MirType::I32)) }
                else { MirType::List(Box::new(ast_type_to_mir(&args[0], struct_defs))) }
            } else if args.is_empty() {
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        return MirType::Struct(name.to_string(), fields.clone());
                    }
                }
                MirType::Struct(name.clone(), vec![])
            } else {
                ast_type_to_mir(&args[0], struct_defs)
            }
        }
        AstType::Optional { inner, .. } => MirType::Ptr(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::Error { inner, .. } => ast_type_to_mir(inner, struct_defs),
    }
}
