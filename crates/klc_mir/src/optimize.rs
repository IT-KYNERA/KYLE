use std::collections::{HashSet, HashMap};
use crate::mir::*;

/// MIR optimization passes.
pub struct Optimizer;

impl Optimizer {
    pub fn new() -> Self {
        Self
    }

    /// Run all optimization passes on a module.
    pub fn optimize(&self, module: &mut MirModule) {
        for func in &mut module.functions {
            self.constant_fold(func);
            self.dead_code_elim(func);
            self.remove_unreachable_blocks(func);
        }
    }

    /// Collect all local IDs referenced by a MirValue.
    fn collect_terminator_refs(term: &MirTerminator, set: &mut HashSet<usize>) {
        match term {
            MirTerminator::Return(v) => Self::collect_value_refs(v, set),
            MirTerminator::CondBr { cond, .. } => Self::collect_value_refs(cond, set),
            _ => {}
        }
    }

    fn collect_value_refs(value: &MirValue, set: &mut HashSet<usize>) {
        if let MirValue::Local(id) = value {
            set.insert(*id);
        }
    }

    /// Constant folding: evaluate constant expressions at compile time.
    fn constant_fold(&self, func: &mut MirFunction) {
        for bb in &mut func.basic_blocks {
            let mut i = 0;
            while i < bb.insts.len() {
                if let MirInst::BinaryOp { dest, op, left, right } = &bb.insts[i] {
                    if let (MirValue::Constant(lc), MirValue::Constant(rc)) = (left, right) {
                        let result = eval_const_binary_op(op, lc, rc);
                        bb.insts[i] = MirInst::Store {
                            dest: *dest,
                            value: MirValue::Constant(result),
                        };
                    }
                }
                i += 1;
            }
        }
    }

    /// Dead code elimination: remove stores whose value is never loaded.
    fn dead_code_elim(&self, func: &mut MirFunction) {
        // Simple DCE: find locals that are stored but never used
        let mut stored = HashSet::new();
        let mut used = HashSet::new();

        for bb in &func.basic_blocks {
            // Track references in terminators (Return, CondBr)
            Self::collect_terminator_refs(&bb.terminator, &mut used);

            for inst in &bb.insts {
                match inst {
                    MirInst::Store { dest, value } => {
                        stored.insert(*dest);
                        Self::collect_value_refs(value, &mut used);
                    }
                    MirInst::Load { dest, src } => {
                        used.insert(*dest);
                        used.insert(*src);
                    }
                    MirInst::Alloca { .. } => {}
                    MirInst::BinaryOp { dest, left, right, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(left, &mut used);
                        Self::collect_value_refs(right, &mut used);
                    }
                    MirInst::UnaryOp { dest, operand, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(operand, &mut used);
                    }
                    MirInst::Call { dest, args, .. } => {
                        if let Some(d) = dest {
                            used.insert(*d);
                        }
                        for arg in args {
                            Self::collect_value_refs(arg, &mut used);
                        }
                    }
                    MirInst::PtrOffset { dest, ptr, index } => {
                        used.insert(*dest);
                        used.insert(*ptr);
                        Self::collect_value_refs(index, &mut used);
                    }
                    MirInst::Cast { dest, value, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(value, &mut used);
                    }
                    MirInst::FieldPtr { dest, ptr, .. } => {
                        used.insert(*dest);
                        used.insert(*ptr);
                    }
                    MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                        used.insert(*dest_ptr_local);
                        used.insert(*src_alloca_local);
                    }
                }
            }
        }

        for bb in &mut func.basic_blocks {
            bb.insts.retain(|inst| {
                match inst {
                    MirInst::Store { dest, .. } => used.contains(dest),
                    MirInst::Alloca { dest, .. } => used.contains(dest) || stored.contains(dest),
                    _ => true,
                }
            });
        }
    }

    /// Remove unreachable basic blocks.
    fn remove_unreachable_blocks(&self, func: &mut MirFunction) {
        if func.basic_blocks.is_empty() {
            return;
        }

        let mut reachable = HashSet::new();
        let mut worklist = vec![0usize];
        reachable.insert(0usize);

        while let Some(idx) = worklist.pop() {
            if let Some(bb) = func.basic_blocks.get(idx) {
                match &bb.terminator {
                    MirTerminator::Br(label) => {
                        if let Some(target) = func.basic_blocks.iter().position(|b| &b.label == label) {
                            if reachable.insert(target) {
                                worklist.push(target);
                            }
                        }
                    }
                    MirTerminator::CondBr { true_block, false_block, .. } => {
                        for label in [true_block, false_block] {
                            if let Some(target) = func.basic_blocks.iter().position(|b| &b.label == label) {
                                if reachable.insert(target) {
                                    worklist.push(target);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Map old indices to new order
        let mut new_blocks = Vec::new();
        let mut old_to_new = HashMap::new();
        for (old_idx, bb) in func.basic_blocks.iter().enumerate() {
            if reachable.contains(&old_idx) {
                old_to_new.insert(old_idx, new_blocks.len());
                new_blocks.push(bb.clone());
            }
        }

        // Fix terminator labels to point to new indices (they're string labels, so they're fine)

        func.basic_blocks = new_blocks;
    }
}

fn eval_const_binary_op(op: &MirBinaryOp, left: &MirConstant, right: &MirConstant) -> MirConstant {
    match (left, right) {
        (MirConstant::I32(l), MirConstant::I32(r)) => {
            match op {
                MirBinaryOp::Add => MirConstant::I32(l.wrapping_add(*r)),
                MirBinaryOp::Sub => MirConstant::I32(l.wrapping_sub(*r)),
                MirBinaryOp::Mul => MirConstant::I32(l.wrapping_mul(*r)),
                MirBinaryOp::Div => if *r != 0 { MirConstant::I32(l.wrapping_div(*r)) } else { MirConstant::I32(0) },
                MirBinaryOp::Rem => if *r != 0 { MirConstant::I32(l.wrapping_rem(*r)) } else { MirConstant::I32(0) },
                MirBinaryOp::And => MirConstant::I32(l & r),
                MirBinaryOp::Or => MirConstant::I32(l | r),
                MirBinaryOp::Xor => MirConstant::I32(l ^ r),
                MirBinaryOp::Eq => MirConstant::Bool(l == r),
                MirBinaryOp::Neq => MirConstant::Bool(l != r),
                MirBinaryOp::Lt => MirConstant::Bool(l < r),
                MirBinaryOp::Gt => MirConstant::Bool(l > r),
                MirBinaryOp::Le => MirConstant::Bool(l <= r),
                MirBinaryOp::Ge => MirConstant::Bool(l >= r),
                _ => MirConstant::I32(0),
            }
        }
        (MirConstant::Bool(l), MirConstant::Bool(r)) => {
            match op {
                MirBinaryOp::And => MirConstant::Bool(*l && *r),
                MirBinaryOp::Or => MirConstant::Bool(*l || *r),
                MirBinaryOp::Eq => MirConstant::Bool(l == r),
                MirBinaryOp::Neq => MirConstant::Bool(l != r),
                _ => MirConstant::Bool(false),
            }
        }
        _ => MirConstant::Void,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_fold_add() {
        let mut func = MirFunction::new("test");
        let mut bb = MirBasicBlock::new("entry");
        bb.insts.push(MirInst::BinaryOp {
            dest: 0,
            op: MirBinaryOp::Add,
            left: MirValue::Constant(MirConstant::I32(2)),
            right: MirValue::Constant(MirConstant::I32(3)),
        });
        bb.terminator = MirTerminator::Return(MirValue::Constant(MirConstant::Void));
        func.basic_blocks.push(bb);

        let opt = Optimizer::new();
        opt.constant_fold(&mut func);

        if let MirInst::Store { value: MirValue::Constant(MirConstant::I32(n)), .. } = &func.basic_blocks[0].insts[0] {
            assert_eq!(*n, 5);
        } else {
            panic!("expected folded constant");
        }
    }

    #[test]
    fn test_remove_unreachable() {
        let mut func = MirFunction::new("test");
        func.basic_blocks.push(MirBasicBlock::new("entry"));
        func.basic_blocks[0].terminator = MirTerminator::Return(MirValue::Constant(MirConstant::Void));
        func.basic_blocks.push(MirBasicBlock::new("dead")); // unreachable

        assert_eq!(func.basic_blocks.len(), 2);
        let opt = Optimizer::new();
        opt.remove_unreachable_blocks(&mut func);
        assert_eq!(func.basic_blocks.len(), 1);
    }
}
