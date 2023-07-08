use std::{vec, collections::BTreeMap};

use move_binary_format::{access::ModuleAccess, views::FunctionHandleView, file_format::FunctionHandleIndex};
use move_stackless_bytecode::{stackless_bytecode::{
    Bytecode::{self, *}, Operation::{self, *}, Constant, AssignKind::{self, *},
}};
use move_model::ty::{Type, TypeDisplayContext};

use super::{generate_bytecode::StacklessBytecodeGenerator, bytecode_display::oper_display};

#[derive(Debug, Clone)]
pub enum Val {
    ByteCode(Bytecode), // 运算符
    // 无子节点
    Const(Constant), // 常量
    ParamType(Type), // 函数参数类型
    AssIgn(Bytecode), // move copy store
}

#[derive(Debug, Clone)]
pub struct Node {
    pub value: Val,
    pub subnodes: Vec<Node>,
}

impl Node {
    pub fn new(value: Val) -> Self {
        Node {
            value,
            subnodes: vec![],
        }
    }

    pub fn newy_with_nodes(value: Val, nodes: Vec<Node>) -> Self {
        Node {
            value,
            subnodes: nodes
        }
    }

    pub fn new_with_node(value: Val, node: Node) -> Self {
        Node {
            value,
            subnodes: vec![node]
        }
    }

    pub fn new_with_binary_nodes(value: Val, lnode: Node, rnode: Node) -> Self {
        Node {
            value,
            subnodes: vec![lnode, rnode]
        }
    }

    pub fn loop_condition_from_copy(&self, condtions: &mut Vec<usize>) {
        match &self.value {
            Val::ByteCode(bc) => {
                match bc {
                    Call(_, _, BorrowLoc, srcs, _) => {
                        condtions.push(srcs[0]);
                    },
                    _ => {
                        for subnode in self.subnodes.iter() {
                            subnode.loop_condition_from_copy(condtions);
                        }
                    }
                }
            },
            Val::AssIgn(Bytecode::Assign(_, _, idx, assignkind)) => {
                match assignkind {
                    AssignKind::Copy => {
                        condtions.push(*idx);
                    },
                    _ => {}
                }
            },
            _ => {}
        };
    }

    pub fn is_const(&self) -> bool {
        let mut is_const = true;
        match &self.value {
            Val::ByteCode(bc) => {
                if let Call(_, _, op, _, _) = bc {
                    for subnode in self.subnodes.iter() {
                        is_const = is_const && subnode.is_const();
                    }
                }
            },
            Val::Const(con) => {
                is_const = is_const && true;
            },
            Val::ParamType(param) => {
                is_const = is_const && false;
            },
            Val::AssIgn(_) => {
                is_const = is_const && self.subnodes[0].is_const();
            }
        };
        is_const
    }

    pub fn display(&self, res: &mut String, stbgr: &StacklessBytecodeGenerator) {
        match &self.value {
            Val::ByteCode(bc) => {
                if let Call(_, _, op, _, _) = bc {
                    let str = oper_display(&op, stbgr).to_string();
                    res.push_str(str.as_str());
                    res.push_str("(");
                    for subnode in self.subnodes.iter() {
                        subnode.display(res, stbgr);
                        res.push_str(", ");
                    }
                    res.truncate(res.len()-2);
                    res.push_str(")");
                }
            },
            Val::Const(con) => {
                let str = format!("{}", con).to_string();
                res.push_str(str.as_str());
            },
            Val::ParamType(param) => {
                let tctx = TypeDisplayContext::WithoutEnv {
                    symbol_pool: &stbgr.symbol_pool,
                    reverse_struct_table: &stbgr.reverse_struct_table,
                };
                let str = param.display(&tctx).to_string();
                res.push_str(str.as_str());
            },
            Val::AssIgn(_) => {
                let str = "Assign";
                res.push_str(str);
                res.push_str("(");
                for subnode in self.subnodes.iter() {
                    subnode.display(res, stbgr);
                    res.push_str(", ");
                }
                res.truncate(res.len()-2);
                res.push_str(")");
            }
        };
    }
}


#[derive(Debug, Clone)]
pub struct DataDepent {
    pub data: BTreeMap<usize, Node>
}

impl DataDepent {
    pub fn insert_or_modify(&mut self, dst: usize, node: Node) {
        self.data.insert(dst, node);
    }

    pub fn get(&self, src: usize) -> Node {
        self.data.get(&src).unwrap().clone()
    }
}

pub fn data_dependency<'a>(stbgr: &'a StacklessBytecodeGenerator, idx: usize) -> DataDepent {
    let function = &stbgr.functions[idx];
    let function_handle = stbgr.module.function_handle_at(FunctionHandleIndex::new(idx as u16));
    let view = FunctionHandleView::new(stbgr.module, function_handle);
    let params = view.parameters();
    let mut data_depent = DataDepent{ data: BTreeMap::new() };

    // 记录函数参数类型
    for i in 0..params.len() {
        let node = Node::new(Val::ParamType(function.local_types[i].clone()));
        data_depent.insert_or_modify(i, node);
    }

    for code in function.code.iter() {
        match code {
            Assign(_, dst, src, kind) => {
                let node = data_depent.get(*src);
                let node = Node::new_with_node(Val::AssIgn(code.clone()), node.clone());
                data_depent.insert_or_modify(*dst, node);
            }
            Call(_, dsts, oper, srcs, _) => {
                match oper {
                    Function(mid, fid, _) => { // 遇到函数调用终止，Vec的使用特殊处理
                        // if mid.eq(&stbgr.vec_module_id) { // Vec
                        //     // length borrow_mut/borrow push_back pop_back swap empty
                        //     let a = fid.symbol().display(&stbgr.symbol_pool).to_string();
                        //     let fname = a.as_str();
                        //     match fname {
                        //         "length" | "pop_back" => {
                        //             let node = data_depent.get(srcs[0]);
                        //             let node = Node::new_with_node(Val::Oper(oper.clone()), node);
                        //             data_depent.insert_or_modify(dsts[0], node);
                        //         },
                        //         "borrow_mut" | "borrow" => {
                        //             let lnode = data_depent.get(srcs[0]);
                        //             let rnode = data_depent.get(srcs[1]);
                        //             let node = Node::new_with_binary_nodes(Val::Oper(oper.clone()), lnode, rnode);
                        //             data_depent.insert_or_modify(dsts[0], node);
                        //         },
                        //         "empty" => { // 0 -> 1
                        //             let node = Node::new(Val::Oper(oper.clone()));
                        //             data_depent.insert_or_modify(dsts[0], node);
                        //         },
                        //         _ => {
                        //             // swap 3 -> 0 push_back 2 -> 0 
                        //             // continue;
                        //             // let node = Node::new(Val::Oper(oper.clone()));
                        //             let mut nodes = vec![];
                        //             for src in srcs {
                        //                 let node = data_depent.get(*src);
                        //                 nodes.push(node);
                        //             }
                        //             let node = Node::newy_with_nodes(Val::Oper(oper.clone()), nodes);
                        //             for dst in dsts {
                        //                 data_depent.insert_or_modify(*dst, node.clone());
                        //             }
                        //         }
                        //     }

                        let mut nodes = vec![];
                        for src in srcs {
                            let node = data_depent.get(*src);
                            nodes.push(node);
                        }
                        let node = Node::newy_with_nodes(Val::ByteCode(code.clone()), nodes);
                        for dst in dsts {
                            data_depent.insert_or_modify(*dst, node.clone());
                        }
                    },
                    Add | Sub | Mul | Div | Mod | BitOr | BitAnd | Xor | Shl | // 二元操作
                        Shr | Lt | Gt | Le | Ge | Or | And | Eq | Neq => {
                        let lnode = data_depent.get(srcs[0]);
                        let rnode = data_depent.get(srcs[1]);
                        let node = Node::new_with_binary_nodes(Val::ByteCode(code.clone()), lnode, rnode);
                        data_depent.insert_or_modify(dsts[0], node);
                    },
                    CastU8 | CastU16 | CastU32 | CastU64 | CastU128 | CastU256 | Not => { // 一元操作
                        let node = data_depent.get(srcs[0]);
                        let node = Node::new_with_node(Val::ByteCode(code.clone()), node);
                        data_depent.insert_or_modify(dsts[0], node);
                    },
                    Pack(_, _, _) => { // n -> 1
                        let mut nodes = vec![];
                        for src in srcs {
                            let node = data_depent.get(*src);
                            nodes.push(node);
                        }
                        let node = Node::newy_with_nodes(Val::ByteCode(code.clone()), nodes);
                        data_depent.insert_or_modify(dsts[0], node);
                    },
                    Unpack(_, _, _) => { // 1 -> n
                        let node = data_depent.get(srcs[0]);
                        let nodes = vec![node];
                        let node = Node::newy_with_nodes(Val::ByteCode(code.clone()), nodes);
                        for dst in dsts {
                            data_depent.insert_or_modify(*dst, node.clone());

                        }
                    },
                    Exists(_, _, _) | FreezeRef | BorrowField(_, _, _, _) | BorrowLoc | // 1 -> 1
                        ReadRef | BorrowGlobal(_, _, _) | MoveFrom(_, _, _) => {
                        let node = data_depent.get(srcs[0]);
                        let node = Node::new_with_node(Val::ByteCode(code.clone()), node);
                        data_depent.insert_or_modify(dsts[0], node);
                    },
                    _ => {
                        // WriteRef MoveTo 2 -> 0
                        continue;
                    }
                }
            }
            Load(_, dst, con) => {
                let node = Node::new(Val::Const(con.clone()));
                data_depent.insert_or_modify(*dst, node.clone());

            }
            _ => {
                continue;
            }
        }
    }
    data_depent
}
