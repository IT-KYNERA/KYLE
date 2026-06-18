use std::cell::RefCell;
use klc_core::ast::*;
use crate::mir::*;

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
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            fn_returns: RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// Lower a program to a MirModule.
    pub fn lower_program(&self, program: &Program) -> MirModule {
        // Pre-scan function declarations to build a return-type map.
        // This allows Expr::FunctionCall to know the return type of user-defined
        // functions (e.g. whether a function returns str vs i32).
        {
            let mut fn_returns = self.fn_returns.borrow_mut();
            fn_returns.clear();
            for decl in &program.declarations {
                if let Decl::Function(f) = decl {
                    let ret_type = f.return_type.as_ref()
                        .map(|rt| ast_type_to_mir(rt))
                        .unwrap_or(MirType::Void);
                    fn_returns.insert(f.name.clone(), ret_type);
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
                            if let Some(func) = self.lower_function(m) {
                                module.functions.push(func);
                            }
                        }
                        if let ClassMember::Constructor(ctor) = member {
                            let mut mir_func = MirFunction::new(format!("{}::new", c.name));
                            mir_func.params = ctor.params.iter()
                                .map(|p| ast_type_to_mir(&p.type_))
                                .collect();
                            mir_func.return_type = MirType::Struct(vec![]);
                            mir_func.local_count = 1;
                            let mut ctx = LowerCtx::new();
                            for stmt in &ctor.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
                            // drop last empty block
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
        let mut mir_func = MirFunction::new(&f.name);
        mir_func.params = f.params.iter()
            .map(|p| ast_type_to_mir(&p.type_))
            .collect();
        mir_func.return_type = f.return_type.as_ref()
            .map(|rt| ast_type_to_mir(rt))
            .unwrap_or(MirType::Void);

        let mut ctx = LowerCtx::new();

        // Allocate and store params
        for (i, param) in f.params.iter().enumerate() {
            let local = ctx.alloc_local(&param.name, ast_type_to_mir(&param.type_));
            ctx.current_block.insts.push(MirInst::Store {
                dest: local,
                value: MirValue::Param(i),
            });
            ctx.locals.insert(param.name.clone(), local);
        }

        // Lower body statements
        for stmt in &body.statements {
            ctx = self.lower_stmt(ctx, stmt);
        }

        // Default terminator: return void
        if ctx.current_block.terminator == MirTerminator::Unreachable {
            ctx.finish_block(MirTerminator::Return(MirValue::Constant(MirConstant::Void)));
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
                let val_ctx = self.lower_expr(ctx, &v.value);
                ctx = val_ctx;
                let val_local = ctx.next_local - 1;
                // Force check string_locals[0..]
                let mut is_str = false;
                for &id in &ctx.string_locals {
                    if id == val_local {
                        is_str = true;
                        break;
                    }
                }
                let var_type = v.type_.as_ref()
                    .map(|t| ast_type_to_mir(t))
                    .unwrap_or(if is_str { MirType::Str } else { MirType::I32 });
                let local = ctx.alloc_local(&v.name, var_type);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: local,
                    value: MirValue::Local(val_local),
                });
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
                ctx.finish_block(MirTerminator::Br(end_label.clone()));

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
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
                }

                // Else block
                if let Some(el) = &s.else_branch {
                    ctx.current_block = MirBasicBlock::new(else_label);
                    for stmt in &el.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
                } else if !s.elif_branches.is_empty() {
                    ctx.current_block = MirBasicBlock::new(else_label);
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
                }

                ctx.current_block = MirBasicBlock::new(end_label);
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
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
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
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.finish_block(MirTerminator::Br(cond_label2.clone()));
                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
            Stmt::Match(s) => {
                let end_label = ctx.fresh_block();
                for arm in &s.arms {
                    let arm_label = ctx.fresh_block();
                    let _next_arm = ctx.fresh_block();
                    ctx.current_block = MirBasicBlock::new(arm_label);
                    for stmt in &arm.body.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
                }
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
                ctx.finish_block(MirTerminator::Unreachable);
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
                    .map(|t| ast_type_to_mir(t))
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
                }
                ctx
            }
            Expr::Binary { left, operator, right, .. } => {
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
                    let result = ctx.alloc_local("_bin", MirType::I64);
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
                    left: MirValue::Local(left_local),
                    right: MirValue::Local(right_local),
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
                let name = if let Expr::Identifier { name, .. } = target.as_ref() {
                    name.clone()
                } else {
                    "_call".to_string()
                };

                // Special case: len() built-in — return string length
                if name == "len" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let arg_local = ctx.next_local - 1;
                    let result = ctx.alloc_local("_len", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "kl_strlen".to_string(),
                        args: vec![MirValue::Local(arg_local)],
                    });
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

                let mut args = Vec::new();
                for arg in arguments {
                    ctx = self.lower_expr(ctx, arg);
                    args.push(MirValue::Local(ctx.next_local - 1));
                }
                let call_type = builtin_return_type(&name).unwrap_or_else(|| {
                    self.fn_returns.borrow().get(&name).cloned().unwrap_or(MirType::I32)
                });
                let dest = ctx.alloc_local("_call", call_type.clone());
                if call_type == MirType::Str {
                    ctx.string_locals.push(dest);
                }

                // For print/println with dynamic string arguments (str() result or string literal store)
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
                        }
                    }
                }

                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(dest),
                    name: name.clone(),
                    args,
                });
                if matches!(name.as_str(), "to_upper" | "to_lower" | "trim" | "replace" | "input" | "read_str") {
                    ctx.string_locals.push(dest);
                }
                ctx
            }
            Expr::Assignment { target, value, .. } => {
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
                }
                ctx
            }
            Expr::PropertyAccess { object, property, .. } => {
                ctx = self.lower_expr(ctx, object);
                let _ = property;
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
                for elem in elements {
                    ctx = self.lower_expr(ctx, elem);
                }
                ctx
            }
            Expr::Dictionary { entries, .. } => {
                for (_, val) in entries {
                    ctx = self.lower_expr(ctx, val);
                }
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
                for stmt in &body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
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
        "sleep" => Some(MirType::Void),
        _ => None,
    }
}

/// Convert an AST type to an MIR type.
fn ast_type_to_mir(ast: &AstType) -> MirType {
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
            _ => MirType::Struct(vec![]),
        },
        AstType::Generic { args, .. } => {
            if args.is_empty() { MirType::Struct(vec![]) }
            else { ast_type_to_mir(&args[0]) }
        }
        AstType::Optional { inner, .. } => MirType::Ptr(Box::new(ast_type_to_mir(inner))),
        AstType::Error { inner, .. } => ast_type_to_mir(inner),
    }
}
