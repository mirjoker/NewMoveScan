use itertools::Itertools;
use move_binary_format::{
    access::ModuleAccess,
    file_format::{
        Bytecode as MoveBytecode, CodeOffset, Constant as VMConstant, FunctionDefinitionIndex,
        StructDefinitionIndex,
    },
    internals::ModuleIndex,
    views::{FunctionDefinitionView, FunctionHandleView},
    CompiledModule,
};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{self, CORE_CODE_ADDRESS};
use move_model::{
    ast::{ModuleName, QualifiedSymbol, TempIndex},
    model::{FunId, FunctionData, Loc, ModuleData, ModuleId, QualifiedId, StructId},
    symbol::SymbolPool,
    ty::{PrimitiveType, Type},
};
use move_stackless_bytecode::{
    stackless_bytecode::{AssignKind, AttrId, Bytecode, Constant, Label, Operation},
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use num::{BigUint, Num};
use petgraph::graph::{Graph, DiGraph, NodeIndex};
use std::{
    collections::{BTreeMap, BTreeSet},
    vec,
};
use super::{utils::*, data_dependency::DataDepent};

pub fn addr_to_big_uint(addr: &AccountAddress) -> BigUint {
    BigUint::from_str_radix(&addr.to_string(), 16).unwrap()
}
pub struct FunctionInfo {
    pub idx: usize,
    pub name: String,
    pub args_count: usize,
    pub code: Vec<Bytecode>,
    pub local_types: Vec<Type>,
    pub location_table: BTreeMap<AttrId, Loc>,
    pub loop_invariants: BTreeSet<AttrId>,
    pub fallthrough_labels: BTreeSet<Label>,
    pub cfg: Option<StacklessControlFlowGraph>,
    pub def_attrid: Vec<Vec<usize>>,
    pub use_attrid: Vec<Vec<usize>>,
}

impl FunctionInfo {
    pub fn new(idx: usize, name: String) -> Self {
        FunctionInfo {
            idx,
            name,
            args_count: 0,
            code: vec![],
            local_types: vec![],
            location_table: BTreeMap::new(),
            loop_invariants: BTreeSet::new(),
            fallthrough_labels: BTreeSet::new(),
            cfg: None,
            def_attrid: vec![],
            use_attrid: vec![],
        }
    }
}
pub struct StacklessBytecodeGenerator<'a> {
    pub module: &'a CompiledModule,
    pub module_data: ModuleData,

    pub temp_count: usize,
    pub temp_stack: Vec<usize>,

    pub symbol_pool: SymbolPool, // 全局符号池，相当于globalenv的符号池
    pub reverse_struct_table: BTreeMap<(ModuleId, StructId), QualifiedSymbol>, // 存放所有module name，其中ModuleId和ModuleHandleIndex在usize上一一对应
    pub module_names: Vec<ModuleName>, // 所有的module handle
    pub func_to_node: BTreeMap<QualifiedId<FunId>, NodeIndex>,
    pub call_graph: Graph<QualifiedId<FunId>, ()>,

    pub vec_module_id: ModuleId, // vector module id

    pub functions: Vec<FunctionInfo>,
    pub data_dependency: Vec<DataDepent>,
}

impl<'a> StacklessBytecodeGenerator<'a> {
    pub fn new(cm: &'a CompiledModule) -> Self {
        let id = cm.self_id();
        let addr = addr_to_big_uint(id.address());
        let symbol_pool = SymbolPool::new();
        let module_name = ModuleName::new(addr, symbol_pool.make(id.name().as_str()));
        let module_id = ModuleId::new(0);
        let mut module_data = ModuleData::stub(module_name.clone(), module_id, cm.clone());

        // add module handle
        let mut module_names = vec![];
        let mut vec_module_id_opt: Option<ModuleId> = None;
        let mut flag = false;
        for (i, module_handle) in cm.module_handles.iter().enumerate() {
            let move_module_id = cm.module_id_for_handle(module_handle);
            let addr = addr_to_big_uint(move_module_id.address());
            let module_name =
                ModuleName::new(addr, symbol_pool.make(move_module_id.name().as_str()));
            // 事先找好vector的module，后面会多次使用
            if move_module_id.name().as_str() == "vector" {
                vec_module_id_opt = Some(ModuleId::new(i));
                flag = true;
            }
            module_names.push(module_name);
        }
        // TODO 目前不确定vector是是否在module_handles之中
        if !flag {
            let storage_id = language_storage::ModuleId::new(
                CORE_CODE_ADDRESS,
                move_core_types::identifier::Identifier::new("vector").unwrap(),
            );
            let vec_module = ModuleName::from_str(
                &storage_id.address().to_string(),
                symbol_pool.make(storage_id.name().as_str()),
            );
            let index = module_names.len();
            module_names.push(vec_module);
            vec_module_id_opt = Some(ModuleId::new(index));
        }

        // add functions
        for (i, def) in cm.function_defs().iter().enumerate() {
            let def_idx = FunctionDefinitionIndex(i as u16);
            let name = cm.identifier_at(cm.function_handle_at(def.function).name);
            let symbol = symbol_pool.make(name.as_str());
            let fun_id = FunId::new(symbol);
            let data = FunctionData::stub(symbol, def_idx, def.function);
            module_data.function_data.insert(fun_id, data);
            module_data.function_idx_to_id.insert(def_idx, fun_id);
        }

        let mut reverse_struct_table = BTreeMap::new();
        // add structs
        for (i, def) in cm.struct_defs().iter().enumerate() {
            let def_idx = StructDefinitionIndex(i as u16);
            let name = cm.identifier_at(cm.struct_handle_at(def.struct_handle).name);
            let symbol = symbol_pool.make(name.as_str());
            let struct_id = StructId::new(symbol);
            let data = create_move_struct_data(
                &symbol_pool,
                cm,
                def_idx,
                symbol,
                Loc::default(),
                Vec::default(),
            );
            module_data.struct_data.insert(struct_id, data);
            module_data.struct_idx_to_id.insert(def_idx, struct_id);

            let addr = addr_to_big_uint(id.address());
            let module_name = ModuleName::new(addr, symbol_pool.make(id.name().as_str()));
            let qsymbol = QualifiedSymbol {
                module_name,
                symbol,
            };
            reverse_struct_table.insert((module_id, struct_id), qsymbol);
        }

        for handle in cm.struct_handles().iter() {
            let module_index = handle.module.into_index();
            let module_name = module_names[module_index].clone();
            let name = cm.identifier_at(handle.name);
            let symbol = symbol_pool.make(name.as_str());
            let struct_id = StructId::new(symbol);
            let qsymbol = QualifiedSymbol {
                module_name,
                symbol,
            };
            reverse_struct_table.insert((ModuleId::new(module_index), struct_id), qsymbol);
        }

        StacklessBytecodeGenerator {
            module: cm,
            module_data,
            temp_count: 0,
            temp_stack: vec![],
            symbol_pool: symbol_pool,
            reverse_struct_table,
            module_names,
            vec_module_id: vec_module_id_opt.unwrap(),
            functions: vec![],
            data_dependency: vec![],
            func_to_node: BTreeMap::new(),
            call_graph: Graph::new(),
        }
    }

    pub fn generate_function(&mut self) {
        for (idx, func_def) in self.module.function_defs.iter().enumerate() {
            let func_def_idx = FunctionDefinitionIndex::new(idx as u16);
            let view = FunctionDefinitionView::new(self.module, func_def);

            let handle = self.module.function_handle_at(func_def.function);
            let fname = self.module.identifier_at(handle.name).to_string();

            let mut function = FunctionInfo::new(idx, fname);
            let local_count = match view.locals_signature() {
                Some(locals_view) => locals_view.len(),
                None => view.parameters().len(),
            };
            let local_types = (0..local_count)
                .map(|i| self.get_local_type(&view, i))
                .collect_vec();
            self.temp_count = local_types.len();
            function.args_count = local_types.len();
            function.local_types = local_types;

            let original_code = match &func_def.code {
                Some(code) => code.code.as_slice(),
                None => &[],
            };
            let mut label_map = BTreeMap::new();
            for (pos, bytecode) in original_code.iter().enumerate() {
                if let MoveBytecode::BrTrue(code_offset)
                | MoveBytecode::BrFalse(code_offset)
                | MoveBytecode::Branch(code_offset) = bytecode
                {
                    let offs = *code_offset as CodeOffset;
                    if label_map.get(&offs).is_none() {
                        let label = Label::new(label_map.len());
                        label_map.insert(offs, label);
                    }
                }
                if let MoveBytecode::BrTrue(_) | MoveBytecode::BrFalse(_) = bytecode {
                    let next_offs = (pos + 1) as CodeOffset;
                    if label_map.get(&next_offs).is_none() {
                        let fall_through_label = Label::new(label_map.len());
                        label_map.insert(next_offs, fall_through_label);
                        function.fallthrough_labels.insert(fall_through_label);
                    }
                };
            }

            // Generate bytecode.
            for (code_offset, bytecode) in original_code.iter().enumerate() {
                self.generate_bytecode(
                    func_def_idx,
                    &view,
                    bytecode,
                    code_offset as CodeOffset,
                    &label_map,
                    &mut function,
                );
            }

            // Eliminate fall-through for non-branching instructions
            let code = std::mem::take(&mut function.code);
            for bytecode in code.into_iter() {
                if let Bytecode::Label(attr_id, label) = bytecode {
                    if !function.code.is_empty()
                        && !function.code[function.code.len() - 1].is_branch()
                    {
                        function.code.push(Bytecode::Jump(attr_id, label));
                    }
                }
                function.code.push(bytecode);
            }

            let n_ty = function.local_types.len();
            let mut def_attrid: Vec<Vec<usize>> = vec![vec![]; n_ty];
            let mut use_attrid: Vec<Vec<usize>> = vec![vec![]; n_ty];
            for (offset, code) in function.code.iter().enumerate() {
                match code {
                    Bytecode::Assign(_, dst, src, _) => {
                        def_attrid[*dst].push(offset);
                        use_attrid[*src].push(offset);
                    }
                    Bytecode::Call(_, dsts, _, srcs, _) => {
                        dsts.iter().for_each(|dst| {
                            def_attrid[*dst].push(offset);
                        });
                        srcs.iter().for_each(|src| {
                            use_attrid[*src].push(offset);
                        });
                    }
                    Bytecode::Ret(_, srcs) => {
                        srcs.iter().for_each(|src| {
                            use_attrid[*src].push(offset);
                        });
                    }
                    Bytecode::Load(_, dst, _) => {
                        def_attrid[*dst].push(offset);
                    }
                    Bytecode::Branch(_, _, _, src) => {
                        use_attrid[*src].push(offset);
                    }
                    Bytecode::Abort(_, src) => {
                        use_attrid[*src].push(offset);
                    }
                    _ => {}
                }
            }

            function.def_attrid = def_attrid;
            function.use_attrid = use_attrid;

            self.functions.push(function);
        }
    }

    pub fn generate_bytecode(
        &mut self,
        func_def_idx: FunctionDefinitionIndex,
        view: &FunctionDefinitionView<'_, CompiledModule>,
        bytecode: &MoveBytecode,
        code_offset: CodeOffset,
        label_map: &BTreeMap<CodeOffset, Label>,
        function: &mut FunctionInfo,
    ) {
        // Add label if defined at this code offset.
        if let Some(label) = label_map.get(&code_offset) {
            let label_attr_id = self.new_loc_attr(func_def_idx, function, code_offset);
            function.code.push(Bytecode::Label(label_attr_id, *label));
        }

        let attr_id = self.new_loc_attr(func_def_idx, function, code_offset);

        let mk_vec_function_operation = |name: &str, tys: Vec<Type>| -> Operation {
            let vec_fun = FunId::new(self.symbol_pool.make(name));
            Operation::Function(self.vec_module_id, vec_fun, tys)
        };

        let mk_call = |op: Operation, dsts: Vec<usize>, srcs: Vec<usize>| -> Bytecode {
            Bytecode::Call(attr_id, dsts, op, srcs, None)
        };
        let mk_unary = |op: Operation, dst: usize, src: usize| -> Bytecode {
            Bytecode::Call(attr_id, vec![dst], op, vec![src], None)
        };
        let mk_binary = |op: Operation, dst: usize, src1: usize, src2: usize| -> Bytecode {
            Bytecode::Call(attr_id, vec![dst], op, vec![src1, src2], None)
        };

        match bytecode {
            MoveBytecode::Pop => {
                let temp_index = self.temp_stack.pop().unwrap();
                function
                    .code
                    .push(mk_call(Operation::Destroy, vec![], vec![temp_index]));
            }
            MoveBytecode::BrTrue(target) => {
                let temp_index = self.temp_stack.pop().unwrap();
                function.code.push(Bytecode::Branch(
                    attr_id,
                    *label_map.get(target).unwrap(),
                    *label_map.get(&(code_offset + 1)).unwrap(),
                    temp_index,
                ));
            }

            MoveBytecode::BrFalse(target) => {
                let temp_index = self.temp_stack.pop().unwrap();
                function.code.push(Bytecode::Branch(
                    attr_id,
                    *label_map.get(&(code_offset + 1)).unwrap(),
                    *label_map.get(target).unwrap(),
                    temp_index,
                ));
            }

            MoveBytecode::Abort => {
                let error_code_index = self.temp_stack.pop().unwrap();
                function
                    .code
                    .push(Bytecode::Abort(attr_id, error_code_index));
            }

            MoveBytecode::StLoc(idx) => {
                let operand_index = self.temp_stack.pop().unwrap();
                function.code.push(Bytecode::Assign(
                    attr_id,
                    *idx as TempIndex,
                    operand_index,
                    AssignKind::Store,
                ));
            }

            MoveBytecode::Ret => {
                let mut return_temps = vec![];
                for _ in 0..view.return_().len() {
                    // OK
                    let return_temp_index = self.temp_stack.pop().unwrap();
                    return_temps.push(return_temp_index);
                }
                return_temps.reverse();
                function.code.push(Bytecode::Ret(attr_id, return_temps));
            }

            MoveBytecode::Branch(target) => {
                // Attempt to eliminate the common pattern `if c goto L1 else L2; L2: goto L3`
                // and replace it with `if c goto L1 else L3`, provided L2 is a fall-through
                // label, i.e. not referenced from elsewhere.
                let target_label = *label_map.get(target).unwrap();
                let at = function.code.len();
                let rewritten = if at >= 2 {
                    match (&function.code[at - 2], &function.code[at - 1]) {
                        (
                            Bytecode::Branch(attr, if_true, if_false, c),
                            Bytecode::Label(_, cont),
                        ) if function.fallthrough_labels.contains(cont) && if_false == cont => {
                            let bc = Bytecode::Branch(*attr, *if_true, target_label, *c);
                            function.code.pop();
                            function.code.pop();
                            function.code.push(bc);
                            true
                        }
                        _ => false,
                    }
                } else {
                    false
                };
                if !rewritten {
                    function.code.push(Bytecode::Jump(attr_id, target_label));
                }
            }

            MoveBytecode::FreezeRef => {
                let mutable_ref_index = self.temp_stack.pop().unwrap();
                let mutable_ref_sig = function.local_types[mutable_ref_index].clone();
                if let Type::Reference(is_mut, signature) = mutable_ref_sig {
                    if is_mut {
                        let immutable_ref_index = self.temp_count;
                        self.temp_stack.push(immutable_ref_index);
                        function.local_types.push(Type::Reference(false, signature));
                        function.code.push(mk_call(
                            Operation::FreezeRef,
                            vec![immutable_ref_index],
                            vec![mutable_ref_index],
                        ));
                        self.temp_count += 1;
                    }
                }
            }

            MoveBytecode::ImmBorrowField(field_handle_index)
            | MoveBytecode::MutBorrowField(field_handle_index) => {
                let field_handle = self.module.field_handle_at(*field_handle_index);
                let struct_def = self.module.struct_def_at(field_handle.owner);
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let (struct_id, field_offset, field_type) =
                    self.get_field_info(*field_handle_index);
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);
                function.code.push(mk_call(
                    Operation::BorrowField(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        struct_id,
                        vec![],
                        field_offset,
                    ),
                    vec![field_ref_index],
                    vec![struct_ref_index],
                ));
                self.temp_count += 1;
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowField(..));
                function
                    .local_types
                    .push(Type::Reference(is_mut, Box::new(field_type)));
            }

            MoveBytecode::ImmBorrowFieldGeneric(field_inst_index)
            | MoveBytecode::MutBorrowFieldGeneric(field_inst_index) => {
                let field_inst = self.module.field_instantiation_at(*field_inst_index);
                let field_handle = self.module.field_handle_at(field_inst.handle);
                let struct_def = self.module.struct_def_at(field_handle.owner);
                let struct_ref_index = self.temp_stack.pop().unwrap();
                let (struct_id, field_offset, base_field_type) =
                    self.get_field_info(field_inst.handle);
                let actuals = self.get_type_params(field_inst.type_parameters);
                let field_type = base_field_type.instantiate(&actuals);
                let field_ref_index = self.temp_count;
                self.temp_stack.push(field_ref_index);
                function.code.push(mk_call(
                    Operation::BorrowField(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        struct_id,
                        actuals,
                        field_offset,
                    ),
                    vec![field_ref_index],
                    vec![struct_ref_index],
                ));
                self.temp_count += 1;
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowFieldGeneric(..));
                function
                    .local_types
                    .push(Type::Reference(is_mut, Box::new(field_type)));
            }

            MoveBytecode::LdU8(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U8));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U8(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU16(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U16));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U16(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU32(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U32));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U32(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU64(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U64));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U64(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU256(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U256));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::from(number)));
                self.temp_count += 1;
            }

            MoveBytecode::LdU128(number) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U128));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::U128(*number)));
                self.temp_count += 1;
            }

            MoveBytecode::CastU8 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U8));
                function
                    .code
                    .push(mk_unary(Operation::CastU8, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU16 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U16));
                function
                    .code
                    .push(mk_unary(Operation::CastU16, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU32 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U32));
                function
                    .code
                    .push(mk_unary(Operation::CastU32, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU64 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U64));
                function
                    .code
                    .push(mk_unary(Operation::CastU64, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU128 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U128));
                function
                    .code
                    .push(mk_unary(Operation::CastU128, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::CastU256 => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U256));
                function
                    .code
                    .push(mk_unary(Operation::CastU256, temp_index, operand_index));
                self.temp_count += 1;
            }

            MoveBytecode::LdConst(idx) => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                let constant = &self.module.constant_pool()[idx.0 as usize];
                let ty = self.globalize_signature(&constant.type_);
                let value = Self::translate_value(
                    &ty,
                    &VMConstant::deserialize_constant(&constant).unwrap(),
                );
                function.local_types.push(ty);
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, value));
                self.temp_count += 1;
            }

            MoveBytecode::LdTrue => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::Bool(true)));
                self.temp_count += 1;
            }

            MoveBytecode::LdFalse => {
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                function
                    .code
                    .push(Bytecode::Load(attr_id, temp_index, Constant::Bool(false)));
                self.temp_count += 1;
            }

            MoveBytecode::CopyLoc(idx) => {
                let signature = self.get_local_type(&view, *idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function.local_types.push(signature); // same type as the value copied
                function.code.push(Bytecode::Assign(
                    attr_id,
                    temp_index,
                    *idx as TempIndex,
                    AssignKind::Copy,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::MoveLoc(idx) => {
                let signature = self.get_local_type(&view, *idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function.local_types.push(signature); // same type as the value copied
                function.code.push(Bytecode::Assign(
                    attr_id,
                    temp_index,
                    *idx as TempIndex,
                    AssignKind::Move,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::MutBorrowLoc(idx) => {
                let signature = self.get_local_type(&view, *idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Reference(true, Box::new(signature)));
                function.code.push(mk_unary(
                    Operation::BorrowLoc,
                    temp_index,
                    *idx as TempIndex,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::ImmBorrowLoc(idx) => {
                let signature = self.get_local_type(&view, *idx as usize);
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function
                    .local_types
                    .push(Type::Reference(false, Box::new(signature)));
                function.code.push(mk_unary(
                    Operation::BorrowLoc,
                    temp_index,
                    *idx as TempIndex,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::Call(idx) => {
                let function_handle = self.module.function_handle_at(*idx);
                let function_handle_view = FunctionHandleView::new(self.module, function_handle);
                let mut arg_temp_indices = vec![];
                let mut return_temp_indices = vec![];
                for _ in function_handle_view.arg_tokens() {
                    let arg_temp_index = self.temp_stack.pop().unwrap();
                    arg_temp_indices.push(arg_temp_index);
                }
                for return_type_view in function_handle_view.return_tokens() {
                    let return_temp_index = self.temp_count;
                    let return_type = self.globalize_signature(return_type_view.signature_token());
                    return_temp_indices.push(return_temp_index);
                    self.temp_stack.push(return_temp_index);
                    function.local_types.push(return_type);
                    self.temp_count += 1;
                }
                arg_temp_indices.reverse();
                function.code.push(mk_call(
                    Operation::Function(
                        ModuleId::new(self.get_module_handle_index_of_func(idx)),
                        self.get_fun_id_by_idx(idx),
                        vec![],
                    ),
                    return_temp_indices,
                    arg_temp_indices,
                ))
            }
            MoveBytecode::CallGeneric(idx) => {
                let func_instantiation = self.module.function_instantiation_at(*idx);

                let type_sigs = self.get_type_params(func_instantiation.type_parameters);
                let function_handle = self.module.function_handle_at(func_instantiation.handle);
                let function_handle_view = FunctionHandleView::new(self.module, function_handle);

                let mut arg_temp_indices = vec![];
                let mut return_temp_indices = vec![];
                for _ in function_handle_view.arg_tokens() {
                    let arg_temp_index = self.temp_stack.pop().unwrap();
                    arg_temp_indices.push(arg_temp_index);
                }
                for return_type_view in function_handle_view.return_tokens() {
                    let return_temp_index = self.temp_count;
                    // instantiate type parameters
                    let return_type = self
                        .globalize_signature(return_type_view.signature_token())
                        .instantiate(&type_sigs);
                    return_temp_indices.push(return_temp_index);
                    self.temp_stack.push(return_temp_index);
                    function.local_types.push(return_type);
                    self.temp_count += 1;
                }
                arg_temp_indices.reverse();
                function.code.push(mk_call(
                    Operation::Function(
                        ModuleId::new(
                            self.get_module_handle_index_of_func(&func_instantiation.handle),
                        ),
                        self.get_fun_id_by_idx(&func_instantiation.handle),
                        type_sigs,
                    ),
                    return_temp_indices,
                    arg_temp_indices,
                ))
            }

            MoveBytecode::Pack(idx) => {
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;

                let struct_def = self.module.struct_def_at(*idx);
                for _ in 0..struct_def.declared_field_count().unwrap() {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }

                let struct_handle = self.module.struct_def_at(*idx).struct_handle;
                function.local_types.push(Type::Struct(
                    ModuleId::new(
                        self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                    ),
                    self.get_struct_id_by_idx(&struct_handle),
                    vec![],
                ));
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                function.code.push(mk_call(
                    Operation::Pack(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_handle),
                        vec![],
                    ),
                    vec![struct_temp_index],
                    field_temp_indices,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::PackGeneric(idx) => {
                let struct_instantiation = &self.module.struct_def_instantiations[idx.into_index()];
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_count;

                let struct_handle = self.module.struct_instantiation_at(*idx);
                let struct_def = self.module.struct_def_at(struct_handle.def);
                let count = struct_def.declared_field_count().unwrap();
                for _ in 0..count {
                    let field_temp_index = self.temp_stack.pop().unwrap();
                    field_temp_indices.push(field_temp_index);
                }
                function.local_types.push(Type::Struct(
                    ModuleId::new(
                        self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                    ),
                    self.get_struct_id_by_idx(&struct_def.struct_handle),
                    actuals.clone(),
                ));
                self.temp_stack.push(struct_temp_index);
                field_temp_indices.reverse();
                function.code.push(mk_call(
                    Operation::Pack(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        actuals,
                    ),
                    vec![struct_temp_index],
                    field_temp_indices,
                ));
                self.temp_count += 1;
            }

            MoveBytecode::Unpack(idx) => {
                let struct_def = self.module.struct_def_at(*idx);
                let name = self
                    .module
                    .identifier_at(self.module.struct_handle_at(struct_def.struct_handle).name);
                let symbol = self.symbol_pool.make(name.as_str());
                let struct_data = create_move_struct_data(
                    &self.symbol_pool,
                    self.module,
                    *idx,
                    symbol,
                    Loc::default(),
                    Vec::default(),
                );
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                for field_data in struct_data
                    .field_data
                    .values()
                    .sorted_by_key(|data| data.offset)
                {
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    function.local_types.push(self.get_type(field_data));
                    self.temp_count += 1;
                }
                function.code.push(mk_call(
                    Operation::Unpack(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        vec![],
                    ),
                    field_temp_indices,
                    vec![struct_temp_index],
                ));
            }

            MoveBytecode::UnpackGeneric(idx) => {
                let struct_instantiation = &self.module.struct_def_instantiations[idx.into_index()];
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                let mut field_temp_indices = vec![];
                let struct_temp_index = self.temp_stack.pop().unwrap();
                let struct_def_id = self.module.struct_instantiation_at(*idx).def;
                let struct_handle = self.module.struct_instantiation_at(*idx);
                let struct_def = self.module.struct_def_at(struct_handle.def);

                let name = self
                    .module
                    .identifier_at(self.module.struct_handle_at(struct_def.struct_handle).name);
                let symbol = self.symbol_pool.make(name.as_str());
                let struct_data = create_move_struct_data(
                    &self.symbol_pool,
                    self.module,
                    struct_def_id,
                    symbol,
                    Loc::default(),
                    Vec::default(),
                );
                for field_data in struct_data
                    .field_data
                    .values()
                    .sorted_by_key(|data| data.offset)
                {
                    let field_type = self.get_type(field_data).instantiate(&actuals);
                    let field_temp_index = self.temp_count;
                    field_temp_indices.push(field_temp_index);
                    self.temp_stack.push(field_temp_index);
                    function.local_types.push(field_type);
                    self.temp_count += 1;
                }
                function.code.push(mk_call(
                    Operation::Unpack(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        actuals,
                    ),
                    field_temp_indices,
                    vec![struct_temp_index],
                ));
            }

            MoveBytecode::ReadRef => {
                let operand_index = self.temp_stack.pop().unwrap();
                let operand_sig = function.local_types[operand_index].clone();
                let temp_index = self.temp_count;
                if let Type::Reference(_, signature) = operand_sig {
                    function.local_types.push(*signature);
                }
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                function
                    .code
                    .push(mk_unary(Operation::ReadRef, temp_index, operand_index));
            }

            MoveBytecode::WriteRef => {
                let ref_operand_index = self.temp_stack.pop().unwrap();
                let val_operand_index = self.temp_stack.pop().unwrap();
                function.code.push(mk_call(
                    Operation::WriteRef,
                    vec![],
                    vec![ref_operand_index, val_operand_index],
                ));
            }

            MoveBytecode::Add
            | MoveBytecode::Sub
            | MoveBytecode::Mul
            | MoveBytecode::Mod
            | MoveBytecode::Div
            | MoveBytecode::BitOr
            | MoveBytecode::BitAnd
            | MoveBytecode::Xor
            | MoveBytecode::Shl
            | MoveBytecode::Shr => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let operand_type = function.local_types[operand1_index].clone();
                let temp_index = self.temp_count;
                function.local_types.push(operand_type);
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                match bytecode {
                    MoveBytecode::Add => {
                        function.code.push(mk_binary(
                            Operation::Add,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Sub => {
                        function.code.push(mk_binary(
                            Operation::Sub,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Mul => {
                        function.code.push(mk_binary(
                            Operation::Mul,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Mod => {
                        function.code.push(mk_binary(
                            Operation::Mod,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Div => {
                        function.code.push(mk_binary(
                            Operation::Div,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::BitOr => {
                        function.code.push(mk_binary(
                            Operation::BitOr,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::BitAnd => {
                        function.code.push(mk_binary(
                            Operation::BitAnd,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Xor => {
                        function.code.push(mk_binary(
                            Operation::Xor,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Shl => {
                        function.code.push(mk_binary(
                            Operation::Shl,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Shr => {
                        function.code.push(mk_binary(
                            Operation::Shr,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    _ => {}
                }
            }
            MoveBytecode::Or => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(mk_binary(
                    Operation::Or,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }

            MoveBytecode::And => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(mk_binary(
                    Operation::And,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }

            MoveBytecode::Not => {
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function
                    .code
                    .push(mk_unary(Operation::Not, temp_index, operand_index));
            }
            MoveBytecode::Eq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(mk_binary(
                    Operation::Eq,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }
            MoveBytecode::Neq => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(mk_binary(
                    Operation::Neq,
                    temp_index,
                    operand1_index,
                    operand2_index,
                ));
            }
            MoveBytecode::Lt | MoveBytecode::Gt | MoveBytecode::Le | MoveBytecode::Ge => {
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                match bytecode {
                    MoveBytecode::Lt => {
                        function.code.push(mk_binary(
                            Operation::Lt,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Gt => {
                        function.code.push(mk_binary(
                            Operation::Gt,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Le => {
                        function.code.push(mk_binary(
                            Operation::Le,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    MoveBytecode::Ge => {
                        function.code.push(mk_binary(
                            Operation::Ge,
                            temp_index,
                            operand1_index,
                            operand2_index,
                        ));
                    }
                    _ => {}
                }
            }
            MoveBytecode::Exists(struct_index) => {
                let struct_def = self.module.struct_def_at(*struct_index);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(mk_unary(
                    Operation::Exists(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        vec![],
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::ExistsGeneric(idx) => {
                let struct_instantiation = &self.module.struct_def_instantiations[idx.into_index()];
                let struct_def = self.module.struct_def_at(struct_instantiation.def);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::Bool));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(mk_unary(
                    Operation::Exists(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        self.get_type_params(struct_instantiation.type_parameters),
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MutBorrowGlobal(idx) | MoveBytecode::ImmBorrowGlobal(idx) => {
                let struct_def = self.module.struct_def_at(*idx);
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowGlobal(..));
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function.local_types.push(Type::Reference(
                    is_mut,
                    Box::new(Type::Struct(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        vec![],
                    )),
                ));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                function.code.push(mk_unary(
                    Operation::BorrowGlobal(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        vec![],
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MutBorrowGlobalGeneric(idx)
            | MoveBytecode::ImmBorrowGlobalGeneric(idx) => {
                let struct_instantiation = &self.module.struct_def_instantiations[idx.into_index()];
                let struct_def = self.module.struct_def_at(struct_instantiation.def);
                let is_mut = matches!(bytecode, MoveBytecode::MutBorrowGlobalGeneric(..));
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                function.local_types.push(Type::Reference(
                    is_mut,
                    Box::new(Type::Struct(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        actuals.clone(),
                    )),
                ));
                self.temp_stack.push(temp_index);
                self.temp_count += 1;
                function.code.push(mk_unary(
                    Operation::BorrowGlobal(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        actuals,
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MoveFrom(idx) => {
                let struct_def = self.module.struct_def_at(*idx);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                function.local_types.push(Type::Struct(
                    ModuleId::new(
                        self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                    ),
                    self.get_struct_id_by_idx(&struct_def.struct_handle),
                    vec![],
                ));
                self.temp_count += 1;
                function.code.push(mk_unary(
                    Operation::MoveFrom(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        vec![],
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MoveFromGeneric(idx) => {
                let struct_instantiation = &self.module.struct_def_instantiations[idx.into_index()];
                let struct_def = self.module.struct_def_at(struct_instantiation.def);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                self.temp_stack.push(temp_index);
                let actuals = self.get_type_params(struct_instantiation.type_parameters);
                function.local_types.push(Type::Struct(
                    ModuleId::new(
                        self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                    ),
                    self.get_struct_id_by_idx(&struct_def.struct_handle),
                    actuals.clone(),
                ));
                self.temp_count += 1;
                function.code.push(mk_unary(
                    Operation::MoveFrom(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        actuals,
                    ),
                    temp_index,
                    operand_index,
                ));
            }

            MoveBytecode::MoveTo(idx) => {
                let struct_def = self.module.struct_def_at(*idx);
                let value_operand_index = self.temp_stack.pop().unwrap();
                let signer_operand_index = self.temp_stack.pop().unwrap();
                function.code.push(mk_call(
                    Operation::MoveTo(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        vec![],
                    ),
                    vec![],
                    vec![value_operand_index, signer_operand_index],
                ));
            }

            MoveBytecode::MoveToGeneric(idx) => {
                let struct_instantiation = &self.module.struct_def_instantiations[idx.into_index()];
                let struct_def = self.module.struct_def_at(struct_instantiation.def);
                let value_operand_index = self.temp_stack.pop().unwrap();
                let signer_operand_index = self.temp_stack.pop().unwrap();
                function.code.push(mk_call(
                    Operation::MoveTo(
                        ModuleId::new(
                            self.get_module_handle_index_of_struct(&struct_def.struct_handle),
                        ),
                        self.get_struct_id_by_idx(&struct_def.struct_handle),
                        self.get_type_params(struct_instantiation.type_parameters),
                    ),
                    vec![],
                    vec![value_operand_index, signer_operand_index],
                ));
            }

            MoveBytecode::Nop => function.code.push(Bytecode::Nop(attr_id)),

            // TODO full prover support for vector bytecode instructions
            // These should go to non-functional call operations
            MoveBytecode::VecLen(sig) => {
                let tys = self.get_type_params(*sig);
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Primitive(PrimitiveType::U64));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(Bytecode::Call(
                    attr_id,
                    vec![temp_index],
                    mk_vec_function_operation("length", tys),
                    vec![operand_index],
                    None,
                ))
            }
            MoveBytecode::VecMutBorrow(sig) | MoveBytecode::VecImmBorrow(sig) => {
                let is_mut = match bytecode {
                    MoveBytecode::VecMutBorrow(_) => true,
                    MoveBytecode::VecImmBorrow(_) => false,
                    _ => unreachable!(),
                };
                let [ty]: [Type; 1] = self.get_type_params(*sig).try_into().unwrap();
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Reference(is_mut, Box::new(ty.clone())));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                let vec_fun = if is_mut { "borrow_mut" } else { "borrow" };
                function.code.push(Bytecode::Call(
                    attr_id,
                    vec![temp_index],
                    mk_vec_function_operation(vec_fun, vec![ty]),
                    vec![operand1_index, operand2_index],
                    None,
                ))
            }
            MoveBytecode::VecPushBack(sig) => {
                let tys = self.get_type_params(*sig);
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                function.code.push(Bytecode::Call(
                    attr_id,
                    vec![],
                    mk_vec_function_operation("push_back", tys),
                    vec![operand1_index, operand2_index],
                    None,
                ))
            }
            MoveBytecode::VecPopBack(sig) => {
                let [ty]: [Type; 1] = self.get_type_params(*sig).try_into().unwrap();
                let operand_index = self.temp_stack.pop().unwrap();
                let temp_index = self.temp_count;
                function.local_types.push(ty.clone());
                self.temp_count += 1;
                self.temp_stack.push(temp_index);
                function.code.push(Bytecode::Call(
                    attr_id,
                    vec![temp_index],
                    mk_vec_function_operation("pop_back", vec![ty]),
                    vec![operand_index],
                    None,
                ))
            }
            MoveBytecode::VecSwap(sig) => {
                let tys = self.get_type_params(*sig);
                let operand3_index = self.temp_stack.pop().unwrap();
                let operand2_index = self.temp_stack.pop().unwrap();
                let operand1_index = self.temp_stack.pop().unwrap();
                function.code.push(Bytecode::Call(
                    attr_id,
                    vec![],
                    mk_vec_function_operation("swap", tys),
                    vec![operand1_index, operand2_index, operand3_index],
                    None,
                ))
            }
            MoveBytecode::VecPack(sig, n) => {
                let n = *n as usize;
                let [ty]: [Type; 1] = self.get_type_params(*sig).try_into().unwrap();
                let operands = self.temp_stack.split_off(self.temp_stack.len() - n);
                let temp_index = self.temp_count;
                function
                    .local_types
                    .push(Type::Vector(Box::new(ty.clone())));
                self.temp_count += 1;
                self.temp_stack.push(temp_index);

                function.code.push(Bytecode::Call(
                    attr_id,
                    vec![temp_index],
                    mk_vec_function_operation("empty", vec![ty.clone()]),
                    vec![],
                    None,
                ));
                if !operands.is_empty() {
                    let mut_ref_index = self.temp_count;
                    function.local_types.push(Type::Reference(
                        true,
                        Box::new(Type::Vector(Box::new(ty.clone()))),
                    ));
                    self.temp_count += 1;

                    function
                        .code
                        .push(mk_unary(Operation::BorrowLoc, mut_ref_index, temp_index));

                    for operand in operands {
                        function.code.push(Bytecode::Call(
                            attr_id,
                            vec![],
                            mk_vec_function_operation("push_back", vec![ty.clone()]),
                            vec![mut_ref_index, operand],
                            None,
                        ));
                    }
                }
            }
            MoveBytecode::VecUnpack(sig, n) => {
                let n = *n as usize;
                let [ty]: [Type; 1] = self.get_type_params(*sig).try_into().unwrap();
                let operand_index = self.temp_stack.pop().unwrap();
                let temps = (0..n).map(|idx| self.temp_count + idx).collect::<Vec<_>>();
                function.local_types.extend(vec![ty.clone(); n]);
                self.temp_count += n;
                self.temp_stack.extend(&temps);

                if !temps.is_empty() {
                    let mut_ref_index = self.temp_count;
                    function.local_types.push(Type::Reference(
                        true,
                        Box::new(Type::Vector(Box::new(ty.clone()))),
                    ));
                    self.temp_count += 1;

                    function.code.push(mk_unary(
                        Operation::BorrowLoc,
                        mut_ref_index,
                        operand_index,
                    ));

                    for temp in temps {
                        function.code.push(Bytecode::Call(
                            attr_id,
                            vec![temp],
                            mk_vec_function_operation("pop_back", vec![ty.clone()]),
                            vec![mut_ref_index],
                            None,
                        ));
                    }
                }

                function.code.push(Bytecode::Call(
                    attr_id,
                    vec![],
                    mk_vec_function_operation("destroy_empty", vec![ty]),
                    vec![operand_index],
                    None,
                ))
            }
        }
    }

    pub fn get_control_flow_graph(&mut self) {
        for (idx, function) in self.functions.iter_mut().enumerate() {
            let func_define = self
                .module
                .function_def_at(FunctionDefinitionIndex::new(idx as u16));
            if !func_define.is_native() {
                function.cfg = Some(StacklessControlFlowGraph::new_forward(&function.code));
            }
        }
    }

    pub fn build_call_graph(&mut self) {
        let mut graph: Graph<QualifiedId<FunId>, ()> = DiGraph::new();
        let mut nodes: BTreeMap<QualifiedId<FunId>, NodeIndex> = BTreeMap::new();
        let cm = self.module;
        for func in cm.function_handles() {
            let name = cm.identifier_at(func.name);
            let symbol = self.symbol_pool.make(name.as_str());
            let func_id = FunId::new(symbol);
            let module_id = ModuleId::new(func.module.into_index());
            let qid = QualifiedId {
                module_id,
                id: func_id,
            };
            let node_idx = graph.add_node(qid);
            nodes.insert(qid, node_idx);
        }

        for (idx, func_id) in self.module_data.function_idx_to_id.iter() {
            let function = &self.functions[idx.into_index()];
            let qid = QualifiedId {
                module_id: self.module_data.id,
                id: *func_id,
            };
            let src_idx = nodes.get(&qid).unwrap();
            let called: BTreeSet<_> = function
                .code
                .iter()
                .filter_map(|c| {
                    if let Bytecode::Call(_, _, Operation::Function(mid, fid, _), _, _) = c {
                        Some(QualifiedId {
                            module_id: *mid,
                            id: *fid,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            for called_qid in called {
                let dst_idx = nodes.get(&called_qid);
                if let Some(dst_idx) = dst_idx {
                    graph.add_edge(*src_idx, *dst_idx, ());
                }
            }
        }
        self.call_graph = graph;
        self.func_to_node = nodes;
    }
}
