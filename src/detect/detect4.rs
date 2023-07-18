// infinite_loop

use std::{collections::BTreeSet, vec};

use move_model::ty::Type;
use move_stackless_bytecode::{stackless_control_flow_graph::BlockContent, stackless_bytecode::{Bytecode, AssignKind, Operation}};

use crate::{move_ir::{generate_bytecode::{StacklessBytecodeGenerator, FunctionInfo}, fatloop::get_loops, data_dependency::data_dependency, control_flow_graph::BlockId, packages::Packages}};


pub fn detect_infinite_loop(packages: &Packages, stbgr: &StacklessBytecodeGenerator, idx: usize) -> bool {
    let function = &stbgr.functions[idx];
    let (natural_loops, fat_loops) = get_loops(function);
    let data_depent = data_dependency(packages, stbgr, idx, 1);
    let cfg = function.cfg.as_ref().unwrap();
    let mut ret_flag = if fat_loops.fat_loops.len() > 0 {true} else {false};
    for (bid, fat_loop) in fat_loops.fat_loops.iter() {
        let mut branchs: BTreeSet<BlockId> = BTreeSet::new();
        let mut unions: BTreeSet<BlockId> = BTreeSet::new();
        // 循环体中所有的block
        for natural_loop in fat_loop.sub_loops.iter() {
            let bodys = natural_loop.loop_body.clone();
            unions.append(&mut bodys.clone());
        }
        // 可能跳出循环的条件
        for union in unions.iter() {
            let children = cfg.successors(*union);
            for child in children {
                if !unions.contains(child) {
                    branchs.insert(*union);
                }
            }
        }

        let mut conditions = vec![];
        for natural_loop in fat_loop.sub_loops.iter() {
            for bid in natural_loop.loop_body.iter() {
                let content = cfg.content(*bid);
                if branchs.contains(bid) {
                    let (mut l, mut u): (u16, u16) = (0, 0);
                    if let BlockContent::Basic { lower, upper } = content {
                        l = *lower;
                        u = *upper;
                    }
                    // 条件分支语句
                    let instr = &function.code[u as usize];
                    if let Bytecode::Branch(_, _, _, src) = instr {
                        let cond = data_depent.get(*src);
                        cond.loop_condition_from_copy(&mut conditions);
                    }
                }
            }
        }

        for natural_loop in fat_loop.sub_loops.iter() {
            for bid in natural_loop.loop_body.iter() {
                for condition in conditions.iter() {
                    let content = cfg.content(*bid);
                    ret_flag = ret_flag & changed_loop_condition(function, content, *condition, 0);
                }
            }
        }
    }
    ret_flag
}

fn changed_loop_condition(function: &FunctionInfo, content: &BlockContent, condition: usize, offset: u16) -> bool {
    let mut flag = true;
    let (mut l, mut u): (u16, u16) = (0, 0);
    if let BlockContent::Basic { lower, upper } = content {
        l = *lower;
        u = *upper;
    }
    for i in (l+offset)..u {
        let instr = &function.code[i as usize];
        match instr {
            Bytecode::Assign(_, dst, src, assginkind) => {
                // 直接进行修改
                if *dst == condition {
                    flag = false;
                } else if *src == condition {
                    let refer = borrow_reference(instr, &function.local_types);
                    if let Some((src, dst, _)) = refer {
                        flag = flag & changed_loop_condition(function, content, dst, i-l+1);
                    }
                }
            },
            Bytecode::Call(_, dsts, oper, srcs, _) => {
                let refer = borrow_reference(instr, &function.local_types);
                if let Some((src, dst, _)) = refer {
                    if src == condition {
                        flag = flag & changed_loop_condition(function, content, dst, i-l+1);
                    }
                }
                if let Operation::Function(_, _, _) = oper {
                    if srcs.contains(&condition) {
                        flag = false;
                    }
                }
            },
            _ => {}
        }
    }
    flag
}

fn borrow_reference(instr: &Bytecode, local_types: &Vec<Type>) ->Option<(usize, usize, bool)> {
    match instr {
        Bytecode::Assign(_, dst, src, kind) => {
            match kind {
                AssignKind::Move => { None }
                AssignKind::Copy => { 
                    if local_types[*src].is_mutable_reference() {
                        Some((*src, *dst, false))
                    } else {
                        None
                    } 
                }
                AssignKind::Store => {
                    if local_types[*src].is_mutable_reference() {
                        Some((*src, *dst, false))
                    } else {
                        None
                    }
                }
            }
        }
        Bytecode::Call(_, dsts, oper, srcs, _) => {
            match oper {
                Operation::BorrowLoc => {
                    Some((srcs[0], dsts[0], false))
                }
                Operation::BorrowGlobal(_, _, _) => {
                    Some((srcs[0], dsts[0], false))
                }
                Operation::BorrowField(_, _, _, _) => {
                    Some((srcs[0], dsts[0], true))
                }
                _ => { None }
            }
        }
        _ => { None }
    }
}