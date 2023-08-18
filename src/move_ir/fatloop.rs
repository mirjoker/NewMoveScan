use std::collections::{BTreeMap, BTreeSet};

use move_binary_format::file_format::CodeOffset;
use move_model::ast::{self, TempIndex};
use move_stackless_bytecode::{stackless_bytecode::{AttrId, Label, Bytecode, Operation, BorrowNode, AbortAction, PropKind}, stackless_control_flow_graph::{StacklessControlFlowGraph, BlockContent}};

use crate::{
    move_ir::{control_flow_graph::BlockId, generate_bytecode::FunctionInfo},
    utils::graph::{Graph, NaturalLoop},
};

#[derive(Debug, Clone)]
pub struct FatLoop {
    pub invariants: BTreeMap<CodeOffset, (AttrId, ast::Exp)>,
    pub val_targets: BTreeSet<TempIndex>,
    pub mut_targets: BTreeMap<TempIndex, bool>,
    pub back_edges: BTreeSet<CodeOffset>,
    pub sub_loops: Vec<NaturalLoop<BlockId>>
}

#[derive(Debug, Clone)]
pub struct LoopAnnotation {
    pub fat_loops: BTreeMap<BlockId, FatLoop>,
}

fn collect_loop_invariants(
    cfg: &StacklessControlFlowGraph,
    function: &FunctionInfo,
    loop_header: BlockId,
) -> BTreeMap<CodeOffset, (AttrId, ast::Exp)> {
    let code = &function.code;
    // let asserts_as_invariants = &func_target.data.loop_invariants;
    let asserts_as_invariants: BTreeSet<AttrId> = BTreeSet::new();

    let mut invariants = BTreeMap::new();
    for (index, code_offset) in cfg.instr_indexes(loop_header).unwrap().enumerate() {
        let bytecode = &code[code_offset as usize];
        if index == 0 {
            assert!(matches!(bytecode, Bytecode::Label(_, _)));
        } else {
            match bytecode {
                Bytecode::Prop(attr_id, PropKind::Assert, exp)
                    if asserts_as_invariants.contains(attr_id) =>
                {
                    invariants.insert(code_offset, (*attr_id, exp.clone()));
                }
                _ => break,
            }
        }
    }
    invariants
}

pub fn modifies(
    bc: &Bytecode,
    function: &FunctionInfo,
) -> (Vec<TempIndex>, Vec<(TempIndex, bool)>) {
    /// Return the temporaries this instruction modifies and how the temporaries are modified.
    ///
    /// For a temporary with TempIndex $t, if $t is modified by the instruction and
    /// 1) $t is a value or an immutable reference, it will show up in the first Vec
    /// 2) $t is a mutable reference and only its value is modified, not the reference itself,
    ///    it will show up in the second Vec as ($t, false).
    /// 3) $t is a mutable reference and the reference itself is modified (i.e., the location and
    ///    path it is pointing to), it will show up in the second Vec as ($t, true).
    use BorrowNode::*;
    use Bytecode::*;
    use Operation::*;
    let add_abort = |mut res: Vec<TempIndex>, aa: &Option<AbortAction>| {
        if let Some(AbortAction(_, dest)) = aa {
            res.push(*dest)
        }
        res
    };

    match bc {
        Assign(_, dest, _, _) => {
            if function.local_types[*dest].is_mutable_reference() {
                // reference assignment completely distorts the reference (value + pointer)
                (vec![], vec![(*dest, true)])
            } else {
                // value assignment
                (vec![*dest], vec![])
            }
        }
        Load(_, dest, _) => {
            // constants can only be values, hence no modifications on the reference
            (vec![*dest], vec![])
        }
        Call(_, _, Operation::WriteBack(LocalRoot(dest), ..), _, aa) => {
            // write-back to a local variable distorts the value
            (add_abort(vec![*dest], aa), vec![])
        }
        Call(_, _, Operation::WriteBack(Reference(dest), ..), _, aa) => {
            // write-back to a reference only distorts the value, but not the pointer itself
            (add_abort(vec![], aa), vec![(*dest, false)])
        }
        Call(_, _, Operation::WriteRef, srcs, aa) => {
            // write-ref only distorts the value of the reference, but not the pointer itself
            (add_abort(vec![], aa), vec![(srcs[0], false)])
        }
        Call(_, dests, Function(..), srcs, aa) => {
            let mut val_targets = vec![];
            let mut mut_targets = vec![];
            for src in srcs {
                if function.local_types[*src].is_mutable_reference() {
                    // values in mutable references can be distorted, but pointer stays the same
                    mut_targets.push((*src, false));
                }
            }
            for dest in dests {
                if function.local_types[*dest].is_mutable_reference() {
                    // similar to reference assignment
                    mut_targets.push((*dest, true));
                } else {
                    // similar to value assignment
                    val_targets.push(*dest);
                }
            }
            (add_abort(val_targets, aa), mut_targets)
        }
        // *** Double-check that this is in Wolfgang's code
        Call(_, dests, _, _, aa) => {
            let mut val_targets = vec![];
            let mut mut_targets = vec![];
            for dest in dests {
                if function.local_types[*dest].is_mutable_reference() {
                    // similar to reference assignment
                    mut_targets.push((*dest, true));
                } else {
                    // similar to value assignment
                    val_targets.push(*dest);
                }
            }
            (add_abort(val_targets, aa), mut_targets)
        }
        _ => (vec![], vec![]),
    }
}

fn collect_loop_targets(
    cfg: &StacklessControlFlowGraph,
    function: &FunctionInfo,
    sub_loops: &[NaturalLoop<BlockId>],
) -> (BTreeSet<TempIndex>, BTreeMap<TempIndex, bool>) {
    let code = &function.code;
    let mut val_targets = BTreeSet::new();
    let mut mut_targets = BTreeMap::new();
    let fat_loop_body: BTreeSet<_> = sub_loops
        .iter()
        .flat_map(|l| l.loop_body.iter())
        .copied()
        .collect();
    for block_id in fat_loop_body {
        for code_offset in cfg
            .instr_indexes(block_id)
            .expect("A loop body should never contain a dummy block")
        {
            let bytecode = &code[code_offset as usize];
            let (bc_val_targets, bc_mut_targets) = modifies(bytecode, function);
            val_targets.extend(bc_val_targets);
            for (idx, is_full_havoc) in bc_mut_targets {
                mut_targets
                    .entry(idx)
                    .and_modify(|v| {
                        *v = *v || is_full_havoc;
                    })
                    .or_insert(is_full_havoc);
            }
        }
    }
    (val_targets, mut_targets)
}

fn collect_loop_back_edges(
    code: &[Bytecode],
    cfg: &StacklessControlFlowGraph,
    header_label: Label,
    sub_loops: &[NaturalLoop<BlockId>],
) -> BTreeSet<CodeOffset> {
    sub_loops
        .iter()
        .map(|l| {
            let code_offset = match cfg.content(l.loop_latch) {
                BlockContent::Dummy => {
                    panic!("A loop body should never contain a dummy block")
                }
                BlockContent::Basic { upper, .. } => *upper,
            };
            match &code[code_offset as usize] {
                Bytecode::Jump(_, goto_label) if *goto_label == header_label => {}
                Bytecode::Branch(_, if_label, else_label, _)
                    if *if_label == header_label || *else_label == header_label => {}
                _ => panic!("The latch bytecode of a loop does not branch into the header"),
            };
            code_offset
        })
        .collect()
}

pub fn get_loops(function: &FunctionInfo) -> (Vec<NaturalLoop<u16>>, LoopAnnotation) {
    let code = &function.code;
    if let Some(cfg) = function.cfg.as_ref(){
        let entry = cfg.entry_block();
        let nodes = cfg.blocks();
        let edges: Vec<(BlockId, BlockId)> = nodes
            .iter()
            .flat_map(|x| {
                cfg.successors(*x)
                    .iter()
                    .map(|y| (*x, *y))
                    .collect::<Vec<(BlockId, BlockId)>>()
            })
            .collect();
        let graph = Graph::new(entry, nodes, edges);
        let natural_loops = graph.compute_reducible().expect(
            "A well-formed Move function is expected to have a reducible control-flow graph",
        );
    
        // collect shared headers from loops
        let mut fat_headers = BTreeMap::new();
        for single_loop in natural_loops.clone() {
            fat_headers
                .entry(single_loop.loop_header)
                .or_insert_with(Vec::new)
                .push(single_loop);
        }
    
        let mut fat_loops = BTreeMap::new();
        for (fat_root, sub_loops) in fat_headers {
            // get the label of the scc root
            let label = match cfg.content(fat_root) {
                BlockContent::Dummy => panic!("A loop header should never be a dummy block"),
                BlockContent::Basic { lower, upper: _ } => match code[*lower as usize] {
                Bytecode::Label(_, label) => label,
                    _ => panic!("A loop header block is expected to start with a Label bytecode"),
                },
            };
    
            let invariants = collect_loop_invariants(cfg, function, fat_root);
            let (val_targets, mut_targets) =
                collect_loop_targets(cfg, function, &sub_loops);
            let back_edges = collect_loop_back_edges(code, &cfg, label, &sub_loops);
    
            // done with all information collection.
            fat_loops.insert(
                fat_root,
                FatLoop {
                    invariants,
                    val_targets,
                    mut_targets,
                    back_edges,
                    sub_loops
                },
            );
        }
    
        ( natural_loops, LoopAnnotation { fat_loops } )
    } else {
        panic!("cfg is none: please check if function is native.");
    }

    // let cfg = function.cfg.as_ref().unwrap();

}
