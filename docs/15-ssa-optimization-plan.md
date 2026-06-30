# SSA Form Optimization — Plan Técnico (Fase 15)

> Cómo convertir el MIR actual (load/store) a SSA Form para que LLVM genere
> código ~10-50x más rápido.

---

## 1. Problema

Kyle genera LLVM IR con `alloca` + `load` + `store` para cada variable local:

```llvm
; CÓDIGO ACTUAL DE KYLE — LLVM no puede optimizar
%total = alloca i64
%i     = alloca i64
store i64 %total_val, %total    ; dependencia falsa
store i64 %i_val, %i            ; dependencia falsa
%loaded_i = load i64, %i
%loaded_t = load i64, %total
%add   = add i64 %loaded_t, %loaded_i
store i64 %add, %total
```

Cada `store` crea una dependencia de memoria que impide que LLVM reordene
o elimine instrucciones. Rust genera SSA directo:

```llvm
; LO QUE RUST GENERA — LLVM optimiza como debe
%total.0 = phi i64 [ 0, %entry ], [ %next, %loop ]
%i.0    = phi i64 [ 0, %entry ], [ %inc, %loop ]
%next   = add i64 %total.0, %i.0
%inc    = add i64 %i.0, 1
%cond   = icmp slt i64 %inc, 10000000
br i1 %cond, label %loop, label %exit
```

**Solución:** Convertir `MirFunction` a `SsaFunction` donde las variables son
valores SSA (cada asignación crea una nueva versión) y los join points tienen
phi nodes.

---

## 2. Estructuras de Datos

### 2.1 SsaFunction — Función en SSA Form

```rust
/// Una función en SSA Form.
/// Cada "variable" es en realidad un conjunto de versiones (SSA values).
/// No hay allocas, no hay load/store.
pub struct SsaFunction {
    pub name: String,
    pub params: Vec<MirType>,      // tipos de los parámetros
    pub return_type: MirType,
    pub blocks: Vec<SsaBlock>,     // basic blocks en SSA
    pub ssa_values: Vec<SsaValue>, // todos los valores SSA de la función
}

/// Un basic block en SSA.
pub struct SsaBlock {
    pub label: String,
    pub phi_nodes: Vec<PhiNode>,   // instrucciones phi al inicio
    pub insts: Vec<SsaInst>,       // instrucciones que usan/definen SSA values
    pub terminator: SsaTerminator,
}

/// Un valor SSA (índice al vector global de la función).
pub type SsaValueRef = usize;

/// Un valor SSA con su tipo e instrucción que lo define.
pub struct SsaValue {
    pub type_: MirType,
    pub defining_inst: Option<usize>,  // índice en el bloque
    pub block: usize,                   // bloque donde se define
}

/// Instrucción phi.
pub struct PhiNode {
    pub dest: SsaValueRef,
    pub incoming: Vec<(SsaValueRef, String)>,  // (valor, bloque_origen)
}

/// Instrucciones SSA (sin load/store — operandos son SsaValueRef).
pub enum SsaInst {
    BinaryOp { dest: SsaValueRef, op: MirBinaryOp, left: SsaValueRef, right: SsaValueRef },
    UnaryOp { dest: SsaValueRef, op: MirUnaryOp, operand: SsaValueRef },
    Call { dest: Option<SsaValueRef>, name: String, args: Vec<SsaValueRef> },
    Cast { dest: SsaValueRef, value: SsaValueRef, to_type: MirType },
    FnAddr { dest: SsaValueRef, name: String },
    CallIndirect { dest: Option<SsaValueRef>, fn_ptr: SsaValueRef, ret_type: MirType, param_types: Vec<MirType>, args: Vec<SsaValueRef> },
    AsyncSpawn { dest: SsaValueRef, function_name: String, arg: SsaValueRef },
    AsyncAwait { dest: SsaValueRef, handle: SsaValueRef },
    // Para tipos heap (escaping allocas) —仍需 store/load
    Store { dest: usize, value: SsaValueRef },     // dest is original alloca index
    Load { dest: SsaValueRef, src: usize },         // src is original alloca index
    PtrOffset { dest: SsaValueRef, ptr: usize, index: SsaValueRef },
    FieldPtr { dest: usize, ptr: usize, field_index: usize, struct_type: Box<MirType> },
    Memcpy { dest_ptr_local: usize, src_alloca_local: usize, struct_type: Box<MirType> },
    Alloca { dest: usize, type_: MirType, name: String },
}
```

### 2.2 Criterios de promoción (Mem2Reg)

Una alloca es promovible si **todas** las siguientes condiciones se cumplen:

1. **Solo accesos directos** — no hay `field_ptr`, `PtrOffset`, o `Memcpy`
   que referencie esta alloca (el puntero no escapa)
2. **Tipo Copy** — el tipo es escalar (i32, i64, f64, bool) o struct pequeño
   (no Str, List, Dict, que son Move types)
3. **Sin alias** — no hay dos allocas diferentes que se carguen/almacenen
   en la misma instrucción

Las allocas NO promovibles (strings, listas, clases, field_ptr) se quedan
en load/store dentro del SSA.

---

## 3. Algoritmo de Construcción SSA

### Fase 1: Mem2Reg (promoción de allocas)

```
Entrada: MirFunction con basic blocks y allocas
Salida: Mapa de alloca → SsaValue para allocas promovibles

Para cada alloca en MirFunction:
  Si es promovible:
    - Reemplazar cada store %dest, %val → definir nueva versión SSA
    - Reemplazar cada load %dest, %src → leer versión SSA actual
    - Las stores en el mismo bloque: simplemente renombrar
    - Las stores en diferentes bloques: insertar phi en join points
```

### Fase 2: Inserción de Phi nodes (algoritmo de dominadores)

```
1. Calcular el árbol de dominadores del CFG
2. Calcular la frontera de dominadores (dominance frontier) de cada bloque
3. Para cada alloca promovible:
   a. Encontrar todos los bloques con stores a esa alloca
   b. Para cada bloque B con store:
      - Para cada bloque DF en dominance_frontier(B):
        - Insertar phi node para la alloca en DF (si no existe ya)
        - Agregar DF a la lista de bloques con definiciones
```

### Fase 3: Renombrado (paseo del árbol de dominadores)

```
1. Para cada alloca promovible, mantener un stack de (versión, bloque)
2. Walk del árbol de dominadores en pre-orden:
   a. Para cada instrucción en el bloque:
      - Si es store → crear nueva versión SSA, pushear al stack
      - Si es load → leer la versión del tope del stack
   b. Para cada phi node en sucesores: agregar (versión_actual, este_bloque)
   c. Procesar hijos en el árbol de dominadores (recursivo)
   d. Al salir del bloque: hacer pop del stack para restaurar estado
```

---

## 4. Codegen SSA

Para funciones en SSA Form, el codegen emite LLVM IR directamente:

```rust
fn codegen_ssa_function(&self, func: &SsaFunction) {
    // 1. Crear LLVM function con los tipos correctos
    // 2. Para cada basic block SSA:
    //    a. Crear LLVM basic block
    //    b. Si hay phi nodes: crear LLVM phi instructions
    //    c. Emitir SSA instructions como LLVM values
    //       (sin allocas, sin load/store)
    //    d. Emitir terminator (br, cond_br, ret)
    // 3. Para instrucciones No promovibles (Store/Load/Alloca):
    //    - Emitir alloca para las que quedan
    //    - Emitir load/store para accesos a memoria
}
```

**Phi → LLVM:** Las instrucciones phi de SSA se traducen directamente a
`LLVMBuildPhi` de LLVM. LLVM optimiza los phi nodes internamente.

**Valores directos:** Una suma `%add = add i64 %a, %b` se genera como
`builder.build_add(a, b, "add")` y el resultado es un valor LLVM que se
pasa directamente como operando a la siguiente instrucción — sin alloca,
sin store, sin load subsecuente.

---

## 5. Quick Wins (Antes de SSA)

### 5.1 i64 por defecto para literales grandes

**Archivo:** `crates/klc_semantic/src/type_checker.rs`

**Cambio:** Los literales enteros > 2^31 (2147483648) se infieren como i64
en lugar de i32. Esto elimina los casts constantes `i32 → i64` que el
lowerer emite actualmente.

```rust
// Antes: todos los literales enteros son i32
// Después:
fn infer_int_literal_type(val: i64) -> MirType {
    if val > i32::MAX as i64 || val < i32::MIN as i64 {
        MirType::I64
    } else {
        MirType::I32
    }
}
```

### 5.2 ThinLTO

**Archivo:** `crates/klc_backend/src/linker.rs`

**Cambio:** Al compilar con `--release`, pasar `-flto=thin -O3` al linker.

```rust
if is_release {
    cmd.arg("-flto=thin");
    cmd.arg("-O3");
}
```

### 5.3 Alias Analysis

**Archivo:** `crates/klc_backend/src/codegen.rs`

**Cambio:** Marcar parámetros de funciones runtime como `noalias` y/o
`readonly` para que LLVM pueda reordenar operaciones.

```rust
// Para parámetros que son solo lectura (p.ej. kl_strlen, kl_list_len):
let param_attr = llvm_ctx.create_enum_attribute(Attribute::get_named_enum_kind_id("readonly"), 0);
fn_value.add_attribute(AttributeLoc::Param(0), param_attr);
```

### 5.4 Inlining

**Archivo:** `crates/klc_mir/src/optimize.rs`

**Cambio:** Inline de funciones con < 10 instrucciones o llamadas una sola vez.

---

## 6. Orden de Implementación

```
Orden  | Optimización       | Archivos               | Impacto | Esfuerzo
-------|--------------------|------------------------|---------|---------
1      | i64 default        | type_checker.rs        | 1.5x    | 4 horas
2      | ThinLTO            | linker.rs              | 1.2x    | 2 horas
3      | Alias Analysis     | codegen.rs             | 1.2x    | 1 día
4      | Inlining           | optimize.rs            | 1.5-3x  | 2 días
5      | SsaFunction struct | mir.rs + ssa.rs        | —       | 2 días
6      | Mem2Reg pass       | ssa/mem2reg.rs         | 3-5x    | 3 días
7      | SSA Codegen        | codegen.rs             | 10-50x  | 3 días
8      | GVN                | ssa/gvn.rs             | 2-3x    | 3 días
```

---

## 7. Benchmarking

Antes y después de cada paso, ejecutar:

```bash
# Compilar Kyle
cargo build --release

# Compilar y ejecutar pruebas de rendimiento
kl run --release examples/bench/primes.kl
kl run --release examples/bench/mandelbrot.kl
kl run --release examples/bench/arithmetic.kl

# Comparar con Rust
rustc -O examples/bench/primes.rs && ./primes
```

---

## 8. Riesgos y Mitigaciones

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Phi mal colocados | Código incorrecto | Tests de regresión con todos los examples |
| Alloca escapada no detectada | Use-after-free | Verificar field_ptr/PtrOffset manualmente |
| Rendimiento empeora | Regression | Benchmark antes/después de cada cambio |
| SSA codegen muy complejo | Retrasos | Implementar incremental: primero funciones sin phi |
