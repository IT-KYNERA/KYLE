# SSA Construction

> Transformación a Static Single Assignment form.
> Crate: `kyc_mir/src/ssa.rs` (947 líneas).

## Responsabilidad

Convierte el MIR a forma SSA (Static Single Assignment), donde cada variable
se asigna exactamente una vez. Esto simplifica análisis y optimizaciones.

## SSA form

En forma SSA, cada variable tiene una única definición:

```
// Antes (MIR normal)
%0 = alloca i32
store i32 10, ptr %0
%1 = load i32, ptr %0
%2 = add i32 %1, 1
store i32 %2, ptr %0

// Después (SSA)
%0 = 10
%1 = add i32 %0, 1
```

## Phi nodes

En puntos de join (if/else, while), se insertan nodos φ (phi) para combinar
múltiples definiciones de una misma variable:

```rust
 struct PhiNode {
     dest: usize,
     incoming: Vec<(usize, usize)>,   // (block_id, value_local)
     type_: MirType,
}

 struct SsaBlock {
     phis: Vec<PhiNode>,
     insts: Vec<SsaInst>,
     terminator: MirTerminator,
}
```

## Algoritmo

```rust
 fn convert_function(func: &MirFunction) -> Option<SsaFunction> {
    // 1. Identify locals that need phi nodes (defined in multiple blocks)
    let phi_candidates = find_phi_candidates(func);
    
    // 2. Insert phi nodes at dominance frontiers
    let dom_tree = build_dominator_tree(func);
    for &local in &phi_candidates {
        for frontier_block in dom_frontier(local, &dom_tree) {
            insert_phi(func, frontier_block, local);
        }
    }
    
    // 3. Rename: replace each local use with its SSA version
    let mut ssa_values = HashMap::new();
    rename_variables(func, &mut ssa_values);
    
    // 4. Build SsaFunction
    SsaFunction { blocks: ssa_blocks }
}
```

## Dominator Tree

El árbol de dominadores determina qué bloques "dominan" a otros:

```
Entry
  │
  ▼
bb0 (check)
  │
  ├──► bb1 (body) ◄──┐
  │       │           │
  │       └───────────┘
  │
  └──► bb2 (done)
```

Dominadores:
- `bb0` domina a `bb1` y `bb2`
- `bb1` solo se domina a sí mismo
- `bb2` solo se domina a sí mismo

## Optimizaciones SSA

### GVN (Global Value Numbering)

Detecta expresiones redundantes y las reemplaza:

```rust
// Antes
%2 = add i32 %0, %1
%3 = add i32 %0, %1
%4 = add i32 %3, %2

// Después (GVN)
%2 = add i32 %0, %1
%4 = add i32 %2, %2
```

### Constant Folding

Evalúa expresiones constantes en compile-time:

```rust
// Antes
%0 = 10
%1 = add i32 %0, 5

// Después
%0 = 15
```

## Estructura de salida

```rust
 struct SsaFunction {
     name: String,
     params: Vec<MirType>,
     return_type: MirType,
     blocks: Vec<SsaBlock>,
     param_modes: Vec<ParamMode>,
}

 struct SsaBlock {
     label: String,
     insts: Vec<SsaInst>,
     terminator: MirTerminator,
}
```

## Ver también

- `mir.md` — MIR antes de SSA
- `optimizer.md` — Optimizaciones post-SSA
- `codegen.md` — Codegen desde SSA
