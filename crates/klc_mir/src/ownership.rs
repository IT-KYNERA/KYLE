use std::collections::{HashMap, HashSet};
use crate::mir::*;

/// RAII ownership inference pass.
///
/// Inserts `kl_release` calls for heap-allocated string values (results
/// of `kl_concat`) right after their last use in each basic block.
///
/// Only releases concat results that are NOT stored to other locals
/// (forwarded). Forwarded values escape the reach of this simple pass and
/// are accepted as leaks for now. Full dataflow tracking would be needed
/// to handle aliased ownership correctly.
pub struct OwnershipPass;

impl OwnershipPass {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, module: &mut MirModule) {
        for func in &mut module.functions {
            self.process_function(func);
        }
    }

    fn process_function(&self, func: &mut MirFunction) {
        // 1. Find all concat result destinations
        let mut concat_dests: HashSet<usize> = HashSet::new();
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Call { dest: Some(d), name, .. } = inst {
                    if name == "kl_concat" {
                        concat_dests.insert(*d);
                    }
                }
            }
        }
        if concat_dests.is_empty() {
            return;
        }

        // 2. Find which concat results are forwarded (stored to another local).
        //    Forwarded values are NOT released here (they leak).
        let mut forwarded: HashSet<usize> = HashSet::new();
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Store { value: MirValue::Local(src), .. } = inst {
                    if concat_dests.contains(src) {
                        forwarded.insert(*src);
                    }
                }
            }
        }

        // 3. Only process concat results that are NOT forwarded.
        let heap_ids: Vec<usize> = concat_dests.iter().filter(|id| !forwarded.contains(id)).copied().collect();
        if heap_ids.is_empty() {
            return;
        }

        // 4. Find the max local ID for temp counter.
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

        // 5. Group non-forwarded concat results by the block they're created in.
        let mut heap_by_block: HashMap<usize, Vec<usize>> = HashMap::new();
        for (bi, bb) in func.basic_blocks.iter().enumerate() {
            for inst in &bb.insts {
                if let MirInst::Call { dest: Some(d), name, .. } = inst {
                    if name == "kl_concat" && !forwarded.contains(d) {
                        heap_by_block.entry(bi).or_default().push(*d);
                    }
                }
            }
        }

        // 6. Find which concat results are used in Return terminators (they
        //    must NOT be released — the caller takes ownership).
        let mut return_used: HashSet<usize> = HashSet::new();
        for bb in &func.basic_blocks {
            if let MirTerminator::Return(MirValue::Local(id)) = &bb.terminator {
                if concat_dests.contains(id) {
                    return_used.insert(*id);
                }
            }
        }

        // 7. For each block, find the last use of each non-forwarded concat result
        //    and insert a release after it. SKIP concat results used in Return.
        let mut tmp_counter = max_local + 1;
        struct ReleaseOp { pos: usize, id: usize, tmp: usize }

        for (bi, ids) in &heap_by_block {
            if let Some(bb) = func.basic_blocks.get_mut(*bi) {
                let mut last_use: HashMap<usize, usize> = HashMap::new();
                for (i, inst) in bb.insts.iter().enumerate() {
                    for s in srcs_of(inst) {
                        if ids.contains(&s) {
                            last_use.insert(s, i);
                        }
                    }
                    if let MirInst::Call { dest: Some(d), name, .. } = inst {
                        if name == "kl_concat" && ids.contains(d) {
                            last_use.entry(*d).or_insert(i);
                        }
                    }
                }
                let mut releases: Vec<ReleaseOp> = ids.iter().filter_map(|id| {
                    if return_used.contains(id) {
                        None  // skip — return value, caller owns it
                    } else {
                        let pos = last_use.get(id).copied().unwrap_or(bb.insts.len().saturating_sub(1));
                        let tmp = tmp_counter;
                        tmp_counter += 1;
                        Some(ReleaseOp { pos, id: *id, tmp })
                    }
                }).collect();
                releases.sort_by(|a, b| b.pos.cmp(&a.pos));
                for op in releases {
                    bb.insts.insert(op.pos + 1, MirInst::Call {
                        dest: None,
                        name: "kl_release".to_string(),
                        args: vec![MirValue::Local(op.tmp)],
                    });
                    bb.insts.insert(op.pos + 1, MirInst::Load {
                        dest: op.tmp,
                        src: op.id,
                    });
                }
            }
        }
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
