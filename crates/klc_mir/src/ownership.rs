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

        // Find the maximum local ID currently in use so we can generate
        // unique temp locals for the Load instructions.
        let mut max_local = 0usize;
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let Some(d) = dest_of(inst) {
                    max_local = max_local.max(d);
                }
                for s in srcs_of(inst) {
                    max_local = max_local.max(s);
                }
            }
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

                if to_release.is_empty() {
                    continue;
                }

                // Load from alloca (guarantees correct value even when the
                // defining basic block does not dominate the return block)
                let mut next_tmp = max_local + 1;
                for id in to_release {
                    let tmp = next_tmp;
                    next_tmp += 1;
                    bb.insts.push(MirInst::Load {
                        dest: tmp,
                        src: id,
                    });
                    bb.insts.push(MirInst::Call {
                        dest: None,
                        name: "kl_release".to_string(),
                        args: vec![MirValue::Local(tmp)],
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

fn dest_of(inst: &MirInst) -> Option<usize> {
    match inst {
        MirInst::Store { dest, .. } => Some(*dest),
        MirInst::Load { dest, .. } => Some(*dest),
        MirInst::Alloca { dest, .. } => Some(*dest),
        MirInst::BinaryOp { dest, .. } => Some(*dest),
        MirInst::UnaryOp { dest, .. } => Some(*dest),
        MirInst::Call { dest: Some(d), .. } => Some(*d),
        MirInst::PtrOffset { dest, .. } => Some(*dest),
        MirInst::Cast { dest, .. } => Some(*dest),
        _ => None,
    }
}

fn srcs_of(inst: &MirInst) -> Vec<usize> {
    match inst {
        MirInst::Store { value: MirValue::Local(id), .. } => vec![*id],
        MirInst::Load { src, .. } => vec![*src],
        MirInst::BinaryOp { left, right, .. } => {
            let mut v = Vec::new();
            if let MirValue::Local(id) = left { v.push(*id); }
            if let MirValue::Local(id) = right { v.push(*id); }
            v
        }
        MirInst::UnaryOp { operand, .. } => {
            if let MirValue::Local(id) = operand { vec![*id] } else { vec![] }
        }
        MirInst::Call { args, .. } => args.iter().filter_map(|a| {
            if let MirValue::Local(id) = a { Some(*id) } else { None }
        }).collect(),
        MirInst::PtrOffset { ptr, index, .. } => {
            let mut v = vec![*ptr];
            if let MirValue::Local(id) = index { v.push(*id); }
            v
        }
        MirInst::Cast { value, .. } => {
            if let MirValue::Local(id) = value { vec![*id] } else { vec![] }
        }
        _ => vec![],
    }
}
