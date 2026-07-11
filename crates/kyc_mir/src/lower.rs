use std::cell::RefCell;
use kyc_core::ast::*;
use crate::mir::*;

thread_local! {
    static TYPE_ALIAS_CACHE: RefCell<std::collections::HashMap<String, AstType>> = RefCell::new(std::collections::HashMap::new());
}

/// Convert a Literal to (MirType, MirConstant).
fn literal_to_mir(value: &Literal) -> (MirType, MirConstant) {
    match value {
        Literal::Integer(n) => {
            if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                (MirType::I32, MirConstant::I32(*n as i32))
            } else {
                (MirType::I64, MirConstant::I64(*n))
            }
        }
        Literal::Float(n) => (MirType::F64, MirConstant::F64(*n)),
        Literal::String(s) => (MirType::Str, MirConstant::String(s.clone())),
        Literal::Boolean(b) => (MirType::Bool, MirConstant::Bool(*b)),
                    Literal::Char(c) => (MirType::Char, MirConstant::I32(*c)),
        Literal::None => (MirType::I32, MirConstant::Void),
        Literal::Null => (MirType::Ptr(Box::new(MirType::Void)), MirConstant::Null),
    }
}

/// Return true if the MIR type is an integer type (i1, i8, i16, i32, i64, char, bool).
fn is_int_type(t: &MirType) -> bool {
    matches!(t, MirType::I1 | MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64 | MirType::Char | MirType::Bool)
}

/// Return the wider of two types, supporting both int and float widening.
/// F64 > F32 > any integer type.
fn wider_type(a: &MirType, b: &MirType) -> MirType {
    use MirType::*;
    if a == b { return a.clone(); }
    if matches!(a, F64) || matches!(b, F64) { return F64; }
    if matches!(a, F32) || matches!(b, F32) { return F32; }
    wider_int_type(a, b)
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
    /// When set, the next Stmt::Match should store each arm's result to this local.
    match_result_local: Option<usize>,
    /// When true, the current function returns a result struct. Return values
    /// are automatically wrapped in {disc: 0, payload: value}, and the `?`
    /// operator propagates errors by early-returning the struct.
    is_fallible: bool,
    /// The MIR return type of the current function (used for fallible result wrapping).
    return_type: MirType,
    /// Deferred call expressions (AST) to execute before function return.
    /// Stored in definition order; emitted in reverse (LIFO).
    deferred_exprs: Vec<Box<Expr>>,
    /// When set, break instructions store `true` here before branching.
    break_flag_local: Option<usize>,
    /// When set, nested if/else branches should store their tail value to this alloca.
    if_result_alloca: Option<usize>,
    /// Set to true when a Stmt::If is the last statement lowered. Reset by each Stmt::If entry.
    last_stmt_was_if: bool,
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
            match_result_local: None,
            is_fallible: false,
            return_type: MirType::Void,
            deferred_exprs: Vec::new(),
            break_flag_local: None,
            if_result_alloca: None,
            last_stmt_was_if: false,
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

    /// Emit a return, wrapping the value in a success result struct if the
    /// function is fallible. Used by both explicit `return` statements and
    /// tail-expression returns.
    fn emit_return(&mut self, value: MirValue) {
        if self.is_fallible {
            if let MirValue::Local(id) = &value {
                if let Some(MirType::Struct(name, _)) = self.local_types.get(id) {
                    if name == "Result" {
                        self.finish_block(MirTerminator::Return(value));
                        return;
                    }
                }
            }
            let result_struct = self.return_type.clone();
            let result_local = self.alloc_local("_ret", result_struct.clone());
            // Set disc = 0 (success/ok)
            let disc_ptr = self.alloc_local("_rsp", MirType::I32);
            self.current_block.insts.push(MirInst::FieldPtr {
                dest: disc_ptr,
                ptr: result_local,
                field_index: 0,
                struct_type: Box::new(result_struct.clone()),
            });
            self.current_block.insts.push(MirInst::Store {
                dest: disc_ptr,
                value: MirValue::Constant(MirConstant::I32(0)),
            });
            // Set payload (widened to I64)
            let payload_ptr = self.alloc_local("_rpp", MirType::I64);
            self.current_block.insts.push(MirInst::FieldPtr {
                dest: payload_ptr,
                ptr: result_local,
                field_index: 1,
                struct_type: Box::new(result_struct.clone()),
            });
            let widened = if let MirValue::Local(id) = &value {
                if self.local_types.get(id).map(|t| *t != MirType::I64).unwrap_or(true) {
                    let cast = self.alloc_local("_rpv", MirType::I64);
                    self.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: value,
                        to_type: MirType::I64,
                    });
                    MirValue::Local(cast)
                } else {
                    value
                }
            } else {
                value
            };
            self.current_block.insts.push(MirInst::Store {
                dest: payload_ptr,
                value: widened,
            });
            self.finish_block(MirTerminator::Return(MirValue::Local(result_local)));
        } else {
            self.finish_block(MirTerminator::Return(value));
        }
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
    /// Static method mangled names (no `this` param).
    static_methods: RefCell<std::collections::HashSet<String>>,
    /// Maps each class to its optional parent class name, used to walk the
    /// inheritance chain when resolving method calls (polymorphism/override).
    class_parent_map: RefCell<std::collections::HashMap<String, Option<String>>>,
    /// Enum variant index map: enum_name -> (variant_name -> index)
    enum_variants: RefCell<std::collections::HashMap<String, std::collections::HashMap<String, usize>>>,
    /// Counter for generating unique closure function names.
    closure_counter: RefCell<usize>,
    /// Closure functions generated during lowering.
    closure_functions: RefCell<Vec<MirFunction>>,
    /// Counter for generating unique async wrapper function names.
    async_counter: RefCell<usize>,
    /// Async wrapper functions generated during lowering.
    async_functions: RefCell<Vec<MirFunction>>,
    /// Generic struct templates (structs with type_params).
    generic_struct_templates: RefCell<std::collections::HashMap<String, StructDecl>>,
    /// Generic function templates (functions with type_params).
    generic_function_templates: RefCell<std::collections::HashMap<String, FunctionDecl>>,
    /// Generic class methods keyed by class name, for monomorphization during struct literal lowering.
    generic_class_methods: RefCell<std::collections::HashMap<String, Vec<FunctionDecl>>>,
    /// All function declarations (for default-value resolution at call sites).
    function_decls: RefCell<std::collections::HashMap<String, FunctionDecl>>,
    /// Specialized MIR functions generated by monomorphization.
    specialized_mir_functions: RefCell<Vec<MirFunction>>,
    /// Field defaults: class_name → (field_name → default_expr)
    field_defaults: RefCell<std::collections::HashMap<String, std::collections::HashMap<String, Expr>>>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            fn_returns: RefCell::new(std::collections::HashMap::new()),
            struct_defs: RefCell::new(std::collections::HashMap::new()),
            class_constructor_map: RefCell::new(std::collections::HashMap::new()),
            const_values: RefCell::new(std::collections::HashMap::new()),
            method_table: RefCell::new(std::collections::HashMap::new()),
            static_methods: RefCell::new(std::collections::HashSet::new()),
            class_parent_map: RefCell::new(std::collections::HashMap::new()),
            enum_variants: RefCell::new(std::collections::HashMap::new()),
            closure_counter: RefCell::new(0),
            closure_functions: RefCell::new(Vec::new()),
            async_counter: RefCell::new(0),
            async_functions: RefCell::new(Vec::new()),
            generic_struct_templates: RefCell::new(std::collections::HashMap::new()),
            generic_function_templates: RefCell::new(std::collections::HashMap::new()),
            generic_class_methods: RefCell::new(std::collections::HashMap::new()),
            function_decls: RefCell::new(std::collections::HashMap::new()),
            specialized_mir_functions: RefCell::new(Vec::new()),
            field_defaults: RefCell::new(std::collections::HashMap::new()),
        }
    }
    /// Lower a program to a MirModule.
    pub fn lower_program(&self, program: &Program) -> MirModule {
        // Pre-scan type alias definitions — must happen before function pre-scans
        // so that ast_type_to_mir can resolve alias names during fn_returns population.
        TYPE_ALIAS_CACHE.with(|cache| {
            let mut aliases = cache.borrow_mut();
            aliases.clear();
            for decl in &program.declarations {
                if let Decl::TypeAlias(t) = decl {
                    aliases.insert(t.name.clone(), t.type_.clone());
                }
            }
        });

        // Pre-scan struct declarations to build struct definition map
        // Two-pass: first register all struct/class names, then resolve field types
        // with the complete map so nested struct references work.
        // Generic structs (those with type_params) are stored as templates for later
        // monomorphization and skipped from normal struct_defs registration.
        {
            let mut struct_defs = self.struct_defs.borrow_mut();
            let mut generic_templates = self.generic_struct_templates.borrow_mut();
            struct_defs.clear();
            generic_templates.clear();
            // Pass 1: insert all names with empty field lists
            for decl in &program.declarations {
                if let Decl::Struct(s) = decl {
                    if !s.type_params.is_empty() {
                        generic_templates.insert(s.name.clone(), s.clone());
                    } else {
                        struct_defs.insert(s.name.clone(), vec![]);
                    }
                }
                if let Decl::Class(c) = decl {
                    if !c.type_params.is_empty() {
                        // Generic class — store as struct template
                        generic_templates.insert(c.name.clone(), StructDecl {
                            name: c.name.clone(),
                            type_params: c.type_params.clone(),
                            fields: c.members.iter().filter_map(|m| {
                                if let ClassMember::Field(f) = m { Some(f.clone()) } else { None }
                            }).collect(),
                            span: c.span.clone(),
                        });
                    } else {
                        struct_defs.insert(c.name.clone(), vec![]);
                    }
                }
                if let Decl::Enum(e) = decl {
                    struct_defs.insert(e.name.clone(), vec![]);
                }
            }
            // Pass 2: resolve field types with full struct_defs map
            for decl in &program.declarations {
                if let Decl::Struct(s) = decl {
                    if !s.type_params.is_empty() {
                        continue; // Generic struct — skip, monomorphized on use
                    }
                    let fields: Vec<(String, MirType)> = s.fields.iter()
                        .map(|f| (f.name.clone(), ast_type_to_mir(&f.type_, Some(&struct_defs))))
                        .collect();
                    struct_defs.insert(s.name.clone(), fields);
                }
                if let Decl::Class(c) = decl {
                    if !c.type_params.is_empty() {
                        continue; // Generic class — monomorphized on use
                    }
                    let fields = Self::collect_class_fields(c, &program, &struct_defs);
                    struct_defs.insert(c.name.clone(), fields);
                }
                if let Decl::Enum(e) = decl {
                    // Enums are represented as tagged unions: { disc: I32, payload: I64 }
                    let fields = vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ];
                    struct_defs.insert(e.name.clone(), fields);
                }
            }
            // Register tuple return types in struct_defs
            for decl in &program.declarations {
                if let Decl::Function(f) = decl {
                    if let Some(ref rt) = f.return_type {
                        if let AstType::Generic { name, args, .. } = rt {
                            if name == "tuple" {
                                let field_types: Vec<(String, MirType)> = args.iter().enumerate()
                                    .map(|(i, el)| (format!("_{}", i), ast_type_to_mir(el, Some(&struct_defs))))
                                    .collect();
                                let type_suffix: String = args.iter()
                                    .map(|a| match a {
                                        AstType::Primitive { name, .. } => name.clone(),
                                        _ => "x".to_string(),
                                    })
                                    .collect();
                                let struct_name = format!("_tuple_{}_{}", field_types.len(), type_suffix);
                                struct_defs.entry(struct_name)
                                    .or_insert_with(|| field_types);
                            }
                        }
                    }
                }
            }
            // Register built-in TypeInfo struct for type introspection
            struct_defs.entry("TypeInfo".to_string()).or_insert_with(|| vec![
                ("name".to_string(), MirType::Str),
                ("kind".to_string(), MirType::Str),
                ("size".to_string(), MirType::I32),
            ]);
        }

        // Populate field defaults for classes
        {
            let mut fds = self.field_defaults.borrow_mut();
            fds.clear();
            for decl in &program.declarations {
                if let Decl::Class(c) = decl {
                    if !c.type_params.is_empty() { continue; }
                    let mut defaults = std::collections::HashMap::new();
                    for member in &c.members {
                        if let ClassMember::Field(f) = member {
                            if let Some(ref default_expr) = f.default {
                                defaults.insert(f.name.clone(), (**default_expr).clone());
                            }
                        }
                    }
                    fds.insert(c.name.clone(), defaults);
                }
            }
        }

        // Pre-scan module-level constants and immutable variables
        {
            let mut cv = self.const_values.borrow_mut();
            cv.clear();
            for decl in &program.declarations {
                match decl {
                    Decl::Constant(c) => {
                        cv.insert(c.name.clone(), *c.value.clone());
                    }
                    Decl::Variable(v) => {
                        // Immutable module-level variables are inlined like
                        // constants — their value never changes at runtime
                        if !v.is_mutable {
                            cv.insert(v.name.clone(), *v.value.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Pre-scan enum variant indices
        {
            let mut ev = self.enum_variants.borrow_mut();
            ev.clear();
            for decl in &program.declarations {
                if let Decl::Enum(e) = decl {
                    let mut variant_map = std::collections::HashMap::new();
                    for (idx, variant) in e.variants.iter().enumerate() {
                        variant_map.insert(variant.name.clone(), idx);
                    }
                    ev.insert(e.name.clone(), variant_map);
                }
            }
            // Register built-in Result enum (used by ok(v)/error(e) patterns)
            let mut result_variants = std::collections::HashMap::new();
            result_variants.insert("Ok".to_string(), 0);
            result_variants.insert("Err".to_string(), 1);
            ev.insert("Result".to_string(), result_variants);
            // Also register Result in struct_defs so ast_type_to_mir can find its fields
            let mut struct_defs = self.struct_defs.borrow_mut();
            if !struct_defs.contains_key("Result") {
                struct_defs.insert("Result".to_string(), vec![
                    ("disc".to_string(), MirType::I32),
                    ("payload".to_string(), MirType::I64),
                ]);
            }
        }

        // Pre-scan function declarations and class constructors to build a return-type map.
        // Generic functions (those with type_params) are stored as templates and
        // monomorphized lazily when concrete call sites are encountered.
        {
            let mut fn_returns = self.fn_returns.borrow_mut();
            let mut cc_map = self.class_constructor_map.borrow_mut();
            let mut method_table = self.method_table.borrow_mut();
            let mut class_parent_map = self.class_parent_map.borrow_mut();
            let mut generic_fn_templates = self.generic_function_templates.borrow_mut();
            fn_returns.clear();
            cc_map.clear();
            method_table.clear();
            class_parent_map.clear();
            generic_fn_templates.clear();
            for decl in &program.declarations {
                if let Decl::Function(f) = decl {
                    if !f.type_params.is_empty() {
                        // Generic function — store as template for lazy monomorphization
                        generic_fn_templates.insert(f.name.clone(), f.clone());
                        continue;
                    }
                    let struct_defs = self.struct_defs.borrow().clone();
                    let ret_type = f.return_type.as_ref()
                        .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
                        .unwrap_or(MirType::Void);
                    fn_returns.insert(f.name.clone(), ret_type);
                    self.function_decls.borrow_mut().insert(f.name.clone(), f.clone());
                }
                if let Decl::Class(c) = decl {
                    class_parent_map.insert(c.name.clone(), c.parent.clone());
                    if !c.type_params.is_empty() {
                        // Generic class — store methods as templates for lazy monomorphization
                        let mut class_methods = Vec::new();
                        for member in &c.members {
                            if let ClassMember::Method(m) = member {
                                let template_name = format!("{}::{}", c.name, m.name);
                                generic_fn_templates.insert(template_name, m.clone());
                                class_methods.push(m.clone());
                            }
                        }
                        self.generic_class_methods.borrow_mut().insert(c.name.clone(), class_methods);
                    }
                    if c.members.iter().any(|m| matches!(m, ClassMember::Constructor(_))) {
                        cc_map.insert(c.name.clone(), format!("{}::new", c.name));
                        for member in &c.members {
                            if let ClassMember::Constructor(_ctor) = member {
                                let defs = self.struct_defs.borrow();
                                let fields = defs.get(&c.name).cloned().unwrap_or_default();
                                fn_returns.insert(format!("{}::new", c.name), MirType::Struct(c.name.clone(), fields));
                            }
                        }
                    } else {
                        // No explicit constructor — auto-register a default one
                        cc_map.insert(c.name.clone(), format!("{}::new", c.name));
                        let defs = self.struct_defs.borrow();
                        let fields = defs.get(&c.name).cloned().unwrap_or_default();
                        fn_returns.insert(format!("{}::new", c.name), MirType::Struct(c.name.clone(), fields));
                    }
                    // Build method dispatch table for this class.
                    // Each method `fn foo()` inside `class C` becomes a free function
                    // named `C::foo` that takes `this: C` as its first parameter.
                    let mut methods: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                    let mut static_set = self.static_methods.borrow_mut();
                    for member in &c.members {
                        if let ClassMember::Method(m) = member {
                            let mangled = format!("{}::{}", c.name, m.name);
                            methods.insert(m.name.clone(), mangled.clone());
                            if m.is_static {
                                static_set.insert(mangled.clone());
                            }
                            let defs = self.struct_defs.borrow();
                            let ret_type = m.return_type.as_ref()
                                .map(|rt| ast_type_to_mir(rt, Some(&defs)))
                                .unwrap_or(MirType::Void);
                            fn_returns.insert(mangled, ret_type);
                        }
                        if let ClassMember::Property(p) = member {
                            let defs = self.struct_defs.borrow();
                            let prop_type = ast_type_to_mir(&p.type_, Some(&defs));
                            if p.getter.is_some() {
                                let mangled = format!("{}::get_{}", c.name, p.name);
                                methods.insert(format!("get_{}", p.name), mangled.clone());
                                fn_returns.insert(mangled, prop_type.clone());
                            }
                            if p.setter.is_some() {
                                let mangled = format!("{}::set_{}", c.name, p.name);
                                methods.insert(format!("set_{}", p.name), mangled.clone());
                                fn_returns.insert(mangled, MirType::Void);
                            }
                        }
                    }
                    if !methods.is_empty() {
                        method_table.insert(c.name.clone(), methods);
                    }
                }
            }
        }

        let mut module = MirModule::new();

        // Collect any closure functions generated during previous lower_program calls
        {
            let cf = self.closure_functions.borrow();
            for func in cf.iter() {
                module.functions.push(func.clone());
            }
        }

        for decl in &program.declarations {
            match decl {
                Decl::Function(f) => {
                    if !f.type_params.is_empty() {
                        continue; // Generic functions are monomorphized lazily
                    }
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
                        if let ClassMember::Property(p) = member {
                            // Generate getter method: Class::get_prop(this) -> PropType
                            if let Some(getter_body) = &p.getter {
                                let getter_fn = FunctionDecl {
                                    name: format!("get_{}", p.name),
                                    type_params: vec![],
                                    params: vec![Parameter {
                                        name: "this".into(),
                                        type_: AstType::User { name: c.name.clone(), span: p.span },
                                        default: None, variadic: false, mode: ParamMode::Move, span: p.span,
                                    }],
                                    return_type: Some(p.type_.clone()),
                                    is_async: false, is_const: false, is_static: false, is_abstract: false, is_extern: false, is_test: false,
                                    visibility: Visibility::Public,
                                    body: Some(getter_body.clone()),
                                    span: p.span,
                                };
                                if let Some(func) = self.lower_method(&getter_fn, &c.name) {
                                    module.functions.push(func);
                                }
                            }
                            // Generate setter method: Class::set_prop(this, value) -> void
                            if let Some((set_param, setter_body)) = &p.setter {
                                let void_type = AstType::Primitive { name: "void".into(), span: p.span };
                                let setter_fn = FunctionDecl {
                                    name: format!("set_{}", p.name),
                                    type_params: vec![],
                                    params: vec![
                                        Parameter {
                                            name: "this".into(),
                                            type_: AstType::User { name: c.name.clone(), span: p.span },
                                            default: None, variadic: false, mode: ParamMode::Move, span: p.span,
                                        },
                                        Parameter {
                                            name: set_param.clone(),
                                            type_: p.type_.clone(),
                                            default: None, variadic: false, mode: ParamMode::Move, span: p.span,
                                        },
                                    ],
                                    return_type: Some(void_type),
                                    is_async: false, is_const: false, is_static: false, is_abstract: false, is_extern: false, is_test: false,
                                    visibility: Visibility::Public,
                                    body: Some(setter_body.clone()),
                                    span: p.span,
                                };
                                if let Some(func) = self.lower_method(&setter_fn, &c.name) {
                                    module.functions.push(func);
                                }
                            }
                        }
                        if let ClassMember::Constructor(ctor) = member {
                            let mut mir_func = MirFunction::new(format!("{}::new", c.name));
                            {
                                let defs = self.struct_defs.borrow();
                                mir_func.params = ctor.params.iter()
                                    .map(|p| ast_type_to_mir(&p.type_, Some(&defs)))
                                    .collect();
                                mir_func.param_modes = ctor.params.iter().map(|p| p.mode).collect();
                                mir_func.return_type = if let Some(fields) = defs.get(&c.name) {
                                    MirType::Struct(c.name.clone(), fields.clone())
                                } else {
                                    MirType::Struct(c.name.clone(), vec![])
                                };
                            }
                            let mut ctx = LowerCtx::new();
                            ctx.struct_defs = self.struct_defs.borrow().clone();
                            // Allocate `this` local for the struct being constructed
                            // so that constructor body can use `this.field = value` syntax.
                            let this_type = if let Some(fields) = ctx.struct_defs.get(&c.name) {
                                MirType::Struct(c.name.clone(), fields.clone())
                            } else {
                                MirType::Struct(c.name.clone(), vec![])
                            };
                            let this_local = ctx.alloc_local("this", this_type);
        ctx.locals.insert("this".to_string(), this_local);
        ctx.locals.insert("self".to_string(), this_local);
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
                            let tail_is_field_assign = ctor.body.statements.last().map_or(false, |s| {
                                if let Stmt::Expression(e) = s {
                                    match e {
                                        Expr::Assignment { target, .. } => {
                                            if let Expr::PropertyAccess { object, .. } = target.as_ref() {
                                                if let Expr::Identifier { name, .. } = object.as_ref() {
                                                    return name == "this";
                                                }
                                            }
                                        }
                                        Expr::Binary { left, operator, .. } if matches!(operator, BinaryOp::Assign) => {
                                            if let Expr::PropertyAccess { object, .. } = left.as_ref() {
                                                if let Expr::Identifier { name, .. } = object.as_ref() {
                                                    return name == "this";
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                false
                            });
                            for stmt in &ctor.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if ctx.current_block.terminator == MirTerminator::Unreachable {
                                if last_is_tail && !tail_is_field_assign {
                                    // Return the tail expression value (e.g., StructLiteral)
                                    let val_local = ctx.next_local - 1;
                                    ctx.emit_return(MirValue::Local(val_local));
                                } else {
                                    // Return `this` (the struct constructed via field assignments)
                                    ctx.emit_return(MirValue::Local(this_local));
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

        // Collect closure functions generated during this lower_program call
        {
            let cf = self.closure_functions.borrow();
            for func in cf.iter() {
                if !module.functions.iter().any(|f| f.name == func.name) {
                    module.functions.push(func.clone());
                }
            }
        }

        // Collect async wrapper functions
        {
            let af = self.async_functions.borrow();
            for func in af.iter() {
                if !module.functions.iter().any(|f| f.name == func.name) {
                    module.functions.push(func.clone());
                }
            }
        }

        // Collect specialized monomorphized functions
        {
            let sf = self.specialized_mir_functions.borrow();
            for func in sf.iter() {
                if !module.functions.iter().any(|f| f.name == func.name) {
                    module.functions.push(func.clone());
                }
            }
        }

        // Generate implicit main() if no explicit main function exists
        if !module.functions.iter().any(|f| f.name == "main" || f.name == "kyle_main") {
            let mut main_stmts: Vec<Stmt> = Vec::new();
            let pg_span = program.span.clone();
            for decl in &program.declarations {
                match decl {
                    Decl::Variable(v) => {
                        main_stmts.push(Stmt::Variable(v.clone()));
                    }
                    Decl::Expression(e) => {
                        main_stmts.push(Stmt::Expression(e.clone()));
                    }
                    _ => {}
                }
            }
            main_stmts.push(Stmt::Return(Some(Box::new(
                Expr::Literal { value: Literal::Integer(0), span: pg_span.clone() }
            ))));
            let int_type = AstType::Primitive { name: "i32".into(), span: pg_span.clone() };
            let main_fn = FunctionDecl {
                name: "main".into(),
                type_params: vec![], params: vec![],
                return_type: Some(int_type),
                is_async: false, is_const: false, is_static: false,
                is_abstract: false, is_extern: false, is_test: false,
                visibility: Visibility::Public,
                body: Some(Block { statements: main_stmts, span: pg_span.clone() }),
                span: pg_span,
            };
            if let Some(mir_main) = self.lower_function(&main_fn) {
                module.functions.push(mir_main);
                // Collect closures generated during implicit main lowering
                let cf2 = self.closure_functions.borrow();
                for func in cf2.iter() {
                    if !module.functions.iter().any(|f| f.name == func.name) {
                        module.functions.push(func.clone());
                    }
                }
            }
        }

        module.links = program.links.clone();
        module
    }

    /// Recursively collect a class's fields, prepending fields from any parent
    /// class (and its parent, …) so that subclass instances laid out as a
    /// struct include the full inherited field set.
    fn collect_class_fields(
        c: &ClassDecl,
        program: &Program,
        struct_defs: &std::collections::HashMap<String, Vec<(String, MirType)>>,
    ) -> Vec<(String, MirType)> {
        let mut fields = Vec::new();
        if let Some(ref parent_name) = c.parent {
            if let Some(parent_fields) = struct_defs.get(parent_name).filter(|f| !f.is_empty()) {
                fields.extend(parent_fields.clone());
            } else {
                for decl in &program.declarations {
                    if let Decl::Class(pc) = decl {
                        if &pc.name == parent_name {
                            fields.extend(Self::collect_class_fields(pc, program, struct_defs));
                            break;
                        }
                    }
                }
            }
        }
        for m in &c.members {
            if let ClassMember::Field(f) = m {
                fields.push((f.name.clone(), ast_type_to_mir(&f.type_, Some(struct_defs))));
            }
        }
        fields
    }

    /// Walk a class's inheritance chain to resolve a method call.  Returns the
    /// mangled `Class::method` name of the most-derived class that declares the
    /// method (or None if no ancestor declares it).  This is what gives us
    /// polymorphism: subclasses override inherited methods by simply shadowing
    /// the entry in `method_table`.
    fn lookup_method_in_chain(
        &self,
        class_name: &str,
        method_name: &str,
        method_table: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
        parent_map: &std::collections::HashMap<String, Option<String>>,
    ) -> Option<String> {
        let mut current = class_name.to_string();
        loop {
            if let Some(methods) = method_table.get(&current) {
                if let Some(mangled) = methods.get(method_name) {
                    return Some(mangled.clone());
                }
            }
            match parent_map.get(&current) {
                Some(Some(parent)) => current = parent.clone(),
                _ => break,
            }
        }
        None
    }

    /// Lower an `async fn` declaration. Generates two MIR functions:
    ///   1. `_async_body_{name}` — the actual body (returns i64)
    ///   2. `{name}` — the wrapper that spawns and returns the task handle
    fn lower_async_fn(&self, f: &FunctionDecl, body: &Block) -> Option<MirFunction> {
        let struct_defs = self.struct_defs.borrow().clone();
        let body_fn_name = format!("_async_body_{}", f.name);
        let real_params: Vec<&Parameter> = f.params.iter()
            .filter(|p| !(p.name == "this" || p.name == "self"))
            .collect();
        let param_count = real_params.len();

        // === 1. Generate the async body function ===
        let mut body_func = MirFunction::new(&body_fn_name);
        body_func.params = vec![MirType::I64]; // receives args_ptr as i64
        body_func.return_type = MirType::I64;

        let mut cctx = LowerCtx::new();
        cctx.struct_defs = struct_defs.clone();

        if param_count == 0 {
            // No params — ignore the arg
        } else if param_count == 1 {
            // Single param: pass value directly as i64
            let ptype = ast_type_to_mir(&real_params[0].type_, Some(&struct_defs));
            let raw_val = cctx.alloc_local("_arg0", ptype.clone());
            cctx.current_block.insts.push(MirInst::Store {
                dest: raw_val,
                value: MirValue::Param(0),
            });
            if ptype != MirType::I64 {
                let cast_local = cctx.alloc_local(&real_params[0].name, ptype.clone());
                cctx.current_block.insts.push(MirInst::Cast {
                    dest: cast_local,
                    value: MirValue::Local(raw_val),
                    to_type: ptype,
                });
                cctx.locals.insert(real_params[0].name.clone(), cast_local);
            } else {
                cctx.locals.insert(real_params[0].name.clone(), raw_val);
            }
        }

        // Lower the body statements
        let last_is_tail = matches!(body.statements.last(), Some(Stmt::Expression(_)));
        let stmt_count = body.statements.len();
        for (i, stmt) in body.statements.iter().enumerate() {
            if i + 1 == stmt_count {
                if let Stmt::If(_) = stmt {
                    cctx.tail_if_as_return = true;
                }
            }
            cctx = self.lower_stmt(cctx, stmt);
        }

        // Emit return — widen tail value to i64
        if cctx.current_block.terminator == MirTerminator::Unreachable {
            let tail_local = if last_is_tail {
                cctx.next_local.checked_sub(1)
            } else { None };
            if let Some(val_local) = tail_local {
                let val_type = cctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I32);
                if val_type != MirType::I64 {
                    let widened = cctx.alloc_local("_aw", MirType::I64);
                    cctx.current_block.insts.push(MirInst::Cast {
                        dest: widened,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });
                    cctx.emit_return(MirValue::Local(widened));
                } else {
                    cctx.emit_return(MirValue::Local(val_local));
                }
            } else {
                cctx.emit_return(MirValue::Constant(MirConstant::I64(0)));
            }
        }
        body_func.local_count = cctx.next_local;
        body_func.basic_blocks = cctx.blocks;
        self.async_functions.borrow_mut().push(body_func);

        // === 2. Generate the wrapper function (returns i64 task handle) ===
        let mut wrapper = MirFunction::new(&f.name);
        wrapper.params = f.params.iter().map(|p| {
            ast_type_to_mir(&p.type_, Some(&struct_defs))
        }).collect();
        wrapper.return_type = MirType::I64;

        let mut ctx = LowerCtx::new();
        ctx.struct_defs = struct_defs;

        // Bind wrapper params to locals (same as regular function lowering)
        for (i, param) in f.params.iter().enumerate() {
            let ptype = ast_type_to_mir(&param.type_, Some(&ctx.struct_defs));
            let local = ctx.alloc_local(&param.name, ptype);
            ctx.current_block.insts.push(MirInst::Store {
                dest: local,
                value: MirValue::Param(i),
            });
            ctx.locals.insert(param.name.clone(), local);
        }

        let dest = ctx.alloc_local("_async_h", MirType::I64);
        if param_count <= 1 {
            // Single param or no param: pass value directly (or 0)
            let arg_val = if param_count == 1 {
                let pname = &real_params[0].name;
                if let Some(&local) = ctx.locals.get(pname) {
                    let ptype = ast_type_to_mir(&real_params[0].type_, Some(&ctx.struct_defs));
                    if ptype != MirType::I64 {
                        let widened = ctx.alloc_local("_pw", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: widened,
                            value: MirValue::Local(local),
                            to_type: MirType::I64,
                        });
                        MirValue::Local(widened)
                    } else {
                        MirValue::Local(local)
                    }
                } else {
                    MirValue::Constant(MirConstant::I64(0))
                }
            } else {
                MirValue::Constant(MirConstant::I64(0))
            };
            ctx.current_block.insts.push(MirInst::AsyncSpawn {
                dest,
                function_name: body_fn_name,
                arg: arg_val,
            });
        } else {
            // Multiple params: pack into heap-allocated array
            // which is TODO for multi-param case
            // For now just pass 0 (params will be lost)
            ctx.current_block.insts.push(MirInst::AsyncSpawn {
                dest,
                function_name: body_fn_name,
                arg: MirValue::Constant(MirConstant::I64(0)),
            });
        }
        ctx.emit_return(MirValue::Local(dest));
        wrapper.local_count = ctx.next_local;
        wrapper.basic_blocks = ctx.blocks;

        Some(wrapper)
    }

    fn lower_function(&self, f: &FunctionDecl) -> Option<MirFunction> {
        let body = f.body.as_ref()?;
        // Handle async fn: generate body + wrapper that spawns on thread pool
        if f.is_async {
            return self.lower_async_fn(f, body);
        }
        // Pre-register Option types in struct_defs for all params
        {
            let mut struct_defs = self.struct_defs.borrow_mut();
            for p in &f.params {
                register_option_type(&p.type_, &mut struct_defs);
            }
            if let Some(rt) = &f.return_type {
                register_option_type(rt, &mut struct_defs);
            }
        }
        let struct_defs = self.struct_defs.borrow().clone();
        let mut mir_func = MirFunction::new(&f.name);
        mir_func.params = f.params.iter()
            .map(|p| if p.variadic {
                MirType::List(Box::new(ast_type_to_mir(&p.type_, Some(&struct_defs))))
            } else {
                ast_type_to_mir(&p.type_, Some(&struct_defs))
            })
            .collect();
        mir_func.param_modes = f.params.iter().map(|p| p.mode).collect();
        mir_func.return_type = f.return_type.as_ref()
            .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
            .unwrap_or(MirType::Void);
        let is_fallible = f.return_type.as_ref().map_or(false, |rt| {
            matches!(rt, AstType::Error { .. })
                || matches!(rt, AstType::Generic { name, .. } if name == "Result")
        });
        mir_func.is_fallible = is_fallible;
        mir_func.is_const = f.is_const;

        let mut ctx = LowerCtx::new();
        ctx.struct_defs = self.struct_defs.borrow().clone();
        ctx.is_fallible = is_fallible;
        ctx.return_type = mir_func.return_type.clone();

        // Allocate and store params
        for (i, param) in f.params.iter().enumerate() {
            let ptype = if param.variadic {
                MirType::List(Box::new(ast_type_to_mir(&param.type_, Some(&ctx.struct_defs))))
            } else {
                ast_type_to_mir(&param.type_, Some(&ctx.struct_defs))
            };
            let local = ctx.alloc_local(&param.name, ptype);
            ctx.current_block.insts.push(MirInst::Store {
                dest: local,
                value: MirValue::Param(i),
            });
            ctx.locals.insert(param.name.clone(), local);
        }

        // Lower body statements
        let last_is_tail = matches!(body.statements.last(), Some(Stmt::Expression(_)));
        let last_is_if_tail = matches!(body.statements.last(), Some(Stmt::If(_)));
        let last_is_match_tail = matches!(body.statements.last(), Some(Stmt::Match(_)));
        let stmt_count = body.statements.len();
        if last_is_match_tail {
            if f.return_type.is_some() {
                let result_local = ctx.alloc_local("_match_res", MirType::I64);
                ctx.match_result_local = Some(result_local);
            }
        }
        for (i, stmt) in body.statements.iter().enumerate() {
            if i + 1 == stmt_count && last_is_if_tail {
                ctx.tail_if_as_return = true;
            }
            ctx = self.lower_stmt(ctx, stmt);
        }

        // Default terminator: return void or tail expression value
        if ctx.current_block.terminator == MirTerminator::Unreachable {
            // Save tail value BEFORE deferred expressions (deferred may add more locals)
            let tail_local = if last_is_tail {
                // The tail expression's result is the last local before deferred
                let t = ctx.next_local.checked_sub(1);
                if let Some(l) = t { Some(l) } else { None }
            } else { None };
            // Emit deferred calls in reverse LIFO order before implicit return
            let deferred = std::mem::take(&mut ctx.deferred_exprs);
            for expr in deferred.iter().rev() {
                ctx = self.lower_expr(ctx, expr);
            }
            if let Some(val_local) = tail_local {
                // If the tail expression is a void call (e.g., print()), return Void
                let is_void = ctx.local_types.get(&val_local).map_or(false, |t| *t == MirType::Void);
                if is_void {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                } else {
                    ctx.emit_return(MirValue::Local(val_local));
                    // Infer return type from the tail expression when no explicit type given
                    if mir_func.return_type == MirType::Void {
                        if let Some(actual_type) = ctx.local_types.get(&val_local) {
                            mir_func.return_type = actual_type.clone();
                        }
                    }
                }
            } else if last_is_match_tail {
                if let Some(result_local) = ctx.match_result_local {
                    let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                    let load = ctx.alloc_local("_match_res_val", result_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: load,
                        src: result_local,
                    });
                    ctx.emit_return(MirValue::Local(load));
                } else {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                }
            } else if !last_is_if_tail {
                ctx.emit_return(MirValue::Constant(MirConstant::Void));
            }
        }

        mir_func.basic_blocks = ctx.blocks;
        mir_func.local_count = ctx.next_local;
        Some(mir_func)
    }

    /// Ensure all methods of a concrete generic class are monomorphized and registered.
    /// `concrete_name` is like "Box__i32" — extract base, look up generic templates,
    /// monomorphize each method, and register in method_table.
    fn ensure_generic_class_methods(&self, concrete_name: &str) {
        // Extract base name: "Box__i32" → "Box"
        let base_name = match concrete_name.split("__").next() {
            Some(n) if n != concrete_name => n.to_string(),
            _ => return,
        };
        // Check if already monomorphized (method_table already has concrete_name)
        if self.method_table.borrow().contains_key(concrete_name) {
            return;
        }
        // Look up class methods from generic_class_methods
        let class_methods;
        let generic_templates;
        let struct_tpl;
        {
            let cm = self.generic_class_methods.borrow();
            class_methods = cm.get(&base_name).cloned().unwrap_or_default();
            generic_templates = self.generic_function_templates.borrow();
            let st = self.generic_struct_templates.borrow();
            struct_tpl = st.get(&base_name).cloned();
        }
        if class_methods.is_empty() || struct_tpl.is_none() {
            return;
        }
        let tpl = struct_tpl.unwrap();
        // Reconstruct concrete fields from struct_defs
        let concrete_fields = self.struct_defs.borrow().get(concrete_name).cloned().unwrap_or_default();
        if concrete_fields.is_empty() {
            return;
        }
        // Infer type substitution from field types
        let mut type_subst: std::collections::HashMap<String, MirType> = std::collections::HashMap::new();
        for tf in &tpl.fields {
            if let Some(concrete_field) = concrete_fields.iter().find(|(n, _)| *n == tf.name) {
                // Check if template field type matches a type param name
                if let AstType::User { name, .. } = &tf.type_ {
                    if tpl.type_params.iter().any(|tp| tp.name == *name) {
                        type_subst.entry(name.clone()).or_insert_with(|| concrete_field.1.clone());
                    }
                }
            }
        }
        // For each method, monomorphize and register
        for method in &class_methods {
            let template_name = format!("{}::{}", base_name, method.name);
            let template = match generic_templates.get(&template_name) {
                Some(t) => t,
                None => continue,
            };
            let specialized_name = format!("{}::{}", concrete_name, method.name);
            let mut full_type_subst = type_subst.clone();
            full_type_subst.insert(base_name.clone(), MirType::Struct(concrete_name.to_string(), concrete_fields.clone()));
            let mut specialized_decl = clone_and_specialize_function(template, &full_type_subst);
            specialized_decl.name = method.name.clone(); // lower_method prepends class_name::
            // Pre-register generic struct types in signature
            {
                let generic_struct_tpls = self.generic_struct_templates.borrow();
                let mut struct_defs = self.struct_defs.borrow_mut();
                if let Some(rt) = &template.return_type {
                    pre_register_generic_type(rt, &type_subst, &generic_struct_tpls, &mut struct_defs);
                }
                for p in &template.params {
                    pre_register_generic_type(&p.type_, &type_subst, &generic_struct_tpls, &mut struct_defs);
                }
            }
            // Compute and register return type
            let struct_defs = self.struct_defs.borrow();
            let ret_type = template.return_type.as_ref()
                .map(|rt| ast_type_to_mir_with_subst(rt, Some(&struct_defs), &type_subst))
                .unwrap_or(MirType::Void);
            drop(struct_defs);
            self.fn_returns.borrow_mut().insert(specialized_name.clone(), ret_type);
            // Lower the specialized method
            if let Some(mir_func) = self.lower_method(&specialized_decl, concrete_name) {
                self.specialized_mir_functions.borrow_mut().push(mir_func);
            }
            // Register in method_table
            self.method_table.borrow_mut()
                .entry(concrete_name.to_string())
                .or_default()
                .insert(method.name.clone(), specialized_name);
        }
    }

    /// Lower a class method. Like `lower_function`, but the method's MIR
    /// signature prepends an implicit `this` parameter of type
    /// `MirType::Struct(class_name, fields)` so the body can reference `this`
    /// and the method can be called as `ClassName::method(obj, args...)`.
    fn lower_method(&self, m: &FunctionDecl, class_name: &str) -> Option<MirFunction> {
        let body = m.body.as_ref()?;
        let struct_defs = self.struct_defs.borrow().clone();
        let mut mir_func = MirFunction::new(&format!("{}::{}", class_name, m.name));
        let is_static = m.is_static;
        let this_type = struct_defs.get(class_name)
            .map(|fields| MirType::Struct(class_name.to_string(), fields.clone()))
            .unwrap_or(MirType::Struct(class_name.to_string(), vec![]));
        // Static methods don't get a `this` parameter; instance methods do.
        let mut ctx = LowerCtx::new();
        ctx.struct_defs = struct_defs.clone();
        if is_static {
            // Static method: no `this`, just explicit params
            let mut params: Vec<MirType> = Vec::new();
            for p in &m.params {
                params.push(ast_type_to_mir(&p.type_, Some(&struct_defs)));
            }
            mir_func.params = params;
            let param_modes: Vec<ParamMode> = m.params.iter().map(|p| p.mode).collect();
            mir_func.param_modes = param_modes;
        } else {
            let mut params: Vec<MirType> = vec![this_type.clone()];
            for (i, p) in m.params.iter().enumerate() {
                if i == 0 && (p.name == "this" || p.name == "self") {
                    continue;
                }
                params.push(ast_type_to_mir(&p.type_, Some(&struct_defs)));
            }
            mir_func.params = params;
            let mut param_modes = vec![ParamMode::MutableBorrow];
            param_modes.extend(m.params.iter().enumerate().filter(|(i, p)| !(*i == 0 && (p.name == "this" || p.name == "self"))).map(|(_, p)| p.mode));
            mir_func.param_modes = param_modes;
        }
        mir_func.return_type = m.return_type.as_ref()
            .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
            .unwrap_or(MirType::Void);
        let is_fallible = m.return_type.as_ref().map_or(false, |rt| {
            matches!(rt, AstType::Error { .. })
                || matches!(rt, AstType::Generic { name, .. } if name == "Result")
        });
        mir_func.is_fallible = is_fallible;

        ctx.struct_defs = struct_defs;
        ctx.is_fallible = is_fallible;
        ctx.return_type = mir_func.return_type.clone();

        if is_static {
            // Static method: bind explicit params directly (no `this`)
            for (i, param) in m.params.iter().enumerate() {
                let local = ctx.alloc_local(&param.name, ast_type_to_mir(&param.type_, Some(&ctx.struct_defs)));
                ctx.current_block.insts.push(MirInst::Store {
                    dest: local,
                    value: MirValue::Param(i),
                });
                ctx.locals.insert(param.name.clone(), local);
            }
        } else {
            // Bind `this` (param 0) into a local so the body's `Expr::PropertyAccess`
            // on `this` resolves to a struct field.
            let this_local = ctx.alloc_local("this", this_type);
            ctx.current_block.insts.push(MirInst::Store {
                dest: this_local,
                value: MirValue::Param(0),
            });
            ctx.locals.insert("this".to_string(), this_local);
            ctx.locals.insert("self".to_string(), this_local);

            // Bind the explicit params (offset by 1 because of implicit `this`).
            let mut param_offset = 1usize;
            for (i, param) in m.params.iter().enumerate() {
                if i == 0 && (param.name == "this" || param.name == "self") {
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
        }

        // Lower body statements
        let last_is_tail = matches!(body.statements.last(), Some(Stmt::Expression(_)));
        let last_is_if_tail = matches!(body.statements.last(), Some(Stmt::If(_)));
        let last_is_match_tail = matches!(body.statements.last(), Some(Stmt::Match(_)));
        let stmt_count = body.statements.len();
        if last_is_match_tail {
            if m.return_type.is_some() {
                let result_local = ctx.alloc_local("_match_res", MirType::I64);
                ctx.match_result_local = Some(result_local);
            }
        }
        for (i, stmt) in body.statements.iter().enumerate() {
            if i + 1 == stmt_count && last_is_if_tail {
                ctx.tail_if_as_return = true;
            }
            ctx = self.lower_stmt(ctx, stmt);
        }

        if ctx.current_block.terminator == MirTerminator::Unreachable {
            if last_is_tail {
                let val_local = ctx.next_local - 1;
                ctx.emit_return(MirValue::Local(val_local));
                // Infer return type from tail expression when no explicit type
                if mir_func.return_type == MirType::Void {
                    if let Some(actual_type) = ctx.local_types.get(&val_local) {
                        mir_func.return_type = actual_type.clone();
                    }
                }
            } else if last_is_match_tail {
                if let Some(result_local) = ctx.match_result_local {
                    let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                    let load = ctx.alloc_local("_match_res_val", result_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: load,
                        src: result_local,
                    });
                    ctx.emit_return(MirValue::Local(load));
                } else {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                }
            } else if !last_is_if_tail {
                ctx.emit_return(MirValue::Constant(MirConstant::Void));
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
                // Pre-register Option struct fields
                if let Some(ref type_) = v.type_ {
                    register_option_type(type_, &mut ctx.struct_defs);
                }
                let has_init = !matches!(v.value.as_ref(), Expr::Literal { value: Literal::None, .. });
                let mut is_list = false;
                let mut is_set = false;
                if !has_init {
                    if let Some(AstType::Generic { name, .. }) = &v.type_ {
                        if name == "list" {
                            is_list = true;
                        } else if name == "set" {
                            is_set = true;
                        }
                    }
                }
                let val_local = if has_init {
                    ctx = self.lower_expr(ctx, &v.value);
                    Some(ctx.next_local - 1)
                } else if is_set {
                    let set_ptr = ctx.alloc_local("_setv", ast_type_to_mir(v.type_.as_ref().unwrap(), Some(&ctx.struct_defs)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(set_ptr),
                        name: "ky_set_new".to_string(),
                        args: vec![],
                    });
                    Some(set_ptr)
                } else if is_list {
                    let list_ptr = ctx.alloc_local("_listv", ast_type_to_mir(v.type_.as_ref().unwrap(), Some(&ctx.struct_defs)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(list_ptr),
                        name: "ky_list_new".to_string(),
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
                                if matches!(t, MirType::List(_) | MirType::Struct(_, _) | MirType::Array(_, _) | MirType::Dict(_, _) | MirType::Ptr(_)) {
                                    t
                                } else if matches!(t, MirType::Str | MirType::I64 | MirType::F64 | MirType::Char) {
                                    t
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
                let is_option_none = val_local.is_none()
                    && matches!(&var_type, MirType::Struct(n, _) if n.starts_with("Option__"));
                let local = ctx.alloc_local(&v.name, var_type.clone());
                if let Some(vl) = val_local {
                    let val_type = ctx.local_types.get(&vl).cloned().unwrap_or(MirType::I32);
                    let store_val = if val_type != var_type {
                        let cast = ctx.alloc_local("_tcast", var_type.clone());
                        ctx.current_block.insts.push(MirInst::Cast { dest: cast, value: MirValue::Local(vl), to_type: var_type.clone() });
                        MirValue::Local(cast)
                    } else {
                        MirValue::Local(vl)
                    };
                    // Skip store for empty array literal to avoid type mismatch in LLVM
                    let is_empty_array = matches!(v.value.as_ref(), Expr::Array { elements, .. } if elements.is_empty());
                    if !is_empty_array || !matches!(val_type, MirType::Array(_, _)) {
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: local,
                            value: store_val,
                        });
                    }
                }
                if is_option_none {
                    // Auto-initialize Option<T> with None (disc = 0)
                    let disc_ptr = ctx.alloc_local("_disc", MirType::I32);
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: disc_ptr,
                        ptr: local,
                        field_index: 0,
                        struct_type: Box::new(ctx.local_types[&local].clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: disc_ptr,
                        value: MirValue::Constant(MirConstant::I32(0)),
                    });
                }
                ctx.locals.insert(v.name.clone(), local);
                ctx
            }
            Stmt::Return(ret_val) => {
                // Emit deferred calls in reverse LIFO order before returning
                let deferred = std::mem::take(&mut ctx.deferred_exprs);
                for expr in deferred.iter().rev() {
                    ctx = self.lower_expr(ctx, expr);
                }
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
                    val_ctx.emit_return(val);
                    val_ctx
                } else {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                    ctx
                }
            }
            Stmt::If(s) => {
                let is_tail = ctx.tail_if_as_return;
                ctx.tail_if_as_return = false;
                ctx.last_stmt_was_if = false;
                let else_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                let then_label = ctx.fresh_block();
                // When tail, allocate shared result alloca so branches with nested
                // if/else store to the same slot and the merge block returns it.
                let result_alloca = if is_tail {
                    let ra = ctx.alloc_local("_if_res", MirType::I64);
                    ctx.if_result_alloca = Some(ra);
                    Some(ra)
                } else {
                    None
                };
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
                    if let Some(ra) = result_alloca {
                        if !ctx.last_stmt_was_if {
                            let last = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                            ctx.local_types.insert(ra, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: ra,
                                value: MirValue::Local(last),
                            });
                        }
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    } else {
                        ctx.emit_return(MirValue::Local(ctx.next_local - 1));
                    }
                } else if let Some(ra) = ctx.if_result_alloca {
                    // Parent if is a tail expression: store last expr to parent's result alloca
                    let last = ctx.next_local - 1;
                    let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                    ctx.local_types.insert(ra, last_type);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: ra,
                        value: MirValue::Local(last),
                    });
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
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
                        if let Some(ra) = result_alloca {
                            if !ctx.last_stmt_was_if {
                                let last = ctx.next_local - 1;
                                let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                                ctx.local_types.insert(ra, last_type);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: ra,
                                    value: MirValue::Local(last),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                        } else {
                            ctx.emit_return(MirValue::Local(ctx.next_local - 1));
                        }
                    } else if let Some(ra) = ctx.if_result_alloca {
                        let last = ctx.next_local - 1;
                        let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                        ctx.local_types.insert(ra, last_type);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: ra,
                            value: MirValue::Local(last),
                        });
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
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
                        if let Some(ra) = result_alloca {
                            if !ctx.last_stmt_was_if {
                                let last = ctx.next_local - 1;
                                let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                                ctx.local_types.insert(ra, last_type);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: ra,
                                    value: MirValue::Local(last),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                        } else {
                            ctx.emit_return(MirValue::Local(ctx.next_local - 1));
                        }
                    } else if let Some(ra) = ctx.if_result_alloca {
                        let last = ctx.next_local - 1;
                        let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                        ctx.local_types.insert(ra, last_type);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: ra,
                            value: MirValue::Local(last),
                        });
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                } else if !s.elif_branches.is_empty() {
                    ctx.current_block = MirBasicBlock::new(else_label);
                    if is_tail {
                        ctx.emit_return(MirValue::Constant(MirConstant::Void));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                }

                if is_tail {
                    ctx.if_result_alloca = None;
                    // Merge block: return result_alloca value (or void if no else)
                    ctx.current_block = MirBasicBlock::new(end_label);
                    if let Some(ra) = result_alloca {
                        let result_type = ctx.local_types.get(&ra).cloned().unwrap_or(MirType::I64);
                        let load = ctx.alloc_local("_if_res_val", result_type);
                        ctx.current_block.insts.push(MirInst::Load {
                            dest: load,
                            src: ra,
                        });
                        ctx.emit_return(MirValue::Local(load));
                    } else {
                        // Only reachable if is_tail && no allocation — impossible
                        ctx.emit_return(MirValue::Constant(MirConstant::Void));
                    }
                } else {
                    ctx.current_block = MirBasicBlock::new(end_label);
                }
                ctx.last_stmt_was_if = true;
                ctx
            }
            Stmt::While(s) => {
                let has_else = s.else_branch.is_some();
                let cond_label = ctx.fresh_block();
                let body_label = ctx.fresh_block();
                let exit_label = ctx.fresh_block();
                let else_label = ctx.fresh_block();
                let skip_else_label = ctx.fresh_block();
                let merge_label = ctx.fresh_block();

                let flag_local = if has_else {
                    let f = ctx.alloc_local("_loop_break", MirType::Bool);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: f,
                        value: MirValue::Constant(MirConstant::Bool(false)),
                    });
                    Some(f)
                } else {
                    None
                };

                ctx.finish_block(MirTerminator::Br(cond_label.clone()));
                ctx.current_block = MirBasicBlock::new(cond_label.clone());
                let cond_ctx = self.lower_expr(ctx, &s.condition);
                ctx = cond_ctx;
                let cond_val = MirValue::Local(ctx.next_local - 1);

                let false_target = if has_else { else_label.clone() } else { exit_label.clone() };
                ctx.finish_block(MirTerminator::CondBr {
                    cond: cond_val,
                    true_block: body_label.clone(),
                    false_block: false_target,
                });

                // Body block
                ctx.current_block = MirBasicBlock::new(body_label);
                let break_target = if has_else { else_label.clone() } else { exit_label.clone() };
                ctx.break_targets.push(break_target);
                ctx.continue_targets.push(cond_label.clone());
                if has_else { ctx.break_flag_local = flag_local; }
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.break_flag_local = None;
                ctx.break_targets.pop();
                ctx.continue_targets.pop();
                ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                if has_else {
                    // else_label: check flag → execute else or skip
                    ctx.current_block = MirBasicBlock::new(else_label);
                    let f_load = ctx.alloc_local("_flag_v", MirType::Bool);
                    ctx.current_block.insts.push(MirInst::Load { dest: f_load, src: flag_local.unwrap() });
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: MirValue::Local(f_load),
                        true_block: skip_else_label.clone(),
                        false_block: exit_label.clone(),
                    });

                    // exit_label: else body
                    ctx.current_block = MirBasicBlock::new(exit_label);
                    if let Some(eb) = &s.else_branch {
                        for stmt in &eb.statements {
                            ctx = self.lower_stmt(ctx, stmt);
                        }
                    }
                    ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                    // skip_else_label
                    ctx.current_block = MirBasicBlock::new(skip_else_label);
                    ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                    ctx.current_block = MirBasicBlock::new(merge_label);
                } else {
                    ctx.current_block = MirBasicBlock::new(exit_label);
                }
                ctx
            }
            Stmt::For(s) => {
                // Check if iterable is a range expression (for i in 0..10)
                let range_op = if let Expr::Binary { operator: op @ (BinaryOp::Range | BinaryOp::RangeInclusive | BinaryOp::RangeExclusive), .. } = &*s.iterable {
                    Some(*op)
                } else { None };
                if let Some(range_op) = range_op {
                    let (left, right) = if let Expr::Binary { left, right, .. } = &*s.iterable {
                        (left, right)
                    } else { unreachable!() };
                    // === RANGE-BASED FOR LOOP ===
                    let cond_label = ctx.fresh_block();
                    let body_label = ctx.fresh_block();
                    let inc_label = ctx.fresh_block();
                    let check_label = ctx.fresh_block();
                    let else_label = ctx.fresh_block();
                    let skip_else_label = ctx.fresh_block();
                    let merge_label = ctx.fresh_block();

                    let has_else = s.else_branch.is_some();

                    let flag_local = if has_else {
                        let f = ctx.alloc_local("_loop_break", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: f,
                            value: MirValue::Constant(MirConstant::Bool(false)),
                        });
                        Some(f)
                    } else {
                        None
                    };

                    // 1. Lower start and end expressions
                    ctx = self.lower_expr(ctx, left);
                    let start_val = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, right);
                    let end_val = ctx.next_local - 1;

                    // Allocate loop variable as I32
                    let var_local = ctx.alloc_local(&s.variable, MirType::I32);
                    ctx.locals.insert(s.variable.clone(), var_local);

                    // Store start value in loop variable
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: var_local,
                        value: MirValue::Local(start_val),
                    });

                    // Store end value in alloca for cross-block access
                    let end_alloca = ctx.alloc_local("_for_end", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: end_alloca,
                        value: MirValue::Local(end_val),
                    });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    // 2. Cond block: load i, compare i < end
                    ctx.current_block = MirBasicBlock::new(cond_label.clone());

                    let i_val = ctx.alloc_local("_for_i", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: i_val, src: var_local });
                    let end_loaded = ctx.alloc_local("_for_el", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: end_loaded, src: end_alloca });

                    let cmp = ctx.alloc_local("_for_cmp", MirType::Bool);
                    let cmp_op = match range_op {
                        BinaryOp::RangeInclusive => MirBinaryOp::Le,
                        _ => MirBinaryOp::Lt,
                    };
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: cmp, op: cmp_op,
                        left: MirValue::Local(i_val), right: MirValue::Local(end_loaded),
                    });

                    let false_target = if has_else { else_label.clone() } else { check_label.clone() };
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: MirValue::Local(cmp),
                        true_block: body_label.clone(),
                        false_block: false_target.clone(),
                    });

                    // 3. Body block
                    ctx.current_block = MirBasicBlock::new(body_label.clone());
                    ctx.break_targets.push(false_target);
                    ctx.continue_targets.push(inc_label.clone());
                    if has_else { ctx.break_flag_local = flag_local; }

                    // Lower body statements
                    for stmt in &s.body.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    ctx.break_flag_local = None;
                    ctx.break_targets.pop();
                    ctx.continue_targets.pop();

                    // Fall through to inc block
                    ctx.finish_block(MirTerminator::Br(inc_label.clone()));

                    // 4. Increment block: i += 1, then goto cond
                    ctx.current_block = MirBasicBlock::new(inc_label.clone());
                    let iv2 = ctx.alloc_local("_for_iv2", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: iv2, src: var_local });
                    let iv3 = ctx.alloc_local("_for_iv3", MirType::I32);
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: iv3, op: MirBinaryOp::Add,
                        left: MirValue::Local(iv2), right: MirValue::Constant(MirConstant::I32(1)),
                    });
                    ctx.current_block.insts.push(MirInst::Store { dest: var_local, value: MirValue::Local(iv3) });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    if has_else {
                        // 5a. Else-label: check flag → execute else or skip
                        ctx.current_block = MirBasicBlock::new(else_label.clone());
                        let f_load = ctx.alloc_local("_flag_v", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Load { dest: f_load, src: flag_local.unwrap() });
                        ctx.finish_block(MirTerminator::CondBr {
                            cond: MirValue::Local(f_load),
                            true_block: skip_else_label.clone(),
                            false_block: check_label.clone(),
                        });

                        // 5b. Check-label: execute else body
                        ctx.current_block = MirBasicBlock::new(check_label.clone());
                        if let Some(eb) = &s.else_branch {
                            for stmt in &eb.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                        }
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        // 5c. Skip else
                        ctx.current_block = MirBasicBlock::new(skip_else_label);
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        ctx.current_block = MirBasicBlock::new(merge_label);
                    } else {
                        // 5. End block
                        ctx.current_block = MirBasicBlock::new(check_label);
                    }
                    ctx
                } else {
                    // === LIST-BASED FOR LOOP ===
                    let cond_label = ctx.fresh_block();
                    let body_label = ctx.fresh_block();
                    let inc_label = ctx.fresh_block();
                    let check_label = ctx.fresh_block();
                    let else_label = ctx.fresh_block();
                    let skip_else_label = ctx.fresh_block();
                    let merge_label = ctx.fresh_block();

                    let has_else = s.else_branch.is_some();

                    let flag_local = if has_else {
                        let f = ctx.alloc_local("_loop_break", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: f,
                            value: MirValue::Constant(MirConstant::Bool(false)),
                        });
                        Some(f)
                    } else {
                        None
                    };

                    // 1. Lower the iterable expression
                    ctx = self.lower_expr(ctx, &s.iterable);
                    let iter_val = ctx.next_local - 1;

                    // Determine element type from the list type
                    let elem_type = match ctx.local_types.get(&iter_val) {
                        Some(MirType::List(inner)) => inner.as_ref().clone(),
                        _ => MirType::I64,
                    };

                    // Allocate loop variable with proper type
                    let var_local = ctx.alloc_local(&s.variable, elem_type.clone());
                    ctx.locals.insert(s.variable.clone(), var_local);
                    if elem_type == MirType::Str {
                        ctx.string_locals.push(var_local);
                    }

                    // Store list pointer in alloca for cross-block access
                    let list_alloca = ctx.alloc_local("_for_list", MirType::List(Box::new(elem_type.clone())));
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: list_alloca,
                        value: MirValue::Local(iter_val),
                    });

                    // Allocate and init index to 0
                    let idx_local = ctx.alloc_local("_for_idx", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: idx_local,
                        value: MirValue::Constant(MirConstant::I32(0)),
                    });

                    // Load list ptr and call kl_list_len
                    let list_tmp = ctx.alloc_local("_for_lt", MirType::List(Box::new(elem_type.clone())));
                    ctx.current_block.insts.push(MirInst::Load { dest: list_tmp, src: list_alloca });
                    let len_local = ctx.alloc_local("_for_len", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(len_local),
                        name: "ky_list_len".to_string(),
                        args: vec![MirValue::Local(list_tmp)],
                    });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    // 2. Cond block: load index, compare with len
                    ctx.current_block = MirBasicBlock::new(cond_label.clone());

                    // Load index
                    let idx = ctx.alloc_local("_for_i", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: idx, src: idx_local });

                    // Load and cast len i64→i32
                    let len_val = ctx.alloc_local("_for_lv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Load { dest: len_val, src: len_local });
                    let len32 = ctx.alloc_local("_for_len32", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: len32, value: MirValue::Local(len_val), to_type: MirType::I32,
                    });

                    // Compare idx < len
                    let cmp = ctx.alloc_local("_for_cmp", MirType::Bool);
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: cmp, op: MirBinaryOp::Lt,
                        left: MirValue::Local(idx), right: MirValue::Local(len32),
                    });

                    let false_target = if has_else { else_label.clone() } else { check_label.clone() };
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: MirValue::Local(cmp),
                        true_block: body_label.clone(),
                        false_block: false_target.clone(),
                    });

                    // 3. Body block
                    ctx.current_block = MirBasicBlock::new(body_label.clone());
                    ctx.break_targets.push(false_target);
                    ctx.continue_targets.push(inc_label.clone());
                    if has_else { ctx.break_flag_local = flag_local; }

                    // Load current index
                    let idx2 = ctx.alloc_local("_for_i2", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: idx2, src: idx_local });

                    // Cast index to i64 for kl_list_get
                    let idx2_64 = ctx.alloc_local("_for_i64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: idx2_64, value: MirValue::Local(idx2), to_type: MirType::I64,
                    });

                    // Load list pointer
                    let list_tmp2 = ctx.alloc_local("_for_lt2", MirType::List(Box::new(elem_type.clone())));
                    ctx.current_block.insts.push(MirInst::Load { dest: list_tmp2, src: list_alloca });

                    // Call kl_list_get
                    let elem_raw = ctx.alloc_local("_for_elem", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(elem_raw),
                        name: "ky_list_get".to_string(),
                        args: vec![MirValue::Local(list_tmp2), MirValue::Local(idx2_64)],
                    });

                    // Store element into loop variable with correct type
                    if elem_type == MirType::Str {
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: var_local, value: MirValue::Local(elem_raw), to_type: MirType::Str,
                        });
                    } else if elem_type != MirType::I64 {
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: var_local, value: MirValue::Local(elem_raw), to_type: elem_type.clone(),
                        });
                    } else {
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: var_local, value: MirValue::Local(elem_raw),
                        });
                    }

                    // Lower body statements
                    for stmt in &s.body.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    ctx.break_flag_local = None;
                    ctx.break_targets.pop();
                    ctx.continue_targets.pop();

                    ctx.finish_block(MirTerminator::Br(inc_label.clone()));

                    // 4. Increment block: idx += 1, then goto cond
                    ctx.current_block = MirBasicBlock::new(inc_label.clone());
                    let idx3 = ctx.alloc_local("_for_i3", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: idx3, src: idx_local });
                    let idx4 = ctx.alloc_local("_for_i4", MirType::I32);
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: idx4, op: MirBinaryOp::Add,
                        left: MirValue::Local(idx3), right: MirValue::Constant(MirConstant::I32(1)),
                    });
                    ctx.current_block.insts.push(MirInst::Store { dest: idx_local, value: MirValue::Local(idx4) });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    if has_else {
                        // 5a. Else-label: check flag → execute else or skip
                        ctx.current_block = MirBasicBlock::new(else_label.clone());
                        let f_load = ctx.alloc_local("_flag_v", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Load { dest: f_load, src: flag_local.unwrap() });
                        ctx.finish_block(MirTerminator::CondBr {
                            cond: MirValue::Local(f_load),
                            true_block: skip_else_label.clone(),
                            false_block: check_label.clone(),
                        });

                        // 5b. Check-label: execute else body
                        ctx.current_block = MirBasicBlock::new(check_label.clone());
                        if let Some(eb) = &s.else_branch {
                            for stmt in &eb.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                        }
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        // 5c. Skip else
                        ctx.current_block = MirBasicBlock::new(skip_else_label);
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        ctx.current_block = MirBasicBlock::new(merge_label);
                    } else {
                        // 5. End block
                        ctx.current_block = MirBasicBlock::new(check_label);
                    }
                    ctx
                }
            }
            Stmt::Match(s) => {
                let end_label = ctx.fresh_block();
                let needs_val = s.arms.iter().any(|a| {
                    matches!(a.pattern, Pattern::Literal { .. } | Pattern::EnumVariant { .. } | Pattern::Tuple { .. } | Pattern::Range { .. })
                        || matches!(&a.pattern, Pattern::Or { patterns, .. } if patterns.iter().any(|p| matches!(p, Pattern::Literal { .. } | Pattern::EnumVariant { .. } | Pattern::Tuple { .. } | Pattern::Range { .. })))
                });
                if needs_val {
                    ctx = self.lower_expr(ctx, &s.expression);
                }
                let match_val = if needs_val { Some(ctx.next_local - 1) } else { None };

                // Lower each arm with condition checks (forward order).
                // Each literal/variant arm: check cond → arm_body, fallthrough → next check.
                // Wildcard/identifier: always execute (no check).
                let arm_count = s.arms.len();
                for (i, arm) in s.arms.iter().enumerate() {
                    let arm_label = ctx.fresh_block();
                    let is_last = i == arm_count - 1;
                    let next_target = if is_last {
                        end_label.clone()
                    } else {
                        ctx.fresh_block()
                    };
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
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::EnumVariant { enum_name, variant, args, .. } => {
                            // Look up variant index from enum_variants map
                            let ev_map = self.enum_variants.borrow();
                            let variant_idx = ev_map.get(enum_name)
                                .and_then(|m| m.get(variant))
                                .copied()
                                .unwrap_or(0);

                            let struct_type = MirType::Struct(
                                if enum_name == "Result" { "Result".to_string() } else { enum_name.clone() },
                                vec![
                                    ("disc".to_string(), MirType::I32),
                                    ("payload".to_string(), MirType::I64),
                                ],
                            );

                            // FieldPtr needs a pointer, but match_val is a loaded value.
                            // Store it to a temp alloca to get an addressable pointer.
                            let mv = match_val.unwrap();
                            let mv_type = ctx.local_types.get(&mv).cloned().unwrap_or(MirType::I64);
                            let struct_ptr = if matches!(mv_type, MirType::Struct(_, _)) {
                                let alloca = ctx.alloc_local("_mvtmp", mv_type);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: alloca,
                                    value: MirValue::Local(mv),
                                });
                                alloca
                            } else {
                                mv
                            };

                            // Load discriminant from match value
                            let disc_ptr = ctx.alloc_local("_disc_ptr", MirType::Ptr(Box::new(MirType::I32)));
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: disc_ptr,
                                ptr: struct_ptr,
                                field_index: 0,
                                struct_type: Box::new(struct_type.clone()),
                            });
                            let disc_val = ctx.alloc_local("_disc", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: disc_val,
                                src: disc_ptr,
                            });

                            // Compare discriminant with variant index
                            let idx_local = ctx.alloc_local("_vidx", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: idx_local,
                                value: MirValue::Constant(MirConstant::I32(variant_idx as i32)),
                            });
                            let eq = ctx.alloc_local("_eq", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: eq, op: MirBinaryOp::Eq,
                                left: MirValue::Local(disc_val),
                                right: MirValue::Local(idx_local),
                            });

                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }

                            // Bind payload values to pattern variables
                            if !args.is_empty() {
                                let payload_ptr = ctx.alloc_local("_pay_ptr", MirType::I64);
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: payload_ptr,
                                    ptr: struct_ptr,
                                    field_index: 1,
                                    struct_type: Box::new(struct_type),
                                });

                                for (_pi, arg) in args.iter().enumerate() {
                                    match arg {
                                        Pattern::Identifier { name, .. } => {
                                            let pay_val = ctx.alloc_local("_pay", MirType::I64);
                                            ctx.current_block.insts.push(MirInst::Load {
                                                dest: pay_val,
                                                src: payload_ptr,
                                            });
                                            let local = ctx.alloc_local(name, MirType::I64);
                                            ctx.current_block.insts.push(MirInst::Store {
                                                dest: local,
                                                value: MirValue::Local(pay_val),
                                            });
                                            ctx.locals.insert(name.clone(), local);
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));

                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::Range { start, end, inclusive, .. } => {
                            let (st, sv) = literal_to_mir(start);
                            let (et, ev) = literal_to_mir(end);
                            let start_lit = ctx.alloc_local("_rs", st);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: start_lit, value: MirValue::Constant(sv),
                            });
                            let end_lit = ctx.alloc_local("_re", et);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: end_lit, value: MirValue::Constant(ev),
                            });
                            let ge = ctx.alloc_local("_ge", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: ge, op: MirBinaryOp::Ge,
                                left: MirValue::Local(match_val.unwrap()),
                                right: MirValue::Local(start_lit),
                            });
                            let le_op = if *inclusive { MirBinaryOp::Le } else { MirBinaryOp::Lt };
                            let le = ctx.alloc_local("_le", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: le, op: le_op,
                                left: MirValue::Local(match_val.unwrap()),
                                right: MirValue::Local(end_lit),
                            });
                            let cond = ctx.alloc_local("_rng", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: cond, op: MirBinaryOp::And,
                                left: MirValue::Local(ge),
                                right: MirValue::Local(le),
                            });
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(cond),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(cond),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            // Execute arm body (handled below in shared code)
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::Tuple { .. } => {
                            // Tuple: treat as wildcard for now (always matches)
                            ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                            ctx.current_block = MirBasicBlock::new(arm_label.clone());
                        }
                        Pattern::IsType { .. } | Pattern::Wildcard { .. } | Pattern::Identifier { .. } => {
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::Br(guard_label.clone()));
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if arm.guard.is_some() {
                                // Guard may fail; continue to next arm (next_target is the false branch)
                                if !is_last {
                                    ctx.current_block = MirBasicBlock::new(next_target);
                                } else {
                                    ctx.current_block = MirBasicBlock::new(end_label.clone());
                                }
                            } else {
                                // No guard: this arm always matches, skip remaining arms
                                ctx.current_block = MirBasicBlock::new(end_label.clone());
                                return ctx;
                            }
                        }
                        Pattern::Or { .. } => {
                            // Or pattern: always matches (wildcard behavior)
                            ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                            ctx.current_block = MirBasicBlock::new(arm_label.clone());
                            // Create the arm body block (shared for all alternatives)
                            ctx.current_block = MirBasicBlock::new(arm_label.clone());
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
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
                ctx = self.lower_expr(ctx, &g.condition);
                let cond_val = MirValue::Local(ctx.next_local - 1);
                let then_label = ctx.fresh_block();
                let else_label = ctx.fresh_block();
                ctx.finish_block(MirTerminator::CondBr {
                    cond: cond_val,
                    true_block: then_label.clone(),
                    false_block: else_label.clone(),
                });
                // then: continue
                ctx.current_block = MirBasicBlock::new(then_label);
                let after = ctx.fresh_block();
                ctx.finish_block(MirTerminator::Br(after.clone()));
                // else: run body (should contain return/break)
                ctx.current_block = MirBasicBlock::new(else_label);
                for stmt in &g.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.current_block = MirBasicBlock::new(after);
                ctx
            }
            Stmt::Unsafe(u) => {
                for stmt in &u.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx
            }
            Stmt::Defer(d) => {
                ctx.deferred_exprs.push(d.call.clone());
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
            Stmt::Break(_, _) => {
                if let Some(flag) = ctx.break_flag_local {
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: flag,
                        value: MirValue::Constant(MirConstant::Bool(true)),
                    });
                }
                if let Some(target) = ctx.break_targets.last().cloned() {
                    ctx.finish_block(MirTerminator::Br(target));
                } else {
                    ctx.finish_block(MirTerminator::Unreachable);
                }
                ctx
            }
            Stmt::Continue(_) => {
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
                ctx = self.lower_expr(ctx, &v.value);
                let val_local = ctx.next_local - 1;
                let val_type = ctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I32);
                let var_type = v.type_.as_ref()
                    .map(|t| ast_type_to_mir(t, Some(&ctx.struct_defs)))
                    .unwrap_or(val_type.clone());
                let store_val = if val_type != var_type {
                    let cast = ctx.alloc_local("_tcast", var_type.clone());
                    ctx.current_block.insts.push(MirInst::Cast { dest: cast, value: MirValue::Local(val_local), to_type: var_type.clone() });
                    MirValue::Local(cast)
                } else {
                    MirValue::Local(val_local)
                };
                let local = ctx.alloc_local(&v.name, var_type.clone());
                ctx.locals.insert(v.name.clone(), local);
                ctx.current_block.insts.push(MirInst::Store { dest: local, value: store_val });
                ctx
            }
        }
    }

    fn lower_match_guard(&self, mut ctx: LowerCtx, guard: &Expr, true_block: &str, false_block: &str) -> LowerCtx {
        let prev_next = ctx.next_local;
        ctx = self.lower_expr(ctx, guard);
        let guard_val = if ctx.next_local > prev_next {
            ctx.next_local - 1
        } else {
            // No expression result (Void) — treat as false
            return ctx;
        };
        let guard_type = ctx.local_types.get(&guard_val).cloned().unwrap_or(MirType::Bool);
        // If guard is not bool, compare with 0 for truthiness
        let cond = if guard_type == MirType::Bool {
            MirValue::Local(guard_val)
        } else {
            let zero = ctx.alloc_local("_zero", guard_type.clone());
            ctx.current_block.insts.push(MirInst::Store {
                dest: zero,
                value: MirValue::Constant(MirConstant::I32(0)),
            });
            let cmp = ctx.alloc_local("_gcmp", MirType::Bool);
            ctx.current_block.insts.push(MirInst::BinaryOp {
                dest: cmp, op: MirBinaryOp::Neq,
                left: MirValue::Local(guard_val),
                right: MirValue::Local(zero),
            });
            MirValue::Local(cmp)
        };
        ctx.finish_block(MirTerminator::CondBr {
            cond,
            true_block: true_block.to_string(),
            false_block: false_block.to_string(),
        });
        ctx
    }

    fn lower_expr(&self, mut ctx: LowerCtx, expr: &Expr) -> LowerCtx {
        match expr {
            Expr::ArrayRepeat { value, count, .. } => {
                ctx = self.lower_expr(ctx, value);
                let elem_local = ctx.next_local - 1;
                let elem_type = ctx.local_types.get(&elem_local).cloned()
                    .unwrap_or(MirType::I32);
                // Evaluate count expression at compile time
                let size = match count.as_ref() {
                    Expr::Literal { value: Literal::Integer(n), .. } => *n as usize,
                    Expr::Identifier { name, .. } => {
                        // Look up constant value
                        if let Some(const_expr) = self.const_values.borrow().get(name) {
                            if let Expr::Literal { value: Literal::Integer(n), .. } = const_expr {
                                *n as usize
                            } else {
                                ctx = self.lower_expr(ctx, count);
                                0usize
                            }
                        } else {
                            ctx = self.lower_expr(ctx, count);
                            0usize
                        }
                    }
                    _ => {
                        ctx = self.lower_expr(ctx, count);
                        0usize
                    }
                };
                let arr_type = MirType::Array(Box::new(elem_type.clone()), size);
                let arr_local = ctx.alloc_local("_arr", arr_type.clone());
                // Fill each element
                let zero = MirConstant::I32(0);
                let one = MirConstant::I32(1);
                let idx_local = ctx.alloc_local("_ar_idx", MirType::I32);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: idx_local,
                    value: MirValue::Constant(zero),
                });
                let body_label = ctx.fresh_block();
                let check_label = ctx.fresh_block();
                let done_label = ctx.fresh_block();
                ctx.finish_block(MirTerminator::Br(check_label.clone()));
                ctx.current_block = MirBasicBlock::new(check_label.clone());
                let idx_loaded = ctx.alloc_local("_ar_ld", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: idx_loaded,
                    src: idx_local,
                });
                let cond = ctx.alloc_local("_ar_cond", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: cond,
                    op: MirBinaryOp::Lt,
                    left: MirValue::Local(idx_loaded),
                    right: MirValue::Constant(MirConstant::I32(size as i32)),
                });
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(cond),
                    true_block: body_label.clone(),
                    false_block: done_label.clone(),
                });
                ctx.current_block = MirBasicBlock::new(body_label.clone());
                let elem_ptr = ctx.alloc_local("_areptr", elem_type.clone());
                ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                    dest: elem_ptr,
                    ptr: arr_local,
                    index: MirValue::Local(idx_loaded),
                    arr_type: Box::new(arr_type.clone()),
                    elem_type: Box::new(elem_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: elem_ptr,
                    value: MirValue::Local(elem_local),
                });
                let inc = ctx.alloc_local("_ar_inc", MirType::I32);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: inc,
                    op: MirBinaryOp::Add,
                    left: MirValue::Local(idx_loaded),
                    right: MirValue::Constant(one),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: idx_local,
                    value: MirValue::Local(inc),
                });
                ctx.finish_block(MirTerminator::Br(check_label.clone()));
                ctx.current_block = MirBasicBlock::new(done_label.clone());
                let arr_val = ctx.alloc_local("_arrv", arr_type.clone());
                ctx.current_block.insts.push(MirInst::Load {
                    dest: arr_val,
                    src: arr_local,
                });
                ctx
            }
            Expr::Array { elements, .. } => {
                if elements.is_empty() {
                    let arr_local = ctx.alloc_local("_arr", MirType::Array(Box::new(MirType::I32), 0));
                    let arr_val = ctx.alloc_local("_arrv", MirType::Array(Box::new(MirType::I32), 0));
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: arr_val,
                        src: arr_local,
                    });
                    return ctx;
                }
                let mut elem_locals = Vec::new();
                for elem in elements {
                    ctx = self.lower_expr(ctx, elem);
                    elem_locals.push(ctx.next_local - 1);
                }
                let elem_type = ctx.local_types.get(&elem_locals[0]).cloned()
                    .unwrap_or(MirType::I32);
                let size = elem_locals.len();
                let arr_type = MirType::Array(Box::new(elem_type.clone()), size);
                let arr_local = ctx.alloc_local("_arr", arr_type.clone());
                // Store each element into the array via computed GEP
                for (i, &elem_local) in elem_locals.iter().enumerate() {
                    let idx_val = MirValue::Constant(MirConstant::I32(i as i32));
                    let elem_ptr = ctx.alloc_local("_aeptr", elem_type.clone());
                    ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                        dest: elem_ptr,
                        ptr: arr_local,
                        index: idx_val,
                        arr_type: Box::new(arr_type.clone()),
                        elem_type: Box::new(elem_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: elem_ptr,
                        value: MirValue::Local(elem_local),
                    });
                }
                // Load the array value from alloca so assignment can store it
                let arr_val = ctx.alloc_local("_arrv", arr_type.clone());
                ctx.current_block.insts.push(MirInst::Load {
                    dest: arr_val,
                    src: arr_local,
                });
                ctx
            }
            Expr::Literal { value, .. } => {
                let (mir_type, constant) = match value {
                    Literal::Integer(n) => {
                        if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                            (MirType::I32, MirConstant::I32(*n as i32))
                        } else {
                            (MirType::I64, MirConstant::I64(*n))
                        }
                    }
                    Literal::Float(n) => (MirType::F64, MirConstant::F64(*n)),
                    Literal::String(s) => (MirType::Str, MirConstant::String(s.clone())),
                    Literal::Boolean(b) => (MirType::Bool, MirConstant::Bool(*b)),
                    Literal::Char(c) => (MirType::Char, MirConstant::I32(*c)),
                    Literal::None => (MirType::I32, MirConstant::Void),
                    Literal::Null => (MirType::Ptr(Box::new(MirType::Void)), MirConstant::Null),
                };
                let is_str = matches!(constant, MirConstant::String(_));
                if is_str {
                    // String literals must be heap-allocated so that move semantics
                    // can safely kl_free them. Store the constant in a temp, then
                    // call kl_clone_str to create an owned heap copy.
                    let const_local = ctx.alloc_local("_lit_const", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: const_local,
                        value: MirValue::Constant(constant),
                    });
                    let local = ctx.alloc_local("_lit", MirType::Str);
                    ctx.string_locals.push(local);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(local),
                        name: "ky_clone_str".to_string(),
                        args: vec![MirValue::Local(const_local)],
                    });
                    ctx
                } else {
                    let local = ctx.alloc_local("_lit", mir_type);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: local,
                        value: MirValue::Constant(constant),
                    });
                    ctx
                }
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
                } else if self.function_decls.borrow().contains_key(name) {
                    // Function name used as value (e.g. worker as ptr)
                    let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                    let dest = ctx.alloc_local("_fnaddr", ptr_type);
                    ctx.current_block.insts.push(MirInst::FnAddr {
                        dest,
                        name: name.clone(),
                    });
                }
                ctx
            }
            Expr::Binary { left, operator, right, .. } => {
                // Helper: map compound assignment operator to its base binary op
                let is_compound = matches!(operator,
                    BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign |
                    BinaryOp::DivAssign | BinaryOp::RemAssign
                );
                let compound_base_op = |op: &BinaryOp| -> Option<MirBinaryOp> {
                    match op {
                        BinaryOp::AddAssign => Some(MirBinaryOp::Add),
                        BinaryOp::SubAssign => Some(MirBinaryOp::Sub),
                        BinaryOp::MulAssign => Some(MirBinaryOp::Mul),
                        BinaryOp::DivAssign => Some(MirBinaryOp::Div),
                        BinaryOp::RemAssign => Some(MirBinaryOp::Rem),
                        _ => None,
                    }
                };

                // Handle compound assignment: x += expr, x -= expr, etc.
                if is_compound {
                    // Only handle simple identifier targets for now
                    if let Expr::Identifier { name, .. } = left.as_ref() {
                        if let Some(var_addr) = ctx.locals.get(name) {
                            let var_addr = *var_addr;
                            let var_type = ctx.local_types.get(&var_addr).cloned().unwrap_or(MirType::I32);
                            let left_is_str = var_type == MirType::Str;

                            // Load current value from variable
                            let loaded = ctx.alloc_local("_ca_load", var_type.clone());
                            if left_is_str {
                                ctx.string_locals.push(loaded);
                            }
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: loaded,
                                src: var_addr,
                            });

                            // Lower right side
                            ctx = self.lower_expr(ctx, right);
                            let right_local = ctx.next_local - 1;
                            let right_is_str = ctx.string_locals.contains(&right_local);

                            // String concatenation for +=
                            if matches!(operator, BinaryOp::AddAssign) && (left_is_str || right_is_str) {
                                let left_len = ctx.alloc_local("_strlen", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(left_len),
                                    name: "ky_strlen".to_string(),
                                    args: vec![MirValue::Local(loaded)],
                                });
                                let right_len = ctx.alloc_local("_strlen", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(right_len),
                                    name: "ky_strlen".to_string(),
                                    args: vec![MirValue::Local(right_local)],
                                });
                                let result = ctx.alloc_local("_bin", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(result),
                                    name: "ky_concat".to_string(),
                                    args: vec![
                                        MirValue::Local(loaded),
                                        MirValue::Local(left_len),
                                        MirValue::Local(right_local),
                                        MirValue::Local(right_len),
                                    ],
                                });
                                ctx.string_locals.push(result);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: var_addr,
                                    value: MirValue::Local(result),
                                });
                                return ctx;
                            }

                            // Determine base operator
                            let base_op = compound_base_op(&operator).unwrap_or(MirBinaryOp::Add);

                            // Coerce operands to same type
                            let left_type = ctx.local_types.get(&loaded).cloned().unwrap_or(MirType::I32);
                            let right_type = ctx.local_types.get(&right_local).cloned().unwrap_or(MirType::I32);
                            let wider = wider_type(&left_type, &right_type);
                            let is_float_op = matches!(wider, MirType::F32 | MirType::F64);
                            let left_operand = if left_type != wider && is_int_type(&left_type) && !is_float_op {
                                let cast = ctx.alloc_local("_widen", wider.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: cast,
                                    value: MirValue::Local(loaded),
                                    to_type: wider.clone(),
                                });
                                MirValue::Local(cast)
                            } else {
                                MirValue::Local(loaded)
                            };
                            let right_operand = if right_type != wider && is_int_type(&right_type) && !is_float_op {
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

                            let result_type = if is_float_op { wider.clone() } else { var_type.clone() };
                            let result = ctx.alloc_local("_ca_result", result_type.clone());
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: result,
                                op: base_op,
                                left: left_operand,
                                right: right_operand,
                            });

                            // Store result back
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: var_addr,
                                value: MirValue::Local(result),
                            });
                            return ctx;
                        }
                    }
                }

                // Handle assignment: target = value
                if matches!(operator, BinaryOp::Assign) {
                    if let Expr::Index { target: index_target, index, .. } = left.as_ref() {
                        // For arrays, use variable alloca directly (skip whole-array Load)
                        let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = index_target.as_ref() {
                            if let Some(&local) = ctx.locals.get(name) {
                                let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                if matches!(t, MirType::Array(_, _)) {
                                    (local, local, Some(t))
                                } else {
                                    ctx = self.lower_expr(ctx, index_target);
                                    let tv = ctx.next_local - 1;
                                    (tv, tv, ctx.local_types.get(&tv).cloned())
                                }
                            } else {
                                ctx = self.lower_expr(ctx, index_target);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned())
                            }
                        } else {
                            // Nested array assignment like m[i][j] = val.
                            let indices = self.collect_array_indices(left);
                            if let Some((root_name, idx_exprs)) = indices {
                                if let Some(&root_local) = ctx.locals.get(&root_name) {
                                    ctx = self.lower_nested_array_geps(ctx, &idx_exprs, root_local);
                                    ctx = self.lower_expr(ctx, right);
                                    let val_local2 = ctx.next_local - 1;
                                    let gep_ptr = ctx.next_local - 2;
                                    ctx.current_block.insts.push(MirInst::Store {
                                        dest: gep_ptr,
                                        value: MirValue::Local(val_local2),
                                    });
                                    return ctx;
                                }
                            }
                            ctx = self.lower_expr(ctx, index_target);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned())
                        };
                        ctx = self.lower_expr(ctx, index);
                        let idx_val = ctx.next_local - 1;
                        // Array set: arr[i] = val → ArrayElemPtr + Store
                        if matches!(&target_type, Some(MirType::Array(_, _))) {
                            ctx = self.lower_expr(ctx, right);
                            let val_local2 = ctx.next_local - 1;
                            if let Some(MirType::Array(ref inner_box, _)) = target_type {
                                let et = inner_box.as_ref().clone();
                                let arr_ty_clone = target_type.clone().unwrap();
                                let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                                ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                                    dest: elem_ptr,
                                    ptr: arr_ptr,
                                    index: MirValue::Local(idx_val),
                                    arr_type: Box::new(arr_ty_clone),
                                    elem_type: Box::new(et),
                                });
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: elem_ptr,
                                    value: MirValue::Local(val_local2),
                                });
                            }
                            return ctx;
                        }
                        // Dict set: dict["key"] = val → kl_dict_set
                        if matches!(&target_type, Some(MirType::Dict(_, _))) {
                            ctx = self.lower_expr(ctx, right);
                            let val_local = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_dict_val", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64,
                                value: MirValue::Local(val_local),
                                to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None,
                                name: "ky_dict_set".to_string(),
                                args: vec![
                                    MirValue::Local(target_val),
                                    MirValue::Local(idx_val),
                                    MirValue::Local(val_i64),
                                ],
                            });
                            return ctx;
                        }
                        // Ptr/Str set: ptr[idx] = val → MirInst::PtrStore
                        if matches!(target_type, Some(MirType::Ptr(_))) || target_type == Some(MirType::Str) {
                            ctx = self.lower_expr(ctx, right);
                            let val_local = ctx.next_local - 1;
                            let result = ctx.alloc_local("_ps", MirType::I32);
                            ctx.current_block.insts.push(MirInst::PtrStore {
                                dest: result,
                                ptr: target_val,
                                index: MirValue::Local(idx_val),
                                value: MirValue::Local(val_local),
                            });
                            return ctx;
                        }
                        // List set: list[idx] = val → kl_list_set
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
                            name: "ky_list_set".to_string(),
                            args: vec![
                                MirValue::Local(target_val),
                                MirValue::Local(idx_i64),
                                MirValue::Local(val_i64),
                            ],
                        });

                        return ctx;
                    }
                    if let Expr::PropertyAccess { object, property, .. } = left.as_ref() {
                        // If field is List and value is empty Dict {}, create list instead
                        if let Expr::Identifier { name, .. } = object.as_ref() {
                            if let Some(&obj_local) = ctx.locals.get(name) {
                                if let Some(MirType::Struct(_, fields)) = ctx.local_types.get(&obj_local).cloned() {
                                    let backing = format!("_{}", property);
                                    let field_idx = fields.iter().position(|(fname, _)| fname == property.as_str())
                                        .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                                    if let Some(fi) = field_idx {
                                        if let Some((_, MirType::List(inner))) = fields.get(fi) {
                                            if let Expr::Dictionary { entries, .. } = right.as_ref() {
                                                if entries.is_empty() {
                                                    let obj_ty = ctx.local_types.get(&obj_local).cloned().unwrap();
                                                    let handle = ctx.alloc_local("_listv", MirType::List(inner.clone()));
                                                    ctx.current_block.insts.push(MirInst::Call {
                                                        dest: Some(handle),
                                                        name: "ky_list_new".to_string(),
                                                        args: vec![],
                                                    });
                                                    let ft = ctx.alloc_local("_fptr", MirType::I64);
                                                    ctx.current_block.insts.push(MirInst::FieldPtr {
                                                        dest: ft,
                                                        ptr: obj_local,
                                                        field_index: fi,
                                                        struct_type: Box::new(obj_ty),
                                                    });
                                                    ctx.current_block.insts.push(MirInst::Store {
                                                        dest: ft,
                                                        value: MirValue::Local(handle),
                                                    });
                                                    return ctx;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        ctx = self.lower_expr(ctx, right);
                        let val_local = ctx.next_local - 1;
                        let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                            ctx.locals.get(name).copied()
                        } else {
                            None
                        };
                        if let Some(obj_ptr) = obj_ptr {
                            if let Some(MirType::Struct(_, fields)) = ctx.local_types.get(&obj_ptr).cloned() {
                                let backing = format!("_{}", property);
                                let field_idx = fields.iter().position(|(fname, _)| fname == property)
                                    .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                                if let Some(field_idx) = field_idx {
                                    let _field_type = fields[field_idx].1.clone();
                                    let ft = ctx.alloc_local("_fptr", MirType::I64);
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

                // Handle plain assignment: x = expr (from deferred expressions parsed as BinaryOp::Assign)
                if matches!(operator, BinaryOp::Assign) {
                    ctx = self.lower_expr(ctx, right);
                    let val_local = ctx.next_local - 1;
                    if let Expr::Index { target: list_expr, index, .. } = left.as_ref() {
                        // For arrays, use variable alloca directly (skip whole-array Load)
                        let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = list_expr.as_ref() {
                            if let Some(&local) = ctx.locals.get(name) {
                                let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                if matches!(t, MirType::Array(_, _)) {
                                    (local, local, t)
                                } else {
                                    ctx = self.lower_expr(ctx, list_expr);
                                    let tv = ctx.next_local - 1;
                                    (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                                }
                            } else {
                                ctx = self.lower_expr(ctx, list_expr);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                            }
                        } else {
                            ctx = self.lower_expr(ctx, list_expr);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        };
                        ctx = self.lower_expr(ctx, index);
                        let idx_val = ctx.next_local - 1;
                        ctx = self.lower_expr(ctx, right);
                        let val_local2 = ctx.next_local - 1;
                        if matches!(&target_type, MirType::Array(_, _)) {
                            let arr_ty_clone = target_type.clone();
                            let et = match &target_type {
                                MirType::Array(inner_box, _) => { inner_box.as_ref().clone() },
                                _ => MirType::I32,
                            };
                            let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                                dest: elem_ptr,
                                ptr: arr_ptr,
                                index: MirValue::Local(idx_val),
                                arr_type: Box::new(arr_ty_clone),
                                elem_type: Box::new(et),
                            });
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: elem_ptr,
                                value: MirValue::Local(val_local2),
                            });
                        } else if matches!(&target_type, MirType::Dict(_, _)) {
                            let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast { dest: val_i64, value: MirValue::Local(val_local2), to_type: MirType::I64 });
                            let key_arg = if ctx.local_types.get(&idx_val).map(|t| *t == MirType::Str).unwrap_or(false) {
                                MirValue::Local(idx_val)
                            } else {
                                MirValue::Local(idx_val)
                            };
                            ctx.current_block.insts.push(MirInst::Call { dest: None, name: "ky_dict_set".to_string(), args: vec![MirValue::Local(target_val), key_arg, MirValue::Local(val_i64)] });
                        } else {
                            let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast { dest: idx_i64, value: MirValue::Local(idx_val), to_type: MirType::I64 });
                            let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast { dest: val_i64, value: MirValue::Local(val_local2), to_type: MirType::I64 });
                            ctx.current_block.insts.push(MirInst::Call { dest: None, name: "ky_list_set".to_string(), args: vec![MirValue::Local(target_val), MirValue::Local(idx_i64), MirValue::Local(val_i64)] });
                        }
                    } else if let Expr::Identifier { name, .. } = left.as_ref() {
                        if let Some(local) = ctx.locals.get(name) {
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: *local,
                                value: MirValue::Local(val_local),
                            });
                        }
                    }
                    return ctx;
                }

                ctx = self.lower_expr(ctx, left);
                let left_local = ctx.next_local - 1;
                let left_is_str = ctx.string_locals.contains(&left_local)
                    || ctx.local_types.get(&left_local).map_or(false, |t| *t == MirType::Str);
                ctx = self.lower_expr(ctx, right);
                let right_local = ctx.next_local - 1;
                let right_is_str = ctx.string_locals.contains(&right_local)
                    || ctx.local_types.get(&right_local).map_or(false, |t| *t == MirType::Str);
                // Range expression outside brackets — no-op
                if matches!(operator, BinaryOp::Range) {
                    return ctx;
                }

                // `is` type test — compare left expression type with right type name
                if matches!(operator, BinaryOp::Is) {
                    let left_local = ctx.next_local - 1;
                    let left_type = ctx.local_types.get(&left_local);
                    let right_mir_type = if let Expr::Identifier { name, .. } = right.as_ref() {
                        match name.as_str() {
                            "i8" => Some(MirType::I8),
                            "i16" => Some(MirType::I16),
                            "i32" => Some(MirType::I32),
                            "i64" => Some(MirType::I64),
                            "f32" => Some(MirType::F32),
                            "f64" => Some(MirType::F64),
                            "bool" => Some(MirType::Bool),
                            "char" => Some(MirType::Char),
                            "str" => Some(MirType::Str),
                            _ => None,
                        }
                    } else { None };
                    let matches = left_type.and_then(|lt| right_mir_type.map(|rt| *lt == rt)).unwrap_or(false);
                    let dest = ctx.alloc_local("_is", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest,
                        value: if matches {
                            MirValue::Constant(MirConstant::I32(1))
                        } else {
                            MirValue::Constant(MirConstant::I32(0))
                        },
                    });
                    return ctx;
                }

                // `as` cast: left as TypeName
                if matches!(operator, BinaryOp::As) {
                    let left_local = ctx.next_local - 1;
                    let to_type = if let Expr::Identifier { name, .. } = right.as_ref() {
                        match name.as_str() {
                            "i8" | "u8" => MirType::I8,
                            "i16" | "u16" => MirType::I16,
                            "i32" | "u32" => MirType::I32,
                            "i64" | "u64" => MirType::I64,
                            "f32" => MirType::F32,
                            "f64" => MirType::F64,
                            "bool" => MirType::Bool,
                            "char" => MirType::Char,
                            "str" => MirType::Str,
                            "ptr" => MirType::Ptr(Box::new(MirType::Void)),
                            name => {
                                if let Some(defs) = ctx.struct_defs.get(name) {
                                    MirType::Struct(name.to_string(), defs.clone())
                                } else {
                                    MirType::I32
                                }
                            }
                        }
                    } else {
                        MirType::I32
                    };
                    let dest = ctx.alloc_local("_cast", to_type.clone());
                    // For str → ptr, use ky_ptr_read_ptr to load the data pointer
                    // from the strBuilder struct. This gives the actual string data pointer.
                    let src_type = ctx.local_types.get(&left_local);
                    if matches!(to_type, MirType::Ptr(_)) && matches!(src_type, Some(MirType::Str | MirType::Ptr(_))) {
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(dest),
                            name: "ky_ptr_read_ptr".to_string(),
                            args: vec![MirValue::Local(left_local)],
                        });
                    } else {
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest,
                            value: MirValue::Local(left_local),
                            to_type,
                        });
                    }
                    return ctx;
                }

                // String concatenation if either operand is a string
                if matches!(operator, BinaryOp::Add) && (left_is_str || right_is_str) {
                    // Get ptr and len for each operand
                    // Left: use the existing local (pointer)
                    // Need length for left if it's a string
                    let left_len = if left_is_str {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(d),
                            name: "ky_strlen".to_string(),
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
                            name: "ky_strlen".to_string(),
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
                        name: "ky_concat".to_string(),
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
                        name: "ky_eq_str".to_string(),
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

                // Operator overloading: dispatch to op_X method for struct types
                let overload_op_name = match operator {
                    BinaryOp::Add => Some("op_+"),
                    BinaryOp::Sub => Some("op_-"),
                    BinaryOp::Mul => Some("op_*"),
                    BinaryOp::Div => Some("op_/"),
                    BinaryOp::Rem => Some("op_%"),
                    BinaryOp::Eq => Some("op_=="),
                    BinaryOp::Neq => Some("op_!="),
                    BinaryOp::Lt => Some("op_<"),
                    BinaryOp::Gt => Some("op_>"),
                    BinaryOp::Le => Some("op_<="),
                    BinaryOp::Ge => Some("op_>="),
                    _ => None,
                };
                if let Some(op_name) = overload_op_name {
                    let left_type = ctx.local_types.get(&left_local).cloned().unwrap_or(MirType::I32);
                    if let MirType::Struct(class_name, _) = &left_type {
                        let method_table = self.method_table.borrow();
                        if let Some(methods) = method_table.get(class_name) {
                            if let Some(mangled) = methods.get(op_name) {
                                let ret_type = self.fn_returns.borrow()
                                    .get(mangled).cloned().unwrap_or(MirType::I64);
                                let dest = ctx.alloc_local("_op", ret_type);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(dest),
                                    name: mangled.clone(),
                                    args: vec![MirValue::Local(left_local), MirValue::Local(right_local)],
                                });
                                return ctx;
                            }
                        }
                    }
                }

                // Coerce operands to the same type for binary operations.
                // Get the actual MIR types of each operand.
                let left_type = ctx.local_types.get(&left_local).cloned().unwrap_or(MirType::I32);
                let right_type = ctx.local_types.get(&right_local).cloned().unwrap_or(MirType::I32);
                let wider = wider_type(&left_type, &right_type);
                let is_float_op = matches!(wider, MirType::F32 | MirType::F64);
                let left_operand = if left_type != wider && is_int_type(&left_type) && !is_float_op {
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
                let right_operand = if right_type != wider && is_int_type(&right_type) && !is_float_op {
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

                // Comparison ops always produce I32. Arithmetic ops produce the wider type.
                let is_cmp = matches!(operator,
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt |
                    BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge);
                let result_type = if is_cmp { MirType::I32 } else { wider.clone() };
                let dest = ctx.alloc_local("_bin", result_type);

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
                    BinaryOp::AddPercent | BinaryOp::SubPercent | BinaryOp::MulPercent => {
                        let fn_name = match operator {
                            BinaryOp::AddPercent => "ky_add_pct",
                            BinaryOp::SubPercent => "ky_sub_pct",
                            BinaryOp::MulPercent => "ky_mul_pct",
                            _ => unreachable!(),
                        };
                        let pct_left = ctx.alloc_local("_pct_l", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pct_left,
                            value: left_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pct_right = ctx.alloc_local("_pct_r", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pct_right,
                            value: right_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pct_dest = ctx.alloc_local("_pct", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(pct_dest),
                            name: fn_name.to_string(),
                            args: vec![
                                MirValue::Local(pct_left),
                                MirValue::Local(pct_right),
                            ],
                        });
                        return ctx;
                    }
                    BinaryOp::Pow => {
                        // Power: generate call to ky_pow(i64, i64) -> i64
                        // Cast operands to i64 first
                        let pow_left = ctx.alloc_local("_pow_l", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pow_left,
                            value: left_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pow_right = ctx.alloc_local("_pow_r", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pow_right,
                            value: right_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pow_dest = ctx.alloc_local("_pow", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(pow_dest),
                            name: "ky_pow".to_string(),
                            args: vec![
                                MirValue::Local(pow_left),
                                MirValue::Local(pow_right),
                            ],
                        });
                        return ctx;
                    }
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
            Expr::FunctionCall { target, arguments, type_args, .. } => {
                // Method dispatch: obj.method(args) → Call ClassName::method(obj, args...)
                // Checked BEFORE the list-specific shortcuts so real class methods win
                // over the generic list.add/list.pop fallbacks.
                if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    // Check for enum variant construction: Option.Some(value)
                    // This must come BEFORE method dispatch because enum type names
                    // are not variables and cannot be lowered as expressions.
                    if let Expr::Identifier { name: enum_name, .. } = object.as_ref() {
                        let ev_map = self.enum_variants.borrow();
                        if let Some(variants) = ev_map.get(enum_name) {
                            if let Some(&variant_idx) = variants.get(property) {
                                // Determine inner type name for Option-like enums
                                let _inner_name = if arguments.len() == 1 {
                                    match &arguments[0] {
                                        Expr::Identifier { name, .. } => {
                                            if let Some(&local) = ctx.locals.get(name) {
                                                if let Some(MirType::Struct(inner, _)) = ctx.local_types.get(&local) {
                                                    Some(inner.clone())
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        }
                                        Expr::Literal { value: Literal::Integer(_), .. } => Some("i32".to_string()),
                                        Expr::Literal { value: Literal::String(_), .. } => Some("str".to_string()),
                                        Expr::Literal { value: Literal::Boolean(_), .. } => Some("bool".to_string()),
                                        _ => None,
                                    }
                                } else {
                                    None
                                };
                                // All enum variant types share the same tagged-union layout
                                // { disc: i32, payload: i64 } regardless of type parameters.
                                // Use the base enum name to match ast_type_to_mir resolution
                                // and function return types.
                                let concrete_name = enum_name.clone();
                                let struct_type = MirType::Struct(concrete_name.clone(), vec![
                                    ("disc".to_string(), MirType::I32),
                                    ("payload".to_string(), MirType::I64),
                                ]);

                                // Register concrete struct type name in struct_defs
                                ctx.struct_defs.entry(concrete_name).or_insert_with(|| vec![
                                    ("disc".to_string(), MirType::I32),
                                    ("payload".to_string(), MirType::I64),
                                ]);

                                // Allocate temps FIRST, struct LAST so the result local is the struct
                                let disc_ptr = ctx.alloc_local("_edp", MirType::I64);
                                let disc_val = MirValue::Constant(MirConstant::I32(variant_idx as i32));

                                let (pay_ptr, arg_val) = if let Some(arg) = arguments.first() {
                                    ctx = self.lower_expr(ctx, arg);
                                    let av = ctx.next_local - 1;
                                    let pp = ctx.alloc_local("_epp", MirType::I64);
                                    let ai = ctx.alloc_local("_epv", MirType::I64);
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: ai,
                                        value: MirValue::Local(av),
                                        to_type: MirType::I64,
                                    });
                                    (Some(pp), Some(ai))
                                } else {
                                    (None, None)
                                };

                                // Allocate struct LAST so the caller sees the correct result local
                                let dest = ctx.alloc_local("_enum", struct_type.clone());

                                // Store discriminant
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: disc_ptr,
                                    ptr: dest,
                                    field_index: 0,
                                    struct_type: Box::new(struct_type.clone()),
                                });
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: disc_ptr,
                                    value: disc_val,
                                });

                                // Store payload
                                if let (Some(pp), Some(av)) = (pay_ptr, arg_val) {
                                    ctx.current_block.insts.push(MirInst::FieldPtr {
                                        dest: pp,
                                        ptr: dest,
                                        field_index: 1,
                                        struct_type: Box::new(struct_type),
                                    });
                                    ctx.current_block.insts.push(MirInst::Store {
                                        dest: pp,
                                        value: MirValue::Local(av),
                                    });
                                }

                                return ctx;
                            }
                        }
                    }
                    // Static method dispatch: ClassName.method(args) — no receiver needed
                    if let Expr::Identifier { name: class_name, .. } = object.as_ref() {
                        let method_table = self.method_table.borrow();
                        if let Some(methods) = method_table.get(class_name) {
                            if let Some(mangled) = methods.get(property) {
                                if self.static_methods.borrow().contains(mangled) {
                                    let mut call_args = Vec::new();
                                    for arg in arguments {
                                        ctx = self.lower_expr(ctx, arg);
                                        call_args.push(MirValue::Local(ctx.next_local - 1));
                                    }
                                    let call_type = self.fn_returns.borrow()
                                        .get(mangled).cloned().unwrap_or(MirType::I64);
                                    let dest = ctx.alloc_local("_modcall", call_type.clone());
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
                    }

                    // Check for module-qualified function call: module.func(args)
                    // where `module` is not a local variable and not an enum name.
                    if let Expr::Identifier { name: mod_name, .. } = object.as_ref() {
                        if !ctx.locals.contains_key(mod_name) {
                            let ev_map = self.enum_variants.borrow();
                            if !ev_map.contains_key(mod_name) {
                                // Map namespaced API to flat function name
                                let resolve_namespace = |ns: &str, func: &str| -> Option<String> {
                                    match (ns, func) {
                                        ("parallel", "for") => Some("ky_parallel_for".into()),
                                        ("thread", "spawn") => Some("ky_spawn_thread".into()),
                                        ("thread", "join") => Some("ky_join_thread".into()),
                                        ("thread", "sleep") => Some("ky_sleep".into()),
                                        ("thread", "yield") => Some("ky_yield".into()),
                                        ("assert", "is_true") => Some("assert".into()),
                                        ("assert", "eq") => Some("assert_eq".into()),
                                        ("assert", "ne") => Some("assert_ne".into()),
                                        ("assert", "str_eq") => Some("assert_str".into()),
                                        ("math", "pow") => Some("ky_pow".into()),
                                        ("math", "ceil") => Some("ceil".into()),
                                        ("math", "floor") => Some("floor".into()),
                                        ("math", "round") => Some("round".into()),
                                        ("json", "parse") => Some("json_parse".into()),
                                        ("json", "stringify") => Some("json_stringify".into()),
                                        ("json", "serialize") => Some("serialize".into()),
                                        ("json", "deserialize") => Some("deserialize".into()),
                                        ("crypto", "sha1") => Some("ky_sha1".into()),
                                        ("crypto", "base64_encode") => Some("ky_base64_encode".into()),
                                        ("process", "env") => Some("ky_getenv".into()),
                                        ("tcp", "listen") => Some("ky_tcp_listen".into()),
                                        ("tcp", "accept") => Some("ky_tcp_accept".into()),
                                        ("tcp", "read") => Some("ky_tcp_read".into()),
                                        ("tcp", "write") => Some("ky_tcp_write".into()),
                                        ("tcp", "close") => Some("ky_tcp_close".into()),
                                        ("dict", "contains") => Some("ky_dict_contains".into()),
                                        ("dict", "remove") => Some("ky_dict_remove".into()),
                                        ("str_builder", "new") => Some("ky_str_builder_new".into()),
                                        ("str_builder", "append") => Some("ky_str_builder_append".into()),
                                        ("str_builder", "to_str") => Some("ky_str_builder_to_str".into()),
                                        ("str_builder", "free") => Some("ky_str_builder_free".into()),
                                        ("fs", "exists") => Some("ky_fs_exists".into()),
                                        ("fs", "is_dir") => Some("ky_fs_is_dir".into()),
                                        ("fs", "is_file") => Some("ky_fs_is_file".into()),
                                        ("fs", "size") => Some("ky_fs_size".into()),
                                        ("fs", "copy") => Some("ky_fs_copy".into()),
                                        ("fs", "remove") => Some("ky_fs_remove".into()),
                                        ("fs", "create_dir") => Some("ky_fs_create_dir".into()),
                                        ("fs", "remove_dir") => Some("ky_fs_remove_dir".into()),
                                        ("fs", "rename") => Some("ky_fs_rename".into()),
                                        ("fs", "read_to_string") => Some("ky_fs_read_to_string".into()),
                                        ("fs", "write_string") => Some("ky_fs_write_string".into()),
                                        ("fs", "list_dir") => Some("ky_fs_list_dir".into()),
                                        ("set", "new") => Some("ky_set_new".into()),
                                        ("set", "free") => Some("ky_set_free".into()),
                                        ("set", "add") => Some("ky_set_add".into()),
                                        ("set", "contains") => Some("ky_set_contains".into()),
                                        ("set", "remove") => Some("ky_set_remove".into()),
                                        ("set", "len") => Some("ky_set_len".into()),
                                        ("file", "open") => Some("ky_open".into()),
                                        ("file", "read") => Some("ky_read_str".into()),
                                        ("file", "write") => Some("ky_write_str".into()),
                                        ("file", "close") => Some("ky_close".into()),
                                        ("time", "now_ms") => Some("ky_time_now_ms".into()),
                                        ("time", "now_us") => Some("ky_time_now_us".into()),
                                        _ => None,
                                    }
                                };
                                let mut fn_name = resolve_namespace(mod_name, property)
                                    .unwrap_or_else(|| property.clone());
                                let mut args = Vec::new();
                                for arg in arguments {
                                    if let Expr::Identifier { name, .. } = arg {
                                        if let Some(&local) = ctx.locals.get(name) {
                                            if let Some(t) = ctx.local_types.get(&local) {
                                                if matches!(t, MirType::Struct(_, _)) || is_move_type(t) {
                                                    args.push(MirValue::Local(local));
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                    ctx = self.lower_expr(ctx, arg);
                                    args.push(MirValue::Local(ctx.next_local - 1));
                                }
                                // Special handling for json.deserialize<T>(str)
                                if fn_name == "deserialize" && args.len() == 1 {
                                    if let Some(first_type_arg) = type_args.first() {
                                        let struct_defs = ctx.struct_defs.clone();
                                        let mir_type = ast_type_to_mir(first_type_arg, Some(&struct_defs));
                                        if let MirType::Struct(_, fields) = &mir_type {
                                            let descriptor = build_descriptor(fields);
                                            let json_arg = args.remove(0);
                                            let out_local = ctx.alloc_local("_dout", mir_type.clone());
                                            args.push(json_arg);
                                            args.push(MirValue::Constant(MirConstant::String(descriptor)));
                                            args.push(MirValue::Local(out_local));
                                            let ret_local = ctx.alloc_local("_dret", MirType::I32);
                                            ctx.current_block.insts.push(MirInst::Call {
                                                dest: Some(ret_local),
                                                name: "ky_json_to_struct".to_string(),
                                                args,
                                            });
                                            let load = ctx.alloc_local("_dval", mir_type.clone());
                                            ctx.current_block.insts.push(MirInst::Load {
                                                dest: load,
                                                src: out_local,
                                            });
                                            return ctx;
                                        }
                                    }
                                    // Without type args, fall back to json_parse (returns Dict<str,i64>)
                                    fn_name = "json_parse".to_string();
                                }
                                let call_type = builtin_return_type(&fn_name)
                                    .or_else(|| self.fn_returns.borrow().get(&fn_name).cloned())
                                    .unwrap_or(MirType::Void);
                                let dest = ctx.alloc_local("_modcall", call_type.clone());
                                if call_type == MirType::Str {
                                    ctx.string_locals.push(dest);
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(dest),
                                    name: fn_name,
                                    args,
                                });
                                return ctx;
                            }
                        }
                    }

                    // Lower the receiver first, but for struct identifiers pass by reference (no copy).
                    let obj_local = if let Expr::Identifier { name, .. } = object.as_ref() {
                        if let Some(&local) = ctx.locals.get(name) {
                            let lt = ctx.local_types.get(&local);
                            if matches!(lt, Some(MirType::Struct(_, _)) | Some(MirType::Ptr(_)) | Some(MirType::Slice(_))) {
                                local
                            } else {
                                ctx = self.lower_expr(ctx, object);
                                ctx.next_local - 1
                            }
                        } else {
                            ctx = self.lower_expr(ctx, object);
                            ctx.next_local - 1
                        }
                    } else {
                        ctx = self.lower_expr(ctx, object);
                        ctx.next_local - 1
                    };
                    let obj_type = ctx.local_types.get(&obj_local).cloned();

                    // If the receiver is a class instance (MirType::Struct) and the class
                    // declares a method named `property`, emit a real method call.
                    // Also unwrap Ptr(Struct(...)) for closure-inferred types.
                    let struct_type = obj_type.as_ref().and_then(|t| {
                        if let MirType::Struct(name, _) = t { Some(name.clone()) }
                        else if let MirType::Ptr(inner) = t {
                            if let MirType::Struct(name, _) = inner.as_ref() { Some(name.clone()) }
                            else { None }
                        } else { None }
                    });
                    if let Some(ref class_name) = struct_type {
                        // For concrete generic types (e.g. "Box__i32"), ensure methods are monomorphized
                        if class_name.contains("__") {
                            self.ensure_generic_class_methods(class_name);
                        }
                        let method_table = self.method_table.borrow();
                        let parent_map = self.class_parent_map.borrow();
                        if let Some(mangled) = self.lookup_method_in_chain(class_name, property, &method_table, &parent_map) {
                                // Static methods don't get `this` as first arg
                                let is_static = self.static_methods.borrow().contains(&mangled);
                                let mut call_args = if is_static {
                                    Vec::new()
                                } else {
                                    vec![MirValue::Local(obj_local)]
                                };
                                for arg in arguments {
                                    // Struct-typed or Move-typed identifiers: pass original local
                                    if let Expr::Identifier { name, .. } = arg {
                                        if let Some(&local) = ctx.locals.get(name) {
                                            if let Some(t) = ctx.local_types.get(&local) {
                                                if matches!(t, MirType::Struct(_, _)) || is_move_type(t) {
                                                    call_args.push(MirValue::Local(local));
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                    ctx = self.lower_expr(ctx, arg);
                                    call_args.push(MirValue::Local(ctx.next_local - 1));
                                }
                                let call_type = self.fn_returns.borrow()
                                    .get(&mangled).cloned().unwrap_or(MirType::Void);
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

                    // Built-in type method dispatch (str, list, array, dict, char)
                    let is_str = obj_type.as_ref().map(|t| *t == MirType::Str).unwrap_or(false);
                    let is_list = obj_type.as_ref().map(|t| matches!(t, MirType::List(_))).unwrap_or(false);
                    let is_array = obj_type.as_ref().map(|t| matches!(t, MirType::Array(_, _))).unwrap_or(false);
                    let is_char = obj_type.as_ref().map(|t| *t == MirType::Char).unwrap_or(false);
                    let is_slice = obj_type.as_ref().map(|t| matches!(t, MirType::Slice(_))).unwrap_or(false);

                    // === UNIVERSAL METHOD: .type() on any value ===
                    if property == "type" && arguments.is_empty() {
                        if let Some(ref obj_t) = obj_type {
                            build_typeinfo_struct(obj_t, &mut ctx);
                            return ctx;
                        }
                    }

                    // === ARRAY METHODS ===
                    if is_array && property == "len" && arguments.is_empty() {
                        // Array length is compile-time constant (N in [T, N])
                        // Return as I32 to match loop variable type in for-range
                        if let MirType::Array(_, size) = obj_type.as_ref().unwrap() {
                            let result = ctx.alloc_local("_arrlen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result,
                                value: MirValue::Constant(MirConstant::I32(*size as i32)),
                            });
                            return ctx;
                        }
                    }

                    // === SLICE METHODS ===
                    if is_slice && property == "len" && arguments.is_empty() {
                        // Slice length is stored in field 1 of the slice struct (i64)
                        if let Some(MirType::Slice(inner)) = obj_type.as_ref() {
                            let slice_type = MirType::Slice(inner.clone());
                            let len_ptr = ctx.alloc_local("_slenp", MirType::I64);
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: len_ptr,
                                ptr: obj_local,
                                field_index: 1,
                                struct_type: Box::new(slice_type.clone()),
                            });
                            let len_i64 = ctx.alloc_local("_slen64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: len_i64,
                                src: len_ptr,
                            });
                            let result = ctx.alloc_local("_slen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: result,
                                value: MirValue::Local(len_i64),
                                to_type: MirType::I32,
                            });
                            return ctx;
                        }
                    }

                    // === STRING METHODS ===
                    if is_str && property == "len" && arguments.is_empty() {
                        let result = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_strlen".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && property == "to_upper" && arguments.is_empty() {
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_to_upper".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && property == "to_lower" && arguments.is_empty() {
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_to_lower".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && property == "trim" && arguments.is_empty() {
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_trim".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && property == "contains" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let arg = ctx.next_local - 1;
                        let result = ctx.alloc_local("_c", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_contains".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(arg)],
                        });
                        return ctx;
                    }
                    if is_str && property == "replace" && arguments.len() == 2 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let from = ctx.next_local - 1;
                        ctx = self.lower_expr(ctx, &arguments[1]);
                        let to = ctx.next_local - 1;
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_replace".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(from), MirValue::Local(to)],
                        });
                        return ctx;
                    }
                    if is_str && property == "substr" && arguments.len() == 2 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let start = ctx.next_local - 1;
                        ctx = self.lower_expr(ctx, &arguments[1]);
                        let count = ctx.next_local - 1;
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_substr".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(start), MirValue::Local(count)],
                        });
                        return ctx;
                    }
                    if is_str && property == "char_at" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let idx = ctx.next_local - 1;
                        let result = ctx.alloc_local("_c", MirType::Char);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_char_at".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(idx)],
                        });
                        return ctx;
                    }
                    // String is_* methods: get first char, then call the char function
                    if is_str && arguments.is_empty() {
                        let is_fn = match property.as_str() {
                            "is_digit" => Some("ky_is_digit"),
                            "is_alpha" => Some("ky_is_alpha"),
                            "is_alnum" => Some("ky_is_alnum"),
                            "is_whitespace" => Some("ky_is_whitespace"),
                            "is_upper" => Some("ky_is_upper"),
                            "is_lower" => Some("ky_is_lower"),
                            _ => None,
                        };
                        if let Some(fn_name) = is_fn {
                            // Get first character code
                            let zero = ctx.alloc_local("_iz", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: zero, value: MirValue::Constant(MirConstant::I32(0)),
                            });
                            let first_char = ctx.alloc_local("_ifc", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(first_char), name: "ky_char_at".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(zero)],
                            });
                            let result = ctx.alloc_local("_ir", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: fn_name.to_string(),
                                args: vec![MirValue::Local(first_char)],
                            });
                            return ctx;
                        }
                    }

                    // === CHAR METHODS ===
                    if is_char && property == "ord" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ord", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_ord".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_digit" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cd", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_digit".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_alpha" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ca", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_alpha".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_alnum" && arguments.is_empty() {
                        let result = ctx.alloc_local("_can", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_alnum".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_whitespace" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cw", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_whitespace".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_upper" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cu", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_upper".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_lower" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cl", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_lower".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }

                    // === UNIVERSAL CONVERSION METHODS (.to_str, .to_int, etc.) ===
                    if property == "to_str" && arguments.is_empty() {
                        let id_type = ctx.local_types.get(&obj_local).cloned().unwrap_or(MirType::I32);
                        if id_type == MirType::Str {
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_clone_str".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        } else if id_type == MirType::Char {
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_char_to_str".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        } else if matches!(id_type, MirType::F32 | MirType::F64) {
                            // F32/F64 → F64 → ky_f64_to_str(str)
                            let f64_val = if id_type == MirType::F32 {
                                let c = ctx.alloc_local("_f2f", MirType::F64);
                                ctx.current_block.insts.push(MirInst::Cast { dest: c, value: MirValue::Local(obj_local), to_type: MirType::F64 });
                                MirValue::Local(c)
                            } else { MirValue::Local(obj_local) };
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_f64_to_str".to_string(),
                                args: vec![f64_val],
                            });
                            return ctx;
                        } else {
                            let i64_val = if id_type == MirType::I32 {
                                let ext = ctx.alloc_local("_i64v", MirType::I64);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: ext,
                                    value: MirValue::Local(obj_local),
                                    to_type: MirType::I64,
                                });
                                MirValue::Local(ext)
                            } else {
                                MirValue::Local(obj_local)
                            };
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_i64_to_str".to_string(),
                                args: vec![i64_val],
                            });
                            return ctx;
                        }
                    }
                    // Type-specific conversion methods
                    if property == "to_i32" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ti32", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I32 });
                        return ctx;
                    }
                    if property == "to_i64" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ti64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I64 });
                        return ctx;
                    }
                    if property == "to_i16" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ti16", MirType::I16);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I16 });
                        return ctx;
                    }
                    if property == "to_i8" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ti8", MirType::I8);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I8 });
                        return ctx;
                    }
                    if property == "to_u32" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu32", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I32 });
                        return ctx;
                    }
                    if property == "to_u64" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I64 });
                        return ctx;
                    }
                    if property == "to_u16" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu16", MirType::I16);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I16 });
                        return ctx;
                    }
                    if property == "to_u8" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu8", MirType::I8);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I8 });
                        return ctx;
                    }
                    if property == "to_f64" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tf64", MirType::F64);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::F64 });
                        return ctx;
                    }
                    if property == "to_f32" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tf32", MirType::F32);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::F32 });
                        return ctx;
                    }
                    if property == "to_char" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tch", MirType::Char);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::Char });
                        return ctx;
                    }
                    // Option<T> methods: is_some, is_none, unwrap
                    if (property == "is_some" || property == "is_none") && arguments.is_empty() {
                        // Option is stored as {disc: i32, payload: i64}
                        let struct_type = MirType::Struct("__option".to_string(), vec![
                            ("disc".to_string(), MirType::I32),
                            ("payload".to_string(), MirType::I64),
                        ]);
                        let disc_ptr = ctx.alloc_local("_odp", MirType::Ptr(Box::new(MirType::I32)));
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: disc_ptr, ptr: obj_local, field_index: 0,
                            struct_type: Box::new(struct_type),
                        });
                        let disc = ctx.alloc_local("_od", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Load { dest: disc, src: disc_ptr });
                        let zero = ctx.alloc_local("_oz", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Store { dest: zero, value: MirValue::Constant(MirConstant::I32(0)) });
                        let eq = ctx.alloc_local("_oe", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: eq, op: if property == "is_some" { MirBinaryOp::Neq } else { MirBinaryOp::Eq },
                            left: MirValue::Local(disc), right: MirValue::Local(zero),
                        });
                        return ctx;
                    }
                    if property == "unwrap" && arguments.is_empty() {
                        let struct_type = MirType::Struct("__option".to_string(), vec![
                            ("disc".to_string(), MirType::I32),
                            ("payload".to_string(), MirType::I64),
                        ]);
                        let payload_ptr = ctx.alloc_local("_opp", MirType::Ptr(Box::new(MirType::I64)));
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: payload_ptr, ptr: obj_local, field_index: 1,
                            struct_type: Box::new(struct_type),
                        });
                        let payload = ctx.alloc_local("_op", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Load { dest: payload, src: payload_ptr });
                        return ctx;
                    }
                    if property == "to_bool" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tb", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::Bool });
                        return ctx;
                    }
                    if property == "to_decimal" && arguments.is_empty() {
                        let i64_val = ctx.alloc_local("_td", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast { dest: i64_val, value: MirValue::Local(obj_local), to_type: MirType::I64 });
                        let result = ctx.alloc_local("_tds", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_decimal_to_str".to_string(),
                            args: vec![MirValue::Local(i64_val)],
                        });
                        return ctx;
                    }
                    if property == "stringify" && arguments.is_empty() {
                        // Convert to JSON: calls json_stringify on the dict/struct
                        let result = ctx.alloc_local("_sj", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_json_stringify".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }

                    // === LIST METHODS ===
                    if is_list {
                        if property == "pop" {
                            let result = ctx.alloc_local("_pop", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_list_pop".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "reserve" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let cap_val = ctx.next_local - 1;
                            let cap_i64 = ctx.alloc_local("_cap64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: cap_i64,
                                value: MirValue::Local(cap_val),
                                to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None,
                                name: "ky_list_reserve".to_string(),
                                args: vec![
                                    MirValue::Local(obj_local),
                                    MirValue::Local(cap_i64),
                                ],
                            });
                            return ctx;
                        }
                        if property == "push" || property == "add" {
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
                                        name: "ky_alloc".to_string(),
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
                                    name: "ky_list_push".to_string(),
                                    args: vec![MirValue::Local(obj_local), MirValue::Local(arg_i64)],
                                });
                            }
                            return ctx;
                        }
                        if is_list && property == "len" && arguments.is_empty() {
                            let result = ctx.alloc_local("_listlen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "pop_first" && arguments.is_empty() {
                            let result = ctx.alloc_local("_popf", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_pop_first".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_clear".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "contains" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let arg = ctx.next_local - 1;
                            let arg_i64 = ctx.alloc_local("_c64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: arg_i64, value: MirValue::Local(arg), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_cres", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_contains".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(arg_i64)],
                            });
                            return ctx;
                        }
                        if is_list && property == "insert" && arguments.len() == 2 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_i64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[1]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_v64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_insert".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if is_list && property == "remove_at" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_i64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_rvat", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_remove_at".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64)],
                            });
                            return ctx;
                        }
                        // Remove by VALUE (find first occurrence, remove it, return 0/1)
                        if is_list && property == "remove" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_rv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_rres", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_remove_value".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        // === GET / SET (direct element access) ===
                        if is_list && property == "get" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_gi64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_get", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_get".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64)],
                            });
                            return ctx;
                        }
                        if is_list && property == "set" && arguments.len() == 2 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_si64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[1]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_sv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_set".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        // === AGGREGATE METHODS (sum, product, max, min) ===
                        if is_list && property == "sum" && arguments.is_empty() {
                            let result = ctx.alloc_local("_sum", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_sum".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "product" && arguments.is_empty() {
                            let result = ctx.alloc_local("_prod", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_product".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "max" && arguments.is_empty() {
                            let result = ctx.alloc_local("_max", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_max".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "min" && arguments.is_empty() {
                            let result = ctx.alloc_local("_min", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_min".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "reverse" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_reverse".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        // === LAZY ITERATOR ===
                        if is_list && property == "iter" && arguments.is_empty() {
                            let list_i64 = ctx.alloc_local("_li64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: list_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_iter", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_new".to_string(),
                                args: vec![MirValue::Local(list_i64)],
                            });
                            return ctx;
                        }
                        // === HIGHER-ORDER METHODS (map, filter, fold, reduce) ===
                        if is_list && property == "map" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_mapres", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_map".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if is_list && property == "filter" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_filres", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_filter".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if is_list && property == "fold" && arguments.len() == 2 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let init_val = ctx.next_local - 1;
                            let init_i64 = ctx.alloc_local("_fldi", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: init_i64, value: MirValue::Local(init_val), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[1]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_fldres", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_fold".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(init_i64), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if is_list && property == "reduce" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_rdcres", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_reduce".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                    }

                    // === ITERATOR METHODS (lazy iteration API) ===
                    let is_iter = obj_type.as_ref().map(|t| matches!(t, MirType::Ptr(_))).unwrap_or(false);
                    if is_iter {
                        if property == "next" && arguments.is_empty() {
                            let iter_i64 = ctx.alloc_local("_it64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_next", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_next".to_string(),
                                args: vec![MirValue::Local(iter_i64)],
                            });
                            return ctx;
                        }
                        if property == "map" && arguments.len() == 1 {
                            let iter_i64 = ctx.alloc_local("_im64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_itermap", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_map".to_string(),
                                args: vec![MirValue::Local(iter_i64), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if property == "filter" && arguments.len() == 1 {
                            let iter_i64 = ctx.alloc_local("_if64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_iterfil", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_filter".to_string(),
                                args: vec![MirValue::Local(iter_i64), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if property == "collect" && arguments.is_empty() {
                            let iter_i64 = ctx.alloc_local("_ic64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_icol", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_collect".to_string(),
                                args: vec![MirValue::Local(iter_i64)],
                            });
                            return ctx;
                        }
                    }

                    // Dict method shortcuts (len, set, get)
                    let is_dict = obj_type.as_ref().map(|t| matches!(t, MirType::Dict(_, _))).unwrap_or(false);
                    if is_dict {
                        if property == "len" {
                            let result = ctx.alloc_local("_dictlen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_dict_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                    }

                    // Clone shortcut for Move types (str, list, dict)
                    if property == "clone" && arguments.is_empty() {
                        if let Some(obj_type) = &obj_type {
                            let fn_name = match obj_type {
                                MirType::Str => Some("ky_clone_str"),
                                MirType::List(_) => Some("ky_clone_list"),
                                MirType::Dict(_, _) => Some("ky_clone_dict"),
                                _ => None,
                            };
                            if let Some(fn_name) = fn_name {
                                let result = ctx.alloc_local("_clone", obj_type.clone());
                                if matches!(obj_type, MirType::Str) {
                                    ctx.string_locals.push(result);
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(result),
                                    name: fn_name.to_string(),
                                    args: vec![MirValue::Local(obj_local)],
                                });
                                return ctx;
                            }
                        }
                    }
                }

                // Handle calling through an expression (e.g. handlers[i](args)):
                // lower the target to get the function pointer, then emit CallIndirect.
                if !matches!(target.as_ref(), Expr::Identifier { .. } | Expr::PropertyAccess { .. }) {
                    ctx = self.lower_expr(ctx, target);
                    let fn_ptr_local = ctx.next_local - 1;
                    let mut args = Vec::new();
                    for arg in arguments {
                        // For struct-typed arguments, pass by pointer (not by value)
                        if let Expr::Identifier { name, .. } = arg {
                            if let Some(&local) = ctx.locals.get(name) {
                                if matches!(ctx.local_types.get(&local), Some(MirType::Struct(_, _))) {
                                    args.push(MirValue::Local(local));
                                    continue;
                                }
                            }
                        }
                        ctx = self.lower_expr(ctx, arg);
                        args.push(MirValue::Local(ctx.next_local - 1));
                    }
                    let param_types: Vec<MirType> = args.iter().map(|a| {
                        match a {
                            MirValue::Local(id) => {
                                let t = ctx.local_types.get(id).cloned().unwrap_or(MirType::I32);
                                // Struct types are passed as pointers in closures
                                if matches!(t, MirType::Struct(_, _)) { MirType::Ptr(Box::new(MirType::Void)) }
                                else { t }
                            }
                            _ => MirType::I32,
                        }
                    }).collect();
                    let dest = ctx.alloc_local("_ccall", MirType::I32);
                    ctx.current_block.insts.push(MirInst::CallIndirect {
                        dest: Some(dest), fn_ptr: fn_ptr_local,
                        ret_type: MirType::I32, param_types, args,
                    });
                    return ctx;
                }

                let name = if let Expr::Identifier { name, .. } = target.as_ref() {
                    name.clone()
                } else {
                    "_call".to_string()
                };

                // Check for closure call: if `name` refers to a local or parameter
                // (not a function declaration), emit an indirect call through the function pointer.
                if let Some(&local) = ctx.locals.get(&name) {
                    // Lower arguments, passing structs by pointer (not by value)
                    let mut args = Vec::new();
                    for arg in arguments {
                        if let Expr::Identifier { name, .. } = arg {
                            if let Some(&arg_local) = ctx.locals.get(name) {
                                if matches!(ctx.local_types.get(&arg_local), Some(MirType::Struct(_, _))) {
                                    args.push(MirValue::Local(arg_local));
                                    continue;
                                }
                            }
                        }
                        ctx = self.lower_expr(ctx, arg);
                        args.push(MirValue::Local(ctx.next_local - 1));
                    }
                    // Infer param_types: struct types become Ptr(Void) for by-ref ABI
                    let param_types: Vec<MirType> = args.iter().map(|a| {
                        match a {
                            MirValue::Local(id) => {
                                let t = ctx.local_types.get(id).cloned().unwrap_or(MirType::I32);
                                if matches!(t, MirType::Struct(_, _)) { MirType::Ptr(Box::new(MirType::Void)) }
                                else { t }
                            }
                            _ => MirType::I32,
                        }
                    }).collect();
                    let ret_type = MirType::I32;
                    let dest = ctx.alloc_local("_ccall", ret_type.clone());
                    ctx.current_block.insts.push(MirInst::CallIndirect {
                        dest: Some(dest),
                        fn_ptr: local,
                        ret_type,
                        param_types,
                        args,
                    });
                    return ctx;
                }

                // Special case: range(n) — create a list [0, 1, ..., n-1]
                if name == "range" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let count_local = ctx.next_local - 1;
                    let count_i64 = ctx.alloc_local("_cnt64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: count_i64,
                        value: MirValue::Local(count_local),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_range", MirType::List(Box::new(MirType::I64)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_range".to_string(),
                        args: vec![MirValue::Local(count_i64)],
                    });
                    return ctx;
                }
                // Special case: range(start, end) — create a list [start, start+1, ..., end-1]
                if name == "range" && arguments.len() == 2 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let start_local = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, &arguments[1]);
                    let end_local = ctx.next_local - 1;
                    let start_i64 = ctx.alloc_local("_rs64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: start_i64,
                        value: MirValue::Local(start_local),
                        to_type: MirType::I64,
                    });
                    let end_i64 = ctx.alloc_local("_re64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: end_i64,
                        value: MirValue::Local(end_local),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_range2", MirType::List(Box::new(MirType::I64)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_range_two".to_string(),
                        args: vec![MirValue::Local(start_i64), MirValue::Local(end_i64)],
                    });
                    return ctx;
                }

                // Special case: len() built-in — return string, list, or dict length
                if name == "len" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let arg_local = ctx.next_local - 1;
                    let t = ctx.local_types.get(&arg_local);
                    let is_list = t.map(|t| matches!(t, MirType::List(_))).unwrap_or(false);
                    let is_dict = t.map(|t| matches!(t, MirType::Dict(_, _))).unwrap_or(false);
                    if is_dict {
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_dict_len".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    } else if is_list {
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_list_len".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    } else {
                        let result = ctx.alloc_local("_len", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_strlen".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    }
                    return ctx;
                }

                // Special case: input() / input(prompt) built-in — read line from stdin
                if name == "input" {
                    if arguments.is_empty() {
                        let result = ctx.alloc_local("_inp", MirType::Str);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_input".to_string(),
                            args: vec![],
                        });
                        ctx.string_locals.push(result);
                        return ctx;
                    }
                }
                if name == "input" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let prompt_local = ctx.next_local - 1;
                    let prompt_len = ctx.alloc_local("_plen", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(prompt_len),
                        name: "ky_strlen".to_string(),
                        args: vec![MirValue::Local(prompt_local)],
                    });
                    let result = ctx.alloc_local("_inp", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_input_with_prompt".to_string(),
                        args: vec![MirValue::Local(prompt_local), MirValue::Local(prompt_len)],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }

                // Special case: str() built-in — convert number to string
                // Note: str(v) standalone conversion removed — use v.to_str() method instead

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
                        name: "ky_substr".to_string(),
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
                        let dest = ctx.alloc_local("_call", MirType::Void);
                        let call_name = if name == "println" { "ky_println" } else { "ky_print" };
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
                let mut resolved_name = self.class_constructor_map.borrow()
                    .get(&name).cloned().unwrap_or_else(|| name.clone());

                let mut args = Vec::new();
                // Check for default parameter values
                let decl_opt = self.function_decls.borrow().get(&resolved_name).cloned();
                if let Some(decl) = decl_opt {
                    let supplied = arguments.len();
                    for (i, param) in decl.params.iter().enumerate() {
                        if param.variadic {
                            // Pack remaining arguments into a list
                            let elem_type = ast_type_to_mir(&param.type_, Some(&ctx.struct_defs));
                            let list_type = MirType::List(Box::new(elem_type.clone()));
                            let list_handle = ctx.alloc_local("_varargs", list_type.clone());
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(list_handle),
                                name: "ky_list_new".to_string(),
                                args: vec![],
                            });
                            for j in i..supplied {
                                let arg = &arguments[j];
                                ctx = self.lower_expr(ctx, arg);
                                let val = ctx.next_local - 1;
                                let val_i64 = ctx.alloc_local("_vvi", MirType::I64);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: val_i64,
                                    value: MirValue::Local(val),
                                    to_type: MirType::I64,
                                });
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: None,
                                    name: "ky_list_push".to_string(),
                                    args: vec![MirValue::Local(list_handle), MirValue::Local(val_i64)],
                                });
                            }
                            let result = ctx.alloc_local("_vr", list_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result,
                                value: MirValue::Local(list_handle),
                            });
                            args.push(MirValue::Local(result));
                        } else if i < supplied {
                            let arg = &arguments[i];
                            // &T (MutableBorrow) params: pass address of local directly
                            if param.mode == ParamMode::MutableBorrow {
                                if let Expr::BorrowRef { expression, .. } = arg {
                                    if let Expr::Identifier { name, .. } = expression.as_ref() {
                                        if let Some(&local) = ctx.locals.get(name) {
                                            args.push(MirValue::Local(local));
                                            continue;
                                        }
                                    }
                                }
                            }
                            // Struct-typed identifiers: pass original local by reference
                            if let Expr::Identifier { name, .. } = arg {
                                if let Some(&local) = ctx.locals.get(name) {
                                    let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                    if matches!(&t, MirType::Struct(_, _)) || is_move_type(&t) {
                                        args.push(MirValue::Local(local));
                                        continue;
                                    }
                                }
                            }
                            ctx = self.lower_expr(ctx, arg);
                            args.push(MirValue::Local(ctx.next_local - 1));
                        } else if let Some(default_expr) = &param.default {
                            // Missing argument with default — lower the default expression
                            ctx = self.lower_expr(ctx, default_expr);
                            args.push(MirValue::Local(ctx.next_local - 1));
                        } else {
                            // Missing argument without default — should be caught by type checker
                            break;
                        }
                    }
                } else {
                    for arg in arguments {
                        // &x: pass address of local directly (codegen handles MutableBorrow lookup)
                        if let Expr::BorrowRef { expression, .. } = arg {
                            if let Expr::Identifier { name, .. } = expression.as_ref() {
                                if let Some(&local) = ctx.locals.get(name) {
                                    args.push(MirValue::Local(local));
                                    continue;
                                }
                            }
                        }
                        if let Expr::Identifier { name, .. } = arg {
                            if let Some(&local) = ctx.locals.get(name) {
                                let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                if matches!(&t, MirType::Struct(_, _)) || is_move_type(&t) {
                                    args.push(MirValue::Local(local));
                                    continue;
                                }
                            }
                        }
                        ctx = self.lower_expr(ctx, arg);
                        args.push(MirValue::Local(ctx.next_local - 1));
                    }
                }

                // Check if this is a call to a generic function — monomorphize lazily
                {
                    let generic_templates = self.generic_function_templates.borrow();
                    let template_opt = generic_templates.get(&resolved_name).cloned();
                    drop(generic_templates);
                    if let Some(template) = template_opt {
                        // Get argument MIR types from the already-lowered args
                        let arg_types: Vec<MirType> = args.iter().map(|a| {
                            match a {
                                MirValue::Local(id) => ctx.local_types.get(id).cloned().unwrap_or(MirType::I32),
                                _ => MirType::I32,
                            }
                        }).collect();
                        // Infer type params from concrete argument types.
                        let type_subst = infer_function_type_params(&template, &arg_types);
                        let type_args: Vec<MirType> = template.type_params.iter()
                            .map(|tp| type_subst.get(&tp.name).cloned().unwrap_or(MirType::I32))
                            .collect();
                        let specialized_name = make_concrete_name(&resolved_name, &type_args);

                        // Check if already specialized
                        if !self.fn_returns.borrow().contains_key(&specialized_name) {
                            // Clone and specialize the function AST
                            let mut specialized_decl = clone_and_specialize_function(&template, &type_subst);
                            specialized_decl.name = specialized_name.clone();

                            // Pre-register any generic struct types in the function signature
                            // so that lower_function can resolve them in struct_defs.
                            {
                                let generic_struct_tpls = self.generic_struct_templates.borrow();
                                let mut struct_defs = self.struct_defs.borrow_mut();
                                if let Some(rt) = &template.return_type {
                                    pre_register_generic_type(rt, &type_subst, &generic_struct_tpls, &mut struct_defs);
                                }
                                for p in &template.params {
                                    pre_register_generic_type(&p.type_, &type_subst, &generic_struct_tpls, &mut struct_defs);
                                }
                            }

                            // Compute specialized return type (struct_defs now has concrete types)
                            let struct_defs = self.struct_defs.borrow();
                            let ret_type = template.return_type.as_ref()
                                .map(|rt| ast_type_to_mir_with_subst(rt, Some(&struct_defs), &type_subst))
                                .unwrap_or(MirType::Void);
                            drop(struct_defs);

                            // Register return type for the specialized function
                            self.fn_returns.borrow_mut().insert(specialized_name.clone(), ret_type.clone());

                            // Lower the specialized function
                            if let Some(mir_func) = self.lower_function(&specialized_decl) {
                                self.specialized_mir_functions.borrow_mut().push(mir_func);
                            }
                        }

                        // Redirect call to the specialized function
                        resolved_name = specialized_name;
                    }
                }

                // Handle serialize(val) and deserialize<T>(str) before call_type setup
                {
                    // type<T>() — return TypeInfo for a type
                    if resolved_name == "type" && type_args.len() == 1 {
                        if let Some(first_type_arg) = type_args.first() {
                            let struct_defs = ctx.struct_defs.clone();
                            let mir_type = ast_type_to_mir(first_type_arg, Some(&struct_defs));
                            let _typeinfo = build_typeinfo_struct(&mir_type, &mut ctx);
                            return ctx;
                        }
                    }
                    // serialize(val) — class to JSON
                    if resolved_name == "serialize" && args.len() == 1 {
                        if let MirValue::Local(id) = &args[0] {
                            if let Some(MirType::Struct(_, fields)) = ctx.local_types.get(id).cloned() {
                                let descriptor = build_descriptor(&fields);
                                args.push(MirValue::Constant(MirConstant::String(descriptor)));
                                resolved_name = "ky_struct_to_json".to_string();
                            }
                        }
                    // deserialize<T>(str) — JSON to class T
                    } else if resolved_name == "deserialize" && args.len() == 1 {
                        if let Some(first_type_arg) = type_args.first() {
                            let struct_defs = ctx.struct_defs.clone();
                            let mir_type = ast_type_to_mir(first_type_arg, Some(&struct_defs));
                            if let MirType::Struct(_, fields) = &mir_type {
                                let descriptor = build_descriptor(fields);
                                let json_arg = args.remove(0);
                                let out_local = ctx.alloc_local("_dout", mir_type.clone());
                                args.push(json_arg);
                                args.push(MirValue::Constant(MirConstant::String(descriptor)));
                                args.push(MirValue::Local(out_local));
                                // Call ky_json_to_struct (returns i32, writes to out_local)
                                let ret_local = ctx.alloc_local("_dret", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(ret_local),
                                    name: "ky_json_to_struct".to_string(),
                                    args,
                                });
                                // Load result from output struct (which ky_json_to_struct wrote to)
                                let load = ctx.alloc_local("_dval", mir_type.clone());
                                ctx.current_block.insts.push(MirInst::Load {
                                    dest: load,
                                    src: out_local,
                                });
                                return ctx;
                            }
                        }
                    }
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
                        name: "ky_println".to_string(),
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
                        let is_str = ctx.string_locals.contains(id)
                            || ctx.local_types.get(id).map_or(false, |t| *t == MirType::Str);
                        if is_str {
                            // This local holds a string pointer — need to get its length
                            let len_dest = ctx.alloc_local("_strlen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(len_dest),
                                name: "ky_strlen".to_string(),
                                args: vec![MirValue::Local(*id)],
                            });
                            // Load a fresh temp for print/println (strlen above consumed *id)
                            let print_arg = ctx.alloc_local("_parg", MirType::Str);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: print_arg,
                                src: *id,
                            });
                            let call_name = if name == "println" { "ky_println" } else { "ky_print" };
                            let pret = ctx.alloc_local("_pret", call_type.clone());
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(pret),
                                name: call_name.to_string(),
                                args: vec![
                                    MirValue::Local(print_arg),
                                    MirValue::Local(len_dest),
                                ],
                            });
                            return ctx;
                        } else {
                            // Non-string argument — convert to string, then print
                            let id_type = if *id < ctx.local_types.len() {
                                ctx.local_types.get(id).cloned().unwrap_or(MirType::I64)
                            } else { MirType::I64 };
                            let str_val = if id_type == MirType::I64 || id_type == MirType::I32 {
                                // Convert integer to string via kl_i64_to_str
                                let i64_val = if id_type == MirType::I32 {
                                    let ext = ctx.alloc_local("_i64v", MirType::I64);
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: ext,
                                        value: MirValue::Local(*id),
                                        to_type: MirType::I64,
                                    });
                                    MirValue::Local(ext)
                                } else { MirValue::Local(*id) };
                                let s = ctx.alloc_local("_strv", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(s),
                                    name: "ky_i64_to_str".to_string(),
                                    args: vec![i64_val],
                                });
                                ctx.string_locals.push(s);
                                s
                            } else {
                                *id
                            };
                            let len_dest = ctx.alloc_local("_strlen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(len_dest),
                                name: "ky_strlen".to_string(),
                                args: vec![MirValue::Local(str_val)],
                            });
                            let print_arg = ctx.alloc_local("_parg", MirType::Str);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: print_arg,
                                src: str_val,
                            });
                            let call_name = if name == "println" { "ky_println" } else { "ky_print" };
                            let pret = ctx.alloc_local("_pret", call_type.clone());
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(pret),
                                name: call_name.to_string(),
                                args: vec![MirValue::Local(print_arg), MirValue::Local(len_dest)],
                            });
                            return ctx;
                        }
                    }
                }

                // Special case: ok(val) — construct success result struct
                if name == "ok" && arguments.len() == 1 {
                    let arg_val = &args[0];
                    let disc_ptr = ctx.alloc_local("_odp", MirType::I32);
                    let payload_val = ctx.alloc_local("_opv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: payload_val,
                        value: arg_val.clone(),
                        to_type: MirType::I64,
                    });
                    let payload_ptr = ctx.alloc_local("_opp", MirType::I64);
                    // Allocate struct LAST
                    let result_local = ctx.alloc_local("_okres", call_type.clone());
                    // disc = 0 (success)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: disc_ptr,
                        ptr: result_local,
                        field_index: 0,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: disc_ptr,
                        value: MirValue::Constant(MirConstant::I32(0)),
                    });
                    // payload = value (extended to i64)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: payload_ptr,
                        ptr: result_local,
                        field_index: 1,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: payload_ptr,
                        value: MirValue::Local(payload_val),
                    });
                    // Load the struct from alloca to return the value (not the pointer)
                    let result_load = ctx.alloc_local("_okres_v", call_type.clone());
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result_load,
                        src: result_local,
                    });
                    return ctx;
                }

                // Special case: error(msg) — construct error result struct
                if name == "error" && arguments.len() == 1 {
                    let arg_val = &args[0];
                    // Allocate temps FIRST, struct LAST
                    let disc_ptr = ctx.alloc_local("_edp", MirType::I32);
                    let payload_val = ctx.alloc_local("_epv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: payload_val,
                        value: arg_val.clone(),
                        to_type: MirType::I64,
                    });
                    let payload_ptr = ctx.alloc_local("_epp", MirType::I64);
                    let result_local = ctx.alloc_local("_erres", call_type.clone());
                    // disc = 1 (error)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: disc_ptr,
                        ptr: result_local,
                        field_index: 0,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: disc_ptr,
                        value: MirValue::Constant(MirConstant::I32(1)),
                    });
                    // payload = argument (cast to i64 if possible)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: payload_ptr,
                        ptr: result_local,
                        field_index: 1,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: payload_ptr,
                        value: MirValue::Local(payload_val),
                    });
                    // Load the struct from alloca to return the value (not the pointer)
                    let result_load = ctx.alloc_local("_erres_v", call_type.clone());
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result_load,
                        src: result_local,
                    });
                    return ctx;
                }

                // Special case: box(val) — heap-allocate T and store val
                if name == "box" && arguments.len() == 1 {
                    let arg_val = &args[0];
                    // Get the inner type from the argument
                    let arg_type = args.first().and_then(|id| {
                        if let MirValue::Local(id) = id {
                            ctx.local_types.get(id).cloned()
                        } else { None }
                    }).unwrap_or(MirType::I32);
                    let size = mir_type_to_size(&arg_type) as i64;
                    let ptr = ctx.alloc_local("_boxptr", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(ptr),
                        name: "ky_alloc".to_string(),
                        args: vec![MirValue::Constant(MirConstant::I64(size))],
                    });
                    // Store the value at the allocated pointer
                    let elem_type = arg_type;
                    let pointee_type = MirType::Ptr(Box::new(elem_type.clone()));
                    // Store through pointer by casting to i64 first (like PtrStore)
                    let val_i64 = ctx.alloc_local("_boxv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: arg_val.clone(),
                        to_type: MirType::I64,
                    });
                    // For now, just return the pointer as the result
                    // The user can dereference it later
                    let result = ctx.alloc_local("_box", MirType::Box(Box::new(elem_type)));
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: result,
                        value: MirValue::Local(ptr),
                    });
                    return ctx;
                }

                // Auto-JSON for client.post(url, data) where data is a class
                if name == "post" && args.len() == 2 {
                    if let MirValue::Local(id) = &args[1] {
                        if let Some(MirType::Struct(_, fields)) = ctx.local_types.get(id).cloned() {
                            let mut desc_parts: Vec<String> = Vec::new();
                            for (fname, ftype) in &fields {
                                let tn = match ftype { MirType::Str => "str", MirType::I32 => "i32", MirType::I64 => "i64", MirType::Bool => "bool", MirType::F64 => "f64", _ => "i32" };
                                desc_parts.push(format!("{}:{}", fname, tn));
                            }
                            let desc = desc_parts.join(",");
                            let data_arg = args.remove(1);
                            let ser_dest = ctx.alloc_local("_ser", MirType::Str);
                            ctx.string_locals.push(ser_dest);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(ser_dest),
                                name: "ky_struct_to_json".to_string(),
                                args: vec![data_arg, MirValue::Constant(MirConstant::String(desc))],
                            });
                            args.push(MirValue::Local(ser_dest));
                        }
                    }
                }

                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(dest),
                    name: resolved_name.clone(),
                    args,
                });
                if matches!(resolved_name.as_str(), "to_upper" | "to_lower" | "trim" | "replace" | "input" | "input_with_prompt" | "read_str" | "substr" | "json_stringify" | "serialize" | "ky_struct_to_json") {
                    ctx.string_locals.push(dest);
                }
                ctx
            }
            Expr::Assignment { target, value, .. } => {

                // Handle list[index] = value → kl_list_set
                // Handle dict[key] = value → kl_dict_set
                if let Expr::Index { target: list_expr, index, .. } = target.as_ref() {
                    // For arrays, use variable alloca directly (skip whole-array Load)
                    let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = list_expr.as_ref() {
                        if let Some(&local) = ctx.locals.get(name) {
                            let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                            if matches!(t, MirType::Array(_, _)) {
                                (local, local, t)
                            } else {
                                ctx = self.lower_expr(ctx, list_expr);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                            }
                        } else {
                            ctx = self.lower_expr(ctx, list_expr);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        }
                    } else {
                        // Nested array assignment like m[i][j] = val.
                        // Walk the index chain to emit GEPs into the root array.
                        let indices = self.collect_array_indices(target);
                        if let Some((root_name, idx_exprs)) = indices {
                            if let Some(&root_local) = ctx.locals.get(&root_name) {
                                ctx = self.lower_nested_array_geps(ctx, &idx_exprs, root_local);
                                // Emit Store directly — GEP chain already computed
                                ctx = self.lower_expr(ctx, value);
                                let val_local = ctx.next_local - 1;
                                let gep_ptr = ctx.next_local - 2;
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: gep_ptr,
                                    value: MirValue::Local(val_local),
                                });
                                return ctx;
                            } else {
                                ctx = self.lower_expr(ctx, list_expr);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                            }
                        } else {
                            ctx = self.lower_expr(ctx, list_expr);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        }
                    };
                    ctx = self.lower_expr(ctx, index);
                    let idx_val = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, value);
                    let val_local = ctx.next_local - 1;
                    let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });

                    if matches!(&target_type, MirType::Array(_, _)) {
                        let arr_ty_clone = target_type.clone();
                        let et = match &target_type {
                            MirType::Array(inner_box, _) => {
                                match inner_box.as_ref() {
                                    t => t.clone(),
                                }
                            },
                            _ => MirType::I32,
                        };
                        let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                        ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                            dest: elem_ptr,
                            ptr: arr_ptr,
                            index: MirValue::Local(idx_val),
                            arr_type: Box::new(arr_ty_clone),
                            elem_type: Box::new(et),
                        });
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: elem_ptr,
                            value: MirValue::Local(val_local),
                        });
                    } else if matches!(&target_type, MirType::Dict(_, _)) {
                        let key_arg = if let MirValue::Local(id) = MirValue::Local(idx_val) {
                            if ctx.local_types.get(&id).map(|t| *t == MirType::Str).unwrap_or(false) {
                                MirValue::Local(id)
                            } else {
                                MirValue::Local(idx_val)
                            }
                        } else {
                            MirValue::Local(idx_val)
                        };
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_dict_set".to_string(),
                            args: vec![
                                MirValue::Local(target_val),
                                key_arg,
                                MirValue::Local(val_i64),
                            ],
                        });
                    } else {
                        let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: idx_i64,
                            value: MirValue::Local(idx_val),
                            to_type: MirType::I64,
                        });
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_list_set".to_string(),
                            args: vec![
                                MirValue::Local(target_val),
                                MirValue::Local(idx_i64),
                                MirValue::Local(val_i64),
                            ],
                        });
                    }
                    return ctx;
                }

                // For struct field assignment with empty Dict {} but field is List, create list instead
                if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    if let Expr::Identifier { name, .. } = object.as_ref() {
                        if let Some(&obj_local) = ctx.locals.get(name) {
                            let obj_type = ctx.local_types.get(&obj_local).cloned();
                            if let Some(MirType::Struct(_, fields)) = &obj_type {
                                let backing = format!("_{}", property);
                                let field_idx = fields.iter().position(|(fname, _)| fname == property.as_str())
                                    .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                                if let Some(fi) = field_idx {
                                    if let Some((_, MirType::List(inner))) = fields.get(fi) {
                                        let obj_ty = obj_type.clone().unwrap();
                                        let handle = ctx.alloc_local("_listv", MirType::List(inner.clone()));
                                        ctx.current_block.insts.push(MirInst::Call {
                                            dest: Some(handle),
                                            name: "ky_list_new".to_string(),
                                            args: vec![],
                                        });
                                        let field_ptr = ctx.alloc_local("_fieldptr", MirType::I64);
                                        ctx.current_block.insts.push(MirInst::FieldPtr {
                                            dest: field_ptr,
                                            ptr: obj_local,
                                            field_index: fi,
                                            struct_type: Box::new(obj_ty),
                                        });
                                        ctx.current_block.insts.push(MirInst::Store {
                                            dest: field_ptr,
                                            value: MirValue::Local(handle),
                                        });
                                        return ctx;
                                    }
                                }
                            }
                        }
                    }
                }
                let adjusted_value = value;

                ctx = self.lower_expr(ctx, adjusted_value);
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
                    // Check for property setter: obj.prop = val → Class::set_prop(obj, val)
                    let obj_id = if let Expr::Identifier { name, .. } = object.as_ref() {
                        ctx.locals.get(name).copied()
                    } else {
                        ctx = self.lower_expr(ctx, object);
                        Some(ctx.next_local - 1)
                    };
                    if let Some(obj_l) = obj_id {
                        if let Some(MirType::Struct(class_name, _)) = ctx.local_types.get(&obj_l) {
                            let method_table = self.method_table.borrow();
                            let parent_map = self.class_parent_map.borrow();
                            let setter_name = format!("set_{}", property);
                            if let Some(mangled) = self.lookup_method_in_chain(class_name, &setter_name, &method_table, &parent_map) {
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: None,
                                    name: mangled.clone(),
                                    args: vec![MirValue::Local(obj_l), MirValue::Local(val_local)],
                                });
                                return ctx;
                            }
                        }
                    }
                    // Struct field assignment: p.x = val
                    let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                        ctx.locals.get(name).copied()
                    } else {
                        None
                    };
                    if let Some(obj_ptr) = obj_ptr {
                        let obj_type = ctx.local_types.get(&obj_ptr).cloned();
                        if let Some(MirType::Struct(_, fields)) = &obj_type {
                            let backing = format!("_{}", property);
                            let field_idx = fields.iter().position(|(fname, _)| fname == property)
                                .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                            if let Some(field_idx) = field_idx {
                                let field_ptr = ctx.alloc_local("_fieldptr", MirType::I64);
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
                } else if let Expr::Tuple { elements: target_elems, .. } = target.as_ref() {
                    // Destructuring: (x, y) = (a, b) or (x, y) = func()
                    if let Expr::Tuple { elements: value_elems, .. } = value.as_ref() {
                        for (target_elem, value_elem) in target_elems.iter().zip(value_elems.iter()) {
                            ctx = self.lower_expr(ctx, value_elem);
                            let elem_val = ctx.next_local - 1;
                            if let Expr::Identifier { name, .. } = target_elem {
                                let var_type = ctx.local_types.get(&elem_val).cloned().unwrap_or(MirType::I32);
                                let local = ctx.alloc_local(name, var_type);
                                ctx.locals.insert(name.clone(), local);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: local,
                                    value: MirValue::Local(elem_val),
                                });
                            }
                        }
                    } else {
                        // (x, y) = func() — lower func call, extract tuple elements
                        ctx = self.lower_expr(ctx, value);
                        let tuple_local = ctx.next_local - 1;
                        let tuple_type = ctx.local_types.get(&tuple_local).cloned().unwrap_or(MirType::I32);
                         if let MirType::Struct(ref _sname, ref fields) = tuple_type {
                             for (i, target_elem) in target_elems.iter().enumerate() {
                                 if let Expr::Identifier { name, .. } = target_elem {
                                     if i < fields.len() {
                                         let field_type = fields[i].1.clone();
                                         let fptr = ctx.alloc_local("_tfptr", MirType::I64);
                                         ctx.current_block.insts.push(MirInst::FieldPtr {
                                             dest: fptr, ptr: tuple_local, field_index: i,
                                             struct_type: Box::new(tuple_type.clone()),
                                         });
                                         let val = ctx.alloc_local("_tval", field_type.clone());
                                         ctx.current_block.insts.push(MirInst::Load { dest: val, src: fptr });
                                         let local = ctx.alloc_local(name, field_type);
                                         ctx.locals.insert(name.clone(), local);
                                         ctx.current_block.insts.push(MirInst::Store { dest: local, value: MirValue::Local(val) });
                                     }
                                 }
                             }
                         }
                         if let MirType::List(ref elem_type) = tuple_type {
                             for (i, target_elem) in target_elems.iter().enumerate() {
                                 if let Expr::Identifier { name, .. } = target_elem {
                                     let idx_i64 = ctx.alloc_local("_ldi", MirType::I64);
                                     ctx.current_block.insts.push(MirInst::Store {
                                         dest: idx_i64, value: MirValue::Constant(MirConstant::I64(i as i64)),
                                     });
                                     let raw = ctx.alloc_local("_ldr", MirType::I64);
                                     ctx.current_block.insts.push(MirInst::Call {
                                         dest: Some(raw), name: "ky_list_get".to_string(),
                                         args: vec![MirValue::Local(tuple_local), MirValue::Local(idx_i64)],
                                     });
                                     let field_type = elem_type.as_ref().clone();
                                     let val = if field_type != MirType::I64 {
                                         let cast = ctx.alloc_local("_ldc", field_type.clone());
                                         ctx.current_block.insts.push(MirInst::Cast { dest: cast, value: MirValue::Local(raw), to_type: field_type.clone() });
                                         cast
                                     } else { raw };
                                     let local = ctx.alloc_local(name, field_type.clone());
                                     ctx.locals.insert(name.clone(), local);
                                     ctx.current_block.insts.push(MirInst::Store { dest: local, value: MirValue::Local(val) });
                                 }
                             }
                         }
                    }
                }
                ctx
            }
            Expr::PropertyAccess { object, property, .. } => {
                // Check for enum variant without payload: Option.None
                if let Expr::Identifier { name: enum_name, .. } = object.as_ref() {
                    let ev_map = self.enum_variants.borrow();
                    if let Some(variants) = ev_map.get(enum_name) {
                        if let Some(&variant_idx) = variants.get(property) {
                            let struct_type = MirType::Struct(enum_name.clone(), vec![
                                ("disc".to_string(), MirType::I32),
                                ("payload".to_string(), MirType::I64),
                            ]);
                            let disc_ptr = ctx.alloc_local("_edp", MirType::I64);
                            let dest = ctx.alloc_local("_enum", struct_type.clone());
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: disc_ptr,
                                ptr: dest,
                                field_index: 0,
                                struct_type: Box::new(struct_type),
                            });
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: disc_ptr,
                                value: MirValue::Constant(MirConstant::I32(variant_idx as i32)),
                            });
                            return ctx;
                        }
                    }
                }
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
                            name: "ky_list_len".to_string(),
                            args: vec![MirValue::Local(obj_val)],
                        });
                        return ctx;
                    }
                    let is_dict = match object.as_ref() {
                        Expr::Identifier { name, .. } => {
                            ctx.locals.get(name)
                                .and_then(|l| ctx.local_types.get(l))
                                .map(|t| matches!(t, MirType::Dict(_, _)))
                                .unwrap_or(false)
                        }
                        _ => false,
                    };
                    if is_dict {
                        ctx = self.lower_expr(ctx, object);
                        let obj_val = ctx.next_local - 1;
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_dict_len".to_string(),
                            args: vec![MirValue::Local(obj_val)],
                        });
                        return ctx;
                    }
                }
                // Check for property getter: obj.prop → Class::get_prop(obj)
                let getter_obj = if let Expr::Identifier { name, .. } = object.as_ref() {
                    ctx.locals.get(name).copied()
                } else { None };
                if let Some(go) = getter_obj {
                    if let Some(MirType::Struct(sname, _)) = ctx.local_types.get(&go) {
                        let method_table = self.method_table.borrow();
                        let parent_map = self.class_parent_map.borrow();
                        if let Some(mangled) = self.lookup_method_in_chain(sname, &format!("get_{}", property), &method_table, &parent_map) {
                            let call_type = self.fn_returns.borrow()
                                .get(&mangled).cloned().unwrap_or(MirType::Void);
                            let dest = ctx.alloc_local("_getter", call_type.clone());
                            if call_type == MirType::Str {
                                ctx.string_locals.push(dest);
                            }
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(dest),
                                name: mangled.clone(),
                                args: vec![MirValue::Local(go)],
                            });
                            return ctx;
                        }
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
                    // Unwrap Ptr(Struct(...)) for closure-inferred types
                    let (struct_name, struct_fields, _is_ptr) = match &obj_type {
                        Some(MirType::Struct(sname, fields)) => (Some(sname.clone()), Some(fields.clone()), false),
                        Some(MirType::Ptr(inner)) => match inner.as_ref() {
                            MirType::Struct(sname, fields) => (Some(sname.clone()), Some(fields.clone()), true),
                            _ => (None, None, false),
                        },
                        _ => (None, None, false),
                    };
                    // Resolve the struct name and fields, searching all struct_defs
                    // if the type is a generic pointer (e.g., closure-inferred Ptr(I8)).
                    let (sname, resolved_fields) = if let (Some(sn), Some(fields)) = (struct_name, struct_fields) {
                        let rf = if fields.is_empty() {
                            ctx.struct_defs.get(&sn).cloned().unwrap_or_default()
                        } else {
                            fields
                        };
                        (sn, rf)
                    } else {
                        // Fallback: scan all struct_defs for matching property
                        let mut found = None;
                        for (sn, sfields) in &ctx.struct_defs {
                            if sfields.iter().any(|(fn_, _)| fn_ == property || fn_ == &format!("_{}", property)) {
                                found = Some((sn.clone(), sfields.clone()));
                                break;
                            }
                        }
                        if let Some(result) = found { result }
                        else { return ctx; }
                    };
                    let backing = format!("_{}", property);
                    let field_idx = resolved_fields.iter().position(|(fname, _)| fname == property)
                        .or_else(|| resolved_fields.iter().position(|(fname, _)| fname == &backing));
                    if let Some(field_idx) = field_idx {
                        let field_type = resolved_fields[field_idx].1.clone();
                        let field_ptr = ctx.alloc_local("_fieldptr", MirType::I64);
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: field_ptr,
                            ptr: obj_ptr,
                            field_index: field_idx,
                            struct_type: Box::new(MirType::Struct(sname.clone(), resolved_fields)),
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
                ctx
            }
            Expr::OptionalChain { target, property, .. } => {
                ctx = self.lower_expr(ctx, target);
                let target_local = ctx.next_local - 1;
                let target_type = ctx.local_types.get(&target_local).cloned()
                    .unwrap_or(MirType::Struct("Result".to_string(), vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]));

                let result_struct = ctx.alloc_local("_ochain", target_type.clone());

                // Get discriminant
                let disc_ptr = ctx.alloc_local("_odp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: disc_ptr,
                    ptr: target_local,
                    field_index: 0,
                    struct_type: Box::new(target_type.clone()),
                });
                let disc = ctx.alloc_local("_odv", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: disc,
                    src: disc_ptr,
                });

                let some_block = ctx.fresh_block();
                let none_block = ctx.fresh_block();
                let merge_block = ctx.fresh_block();

                let is_none = ctx.alloc_local("_oneq", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: is_none,
                    op: MirBinaryOp::Eq,
                    left: MirValue::Local(disc),
                    right: MirValue::Constant(MirConstant::I32(0)),
                });
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(is_none),
                    true_block: none_block.clone(),
                    false_block: some_block.clone(),
                });

                // None block: set disc=0, branch to merge
                ctx.current_block = MirBasicBlock::new(none_block);
                let rn_disc_ptr = ctx.alloc_local("_rndp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: rn_disc_ptr,
                    ptr: result_struct,
                    field_index: 0,
                    struct_type: Box::new(target_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: rn_disc_ptr,
                    value: MirValue::Constant(MirConstant::I32(0)),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                // Some block: extract payload, access property, wrap in Some
                ctx.current_block = MirBasicBlock::new(some_block);
                let src_payload_ptr = ctx.alloc_local("_spp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: src_payload_ptr,
                    ptr: target_local,
                    field_index: 1,
                    struct_type: Box::new(target_type.clone()),
                });
                let payload_val = ctx.alloc_local("_spv", MirType::I64);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: payload_val,
                    src: src_payload_ptr,
                });

                // Access property on the inner struct
                // Strategy: look for the inner struct either by name (mangled) or by field search
                let result_payload = if let MirType::Struct(struct_name, _) = &target_type {
                    // Try mangled name first: "Option__Person" → "Person"
                    let inner_by_name = struct_name.split("__").nth(1)
                        .and_then(|n| ctx.struct_defs.get(n))
                        .and_then(|fields| fields.iter().position(|(fn_, _)| fn_ == property))
                        .map(|field_idx| (struct_name.split("__").nth(1).unwrap().to_string(), field_idx));

                    // Try search-based approach: find any struct that has this field
                    let inner_by_search = ctx.struct_defs.iter()
                        .find(|(_, fields)| fields.iter().any(|(fn_, _)| fn_ == property))
                        .and_then(|(name, fields)| {
                            fields.iter().position(|(fn_, _)| fn_ == property)
                                .map(|field_idx| (name.clone(), field_idx))
                        });

                    if let Some((inner_name, field_idx)) = inner_by_name.or(inner_by_search) {
                        if let Some(inner_fields) = ctx.struct_defs.get(&inner_name) {
                            let field_type = inner_fields[field_idx].1.clone();
                            let inner_mir = MirType::Struct(inner_name, inner_fields.clone());
                            let struct_val = ctx.alloc_local("_och_s", inner_mir.clone());
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: struct_val,
                                value: MirValue::Local(payload_val),
                                to_type: inner_mir.clone(),
                            });
                            let field_ptr = ctx.alloc_local("_och_fp", field_type.clone());
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: field_ptr,
                                ptr: struct_val,
                                field_index: field_idx,
                                struct_type: Box::new(inner_mir),
                            });
                            let field_val = ctx.alloc_local("_och_fv", field_type.clone());
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: field_val,
                                src: field_ptr,
                            });
                            if field_type == MirType::Str {
                                ctx.string_locals.push(field_val);
                            }
                            if field_type != MirType::I64 {
                                let casted = ctx.alloc_local("_och_c", MirType::I64);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: casted,
                                    value: MirValue::Local(field_val),
                                    to_type: MirType::I64,
                                });
                                MirValue::Local(casted)
                            } else {
                                MirValue::Local(field_val)
                            }
                        } else {
                            MirValue::Local(payload_val)
                        }
                    } else {
                        MirValue::Local(payload_val)
                    }
                } else {
                    MirValue::Local(payload_val)
                };

                // Set disc=1 and store payload
                let r_disc_ptr = ctx.alloc_local("_rdp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: r_disc_ptr,
                    ptr: result_struct,
                    field_index: 0,
                    struct_type: Box::new(target_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: r_disc_ptr,
                    value: MirValue::Constant(MirConstant::I32(1)),
                });
                let r_payload_ptr = ctx.alloc_local("_rpp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: r_payload_ptr,
                    ptr: result_struct,
                    field_index: 1,
                    struct_type: Box::new(target_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: r_payload_ptr,
                    value: result_payload,
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                ctx.current_block = MirBasicBlock::new(merge_block);
                // Load result struct as the expression's final value
                let result_type = ctx.local_types.get(&result_struct).cloned()
                    .unwrap_or(target_type.clone());
                let result_val = ctx.alloc_local("_och_res", result_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result_val,
                    src: result_struct,
                });
                // Ensure string tracking for Option<T> where T=Str

                ctx
            }
            Expr::ErrorProp { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                let result_local = ctx.next_local - 1;
                let result_type = ctx.local_types.get(&result_local).cloned()
                    .unwrap_or(MirType::Struct("Result".to_string(), vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]));

                // Get discriminant (field 0)
                let disc_ptr = ctx.alloc_local("_edp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: disc_ptr,
                    ptr: result_local,
                    field_index: 0,
                    struct_type: Box::new(result_type.clone()),
                });
                let disc = ctx.alloc_local("_edv", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: disc,
                    src: disc_ptr,
                });

                // Check if disc == 0 (error / None)
                let is_error = ctx.alloc_local("_eeq", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: is_error,
                    op: MirBinaryOp::Eq,
                    left: MirValue::Local(disc),
                    right: MirValue::Constant(MirConstant::I32(0)),
                });

                let error_block = ctx.fresh_block();
                let continue_block = ctx.fresh_block();
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(is_error),
                    true_block: error_block.clone(),
                    false_block: continue_block.clone(),
                });

                // Error block: early-return the struct as-is (already has disc = 0)
                ctx.current_block = MirBasicBlock::new(error_block);
                ctx.finish_block(MirTerminator::Return(MirValue::Local(result_local)));

                // Continue block: extract payload (field 1)
                ctx.current_block = MirBasicBlock::new(continue_block);
                let payload_ptr = ctx.alloc_local("_epp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: payload_ptr,
                    ptr: result_local,
                    field_index: 1,
                    struct_type: Box::new(result_type),
                });
                let payload = ctx.alloc_local("_epv", MirType::I64);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: payload,
                    src: payload_ptr,
                });
                ctx
            }
            Expr::StringInterp { parts, .. } => {
                let mut str_locals: Vec<usize> = Vec::new();
                for part in parts {
                    ctx = self.lower_expr(ctx, part);
                    let val_local = ctx.next_local - 1;
                    let is_str = ctx.local_types.get(&val_local).map_or(false, |t| *t == MirType::Str);
                    if is_str {
                        str_locals.push(val_local);
                    } else {
                        let cast_local = ctx.alloc_local("_cast64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: cast_local,
                            value: MirValue::Local(val_local),
                            to_type: MirType::I64,
                        });
                        let str_local = ctx.alloc_local("_strptr", MirType::Str);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(str_local),
                            name: "ky_i64_to_str".to_string(),
                            args: vec![MirValue::Local(cast_local)],
                        });
                        ctx.string_locals.push(str_local);
                        str_locals.push(str_local);
                    }
                }
                let mut result = str_locals[0];
                for i in 1..str_locals.len() {
                    let left = result;
                    let right = str_locals[i];
                    let left_len = ctx.alloc_local("_strlen", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(left_len),
                        name: "ky_strlen".to_string(),
                        args: vec![MirValue::Local(left)],
                    });
                    let right_len = ctx.alloc_local("_strlen", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(right_len),
                        name: "ky_strlen".to_string(),
                        args: vec![MirValue::Local(right)],
                    });
                    let new_result = ctx.alloc_local("_sinterp", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(new_result),
                        name: "ky_concat".to_string(),
                        args: vec![
                            MirValue::Local(left),
                            MirValue::Local(left_len),
                            MirValue::Local(right),
                            MirValue::Local(right_len),
                        ],
                    });
                    ctx.string_locals.push(new_result);
                    result = new_result;
                }
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
                    name: "ky_list_new".to_string(),
                    args: vec![],
                });
                for elem in elements {
                    // Handle spread operator: [...list, new_elem]
                    if let Expr::Spread { expression, .. } = elem {
                        ctx = self.lower_expr(ctx, expression);
                        let spread_val = ctx.next_local - 1;
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_list_extend".to_string(),
                            args: vec![MirValue::Local(handle), MirValue::Local(spread_val)],
                        });
                    } else {
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
                            name: "ky_list_push".to_string(),
                            args: vec![MirValue::Local(handle), MirValue::Local(val_i64)],
                        });
                    }
                }
                let result = ctx.alloc_local("_listv", MirType::List(Box::new(elem_type)));
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result,
                    value: MirValue::Local(handle),
                });
                ctx
            }
            Expr::Index { target, index, .. } => {
                // For arrays, avoid lowering the target (which generates unnecessary whole-array Load)
                let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = target.as_ref() {
                    if let Some(&local) = ctx.locals.get(name) {
                        let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                        // Only skip Load for arrays and slices (zero-copy GEP). For other types, use normal expression lowering.
                        if matches!(t, MirType::Array(_, _) | MirType::Slice(_)) {
                            // Use variable's alloca directly (no Load)
                            (local, local, t)
                        } else {
                            ctx = self.lower_expr(ctx, target);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        }
                    } else {
                        ctx = self.lower_expr(ctx, target);
                        let tv = ctx.next_local - 1;
                        (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                    }
                } else {
                    ctx = self.lower_expr(ctx, target);
                    let tv = ctx.next_local - 1;
                    let t = ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32);
                    // For non-identity array targets (e.g., nested mat[i][j]), the value
                    // needs a temp alloca so ArrayElemPtr can GEP into it
                    if matches!(t, MirType::Array(_, _)) {
                        let arr_tmp = ctx.alloc_local("_arrtmp", t.clone());
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: arr_tmp,
                            value: MirValue::Local(tv),
                        });
                        (tv, arr_tmp, t)
                    } else {
                        (tv, tv, t)
                    }
                };
                ctx = self.lower_expr(ctx, index);
                let index_val = ctx.next_local - 1;
                let target_type = ctx.local_types.get(&target_val).cloned().unwrap_or(MirType::I32);
                if target_type == MirType::Str {
                    // String indexing: source[i] -> substr(source, i, 1) -> returns str
                    let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: idx_i64,
                        value: MirValue::Local(index_val),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_substr", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_substr".to_string(),
                        args: vec![
                            MirValue::Local(target_val),
                            MirValue::Local(idx_i64),
                            MirValue::Constant(MirConstant::I64(1)),
                        ],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }
                if matches!(&target_type, MirType::Array(_, _)) {
                    let et = match &target_type { MirType::Array(e, _) => *(e.clone()), _ => MirType::I32 };
                    let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                        dest: elem_ptr,
                        ptr: arr_ptr,
                        index: MirValue::Local(index_val),
                        arr_type: Box::new(target_type.clone()),
                        elem_type: Box::new(et.clone()),
                    });
                    let loaded = ctx.alloc_local("_aelem_val", et);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: loaded,
                        src: elem_ptr,
                    });
                    return ctx;
                }
                // === SLICE INDEXING ===
                if let MirType::Slice(inner) = &target_type {
                    let et = *inner.clone();
                    let slice_type = MirType::Slice(inner.clone());
                    // Get ptr field (field 0) from slice struct
                    let ptr_field = ctx.alloc_local("_sptrf", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: ptr_field,
                        ptr: target_val,
                        field_index: 0,
                        struct_type: Box::new(slice_type.clone()),
                    });
                    let base_ptr = ctx.alloc_local("_sbase", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: base_ptr,
                        src: ptr_field,
                    });
                    // GEP: base_ptr + index
                    let index_i64 = ctx.alloc_local("_si64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: index_i64,
                        value: MirValue::Local(index_val),
                        to_type: MirType::I64,
                    });
                    let elem_ptr = ctx.alloc_local("_selem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                    let et_clone = et.clone();
                    ctx.current_block.insts.push(MirInst::PtrOffset {
                        dest: elem_ptr,
                        ptr: base_ptr,
                        index: MirValue::Local(index_i64),
                        elem_type: Box::new(et_clone),
                    });
                    let loaded = ctx.alloc_local("_selem_val", et);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: loaded,
                        src: elem_ptr,
                    });
                    return ctx;
                }
                if let MirType::Dict(_, v) = &target_type {
                    let key_arg = if matches!(ctx.local_types.get(&index_val), Some(MirType::Str)) {
                        MirValue::Local(index_val)
                    } else {
                        // Index must be a string for dict access
                        return ctx;
                    };
                    let result = ctx.alloc_local("_dict_idx", MirType::I64);

                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_dict_get".to_string(),
                        args: vec![MirValue::Local(target_val), key_arg],
                    });
                    let elem_type = v.as_ref().clone();
                    if elem_type == MirType::Str {
                        let str_res = ctx.alloc_local("_dict_idx_str", MirType::Str);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: str_res,
                            value: MirValue::Local(result),
                            to_type: MirType::Str,
                        });
                        ctx.string_locals.push(str_res);
                    } else if elem_type != MirType::I64 {
                        let casted = ctx.alloc_local("_dict_idx_cast", elem_type.clone());
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: casted,
                            value: MirValue::Local(result),
                            to_type: elem_type,

                        });
                    }
                    return ctx;
                }
                // Ptr type indexing: ptr[index] → PtrOffset + Load
                if matches!(target_type, MirType::Ptr(_)) {
                    let ptr_type = ctx.local_types.get(&target_val).cloned().unwrap_or(MirType::I64);
                    let elem_type = if target_type == MirType::I64 { MirType::I8 } else { MirType::I8 };
                    let elem_type2 = elem_type.clone();
                    let offset = ctx.alloc_local("_ptroff", ptr_type.clone());
                    ctx.current_block.insts.push(MirInst::PtrOffset {
                        dest: offset,
                        ptr: target_val,
                        index: MirValue::Local(index_val),
                        elem_type: Box::new(elem_type2),
                    });
                    let result = ctx.alloc_local("_ptrload", elem_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result,
                        src: offset,
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
                    name: "ky_list_get".to_string(),
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
                // Determine value type from entries
                let val_type = entries.first()
                    .and_then(|(_, v)| {
                        if let Expr::Literal { value: Literal::String(_), .. } = v { Some(MirType::Str) }
                        else if let Expr::Literal { value: Literal::Integer(_), .. } = v { Some(MirType::I64) }
                        else { None }
                    })
                    .unwrap_or(MirType::I64);
                let dict_type = MirType::Dict(Box::new(MirType::Str), Box::new(val_type));
                // Allocate dict via runtime call (into ptr-typed temp, then store to typed alloca)
                let handle = ctx.alloc_local("_dict_raw", MirType::Ptr(Box::new(MirType::Void)));
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(handle),
                    name: "ky_dict_new".to_string(),
                    args: vec![],
                });
                // Insert each entry
                for (key_str, val_expr) in entries {
                    ctx = self.lower_expr(ctx, val_expr);
                    let val_local = ctx.next_local - 1;

                    let val_i64 = ctx.alloc_local("_dv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: None,
                        name: "ky_dict_set".to_string(),
                        args: vec![
                            MirValue::Local(handle),
                            MirValue::Constant(MirConstant::String(key_str.clone())),

                            MirValue::Local(val_i64),
                        ],
                    });
                }
                // Allocate a correctly-typed local as the final result
                let result = ctx.alloc_local("_dict", dict_type);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result,
                    value: MirValue::Local(handle),

                });
                ctx
            }
            Expr::StructLiteral { struct_name, type_args: _, fields, .. } => {
                let struct_defs = ctx.struct_defs.clone();
                // Check if this is a generic struct template
                let generic_struct: Option<StructDecl> = (!struct_defs.contains_key(struct_name.as_str()))
                    .then(|| {
                        let templates = self.generic_struct_templates.borrow();
                        templates.get(struct_name.as_str()).cloned()
                    })
                    .flatten();

                if let Some(tpl) = generic_struct {
                    // --- Generic struct: monomorphize on the fly ---
                    // First, lower all field expressions to get their concrete MIR types
                    let mut field_val_ids: Vec<usize> = Vec::new();
                    for (_, field_expr) in fields {
                        ctx = self.lower_expr(ctx, field_expr);
                        field_val_ids.push(ctx.next_local - 1);
                    }
                    // Infer type params from concrete field value types
                    let mut type_subst: std::collections::HashMap<String, MirType> = std::collections::HashMap::new();
                    for ((field_name, _), val_id) in fields.iter().zip(&field_val_ids) {
                        if let Some(tf) = tpl.fields.iter().find(|f| f.name == *field_name) {
                            if let Some(concrete_type) = ctx.local_types.get(val_id) {
                                for tp in &tpl.type_params {
                                    if is_type_ref(&tf.type_, &tp.name) {
                                        type_subst.entry(tp.name.clone()).or_insert_with(|| concrete_type.clone());
                                    }
                                }
                            }
                        }
                    }
                    // Build ordered type args matching template's type_params order
                    let type_args: Vec<MirType> = tpl.type_params.iter()
                        .map(|tp| type_subst.get(&tp.name).cloned().unwrap_or(MirType::I32))
                        .collect();
                    let concrete_name = make_concrete_name(struct_name, &type_args);
                    // Create concrete field types with substitution and register in struct_defs
                    let struct_defs = ctx.struct_defs.clone(); // Use struct_defs from ctx for field resolution
                    let concrete_fields: Vec<(String, MirType)> = tpl.fields.iter()
                        .map(|f| (f.name.clone(), ast_type_to_mir_with_subst(&f.type_, Some(&struct_defs), &type_subst)))
                        .collect();
                    // Register in Lowerer's struct_defs
                    self.struct_defs.borrow_mut().insert(concrete_name.clone(), concrete_fields.clone());
                    // Now create the struct alloca and store fields
                    let struct_type = MirType::Struct(concrete_name.clone(), concrete_fields.clone());
                    let struct_local = ctx.alloc_local("_slit", struct_type.clone());
                    for ((field_name, _), val_local) in fields.iter().zip(&field_val_ids) {
                        if let Some(field_idx) = concrete_fields.iter().position(|(fn_, _)| *fn_ == *field_name) {
                            let fptr = ctx.alloc_local("_sfptr", MirType::Void);
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: fptr,
                                ptr: struct_local,
                                field_index: field_idx,
                                struct_type: Box::new(MirType::Struct(concrete_name.clone(), concrete_fields.clone())),
                            });
                            let field_type = concrete_fields[field_idx].1.clone();
                            let cast_val = if field_type != ctx.local_types.get(val_local).cloned().unwrap_or(MirType::I64) {
                                let cast_local = ctx.alloc_local("_scast", field_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: cast_local,
                                    value: MirValue::Local(*val_local),
                                    to_type: field_type,
                                });
                                MirValue::Local(cast_local)
                            } else {
                                MirValue::Local(*val_local)
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
                } else if let Some(def_fields) = struct_defs.get(struct_name.as_str()) {
                    // --- Non-generic struct ---
                    let def_fields = def_fields.clone();
                    let struct_type = MirType::Struct(struct_name.clone(), def_fields.clone());
                    let struct_local = ctx.alloc_local("_slit", struct_type.clone());
                    let mut provided_fields: std::collections::HashSet<String> = std::collections::HashSet::new();
                    for (field_name, field_expr) in fields {
                        ctx = self.lower_expr(ctx, field_expr);
                        let val_local = ctx.next_local - 1;
                        if let Some(field_idx) = def_fields.iter()
                            .position(|(fname, _)| fname == field_name)
                        {
                            provided_fields.insert(field_name.clone());
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
                    // Apply field defaults for any missing fields
                    {
                        let fds = self.field_defaults.borrow();
                        if let Some(field_defaults) = fds.get(struct_name.as_str()) {
                            for (field_idx, (fn_name, _)) in def_fields.iter().enumerate() {
                                if !provided_fields.contains(fn_name) {
                                    if let Some(default_expr) = field_defaults.get(fn_name) {
                                        ctx = self.lower_expr(ctx, default_expr);
                                        let val_local = ctx.next_local - 1;
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
                            }
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
                if elements.len() <= 1 {
                    // Single-element tuple is just the element
                    if elements.is_empty() {
                        ctx
                    } else {
                        ctx = self.lower_expr(ctx, &elements[0]);
                        ctx
                    }
                } else {
                    // Multi-element tuple: build a struct
                    let mut elem_ids = Vec::new();
                    for elem in elements {
                        ctx = self.lower_expr(ctx, elem);
                        elem_ids.push(ctx.next_local - 1);
                    }
                    // Build struct type from element types
                    let field_types: Vec<(String, MirType)> = elem_ids.iter().enumerate()
                        .map(|(i, id)| (format!("_{}", i), ctx.local_types.get(id).cloned().unwrap_or(MirType::I32)))
                        .collect();
                    let type_suffix: String = field_types.iter()
                        .map(|(_, t)| match t {
                            MirType::I32 => "i32",
                            MirType::I64 => "i64",
                            MirType::Str => "str",
                            MirType::Bool => "bool",
                            MirType::F64 => "f64",
                            _ => "x",
                        })
                        .collect();
                    let struct_type = MirType::Struct(format!("_tuple_{}_{}", elements.len(), type_suffix), field_types.clone());
                    let struct_local = ctx.alloc_local("_tup", struct_type.clone());
                    for (i, elem_id) in elem_ids.iter().enumerate() {
                        let fptr = ctx.alloc_local("_tfptr", MirType::I64);
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: fptr,
                            ptr: struct_local,
                            field_index: i,
                            struct_type: Box::new(struct_type.clone()),
                        });
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: fptr,
                            value: MirValue::Local(*elem_id),
                        });
                    }
                    // Load the struct as the expression result (last local)
                    let load_local = ctx.alloc_local("_tload", struct_type.clone());
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: load_local,
                        src: struct_local,
                    });
                    ctx
                }
            }
            Expr::Closure { params, body, .. } => {
                let mut counter = self.closure_counter.borrow_mut();
                let fn_name = format!("_closure_{}", *counter);
                *counter += 1;
                drop(counter);

                let mut mir_func = MirFunction::new(&fn_name);
                // Use type annotations from AST, fall back to inference
                let param_types: Vec<MirType> = params.iter()
                    .map(|(p, t)| param_type_from_annotation(t)
                        .unwrap_or_else(|| infer_closure_param_type(p, body)))
                    .collect();
                mir_func.params = param_types.clone();
                mir_func.return_type = MirType::I32; // default, will be inferred

                let mut cctx = LowerCtx::new();
                cctx.struct_defs = ctx.struct_defs.clone();
                // Bind params to locals with inferred types
                for (i, (pname, _)) in params.iter().enumerate() {
                    let pt = param_types[i].clone();
                    let local = cctx.alloc_local(pname, pt.clone());
                    cctx.current_block.insts.push(MirInst::Store {
                        dest: local,
                        value: MirValue::Param(i),
                    });
                    cctx.locals.insert(pname.clone(), local);
                    // Record type for use in body lowering
                    cctx.local_types.insert(local, pt.clone());
                }
                // Lower body expression
                cctx = self.lower_expr(cctx, body);
                let val_local = cctx.next_local - 1;
                // Infer return type if needed
                if mir_func.return_type == MirType::I32 {
                    if let Some(actual_type) = cctx.local_types.get(&val_local) {
                        mir_func.return_type = actual_type.clone();
                    }
                }
                cctx.emit_return(MirValue::Local(val_local));
                mir_func.local_count = cctx.next_local;
                mir_func.basic_blocks = cctx.blocks;

                // Store the closure function
                self.closure_functions.borrow_mut().push(mir_func);

                // Emit FnAddr to get the function pointer
                let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                let dest = ctx.alloc_local("_closure", ptr_type);
                ctx.current_block.insts.push(MirInst::FnAddr {
                    dest,
                    name: fn_name,
                });
                ctx
            }
            Expr::Await { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                let handle_local = ctx.next_local - 1;
                let result = ctx.alloc_local("_await", MirType::I64);
                ctx.current_block.insts.push(MirInst::AsyncAwait {
                    dest: result,
                    handle: handle_local,
                });
                // Cast i64 back to the expected result type
                let result_type = ctx.local_types.get(&result).cloned().unwrap_or(MirType::I64);
                if result_type != MirType::I64 {
                    let cast = ctx.alloc_local("_awaitcast", result_type.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(result),
                        to_type: result_type,
                    });
                }
                ctx
            }
            Expr::Async { expression, .. } => {
                let mut counter = self.async_counter.borrow_mut();
                let fn_name = format!("_async_{}", *counter);
                *counter += 1;
                drop(counter);

                let mut mir_func = MirFunction::new(&fn_name);
                mir_func.params = vec![];
                mir_func.return_type = MirType::I64;

                let mut cctx = LowerCtx::new();
                cctx.struct_defs = self.struct_defs.borrow().clone();
                cctx = self.lower_expr(cctx, expression);
                let val_local = cctx.next_local - 1;
                // Widen to i64
                let val_type = cctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I32);
                let ret_local = if val_type != MirType::I64 {
                    let widened = cctx.alloc_local("_widen", MirType::I64);
                    cctx.current_block.insts.push(MirInst::Cast {
                        dest: widened,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });
                    widened
                } else {
                    val_local
                };
                cctx.emit_return(MirValue::Local(ret_local));
                mir_func.local_count = cctx.next_local;
                mir_func.basic_blocks = cctx.blocks;

                self.async_functions.borrow_mut().push(mir_func);

                let dest = ctx.alloc_local("_async_h", MirType::I64);
                ctx.current_block.insts.push(MirInst::AsyncSpawn {
                    dest,
                    function_name: fn_name,
                    arg: MirValue::Constant(MirConstant::I64(0)),
                });
                ctx
            }
            Expr::AsyncBlock { body, .. } => {
                // async: block — generate a zero-param function and spawn it
                let mut counter = self.async_counter.borrow_mut();
                let fn_name = format!("_async_block_{}", *counter);
                *counter += 1;
                drop(counter);

                let mut mir_func = MirFunction::new(&fn_name);
                mir_func.params = vec![];
                mir_func.return_type = MirType::I64;

                let mut cctx = LowerCtx::new();
                cctx.struct_defs = self.struct_defs.borrow().clone();

                let last_is_tail = matches!(body.statements.last(), Some(Stmt::Expression(_)));
                let stmt_count = body.statements.len();
                for (i, stmt) in body.statements.iter().enumerate() {
                    if i + 1 == stmt_count {
                        if let Stmt::If(_) = stmt {
                            cctx.tail_if_as_return = true;
                        }
                    }
                    cctx = self.lower_stmt(cctx, stmt);
                }

                if cctx.current_block.terminator == MirTerminator::Unreachable {
                    if last_is_tail {
                        let val_local = cctx.next_local.checked_sub(1);
                        if let Some(vl) = val_local {
                            let vt = cctx.local_types.get(&vl).cloned().unwrap_or(MirType::I32);
                            if vt != MirType::I64 {
                                let w = cctx.alloc_local("_bw", MirType::I64);
                                cctx.current_block.insts.push(MirInst::Cast {
                                    dest: w, value: MirValue::Local(vl), to_type: MirType::I64,
                                });
                                cctx.emit_return(MirValue::Local(w));
                            } else {
                                cctx.emit_return(MirValue::Local(vl));
                            }
                        } else {
                            cctx.emit_return(MirValue::Constant(MirConstant::I64(0)));
                        }
                    } else {
                        cctx.emit_return(MirValue::Constant(MirConstant::I64(0)));
                    }
                }
                mir_func.local_count = cctx.next_local;
                mir_func.basic_blocks = cctx.blocks;
                self.async_functions.borrow_mut().push(mir_func);

                let dest = ctx.alloc_local("_async_h", MirType::I64);
                ctx.current_block.insts.push(MirInst::AsyncSpawn {
                    dest, function_name: fn_name,
                    arg: MirValue::Constant(MirConstant::I64(0)),
                });
                ctx
            }
            Expr::Spread { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                ctx
            }
            Expr::RangeSlice { target, start, end, .. } => {
                // For arrays, don't lower expression (use alloca directly)
                let (target_val, target_type) = if let Expr::Identifier { name, .. } = target.as_ref() {
                    if let Some(&local) = ctx.locals.get(name) {
                        let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                        if matches!(t, MirType::Array(_, _)) {
                            (local, t)
                        } else {
                            ctx = self.lower_expr(ctx, target);
                            (ctx.next_local - 1, ctx.local_types.get(&(ctx.next_local - 1)).cloned().unwrap_or(MirType::I32))
                        }
                    } else {
                        ctx = self.lower_expr(ctx, target);
                        (ctx.next_local - 1, ctx.local_types.get(&(ctx.next_local - 1)).cloned().unwrap_or(MirType::I32))
                    }
                } else {
                    ctx = self.lower_expr(ctx, target);
                    (ctx.next_local - 1, ctx.local_types.get(&(ctx.next_local - 1)).cloned().unwrap_or(MirType::I32))
                };
                let start_i64 = if let Some(s) = start {
                    ctx = self.lower_expr(ctx, s);
                    let val = ctx.next_local - 1;
                    let cast = ctx.alloc_local("_sli64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(val),
                        to_type: MirType::I64,
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Constant(MirConstant::I64(0))
                };
                let end_i64 = if let Some(e) = end {
                    ctx = self.lower_expr(ctx, e);
                    let val = ctx.next_local - 1;
                    let cast = ctx.alloc_local("_eli64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(val),
                        to_type: MirType::I64,
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Constant(MirConstant::I64(-1))
                };
                // Handle array ranges: produce &[T] slice
                if let MirType::Array(inner, _) = &target_type {
                    // Get element pointer via ArrayElemPtr
                    let start_i32 = ctx.alloc_local("_sli32", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: start_i32,
                        value: start_i64.clone(),
                        to_type: MirType::I32,
                    });
                    let elem_ptr = ctx.alloc_local("_sptr", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                        dest: elem_ptr,
                        ptr: target_val,
                        index: MirValue::Local(start_i32),
                        arr_type: Box::new(target_type.clone()),
                        elem_type: inner.clone(),
                    });
                    // Compute length: end - start (or arr_len for full slice)
                    let len = ctx.alloc_local("_slen", MirType::I64);
                    let len_i64 = if let Some(_) = end {
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: len,
                            op: MirBinaryOp::Sub,
                            left: end_i64,
                            right: start_i64.clone(),
                        });
                        MirValue::Local(len)
                    } else if let MirType::Array(_, size) = &target_type {
                        MirValue::Constant(MirConstant::I64(*size as i64))
                    } else {
                        MirValue::Constant(MirConstant::I64(-1))
                    };
                    let result = ctx.alloc_local("_slice", MirType::Slice(inner.clone()));
                    ctx.current_block.insts.push(MirInst::SliceMake {
                        dest: result,
                        ptr: MirValue::Local(elem_ptr),
                        len: len_i64,
                        elem_type: inner.clone(),
                    });
                    return ctx;
                }
                let result = ctx.alloc_local("_slice", MirType::List(Box::new(MirType::I64)));
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(result),
                    name: "ky_list_slice".to_string(),
                    args: vec![
                        MirValue::Local(target_val),
                        start_i64,
                        end_i64,
                    ],
                });
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
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                let result_local = ctx.alloc_local("_tern_res", MirType::I64);
                ctx = self.lower_expr(ctx, cond);
                let cond_val = MirValue::Local(ctx.next_local - 1);
                let then_block = ctx.fresh_block();
                let else_block = ctx.fresh_block();
                let merge_block = ctx.fresh_block();
                ctx.finish_block(MirTerminator::CondBr {
                    cond: cond_val,
                    true_block: then_block.clone(),
                    false_block: else_block.clone(),
                });
                // Then block
                ctx.current_block = MirBasicBlock::new(then_block.clone());
                ctx = self.lower_expr(ctx, then_expr);
                let then_val = ctx.next_local - 1;
                let then_type = ctx.local_types.get(&then_val).cloned().unwrap_or(MirType::I64);
                ctx.local_types.insert(result_local, then_type.clone());
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(then_val),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                // Else block
                ctx.current_block = MirBasicBlock::new(else_block.clone());
                ctx = self.lower_expr(ctx, else_expr);
                let else_val = ctx.next_local - 1;
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(else_val),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                // Merge block — load result as the expression's return value
                ctx.current_block = MirBasicBlock::new(merge_block);
                let result = ctx.alloc_local("_tern_val", then_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result,
                    src: result_local,
                });
                ctx
            }
            Expr::MatchExpr { expression, arms, .. } => {
                // Lower the matched expression
                ctx = self.lower_expr(ctx, expression);
                let match_val = ctx.next_local - 1;
                let result_local = ctx.alloc_local("_match_res", MirType::I64);
                let mut arm_types: Vec<MirType> = Vec::new();
                let merge_block = ctx.fresh_block();
                let arm_count = arms.len();
                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = ctx.fresh_block();
                    let is_last = i == arm_count - 1;
                    let next_target = if is_last {
                        merge_block.clone()
                    } else {
                        ctx.fresh_block()
                    };
                    match &arm.pattern {
                        Pattern::Tuple { .. } | Pattern::Range { .. } | Pattern::IsType { .. } | Pattern::Wildcard { .. } | Pattern::Identifier { .. } => {
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::Br(guard_label.clone()));
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            // Bind identifier pattern
                            if let Pattern::Identifier { name, .. } = &arm.pattern {
                                let local = ctx.alloc_local(name, MirType::I64);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: local,
                                    value: MirValue::Local(match_val),
                                });
                                ctx.locals.insert(name.clone(), local);
                            }
                            // Lower body, store last expression to result
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type.clone());
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            // After Wildcard, no more arms
                            ctx.current_block = MirBasicBlock::new(merge_block.clone());
                            let result_type = last_type;
                            ctx.local_types.insert(result_local, result_type);
                            break;
                        }
                        Pattern::Literal { value, .. } => {
                            let (vt, lc) = literal_to_mir(value);
                            let lit = ctx.alloc_local("_lit", vt);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: lit, value: MirValue::Constant(lc),
                            });
                            let eq = ctx.alloc_local("_eq", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: eq, op: MirBinaryOp::Eq,
                                left: MirValue::Local(match_val),
                                right: MirValue::Local(lit),
                            });
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::Or { .. } => {
                            ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                            ctx.current_block = MirBasicBlock::new(arm_label);
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::EnumVariant { enum_name, variant, args, .. } => {
                            let ev_map = self.enum_variants.borrow();
                            let variant_idx = ev_map.get(enum_name)
                                .and_then(|m| m.get(variant))
                                .copied()
                                .unwrap_or(0);
                            let struct_type = MirType::Struct(
                                if enum_name == "Result" { "Result".to_string() } else { enum_name.clone() },
                                vec![
                                    ("disc".to_string(), MirType::I32),
                                    ("payload".to_string(), MirType::I64),
                                ],
                            );
                            // FieldPtr needs a pointer, but match_val is a loaded value.
                            let mv_type = ctx.local_types.get(&match_val).cloned().unwrap_or(MirType::I64);
                            let struct_ptr = if matches!(mv_type, MirType::Struct(_, _)) {
                                let alloca = ctx.alloc_local("_mvtmp", mv_type);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: alloca,
                                    value: MirValue::Local(match_val),
                                });
                                alloca
                            } else {
                                match_val
                            };
                            let disc_ptr = ctx.alloc_local("_disc_ptr", MirType::Ptr(Box::new(MirType::I32)));
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: disc_ptr,
                                ptr: struct_ptr,
                                field_index: 0,
                                struct_type: Box::new(struct_type.clone()),
                            });
                            let disc_val = ctx.alloc_local("_disc", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: disc_val,
                                src: disc_ptr,
                            });
                            let idx_local = ctx.alloc_local("_vidx", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: idx_local,
                                value: MirValue::Constant(MirConstant::I32(variant_idx as i32)),
                            });
                            let eq = ctx.alloc_local("_eq", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: eq, op: MirBinaryOp::Eq,
                                left: MirValue::Local(disc_val),
                                right: MirValue::Local(idx_local),
                            });
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            if !args.is_empty() {
                                let payload_ptr = ctx.alloc_local("_pay_ptr", MirType::I64);
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: payload_ptr,
                                    ptr: struct_ptr,
                                    field_index: 1,
                                    struct_type: Box::new(struct_type),
                                });
                                for arg in args.iter() {
                                    if let Pattern::Identifier { name, .. } = arg {
                                        let val = ctx.alloc_local(name, MirType::I64);
                                        ctx.current_block.insts.push(MirInst::Load {
                                            dest: val,
                                            src: payload_ptr,
                                        });
                                        ctx.locals.insert(name.clone(), val);
                                    }
                                }
                            }
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        _ => {
                            // Unsupported pattern — fall through
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            ctx.current_block = MirBasicBlock::new(merge_block.clone());
                            break;
                        }
                    }
                }
                // Set current block to merge
                ctx.current_block = MirBasicBlock::new(merge_block);
                let res_type = arm_types.first().cloned().unwrap_or(MirType::I64);
                let result = ctx.alloc_local("_match_val", res_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result,
                    src: result_local,
                });
                ctx
            }
            Expr::BorrowRef { expression, .. } => {
                if let Expr::Identifier { name, .. } = expression.as_ref() {
                    if let Some(&local_id) = ctx.locals.get(name) {
                        let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                        let dest = ctx.alloc_local("_addr", ptr_type);
                        ctx.current_block.insts.push(MirInst::AddressOf {
                            dest,
                            local_id,
                        });
                        return ctx;
                    }
                }
                ctx = self.lower_expr(ctx, expression);
                let inner_local = ctx.next_local - 1;
                let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                let dest = ctx.alloc_local("_addr", ptr_type);
                ctx.current_block.insts.push(MirInst::AddressOf {
                    dest,
                    local_id: inner_local,
                });
                ctx
            }
            Expr::NullCoalesce { left, right, .. } => {
                // Lower left expression (Option<T>)
                ctx = self.lower_expr(ctx, left);
                let left_local = ctx.next_local - 1;
                let left_type = ctx.local_types.get(&left_local).cloned()
                    .unwrap_or(MirType::Struct("Result".to_string(), vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]));

                // Determine inner type T from Option__T struct name
                let inner_type = if let MirType::Struct(name, _) = &left_type {
                    extract_inner_type(name)
                } else {
                    MirType::I32
                };

                // Allocate result local
                let result_local = ctx.alloc_local("_ncres", inner_type.clone());

                // Allocate discriminant pointer
                let disc_ptr = ctx.alloc_local("_ncdp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: disc_ptr,
                    ptr: left_local,
                    field_index: 0,
                    struct_type: Box::new(left_type.clone()),
                });
                let disc_val = ctx.alloc_local("_ncdv", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: disc_val,
                    src: disc_ptr,
                });

                let some_block = ctx.fresh_block();
                let none_block = ctx.fresh_block();
                let merge_block = ctx.fresh_block();

                // Check if disc != 0 (Some)
                let is_some = ctx.alloc_local("_ncis", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: is_some,
                    op: MirBinaryOp::Neq,
                    left: MirValue::Local(disc_val),
                    right: MirValue::Constant(MirConstant::I32(0)),
                });
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(is_some),
                    true_block: some_block.clone(),
                    false_block: none_block.clone(),
                });

                // Some block: extract payload, cast to inner type, store
                ctx.current_block = MirBasicBlock::new(some_block);
                let payload_ptr = ctx.alloc_local("_ncpp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: payload_ptr,
                    ptr: left_local,
                    field_index: 1,
                    struct_type: Box::new(left_type.clone()),
                });
                let payload_val = ctx.alloc_local("_ncpv", MirType::I64);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: payload_val,
                    src: payload_ptr,
                });
                // Cast payload I64 → inner_type if needed
                let some_result = if inner_type != MirType::I64 {
                    let casted = ctx.alloc_local("_ncc", inner_type.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: casted,
                        value: MirValue::Local(payload_val),
                        to_type: inner_type.clone(),
                    });
                    casted
                } else {
                    payload_val
                };
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(some_result),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                // None block: evaluate right expression, store
                ctx.current_block = MirBasicBlock::new(none_block);
                ctx = self.lower_expr(ctx, right);
                let right_val = ctx.next_local - 1;
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(right_val),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                // Merge block: load result
                ctx.current_block = MirBasicBlock::new(merge_block);
                let result_val = ctx.alloc_local("_ncresv", inner_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result_val,
                    src: result_local,
                });
                ctx
            }
        }
    }

    /// Collect root identifier and all index expressions from a nested Index chain.
    /// e.g., m[i][j] → Some(("m", [i_expr, j_expr]))
    fn collect_array_indices<'a>(&self, expr: &'a Expr) -> Option<(String, Vec<&'a Expr>)> {
        match expr {
            Expr::Index { target, index, .. } => {
                let mut result = self.collect_array_indices(target)?;
                result.1.push(index);
                Some(result)
            }
            Expr::Identifier { name, .. } => Some((name.clone(), vec![])),
            _ => None,
        }
    }

    /// Given a root array local and a list of index expressions (innermost first),
    /// compute the final element pointer via nested GEP into the root array.
    /// Uses the original Expr::Index lowering logic which correctly handles array
    /// identifiers by using their alloca directly.
    fn lower_nested_array_geps(&self, mut ctx: LowerCtx, idx_exprs: &[&Expr], root_local: usize) -> LowerCtx {
        let mut ptr = root_local;
        let mut cur_type = ctx.local_types.get(&root_local).cloned().unwrap_or(MirType::I32);
        for idx_expr in idx_exprs {
            ctx = self.lower_expr(ctx, idx_expr);
            let idx_val = ctx.next_local - 1;
            if let MirType::Array(ref inner, _) = cur_type {
                let elem_ptr = ctx.alloc_local("_nested_aep", MirType::Ptr(Box::new(MirType::I8)));
                ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                    dest: elem_ptr,
                    ptr: ptr,
                    index: MirValue::Local(idx_val),
                    arr_type: Box::new(cur_type.clone()),
                    elem_type: Box::new(inner.as_ref().clone()),
                });
                ptr = elem_ptr;
                cur_type = inner.as_ref().clone();
            }
        }
        ctx
    }
}

/// Build a struct descriptor string for JSON serialize/deserialize.
/// Format: "field1:type1,field2:type2" where type is str/i32/i64/bool/f64.
fn build_descriptor(fields: &[(String, MirType)]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (fname, ftype) in fields {
        let type_name = match ftype {
            MirType::Str => "str",
            MirType::I32 => "i32",
            MirType::I64 => "i64",
            MirType::Bool => "bool",
            MirType::F64 => "f64",
            _ => "i32",
        };
        parts.push(format!("{}:{}", fname, type_name));
    }
    parts.join(",")
}

/// Check if a call name refers to a builtin that returns a string.
#[allow(dead_code)]
fn is_string_builtin_name(name: &str) -> bool {
    matches!(name, "ky_strlen" | "ky_i64_to_str" | "ky_input" | "ky_concat"
        | "ky_str_to_upper" | "ky_str_to_lower" | "ky_str_trim" | "ky_str_replace"
        | "ky_read_str"
        | "to_upper" | "to_lower" | "trim" | "replace" | "input" | "input_with_prompt" | "read_str")
}

/// Return the MIR type for known builtin functions, or None for generic functions.
fn builtin_return_type(name: &str) -> Option<MirType> {
    match name {
        "print" | "println" => Some(MirType::Void),
        "contains" => Some(MirType::I32),
        "to_upper" | "to_lower" | "trim" | "replace" | "input" | "input_with_prompt" => Some(MirType::Str),
        "open" | "close" | "write_str" => Some(MirType::I32),
        "read_str" => Some(MirType::Str),
        "char_at" => Some(MirType::Char),
        "ord" => Some(MirType::I32),
        "is_digit" | "is_alpha" | "is_alnum" | "is_whitespace" | "is_upper" | "is_lower" => Some(MirType::I32),
        "now" => Some(MirType::I64),
        "sleep" | "list_push" | "list_set" | "assert" | "assert_eq" | "assert_ne" | "assert_str" => Some(MirType::Void),
        "list_new" => Some(MirType::List(Box::new(MirType::I32))),
        "list_get" => Some(MirType::I64),
        "list_len" => Some(MirType::I64),
        "substr" => Some(MirType::Str),
        "eq_str" => Some(MirType::I32),
        "json_parse" => Some(MirType::Dict(Box::new(MirType::Str), Box::new(MirType::I64))),
        "json_stringify" | "json_stringify_str" => Some(MirType::Str),
        "serialize" => Some(MirType::Str),
        "ky_struct_to_json" => Some(MirType::Str),
        "type" => Some(MirType::Struct("TypeInfo".to_string(), vec![
            ("name".into(), MirType::Str),
            ("kind".into(), MirType::Str),
            ("size".into(), MirType::I32),
        ])),
        "ceil" | "floor" | "round" => Some(MirType::F64),
        "ky_getenv" | "ky_setenv" | "ky_base64_encode" | "ky_sha1" => Some(MirType::Str),
        "ky_spawn_thread" | "ky_join_thread" | "ky_parallel_for" => Some(MirType::I64),
        "ky_channel_new" | "ky_channel_send" | "ky_channel_recv" | "ky_channel_len" | "ky_channel_free" => Some(MirType::I64),
        "ky_channel_close" => Some(MirType::Void),
        "ok" | "error" => Some(MirType::Struct("Result".to_string(), vec![
            ("disc".to_string(), MirType::I32),
            ("payload".to_string(), MirType::I64),
        ])),
        "box" => None, // handled specially
        "ky_list_remove_value" => Some(MirType::I32),
        "ky_dict_contains" => Some(MirType::I32),
        "ky_dict_remove" => Some(MirType::I64),
        "ky_str_builder_new" => Some(MirType::Str),
        "ky_str_builder_append" | "ky_str_builder_free" => Some(MirType::Void),
        "ky_str_builder_to_str" => Some(MirType::Str),
        "ky_fs_exists" | "ky_fs_is_dir" | "ky_fs_is_file"
            | "ky_fs_copy" | "ky_fs_remove" | "ky_fs_create_dir"
            | "ky_fs_remove_dir" | "ky_fs_rename" | "ky_fs_write_string" => Some(MirType::I32),
        "ky_fs_size" | "ky_time_now_ms" | "ky_time_now_us" | "ky_fs_list_dir" | "ky_set_len" => Some(MirType::I64),
        "ky_set_new" => Some(MirType::Set(Box::new(MirType::I32))),
        "ky_set_add" | "ky_set_free" => Some(MirType::Void),
        "ky_set_contains" | "ky_set_remove" => Some(MirType::I32),
        "ky_open" => Some(MirType::I32),
        "ky_read_str" => Some(MirType::Str),
        "ky_write_str" | "ky_close" => Some(MirType::I32),
        "ky_fs_read_to_string" => Some(MirType::Str),
        _ => None,
    }
}

/// Convert an optional AstType annotation to a MirType.
fn param_type_from_annotation(ann: &Option<AstType>) -> Option<MirType> {
    match ann {
        Some(AstType::Primitive { name, .. } | AstType::User { name, .. }) => match name.as_str() {
            "str" => Some(MirType::Str),
            "i32" => Some(MirType::I32),
            "i64" => Some(MirType::I64),
            "f64" => Some(MirType::F64),
            "bool" => Some(MirType::I32),
            _ => None,
        },
        _ => None,
    }
}

/// Infer a closure parameter's MIR type by analyzing how it's used in the body.
/// Checks if the param participates in string concatenation (Str) or arithmetic (I32).
fn infer_closure_param_type(param: &str, body: &Expr) -> MirType {
    if let Some(t) = infer_expr_param_type(param, body) {
        return t;
    }
    MirType::Ptr(Box::new(MirType::I8))
}

fn infer_expr_param_type(param: &str, expr: &Expr) -> Option<MirType> {
    match expr {
        Expr::Binary { left, right, operator, .. } => {
            if matches!(operator, BinaryOp::Add) {
                if contains_param(param, left) && is_str_expr(right) {
                    return Some(MirType::Str);
                }
                if contains_param(param, right) && is_str_expr(left) {
                    return Some(MirType::Str);
                }
            }
            if let Some(t) = infer_expr_param_type(param, left) { return Some(t); }
            if let Some(t) = infer_expr_param_type(param, right) { return Some(t); }
        }
        Expr::Unary { operand, .. } => {
            if let Some(t) = infer_expr_param_type(param, operand) { return Some(t); }
        }
        Expr::Ternary { then_expr, else_expr, .. } => {
            if contains_param(param, then_expr) && !is_str_expr(then_expr) {
                if let Some(t) = infer_expr_param_type(param, then_expr) { return Some(t); }
            }
            if let Some(t) = infer_expr_param_type(param, else_expr) { return Some(t); }
        }
        Expr::FunctionCall { target, arguments, .. } => {
            // Check if param is the receiver of a method call: param.method(args)
            if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                if let Expr::Identifier { name, .. } = object.as_ref() {
                    if name == param {
                        // Infer type from known method patterns
                        match property.as_str() {
                            "json" | "text" | "status" | "send" | "redirect" => {
                                return Some(MirType::Ptr(Box::new(MirType::Struct("Res".into(), vec![]))));
                            }
                            "param" | "header" | "body" | "query" => {
                                return Some(MirType::Ptr(Box::new(MirType::Struct("Request".into(), vec![]))));
                            }
                            _ => {}
                        }
                    }
                }
            }
            for arg in arguments {
                if let Some(t) = infer_expr_param_type(param, arg) { return Some(t); }
            }
        }
        Expr::PropertyAccess { object, property, .. } => {
            if let Expr::Identifier { name, .. } = object.as_ref() {
                if name == param {
                    // Known struct field access patterns
                    match property.as_str() {
                        "fd" | "sent" | "status" | "json" | "text" => {
                            return Some(MirType::Ptr(Box::new(MirType::Struct("Res".into(), vec![]))));
                        }
                        "path" | "method" | "body" | "_pattern" => {
                            return Some(MirType::Ptr(Box::new(MirType::Struct("Request".into(), vec![]))));
                        }
                        _ => {
                            return Some(MirType::Ptr(Box::new(MirType::I8)));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    None
}

fn contains_param(param: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Identifier { name, .. } => name == param,
        Expr::Binary { left, right, .. } => contains_param(param, left) || contains_param(param, right),
        Expr::Unary { operand, .. } => contains_param(param, operand),
        Expr::Ternary { then_expr, else_expr, .. } => {
            contains_param(param, then_expr) || contains_param(param, else_expr)
        }
        Expr::FunctionCall { target, arguments, .. } => {
            // Check target for method calls: param.method(args)
            if let Expr::PropertyAccess { object, .. } = target.as_ref() {
                if contains_param(param, object) { return true; }
            }
            arguments.iter().any(|a| contains_param(param, a))
        }
        Expr::PropertyAccess { object, .. } => contains_param(param, object),
        _ => false,
    }
}

fn is_str_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Literal { value, .. } => matches!(value, Literal::String(_)),
        Expr::Binary { left, right, operator, .. } if matches!(operator, BinaryOp::Add) => {
            is_str_expr(left) || is_str_expr(right)
        }
        Expr::Ternary { then_expr, else_expr, .. } => is_str_expr(then_expr) || is_str_expr(else_expr),
        _ => false,
    }
}

/// Convert an AST type to an MIR type.
/// Check if an AstType references a specific type param name (e.g., `T` in `first: T`).
fn is_type_ref(ast: &AstType, tp_name: &str) -> bool {
    match ast {
        AstType::User { name, .. } | AstType::Primitive { name, .. } => name == tp_name,
        AstType::Generic { name, args, .. } => {
            name == tp_name || args.iter().any(|a| is_type_ref(a, tp_name))
        }
        _ => false,
    }
}

/// Build a TypeInfo struct literal for a given MirType.
/// Allocates locals, stores field values, and returns the final struct local.
fn build_typeinfo_struct(mir_type: &MirType, ctx: &mut LowerCtx) {
    let type_name = mir_type_to_type_name(mir_type);
    let type_kind = mir_type_to_kind(mir_type);
    let type_size = mir_type_to_size(mir_type);

    let struct_type = MirType::Struct("TypeInfo".into(), vec![
        ("name".into(), MirType::Str),
        ("kind".into(), MirType::Str),
        ("size".into(), MirType::I32),
    ]);

    let struct_local = ctx.alloc_local("_ti", struct_type.clone());
    let tin = ctx.alloc_local("_tin", MirType::Str);
    let tik = ctx.alloc_local("_tik", MirType::Str);
    ctx.string_locals.push(tin);
    ctx.string_locals.push(tik);

    // Field 0: name (str)
    let name_const = ctx.alloc_local("_tiname", MirType::Str);
    ctx.current_block.insts.push(MirInst::Store {
        dest: name_const,
        value: MirValue::Constant(MirConstant::String(type_name)),
    });
    let name_str = ctx.alloc_local("_tistr", MirType::Str);
    ctx.string_locals.push(name_str);
    ctx.current_block.insts.push(MirInst::Call {
        dest: Some(name_str),
        name: "ky_clone_str".to_string(),
        args: vec![MirValue::Local(name_const)],
    });
    let f0 = ctx.alloc_local("_tif0", MirType::I64);
    ctx.current_block.insts.push(MirInst::FieldPtr {
        dest: f0, ptr: struct_local, field_index: 0,
        struct_type: Box::new(struct_type.clone()),
    });
    ctx.current_block.insts.push(MirInst::Store {
        dest: f0,
        value: MirValue::Local(name_str),
    });

    // Field 1: kind (str)
    let kind_const = ctx.alloc_local("_tikind", MirType::Str);
    ctx.current_block.insts.push(MirInst::Store {
        dest: kind_const,
        value: MirValue::Constant(MirConstant::String(type_kind)),
    });
    let kind_str = ctx.alloc_local("_tikstr", MirType::Str);
    ctx.string_locals.push(kind_str);
    ctx.current_block.insts.push(MirInst::Call {
        dest: Some(kind_str),
        name: "ky_clone_str".to_string(),
        args: vec![MirValue::Local(kind_const)],
    });
    let f1 = ctx.alloc_local("_tif1", MirType::I64);
    ctx.current_block.insts.push(MirInst::FieldPtr {
        dest: f1, ptr: struct_local, field_index: 1,
        struct_type: Box::new(struct_type.clone()),
    });
    ctx.current_block.insts.push(MirInst::Store {
        dest: f1,
        value: MirValue::Local(kind_str),
    });

    // Field 2: size (i32)
    let f2 = ctx.alloc_local("_tif2", MirType::I64);
    ctx.current_block.insts.push(MirInst::FieldPtr {
        dest: f2, ptr: struct_local, field_index: 2,
        struct_type: Box::new(struct_type.clone()),
    });
    ctx.current_block.insts.push(MirInst::Store {
        dest: f2,
        value: MirValue::Constant(MirConstant::I32(type_size)),
    });

    // Load struct as result
    let load = ctx.alloc_local("_tires", struct_type);
    ctx.current_block.insts.push(MirInst::Load {
        dest: load,
        src: struct_local,
    });
}

/// Get the string name of a MirType (for TypeInfo.name)
fn mir_type_to_type_name(t: &MirType) -> String {
    match t {
        MirType::I1 => "i1".into(),
        MirType::I8 => "i8".into(),
        MirType::I16 => "i16".into(),
        MirType::I32 => "i32".into(),
        MirType::I64 => "i64".into(),
        MirType::F32 => "f32".into(),
        MirType::F64 => "f64".into(),
        MirType::Bool => "bool".into(),
        MirType::Char => "char".into(),
        MirType::Str => "str".into(),
        MirType::Void => "void".into(),
        MirType::Ptr(_) => "ptr".into(),
        MirType::List(inner) => format!("list<{}>", mir_type_to_type_name(inner)),
        MirType::Struct(name, _) => name.clone(),
        MirType::Dict(k, v) => format!("dict<{},{}>", mir_type_to_type_name(k), mir_type_to_type_name(v)),
        MirType::Set(inner) => format!("set<{}>", mir_type_to_type_name(inner)),
        MirType::Array(inner, _size) => format!("[{}]", mir_type_to_type_name(inner)),
        MirType::Slice(inner) => format!("&[{}]", mir_type_to_type_name(inner)),
        MirType::Box(inner) => format!("box<{}>", mir_type_to_type_name(inner)),
    }
}

/// Get the kind string of a MirType (for TypeInfo.kind)
fn mir_type_to_kind(t: &MirType) -> String {
    match t {
        MirType::I1 | MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64
        | MirType::F32 | MirType::F64 | MirType::Bool | MirType::Char
        | MirType::Str | MirType::Void => "primitive".into(),
        MirType::Ptr(_) => "ptr".into(),
        MirType::List(_) => "list".into(),
        MirType::Struct(_, _) => "struct".into(),
        MirType::Dict(_, _) => "dict".into(),
        MirType::Set(_) => "set".into(),
        MirType::Array(_, _) => "array".into(),
        MirType::Slice(_) => "slice".into(),
        MirType::Box(_) => "box".into(),
    }
}

/// Get the byte size of a MirType (for TypeInfo.size)
fn mir_type_to_size(t: &MirType) -> i32 {
    match t {
        MirType::I1 | MirType::Bool => 1,
        MirType::I8 | MirType::Char => 1,
        MirType::I16 => 2,
        MirType::I32 => 4,
        MirType::I64 => 8,
        MirType::F32 => 4,
        MirType::F64 => 8,
        MirType::Str | MirType::Ptr(_) | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) => 8,
        MirType::Void => 0,
        MirType::Struct(_, fields) => {
            // Estimate size: sum of field sizes (approximately, without padding)
            fields.iter().map(|(_, ft)| mir_type_to_size(ft) as i32).sum()
        }
        MirType::Array(inner, _) => mir_type_to_size(inner) * 8, // rough estimate
        MirType::Slice(_) => 16, // ptr (8) + len (8)
        MirType::Box(_) => 8, // ptr
    }
}

/// Serialize a MirType to a string for concrete struct name mangling.
fn mir_type_to_string(t: &MirType) -> String {
    match t {
        MirType::I8 => "i8".into(),
        MirType::I16 => "i16".into(),
        MirType::I32 => "i32".into(),
        MirType::I64 => "i64".into(),
        MirType::F32 => "f32".into(),
        MirType::F64 => "f64".into(),
        MirType::Bool => "bool".into(),
        MirType::Char => "char".into(),
        MirType::Str => "str".into(),
        MirType::Void => "void".into(),
        MirType::Ptr(_) => "ptr".into(),
        MirType::List(inner) => format!("list_{}", mir_type_to_string(inner)),
        MirType::Struct(n, _) => n.clone(),
        MirType::I1 => "i1".into(),
        MirType::Array(inner, _) => format!("arr_{}", mir_type_to_string(inner)),
        MirType::Dict(key, val) => format!("dict_{}_{}", mir_type_to_string(key), mir_type_to_string(val)),
        MirType::Set(inner) => format!("set_{}", mir_type_to_string(inner)),
        MirType::Slice(inner) => format!("slice_{}", mir_type_to_string(inner)),
        MirType::Box(inner) => format!("box_{}", mir_type_to_string(inner)),
    }
}

/// Create a mangled concrete struct name from a generic name and concrete type args.
fn make_concrete_name(name: &str, type_args: &[MirType]) -> String {
    if type_args.is_empty() {
        return name.to_string();
    }
    let args_str: Vec<String> = type_args.iter().map(mir_type_to_string).collect();
    format!("{}__{}", name, args_str.join("_"))
}

/// Extract the inner MirType from an Option__T struct name.
fn extract_struct_inner(name: &str) -> MirType {
    extract_inner_type(name)
}

/// Extract the inner type T from an Option__T struct name.
/// For example, "Option__i32" → MirType::I32, "Option__str" → MirType::Str.
fn extract_inner_type(struct_name: &str) -> MirType {
    if let Some(inner) = struct_name.strip_prefix("Option__") {
        match inner {
            "i8" | "u8" => MirType::I8,
            "i16" | "u16" => MirType::I16,
            "i32" | "u32" => MirType::I32,
            "i64" | "u64" => MirType::I64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            "bool" => MirType::Bool,
            "char" => MirType::Char,
            "str" => MirType::Str,
            "void" => MirType::Void,
            _ => MirType::I32, // fallback
        }
    } else {
        MirType::I32
    }
}

/// Register Option__T struct fields in struct_defs for a given AstType
/// that may contain Option types (e.g. `i32?`, `Option<i32>`).
fn register_option_type(ast: &AstType, struct_defs: &mut std::collections::HashMap<String, Vec<(String, MirType)>>) {
    match ast {
        AstType::Optional { inner, .. } => {
            let inner_mir = ast_type_to_mir(inner, Some(struct_defs));
            let concrete_name = make_concrete_name("Option", &[inner_mir]);
            if !struct_defs.contains_key(&concrete_name) {
                struct_defs.insert(concrete_name, vec![
                    ("disc".to_string(), MirType::I32),
                    ("payload".to_string(), MirType::I64),
                ]);
            }
        }
        AstType::Generic { name, args, .. } if name == "Option" && args.len() == 1 => {
            register_option_type(&args[0], struct_defs);
            let inner_mir = ast_type_to_mir(&args[0], Some(struct_defs));
            let concrete_name = make_concrete_name("Option", &[inner_mir]);
            if !struct_defs.contains_key(&concrete_name) {
                struct_defs.insert(concrete_name, vec![
                    ("disc".to_string(), MirType::I32),
                    ("payload".to_string(), MirType::I64),
                ]);
            }
        }
        _ => {}
    }
}

/// Extract generic type bindings by matching an AstType (parameter type with type params)
/// against a concrete MirType (argument type).
/// Scan an AstType for generic struct references and pre-register concrete versions.
/// This must run before lower_function so that the function signature can resolve
/// return types like `Pair<i32, str>` as `MirType::Struct("Pair__i32_str", fields)`.
fn pre_register_generic_type(
    ast: &AstType,
    type_subst: &std::collections::HashMap<String, MirType>,
    generic_struct_templates: &std::collections::HashMap<String, StructDecl>,
    struct_defs: &mut std::collections::HashMap<String, Vec<(String, MirType)>>,
) -> Option<MirType> {
    match ast {
        AstType::Generic { name, args, .. } if name != "list" && !args.is_empty() => {
            if let Some(tpl) = generic_struct_templates.get(name) {
                let concrete_args: Vec<MirType> = args.iter()
                    .map(|a| {
                        let mir = ast_type_to_mir_with_subst(a, Some(struct_defs), type_subst);
                        // Recurse for nested generic types
                        if let AstType::Generic { .. } = a {
                            if let Some(m) = pre_register_generic_type(a, type_subst, generic_struct_templates, struct_defs) {
                                return m;
                            }
                        }
                        mir
                    })
                    .collect();
                let concrete_name = make_concrete_name(name, &concrete_args);
                if !struct_defs.contains_key(&concrete_name) {
                    let concrete_fields: Vec<(String, MirType)> = tpl.fields.iter()
                        .map(|f| (f.name.clone(), ast_type_to_mir_with_subst(&f.type_, Some(struct_defs), type_subst)))
                        .collect();
                    struct_defs.insert(concrete_name.clone(), concrete_fields.clone());
                }
                return struct_defs.get(&concrete_name).map(|f| MirType::Struct(concrete_name, f.clone()));
            }
        }
        _ => {}
    }
    None
}

fn extract_generic_bindings(
    param_type: &AstType,
    arg_type: &MirType,
    type_params: &[TypeParam],
    subst: &mut std::collections::HashMap<String, MirType>,
) {
    match (param_type, arg_type) {
        (AstType::User { name, .. } | AstType::Primitive { name, .. }, _) => {
            if type_params.iter().any(|tp| tp.name == *name) && !subst.contains_key(name) {
                subst.insert(name.clone(), arg_type.clone());
            }
        }
        (AstType::Generic { name, args, .. }, MirType::List(inner)) if name == "list" => {
            if let Some(elem_type) = args.first() {
                extract_generic_bindings(elem_type, inner, type_params, subst);
            }
        }
        (AstType::Optional { inner, .. }, MirType::Ptr(inner_type)) => {
            extract_generic_bindings(inner, inner_type, type_params, subst);
        }
        (AstType::Optional { inner, .. }, MirType::Struct(name, _)) if name.starts_with("Option__") => {
            let inner_type = extract_struct_inner(name);
            extract_generic_bindings(inner, &inner_type, type_params, subst);
        }
        _ => {}
    }
}

/// Infer type param bindings for a generic function call from the concrete argument types.
fn infer_function_type_params(
    template: &FunctionDecl,
    arg_types: &[MirType],
) -> std::collections::HashMap<String, MirType> {
    let mut subst: std::collections::HashMap<String, MirType> = std::collections::HashMap::new();
    for (param, arg_type) in template.params.iter().zip(arg_types) {
        extract_generic_bindings(&param.type_, arg_type, &template.type_params, &mut subst);
    }
    subst
}

/// Convert a MirType to an AstType for substitution into function AST.
fn mir_type_to_ast_type(t: &MirType, _span: kyc_core::span::Span) -> AstType {
    match t {
        MirType::I8 => AstType::Primitive { name: "i8".into(), span: _span },
        MirType::I16 => AstType::Primitive { name: "i16".into(), span: _span },
        MirType::I32 => AstType::Primitive { name: "i32".into(), span: _span },
        MirType::I64 => AstType::Primitive { name: "i64".into(), span: _span },
        MirType::F32 => AstType::Primitive { name: "f32".into(), span: _span },
        MirType::F64 => AstType::Primitive { name: "f64".into(), span: _span },
        MirType::Bool => AstType::Primitive { name: "bool".into(), span: _span },
        MirType::Char => AstType::Primitive { name: "char".into(), span: _span },
        MirType::Str => AstType::Primitive { name: "str".into(), span: _span },
        MirType::Void => AstType::Primitive { name: "void".into(), span: _span },
        MirType::List(inner) => AstType::Generic {
            name: "list".into(),
            args: vec![mir_type_to_ast_type(inner, _span)],
            span: _span,
        },
        MirType::Struct(name, _) => AstType::User { name: name.clone(), span: _span },
        MirType::Ptr(_) => AstType::User { name: "ptr".into(), span: _span },
        MirType::Dict(key, value) => AstType::Dict {
            key: Box::new(mir_type_to_ast_type(key, _span)),
            value: Box::new(mir_type_to_ast_type(value, _span)),
            span: _span,
        },
        MirType::Set(inner) => AstType::Generic {
            name: "set".into(),
            args: vec![mir_type_to_ast_type(inner, _span)],
            span: _span,
        },
        MirType::I1 => AstType::Primitive { name: "bool".into(), span: _span },
        MirType::Array(inner, size) => AstType::Array {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            size: *size,
            span: _span,
        },
        MirType::Slice(inner) => AstType::Slice {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::Box(inner) => AstType::Generic {
            name: "box".into(),
            args: vec![mir_type_to_ast_type(inner, _span)],
            span: _span,
        },
    }
}

/// Substitute type params in an AstType with concrete AstTypes.
fn substitute_ast_type(ast: &AstType, subst: &std::collections::HashMap<String, AstType>) -> AstType {
    match ast {
        AstType::User { name, .. } => {
            if let Some(replacement) = subst.get(name) {
                replacement.clone()
            } else {
                ast.clone()
            }
        }
        AstType::Primitive { name, .. } => {
            if let Some(replacement) = subst.get(name) {
                replacement.clone()
            } else {
                ast.clone()
            }
        }
        AstType::Generic { name, args, span } => {
            AstType::Generic {
                name: name.clone(),
                args: args.iter().map(|a| substitute_ast_type(a, subst)).collect(),
                span: *span,
            }
        }
        AstType::Optional { inner, span } => {
            AstType::Optional {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Error { inner, span } => {
            AstType::Error {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Dict { key, value, span } => {
            AstType::Dict {
                key: Box::new(substitute_ast_type(key, subst)),
                value: Box::new(substitute_ast_type(value, subst)),
                span: *span,
            }
        }
        AstType::FnPtr { params, return_, span } => {
            AstType::FnPtr {
                params: params.iter().map(|p| substitute_ast_type(p, subst)).collect(),
                return_: Box::new(substitute_ast_type(return_, subst)),
                span: *span,
            }
        }
        AstType::Mutable { inner, span } => {
            AstType::Mutable {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Borrow { inner, span } => {
            AstType::Borrow {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Borrow { inner, span } => {
            AstType::Borrow {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Ptr { span } => AstType::Ptr { span: *span },
        AstType::Array { inner, size, span, .. } => AstType::Array {
            inner: Box::new(substitute_ast_type(inner, subst)),
            size: *size,
            span: *span,
        },
        AstType::Slice { inner, span } => AstType::Slice {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
    }
}

/// Clone a FunctionDecl, substituting its type params with concrete type args.
fn clone_and_specialize_function(
    template: &FunctionDecl,
    type_subst: &std::collections::HashMap<String, MirType>,
) -> FunctionDecl {
    let mut f = template.clone();
    // Build AstType substitution map
    let ast_subst: std::collections::HashMap<String, AstType> = type_subst.iter()
        .map(|(name, mir_type)| (name.clone(), mir_type_to_ast_type(mir_type, f.span)))
        .collect();
    // Substitute param types
    for p in &mut f.params {
        p.type_ = substitute_ast_type(&p.type_, &ast_subst);
    }
    // Substitute return type
    if let Some(rt) = &mut f.return_type {
        *rt = substitute_ast_type(rt, &ast_subst);
    }
    // Substitute types in block statements (variable declarations)
    if let Some(body) = &mut f.body {
        for stmt in &mut body.statements {
            substitute_stmt_types(stmt, &ast_subst);
        }
    }
    f
}

/// Substitute type params in variable declarations inside a statement.
fn substitute_stmt_types(stmt: &mut Stmt, subst: &std::collections::HashMap<String, AstType>) {
    match stmt {
        Stmt::Variable(v) | Stmt::TypedVariable(v) => {
            if let Some(t) = &mut v.type_ {
                *t = substitute_ast_type(t, subst);
            }
        }
        Stmt::If(s) => {
            for s_ in &mut s.body.statements { substitute_stmt_types(s_, subst); }
            for el in &mut s.elif_branches {
                for s_ in &mut el.body.statements { substitute_stmt_types(s_, subst); }
            }
            if let Some(b) = &mut s.else_branch {
                for s_ in &mut b.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::While(w) => {
            for s_ in &mut w.body.statements { substitute_stmt_types(s_, subst); }
            if let Some(b) = &mut w.else_branch {
                for s_ in &mut b.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::For(f) => {
            for s_ in &mut f.body.statements { substitute_stmt_types(s_, subst); }
            if let Some(b) = &mut f.else_branch {
                for s_ in &mut b.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::Match(m) => {
            for arm in &mut m.arms {
                for s_ in &mut arm.body.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::Unsafe(u) => {
            for s_ in &mut u.body.statements { substitute_stmt_types(s_, subst); }
        }
        Stmt::Guard(g) => {
            for s_ in &mut g.body.statements { substitute_stmt_types(s_, subst); }
        }
        Stmt::BindingIf(b) => {
            for s_ in &mut b.body.statements { substitute_stmt_types(s_, subst); }
            if let Some(el) = &mut b.else_branch {
                for s_ in &mut el.statements { substitute_stmt_types(s_, subst); }
            }
        }
        _ => {}
    }
}

/// Convert an AstType to MirType with type param substitution.
fn ast_type_to_mir_with_subst(
    ast: &AstType,
    struct_defs: Option<&std::collections::HashMap<String, Vec<(String, MirType)>>>,
    subst: &std::collections::HashMap<String, MirType>,
) -> MirType {
    match ast {
        AstType::Primitive { name, .. } => {
            if let Some(t) = subst.get(name) { return t.clone(); }
            match name.as_str() {
                "i8" | "u8" => MirType::I8,
                "i16" | "u16" => MirType::I16,
                "i32" | "u32" => MirType::I32,
                "i64" | "u64" => MirType::I64,
                "f32" => MirType::F32,
                "f64" => MirType::F64,
                "bool" => MirType::Bool,
                "char" => MirType::Char,
                "str" => MirType::Str,
                "void" => MirType::Void,
                _ => MirType::I32,
            }
        }
        AstType::User { name, .. } => {
            if let Some(t) = subst.get(name) { return t.clone(); }
            match name.as_str() {
                "i8" | "u8" => MirType::I8,
                "i16" | "u16" => MirType::I16,
                "i32" | "u32" => MirType::I32,
                "i64" | "u64" => MirType::I64,
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
            }
        }
        AstType::Generic { name, args, .. } => {
            if let Some(t) = subst.get(name) { return t.clone(); }
            if name == "set" {
                if args.is_empty() { MirType::Set(Box::new(MirType::I32)) }
                else { MirType::Set(Box::new(ast_type_to_mir_with_subst(&args[0], struct_defs, subst))) }
            } else if name == "list" {
                if args.is_empty() { MirType::List(Box::new(MirType::I32)) }
                else { MirType::List(Box::new(ast_type_to_mir_with_subst(&args[0], struct_defs, subst))) }
            } else if name == "tuple" {
                // (T, U, ...) → anonymous struct with _0, _1, ... fields
                let field_types: Vec<(String, MirType)> = args.iter().enumerate()
                    .map(|(i, el)| (format!("_{}", i), ast_type_to_mir_with_subst(el, struct_defs, subst)))
                    .collect();
                // Create unique name based on field types
                let type_suffix: String = field_types.iter()
                    .map(|(_, t)| match t {
                        MirType::I32 => "i32",
                        MirType::I64 => "i64",
                        MirType::Str => "str",
                        MirType::Bool => "bool",
                        MirType::F64 => "f64",
                        MirType::F32 => "f32",
                        MirType::I16 => "i16",
                        MirType::I8 => "i8",
                        MirType::Char => "char",
                        _ => "x",
                    })
                    .collect();
                let struct_name = format!("_tuple_{}_{}", field_types.len(), type_suffix);
                MirType::Struct(struct_name, field_types)
            } else if args.is_empty() {
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        return MirType::Struct(name.to_string(), fields.clone());
                    }
                }
                MirType::Struct(name.clone(), vec![])
            } else {
                // Check if the base name is already registered in struct_defs with known
                // fields (e.g. enums). Enums and non-generic structs use the base name
                // directly; generic structs/classes are registered with concrete names.
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        if !fields.is_empty() {
                            return MirType::Struct(name.to_string(), fields.clone());
                        }
                    }
                }
                // Handle Option<T> — always create struct with disc + payload fields
                if name == "Option" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir_with_subst(&args[0], struct_defs, subst);
                    let concrete_name = make_concrete_name("Option", &[inner_mir]);
                    return MirType::Struct(concrete_name, vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]);
                }
                // Handle box<T> — built-in heap pointer type
                if name == "box" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir_with_subst(&args[0], struct_defs, subst);
                    return MirType::Box(Box::new(inner_mir));
                }
                // User-defined generic with concrete args — create concrete version
                let concrete_args: Vec<MirType> = args.iter()
                    .map(|a| ast_type_to_mir_with_subst(a, struct_defs, subst))
                    .collect();
                let concrete_name = make_concrete_name(name, &concrete_args);
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(&concrete_name) {
                        return MirType::Struct(concrete_name, fields.clone());
                    }
                }
                MirType::Struct(concrete_name, vec![])
            }
        }
        AstType::Optional { inner, .. } => {
            let inner_mir = ast_type_to_mir_with_subst(inner, struct_defs, subst);
            let concrete_name = make_concrete_name("Option", &[inner_mir.clone()]);
            MirType::Struct(concrete_name, vec![
                ("disc".to_string(), MirType::I32),
                ("payload".to_string(), MirType::I64),
            ])
        }
        AstType::Dict { key, value, .. } => MirType::Dict(
            Box::new(ast_type_to_mir_with_subst(key, struct_defs, subst)),
            Box::new(ast_type_to_mir_with_subst(value, struct_defs, subst)),
        ),
        AstType::Error { inner: _, .. } => MirType::Struct("Result".to_string(), vec![
            ("disc".to_string(), MirType::I32),
            ("payload".to_string(), MirType::I64),
        ]),
        AstType::FnPtr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } => ast_type_to_mir_with_subst(inner, struct_defs, subst),
        AstType::Borrow { inner, .. } => ast_type_to_mir_with_subst(inner, struct_defs, subst),
        AstType::Ptr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Array { inner, size, .. } => MirType::Array(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst)), *size),
        AstType::Slice { inner, .. } => MirType::Slice(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
    }
}

fn ast_type_to_mir(ast: &AstType, struct_defs: Option<&std::collections::HashMap<String, Vec<(String, MirType)>>>) -> MirType {
    match ast {
        AstType::Primitive { name, .. } => match name.as_str() {
            "i8" | "u8" => MirType::I8,
            "i16" | "u16" => MirType::I16,
            "i32" | "u32" => MirType::I32,
            "i64" | "u64" => MirType::I64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            "bool" => MirType::Bool,
            "char" => MirType::Char,
            "str" => MirType::Str,
            "void" => MirType::Void,
            _ => MirType::I32,
        },
        AstType::User { name, .. } => match name.as_str() {
            "i8" | "u8" => MirType::I8,
            "i16" | "u16" => MirType::I16,
            "i32" | "u32" => MirType::I32,
            "i64" | "u64" => MirType::I64,
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
                // Resolve type alias if defined
                let alias_mir = TYPE_ALIAS_CACHE.with(|cache| {
                    let defs = cache.borrow();
                    defs.get(name).and_then(|ast| {
                        // Recursively resolve alias chain
                        let mut seen = std::collections::HashSet::new();
                        let mut current = ast;
                        let mut current_name = name;
                        loop {
                            if !seen.insert(current_name.to_string()) {
                                return None; // cycle
                            }
                            match current {
                                AstType::Primitive { name, .. } | AstType::User { name, .. } => {
                                    let n = name.as_str();
                                    let mir = match n {
                                        "i8" => Some(MirType::I8), "i16" => Some(MirType::I16),
                                        "i32" => Some(MirType::I32), "i64" => Some(MirType::I64),
                                        "f32" => Some(MirType::F32), "f64" => Some(MirType::F64),
                                        "bool" => Some(MirType::Bool), "char" => Some(MirType::Char),
                                        "str" => Some(MirType::Str),
                                        other => {
                                            if let Some(defs) = struct_defs {
                                                if let Some(_) = defs.get(other) {
                                                    // Let ast_type_to_mir handle structs
                                                    return None;
                                                }
                                            }
                                            // Try dereferencing another alias
                                            if let Some(next) = defs.get(other) {
                                                current = next;
                                                current_name = other;
                                                continue;
                                            }
                                            None
                                        }
                                    };
                                    return mir;
                                }
                                _ => return None,
                            }
                        }
                    })
                });
                if let Some(mir) = alias_mir {
                    return mir;
                }
                MirType::Struct(name.to_string(), vec![])
            }
        },
        AstType::Generic { name, args, .. } => {
            if name == "set" {
                if args.is_empty() { MirType::Set(Box::new(MirType::I32)) }
                else { MirType::Set(Box::new(ast_type_to_mir(&args[0], struct_defs))) }
            } else if name == "list" {
                if args.is_empty() { MirType::List(Box::new(MirType::I32)) }
                else { MirType::List(Box::new(ast_type_to_mir(&args[0], struct_defs))) }
            } else if name == "tuple" {
                // (T, U, ...) → anonymous struct
                let field_types: Vec<(String, MirType)> = args.iter().enumerate()
                    .map(|(i, el)| (format!("_{}", i), ast_type_to_mir(el, struct_defs)))
                    .collect();
                let type_suffix: String = field_types.iter()
                    .map(|(_, t)| match t {
                        MirType::I32 => "i32",
                        MirType::I64 => "i64",
                        MirType::Str => "str",
                        MirType::Bool => "bool",
                        MirType::F64 => "f64",
                        MirType::F32 => "f32",
                        MirType::I16 => "i16",
                        MirType::I8 => "i8",
                        MirType::Char => "char",
                        _ => "x",
                    })
                    .collect();
                let struct_name = format!("_tuple_{}_{}", field_types.len(), type_suffix);
                MirType::Struct(struct_name, field_types)
            } else if args.is_empty() {
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        return MirType::Struct(name.to_string(), fields.clone());
                    }
                }
                MirType::Struct(name.clone(), vec![])
            } else {
                // Check if the base name is already registered in struct_defs with known
                // fields (e.g. enums). Enums and non-generic structs use the base name
                // directly; generic structs/classes are registered with concrete names.
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        if !fields.is_empty() {
                            return MirType::Struct(name.to_string(), fields.clone());
                        }
                    }
                }
                // Handle Option<T> — always create struct with disc + payload fields
                if name == "Option" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir(&args[0], struct_defs);
                    let concrete_name = make_concrete_name("Option", &[inner_mir]);
                    return MirType::Struct(concrete_name, vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]);
                }
                // Handle box<T> — built-in heap pointer type
                if name == "box" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir(&args[0], struct_defs);
                    return MirType::Box(Box::new(inner_mir));
                }
                // User-defined generic with concrete type args — create concrete name
                let concrete_args: Vec<MirType> = args.iter()
                    .map(|a| ast_type_to_mir(a, struct_defs))
                    .collect();
                let concrete_name = make_concrete_name(name, &concrete_args);
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(&concrete_name) {
                        return MirType::Struct(concrete_name, fields.clone());
                    }
                }
                MirType::Struct(concrete_name, vec![])
            }
        }
        AstType::Optional { inner, .. } => {
            let inner_mir = ast_type_to_mir(inner, struct_defs);
            let concrete_name = make_concrete_name("Option", &[inner_mir.clone()]);
            MirType::Struct(concrete_name, vec![
                ("disc".to_string(), MirType::I32),
                ("payload".to_string(), MirType::I64),
            ])
        }
        AstType::Dict { key, value, .. } => MirType::Dict(
            Box::new(ast_type_to_mir(key, struct_defs)),
            Box::new(ast_type_to_mir(value, struct_defs)),
        ),
        AstType::Error { inner: _, .. } => MirType::Struct("Result".to_string(), vec![
            ("disc".to_string(), MirType::I32),
            ("payload".to_string(), MirType::I64),
        ]),
        AstType::FnPtr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } => ast_type_to_mir(inner, struct_defs),
        AstType::Ptr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Array { inner, size, .. } => MirType::Array(Box::new(ast_type_to_mir(inner, struct_defs)), *size),
        AstType::Slice { inner, .. } => MirType::Slice(Box::new(ast_type_to_mir(inner, struct_defs))),
    }
}
