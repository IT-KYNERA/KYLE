# MIR Optimizer

> Optimizaciones a nivel MIR: constant folding, dead code elimination, simplificaciones.
> Crate: `kyc_mir/src/optimize.rs` (888 líneas).

## Responsabilidad

Aplica optimizaciones sobre el MIR (y SSA) para mejorar el código generado.
Opera antes de la generación de LLVM IR.

## Pases de optimización

### 1. Constant Folding

Evalúa operaciones con operandos constantes en tiempo de compilación:

```rust
fn constant_fold(inst: &MirInst) -> Option<MirConstant> {
    match inst {
        BinaryOp { op: Add, left: Constant(I32(a)), right: Constant(I32(b)) } => {
            Some(I32(a.wrapping_add(*b)))
        }
        BinaryOp { op: Mul, left: Constant(I64(a)), right: Constant(I64(b)) } => {
            Some(I64(a.wrapping_mul(*b)))
        }
        // ... otros casos
    }
}
```

```ky
# Antes
x = 2 + 3 * 4

# Después (constant folding)
x = 14
```

### 2. Dead Code Elimination (DCE)

Elimina instrucciones cuyo resultado nunca se usa:

```rust
fn dead_code_elimination(block: &mut MirBasicBlock, used: &HashSet<usize>) {
    block.insts.retain(|inst| {
        let dest = inst_dest(inst);
        dest.map_or(true, |d| used.contains(&d))
    });
}
```

```ky
# Antes
x = expensive_function()  # resultado nunca usado
println("hello")

# Después
println("hello")           # llamada eliminada
```

### 3. Alloca Elimination

Elimina allocas que nunca se usan o que se pueden reemplazar por SSA values:

```rust
fn remove_unused_allocas(block: &mut MirBasicBlock, local_types: &HashMap<usize, MirType>) {
    // Simple allocas (I32, I64, etc.) that are only loaded/stored can be
    // fully promoted to SSA values and don't need an alloca
}
```

### 4. Store-Store Elimination

Elimina stores redundantes donde el mismo valor se escribe dos veces:

```rust
store i32 5, ptr %0   # se elimina si hay otro store después sin load intermedio
store i32 10, ptr %0  # solo este se mantiene
```

### 5. Load-Forwarding

Reemplaza loads con el valor stored más reciente:

```rust
store i32 42, ptr %0
%1 = load i32, ptr %0  # → reemplazar con 42 directamente
```

## Pipeline de optimización

```rust
pub fn optimize(func: &mut MirFunction) {
    constant_folding(func);        // 1. Fold constants
    dead_code_elimination(func);   // 2. Remove dead code
    simplify_cfg(func);            // 3. Simplify control flow
    // Repetir hasta punto fijo
    loop {
        let changed = false;
        changed |= constant_folding(func);
        changed |= dead_code_elimination(func);
        if !changed { break; }
    }
}
```

## Ver también

- `ssa.md` — SSA form que habilita optimizaciones avanzadas
- `codegen.md` — LLVM codegen que recibe el MIR optimizado
