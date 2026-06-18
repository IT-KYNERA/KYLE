use std::collections::HashSet;
use crate::mir::*;

/// RAII ownership inference pass.
///
/// Inserts `kl_release` calls for heap-allocated values (e.g., results of
/// `kl_concat`) at the end of their lifetime, preventing memory leaks.
pub struct OwnershipPass;

impl OwnershipPass {
    pub fn new() -> Self {
        Self
    }

    /// Run the ownership inference pass on a MIR module.
    /// Inserts `kl_release` calls before each `Return` for any heap-allocated
    /// pointers that are not the return value.
    pub fn run(&self, module: &mut MirModule) {
        for func in &mut module.functions {
            self.process_function(func);
        }
    }

    fn process_function(&self, func: &mut MirFunction) {
        // Find all locals that hold heap-allocated pointers from kl_concat
        let heap_locals = self.find_heap_locals(func);

        if heap_locals.is_empty() {
            return;
        }

        // For each basic block, add kl_release calls before Return terminators
        for bb in &mut func.basic_blocks {
            if let MirTerminator::Return(ref val) = bb.terminator {
                // Determine which locals to release: all heap_locals except the
                // one being returned (if it's a local)
                let return_local = match val {
                    MirValue::Local(id) => Some(*id),
                    _ => None,
                };

                let to_release: Vec<usize> = heap_locals
                    .iter()
                    .copied()
                    .filter(|id| Some(*id) != return_local)
                    .collect();

                for id in to_release {
                    bb.insts.push(MirInst::Call {
                        dest: None,
                        name: "kl_release".to_string(),
                        args: vec![MirValue::Local(id)],
                    });
                }
            }
        }
    }

    /// Find all local IDs that receive the result of `kl_concat`.
    fn find_heap_locals(&self, func: &MirFunction) -> HashSet<usize> {
        let mut heap = HashSet::new();
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Call { dest: Some(d), name, .. } = inst {
                    if name == "kl_concat" {
                        heap.insert(*d);
                    }
                }
            }
        }
        heap
    }
}
