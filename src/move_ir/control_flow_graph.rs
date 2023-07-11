use crate::{move_ir::generate_bytecode::StacklessBytecodeGenerator, utils::utils::DotWeight};
use move_stackless_bytecode::{
    stackless_bytecode::{Bytecode, Label},
    stackless_control_flow_graph::{StacklessControlFlowGraph, BlockContent},
};
use move_binary_format::file_format::CodeOffset;
use petgraph::{dot::Dot, graph::Graph};
use std::{collections::BTreeMap, path::PathBuf, fs};

use super::bytecode_display::display;
use super::generate_bytecode::FunctionInfo;
pub type BlockId = CodeOffset;

struct DotCFGBlock<'env> {
    block_id: BlockId,
    content: BlockContent,
    label_offsets: &'env BTreeMap<Label, CodeOffset>,
    function: &'env FunctionInfo,
    stbgr: &'env StacklessBytecodeGenerator<'env>,
}

impl<'env> std::fmt::Display for DotCFGBlock<'env> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code_range = match self.content {
            BlockContent::Basic { lower, upper } => format!("offset {}..={}", lower, upper),
            BlockContent::Dummy => "X".to_owned(),
        };
        writeln!(f, "[Block {} - {}]", self.block_id, code_range)?;
        match self.content {
            BlockContent::Basic { lower, upper } => {
                let code = &self.function.code;
                for (offset, instruction) in
                    (lower..=upper).zip(&code[(lower as usize)..=(upper as usize)])
                {
                    let text = pretty_print_bytecode(
                        &self.label_offsets,
                        offset as usize,
                        instruction,
                        self.stbgr
                    );
                    writeln!(f, "{}", text)?;
                }
            }
            BlockContent::Dummy => {}
        }
        Ok(())
    }
}

pub fn pretty_print_bytecode(
    label_offsets: &BTreeMap<Label, CodeOffset>,
    offset: usize,
    code: &Bytecode,
    stbgr: &StacklessBytecodeGenerator
) -> String {
    let mut texts = vec![];
    texts.push(format!(
        "{:>3}: {}",
        offset,
        display(code, label_offsets, stbgr)
    ));

    texts.join("\n")
}

pub fn generate_cfg_in_dot_format<'env>(function: &'env FunctionInfo, dotfile: PathBuf, stbgr: &'env StacklessBytecodeGenerator) {
    let code = &function.code;
    let cfg = StacklessControlFlowGraph::new_forward(code);
    let label_offsets = Bytecode::label_offsets(code);
    let mut graph = Graph::new();

    let mut node_map = BTreeMap::new();
    for (block_id, block) in &cfg.blocks {
        let dot_block = DotCFGBlock {
            block_id: *block_id,
            content: block.content,
            label_offsets: &label_offsets,
            function,
            stbgr,
        };
        let node_index = graph.add_node(dot_block);
        node_map.insert(block_id, node_index);
    }

    // add edges
    for (block_id, block) in &cfg.blocks {
        for successor in &block.successors {
            graph.add_edge(
                *node_map.get(block_id).unwrap(),
                *node_map.get(successor).unwrap(),
                DotWeight {},
            );
        }
    }
    
    let dot_graph = format!(
        "{}",
        Dot::with_attr_getters(&graph, &[], &|_, _| "".to_string(), &|_, _| {
            "shape=box".to_string()
        })
    );
    fs::write(&dotfile, &dot_graph).expect("generating dot file for CFG");
}
