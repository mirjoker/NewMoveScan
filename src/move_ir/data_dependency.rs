use ethnum::U256;
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    ops::{Rem, Sub},
    rc::Rc,
    str::FromStr,
    vec,
};

use move_binary_format::{
    access::ModuleAccess,
    file_format::{FunctionDefinitionIndex, FunctionHandleIndex},
    views::FunctionHandleView,
};
use move_model::ty::{PrimitiveType, Type, TypeDisplayContext};
use move_stackless_bytecode::stackless_bytecode::{
    AssignKind,
    Bytecode::{self, *},
    Constant,
    Operation::{self, *},
};

use super::{bytecode_display::oper_display, generate_bytecode::StacklessBytecodeGenerator};

#[derive(Debug, Clone)]
pub enum Val {
    ByteCode(Bytecode), // 运算符
    // 无子节点
    Const(Constant),  // 常量
    ParamType(Type),  // 函数参数类型
    AssIgn(Bytecode), // move copy store
}

#[derive(Debug, Clone)]
pub struct Node {
    pub value: Val,
    pub subnodes: Vec<Rc<RefCell<Node>>>,
    pub max: Option<U256>,
    pub is_constant: bool,
}

impl Node {
    pub fn new(value: Val, max: Option<U256>, is_constant: bool) -> Self {
        Node {
            value,
            subnodes: vec![],
            max,
            is_constant,
        }
    }

    pub fn newy_with_nodes(
        value: Val,
        nodes: Vec<Rc<RefCell<Node>>>,
        max: Option<U256>,
        is_constant: bool,
    ) -> Self {
        Node {
            value,
            subnodes: nodes,
            max,
            is_constant,
        }
    }

    pub fn new_with_node(
        value: Val,
        node: Rc<RefCell<Node>>,
        max: Option<U256>,
        is_constant: bool,
    ) -> Self {
        Node {
            value,
            subnodes: vec![node],
            max,
            is_constant,
        }
    }

    pub fn new_with_binary_nodes(
        value: Val,
        lnode: Rc<RefCell<Node>>,
        rnode: Rc<RefCell<Node>>,
        max: Option<U256>,
        is_constant: bool,
    ) -> Self {
        Node {
            value,
            subnodes: vec![lnode, rnode],
            max,
            is_constant,
        }
    }

    pub fn loop_condition_from_copy(&self, conditions: &mut Vec<usize>, params: &mut VecDeque<usize>) {
        match &self.value {
            Val::ByteCode(bc) => match bc {
                Call(_, _, BorrowLoc, srcs, _) => {
                    conditions.push(srcs[0]);
                },
                Call(_, _, Function(_, _, _), srcs, _) => {
                    params.extend(srcs);
                },
                _ => {
                    for subnode in self.subnodes.iter() {
                        subnode.borrow().loop_condition_from_copy(conditions, params);
                    }
                }
            },
            Val::AssIgn(Bytecode::Assign(_, _, idx, assignkind)) => match assignkind {
                AssignKind::Copy => {
                    conditions.push(*idx);
                },
                AssignKind::Move => {
                    conditions.push(*idx);
                },
                _ => {}
            },
            _ => {}
        };
    }

    pub fn is_const(&self) -> bool {
        let mut is_const = true;
        match &self.value {
            Val::ByteCode(bc) => {
                if let Call(_, _, _, _, _) = bc {
                    for subnode in self.subnodes.iter() {
                        is_const = is_const && subnode.borrow().is_const();
                    }
                }
            }
            Val::Const(_) => {
                is_const = is_const && true;
            }
            Val::ParamType(_) => {
                is_const = is_const && false;
            }
            Val::AssIgn(_) => {
                is_const = is_const && self.subnodes[0].borrow().is_const();
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
                        subnode.borrow().display(res, stbgr);
                        res.push_str(", ");
                    }
                    if !self.subnodes.is_empty() {
                        res.truncate(res.len() - 2);
                    }
                    res.push_str(")");
                }
            }
            Val::Const(con) => {
                let str = format!("{}", con).to_string();
                res.push_str(str.as_str());
            }
            Val::ParamType(param) => {
                let tctx = TypeDisplayContext::WithoutEnv {
                    symbol_pool: &stbgr.symbol_pool,
                    reverse_struct_table: &stbgr.reverse_struct_table,
                };
                let str = param.display(&tctx).to_string();
                res.push_str(str.as_str());
            }
            Val::AssIgn(_) => {
                // TODO 简化fmt结果
                // let str = "Assign";
                // res.push_str(str);
                // res.push_str("(");
                for subnode in self.subnodes.iter() {
                    subnode.borrow().display(res, stbgr);
                    // res.push_str(", ");
                }
                // res.truncate(res.len()-2);
                // res.push_str(")");
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct DataDepent {
    pub data: BTreeMap<usize, Node>,
}

impl DataDepent {
    pub fn insert_or_modify(&mut self, dst: usize, node: Node) {
        self.data.insert(dst, node);
    }

    pub fn get(&self, src: usize) -> Node {
        self.data.get(&src).unwrap().clone()
    }
}

impl<'a> StacklessBytecodeGenerator<'a> {
    pub fn get_data_dependency(&mut self, stbgrs: &mut Vec<StacklessBytecodeGenerator>) {
        for (idx, function) in self.functions.iter().enumerate() {
            let function_handle_idx = FunctionHandleIndex::new(idx as u16);
            let function_handle = self.module.function_handle_at(function_handle_idx);
            let view = FunctionHandleView::new(self.module, function_handle);
            let mut data_depent = DataDepent {
                data: BTreeMap::new(),
            };

            let function_defintion_idx = FunctionDefinitionIndex::new(idx as u16);
            let _self_fid = self
                .module_data
                .function_idx_to_id
                .get(&function_defintion_idx)
                .unwrap();

            // 记录函数参数类型
            for i in 0..view.arg_count() {
                let ty = &function.local_types[i];
                let uint_max = get_uint_max(ty);
                let node = Node::new(
                    Val::ParamType(function.local_types[i].clone()),
                    uint_max,
                    false,
                );
                data_depent.insert_or_modify(i, node);
            }

            for code in function.code.iter() {
                match code {
                    Assign(_, dst, src, _) => {
                        let node = data_depent.get(*src);
                        let node = Rc::new(RefCell::new(node));
                        let node = Node::new_with_node(
                            Val::AssIgn(code.clone()),
                            node.clone(),
                            node.borrow().max,
                            false,
                        );
                        data_depent.insert_or_modify(*dst, node);
                    }
                    Call(_, dsts, oper, srcs, _) => {
                        match oper {
                            // 简单的跨函数分析，如果结果来自函数调用的结果，则进入函数内部通过return指令拿到返回值的依赖
                            Function(mid, fid, _) => {
                                let mut nodes: Vec<Rc<RefCell<Node>>> = vec![];
                                // packages通过ModuleName找到被调函数的module
                                let mut option_stbgr = None;
                                let mname = &self.module_names[mid.to_usize()];
                                for stbgr in stbgrs.iter() {
                                    if stbgr.module_names[0] == *mname {
                                        option_stbgr = Some(stbgr);
                                        break;
                                    }
                                }

                                // 如果有这个module存在，则去找这个函数，如果不存在，按照原来的处理逻辑
                                if let Some(other_stbgr) = option_stbgr {
                                    // 拿到被调函数的data_dependency
                                    let mut idx = None;
                                    for (def_idx, fun_id) in other_stbgr.module_data.function_idx_to_id.iter() {
                                        if *fid == *fun_id {
                                            idx = Some(def_idx.0 as usize);
                                            break;
                                        }
                                    }
                                    if let Some(idx) = idx {
                                        let other_dd = &other_stbgr.data_dependency[idx];
                                        let other_funtion = &other_stbgr.functions[idx];
                                        if let Some(Bytecode::Ret(_, rets)) = other_funtion.code.last() {
                                            for i in 0..rets.len() {
                                                let node = other_dd.get(rets[i]);
                                                nodes.push(Rc::new(RefCell::new(node)));
                                            }
                                        }
                                    }
                                } else if mid.to_usize() == 0 { // 本cm的函数
                                    let mut idx = None;
                                    for (def_idx, fun_id) in self.module_data.function_idx_to_id.iter() {
                                        if *fid == *fun_id {
                                            idx = Some(def_idx.0 as usize);
                                            break;
                                        }
                                    }
                                    if let Some(idx) = idx {
                                        if idx < self.data_dependency.len() {
                                            let other_dd = &self.data_dependency[idx];
                                            let other_funtion = &self.functions[idx];
                                            if let Some(Bytecode::Ret(_, rets)) = other_funtion.code.last() {
                                                for i in 0..rets.len() {
                                                    let node = other_dd.get(rets[i]);
                                                    nodes.push(Rc::new(RefCell::new(node)));
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    for src in srcs {
                                        let node = data_depent.get(*src);
                                        nodes.push(Rc::new(RefCell::new(node)));
                                    }
                                }

                                for dst in dsts {
                                    let ty = &function.local_types[*dst];
                                    let max = get_uint_max(ty);
                                    let node = Node::newy_with_nodes(Val::ByteCode(code.clone()), nodes.clone(), max, false);
                                    data_depent.insert_or_modify(*dst, node.clone());
                                }
                            },
                            Operation::Sub | Add | Operation::Mul | Div | Mod | BitOr | BitAnd | Xor | Shl | Shr  => { // 二元操作
                                let lnode = data_depent.get(srcs[0]);
                                let rnode = data_depent.get(srcs[1]);
                                let ty = &function.local_types[dsts[0]];
                                // println!("{} {}", srcs[0], srcs[1]);
                                // let mut res = "".to_string();
                                // rnode.display(&mut res, &stbgr);
                                // println!("{}", res);
                                let (max, is_constant) = binary_operation_max(oper, lnode.max, rnode.max, lnode.is_constant, rnode.is_constant, ty);
                                let node = Node::new_with_binary_nodes(Val::ByteCode(code.clone()),Rc::new(RefCell::new(lnode)),Rc::new(RefCell::new(rnode)), max, is_constant);
                                data_depent.insert_or_modify(dsts[0], node);
                            },
                            Lt | Gt | Le | Ge | Or | And | Eq | Neq => { // 二元操作，返回值为bool，参数类型不确定
                                let lnode = data_depent.get(srcs[0]);
                                let rnode = data_depent.get(srcs[1]);
                                let node = Node::new_with_binary_nodes(Val::ByteCode(code.clone()),Rc::new(RefCell::new(lnode)),Rc::new(RefCell::new(rnode)), None, false);
                                data_depent.insert_or_modify(dsts[0], node);
                            },
                            CastU8 | CastU16 | CastU32 | CastU64 | CastU128 | CastU256 => { // 一元操作
                                let node = data_depent.get(srcs[0]);
                                // 源数据的最大值和cast的范围，取最小值
                                let ty = &function.local_types[dsts[0]];
                                let ty_max = get_uint_max(ty);
                                let max = get_min_uint(node.max, ty_max);
                                let is_constant = node.is_constant;
                                let node = Node::new_with_node(Val::ByteCode(code.clone()),Rc::new(RefCell::new(node)), max, is_constant);
                                data_depent.insert_or_modify(dsts[0], node);
                            },
                            Not => {
                                let node = data_depent.get(srcs[0]);
                                let node = Node::new_with_node(Val::ByteCode(code.clone()),Rc::new(RefCell::new(node)), None, false);
                                data_depent.insert_or_modify(dsts[0], node);
                            },
                            Pack(_, _, _) => { // n -> 1
                                let mut nodes = vec![];
                                for src in srcs {
                                    let node = data_depent.get(*src);
                                    nodes.push(Rc::new(RefCell::new(node)));
                                }
                                let node = Node::newy_with_nodes(Val::ByteCode(code.clone()), nodes, None, false);
                                data_depent.insert_or_modify(dsts[0], node);
                            },
                            Unpack(_, _, _) => { // 1 -> n
                                let node = data_depent.get(srcs[0]);
                                let node_rc = Rc::new(RefCell::new(node.clone()));
                                let children = &node.subnodes;
                                for (i, dst) in dsts.iter().enumerate() {
                                    // 如果结构体来自函数pack操作，可以拿到pack时，每一个成员变量的约束，否则不行
                                    let ty = &function.local_types[*dst];
                                    let max = if children.len() == dsts.len() {
                                        if children[i].borrow().max.is_none() {
                                            get_uint_max(ty)
                                        } else {
                                            children[i].borrow().max
                                        }
                                    } else {
                                        get_uint_max(ty)
                                    };
                                    let node = Node::new_with_node(Val::ByteCode(code.clone()), node_rc.clone(), max, false);
                                    data_depent.insert_or_modify(*dst, node.clone());

                                }
                            },
                            Exists(_, _, _) | FreezeRef | BorrowField(_, _, _, _) | BorrowLoc | // 1 -> 1 TODO
                                ReadRef | BorrowGlobal(_, _, _) | MoveFrom(_, _, _) => {
                                let node = data_depent.get(srcs[0]);
                                let ty = &function.local_types[dsts[0]];
                                let max = get_uint_max(ty);
                                let node = Node::new_with_node(Val::ByteCode(code.clone()),Rc::new(RefCell::new(node)), max, false);
                                data_depent.insert_or_modify(dsts[0], node);
                            },
                            _ => {
                                // WriteRef MoveTo 2 -> 0
                                continue;
                            }
                        }
                    }
                    Load(_, dst, con) => {
                        let constant = get_uint_constant(con);
                        let node = Node::new(Val::Const(con.clone()), constant, true);
                        data_depent.insert_or_modify(*dst, node.clone());
                    }
                    _ => {
                        continue;
                    }
                }
            }
            self.data_dependency.push(data_depent);
        }
    }
}

#[allow(unused)]
fn is_uint(ty: &Type) -> bool {
    let mut flag = false;
    if let Type::Primitive(bty) = ty {
        match bty {
            PrimitiveType::U8
            | PrimitiveType::U16
            | PrimitiveType::U32
            | PrimitiveType::U64
            | PrimitiveType::U128
            | PrimitiveType::U256 => {
                flag = true;
            }
            _ => {}
        }
    }
    flag
}

fn get_uint_max(ty: &Type) -> Option<U256> {
    if let Type::Primitive(bty) = ty {
        match bty {
            PrimitiveType::U8 => Some(U256::from_str("255").unwrap()),
            PrimitiveType::U16 => Some(U256::from_str("65535").unwrap()),
            PrimitiveType::U32 => Some(U256::from_str("4294967295").unwrap()),
            PrimitiveType::U64 => Some(U256::from_str("18446744073709551615").unwrap()),
            PrimitiveType::U128 => Some(U256::from_str("340282366920938463463374607431768211455").unwrap()),
            PrimitiveType::U256 => Some(U256::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap()),
            _ => None
        }
    } else {
        None
    }
}

fn get_uint_constant(constant: &Constant) -> Option<U256> {
    match constant {
        Constant::U8(c) => Some(U256::from(*c)),
        Constant::U16(c) => Some(U256::from(*c)),
        Constant::U32(c) => Some(U256::from(*c)),
        Constant::U64(c) => Some(U256::from(*c)),
        Constant::U128(c) => Some(U256::from(*c)),
        Constant::U256(c) => Some(U256::from(*c)),
        _ => None,
    }
}

fn get_min_uint(u1: Option<U256>, u2: Option<U256>) -> Option<U256> {
    let uint1 = u1.expect("Missing the value of U256");
    let uint2 = u2.expect("Missing the value of U256");
    Some(uint1.min(uint2))
}

fn binary_operation_max(
    oper: &Operation,
    u1: Option<U256>,
    u2: Option<U256>,
    c1: bool,
    c2: bool,
    ty: &Type,
) -> (Option<U256>, bool) {
    let uint1 = u1.expect("Missing the value of U256");
    let uint2 = u2.expect("Missing the value of U256");
    let mut is_constant = false;
    let mut res = U256::ZERO;
    match oper {
        Operation::Mod => {
            if c1 && c2 {
                res = uint1.rem(uint2);
                is_constant = true;
            } else if uint1 < uint2 {
                res = uint1;
                is_constant = c1;
            } else {
                res = uint2.sub(1);
            }
        }
        Operation::Sub => {
            if c1 && c2 {
                res = uint1.sub(uint2);
                is_constant = true;
            } else if c2 {
                // TODO 这里有一个问题，可能是循环引起的
                if uint1 >= uint2 {
                    res = uint1.sub(uint2);
                } else {
                    res = uint1;
                }
            } else {
                res = uint1;
            }
        }
        Operation::Add | Operation::Mul | Div | BitOr | BitAnd | Xor | Shl | Shr => match oper {
            Operation::Add | BitAnd | BitOr | Xor => {
                let (u, flag) = uint1.overflowing_add(uint2);
                res = if flag {
                    get_uint_max(ty).unwrap()
                } else {
                    u.min(get_uint_max(ty).unwrap())
                }
            }
            Operation::Mul => {
                let (u, flag) = uint1.overflowing_mul(uint2);
                res = if flag {
                    get_uint_max(ty).unwrap()
                } else {
                    u.min(get_uint_max(ty).unwrap())
                }
            }
            Div | Shr => res = uint1,
            Shl => res = uint1.wrapping_shl(uint2.as_u32()),
            _ => {}
        },
        _ => {}
    }
    (Some(res), is_constant)
}
