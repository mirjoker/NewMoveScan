use itertools::Itertools;
use std::{collections::BTreeMap, fmt::Write};

use crate::{move_ir::generate_bytecode::StacklessBytecodeGenerator, utils::utils::DotWeight};
use move_binary_format::{
    access::ModuleAccess, file_format::FunctionDefinitionIndex, internals::ModuleIndex,
};
use move_model::model::{FunId, ModuleId, QualifiedId};
use move_stackless_bytecode::stackless_bytecode::Bytecode;
use petgraph::{
    graph::{DiGraph, Graph, NodeIndex},
    visit::EdgeRef,
};

use move_binary_format::{
    file_format::{
        CodeOffset, FieldHandleIndex, FunctionHandleIndex, SignatureIndex, SignatureToken,
        StructDefinitionIndex, StructFieldInformation, StructHandleIndex,
    },
    views::FunctionDefinitionView,
    CompiledModule,
};
use move_core_types::value::MoveValue;
use move_model::{
    ast::{Attribute, Spec, TempIndex},
    model::{FieldData, FieldId, FieldInfo, Loc, StructData, StructId, StructInfo},
    symbol::{Symbol, SymbolPool},
    ty::{PrimitiveType, Type, TypeDisplayContext},
};
use move_stackless_bytecode::stackless_bytecode::{AttrId, Constant};
use num::BigUint;

use crate::move_ir::bytecode_display;
use crate::utils::utils;

use super::generate_bytecode::FunctionInfo;

impl<'a> StacklessBytecodeGenerator<'a> {
    pub fn call_graph2str(&self) -> Graph<String, DotWeight> {
        let mut graph: Graph<String, DotWeight> = DiGraph::new();
        let mut nodes: BTreeMap<String, NodeIndex> = BTreeMap::new();
        for node in self.func_to_node.iter() {
            let fname = self.get_fname_by_qid(node.0);
            let node_idx = graph.add_node(fname.clone());
            nodes.insert(fname, node_idx);
        }
        for edge in self.call_graph.edge_references() {
            let source_node = edge.source();
            let target_node = edge.target();
            graph.add_edge(source_node, target_node, DotWeight {});
        }
        graph
    }
    /// Create a new attribute id and populate location table.
    pub fn new_loc_attr(
        &mut self,
        func_def_idx: FunctionDefinitionIndex,
        function: &mut FunctionInfo,
        code_offset: CodeOffset,
    ) -> AttrId {
        let loc = self.get_bytecode_loc(func_def_idx, code_offset);
        let attr = AttrId::new(function.location_table.len());
        function.location_table.insert(attr, loc);
        attr
    }

    pub fn get_bytecode_loc(&self, func_def_idx: FunctionDefinitionIndex, _: u16) -> Loc {
        let func_id = self.module_data.function_idx_to_id[&func_def_idx];
        let func_data = &self.module_data.function_data[&func_id];
        func_data.loc.clone()
    }

    pub fn get_field_info(&self, field_handle_index: FieldHandleIndex) -> (StructId, usize, Type) {
        let field_handle = self.module.field_handle_at(field_handle_index);
        let struct_def = self.module.struct_def_at(field_handle.owner);
        let struct_id = self.get_struct_id_by_idx(&struct_def.struct_handle);
        let name = self
            .module
            .identifier_at(self.module.struct_handle_at(struct_def.struct_handle).name);
        let symbol = self.symbol_pool.make(name.as_str());
        let struct_data = create_move_struct_data(
            &self.symbol_pool,
            self.module,
            field_handle.owner,
            symbol,
            Loc::default(),
            Vec::default(),
        );
        let offset = field_handle.field as usize;
        let mut filed_data = struct_data.field_data.first_key_value().unwrap().1;
        for data in struct_data.field_data.values() {
            if data.offset == offset {
                filed_data = data;
            }
        }
        (
            struct_id,
            field_handle.field as usize,
            self.get_type(filed_data),
        )
    }

    pub fn globalize_signature(&self, sig: &SignatureToken) -> Type {
        match sig {
            SignatureToken::Bool => Type::Primitive(PrimitiveType::Bool),
            SignatureToken::U8 => Type::Primitive(PrimitiveType::U8),
            SignatureToken::U16 => Type::Primitive(PrimitiveType::U16),
            SignatureToken::U32 => Type::Primitive(PrimitiveType::U32),
            SignatureToken::U64 => Type::Primitive(PrimitiveType::U64),
            SignatureToken::U128 => Type::Primitive(PrimitiveType::U128),
            SignatureToken::U256 => Type::Primitive(PrimitiveType::U256),
            SignatureToken::Address => Type::Primitive(PrimitiveType::Address),
            SignatureToken::Signer => Type::Primitive(PrimitiveType::Signer),
            SignatureToken::Reference(t) => {
                Type::Reference(false, Box::new(self.globalize_signature(t)))
            }
            SignatureToken::MutableReference(t) => {
                Type::Reference(true, Box::new(self.globalize_signature(t)))
            }
            SignatureToken::TypeParameter(index) => Type::TypeParameter(*index),
            SignatureToken::Vector(bt) => Type::Vector(Box::new(self.globalize_signature(bt))),
            SignatureToken::Struct(handle_idx) => Type::Struct(
                ModuleId::new(self.get_module_handle_index_of_struct(handle_idx)),
                self.get_struct_id_by_idx(handle_idx),
                vec![],
            ),
            SignatureToken::StructInstantiation(handle_idx, args) => Type::Struct(
                ModuleId::new(self.get_module_handle_index_of_struct(handle_idx)),
                self.get_struct_id_by_idx(handle_idx),
                self.globalize_signatures(args),
            ),
        }
    }

    pub fn globalize_signatures(&self, sigs: &[SignatureToken]) -> Vec<Type> {
        sigs.iter()
            .map(|s| self.globalize_signature(s))
            .collect_vec()
    }

    pub fn get_type_params(&self, type_params_index: SignatureIndex) -> Vec<Type> {
        let actuals = &self.module.signature_at(type_params_index).0;
        self.globalize_signatures(&actuals)
    }

    pub fn translate_value(ty: &Type, value: &MoveValue) -> Constant {
        match (ty, &value) {
            (Type::Vector(inner), MoveValue::Vector(vs)) => match **inner {
                Type::Primitive(PrimitiveType::U8) => {
                    let b = vs
                        .iter()
                        .map(|v| match Self::translate_value(inner, v) {
                            Constant::U8(u) => u,
                            _ => panic!("Expected u8, but found: {:?}", inner),
                        })
                        .collect::<Vec<u8>>();
                    Constant::ByteArray(b)
                }
                Type::Primitive(PrimitiveType::Address) => {
                    let b = vs
                        .iter()
                        .map(|v| match Self::translate_value(inner, v) {
                            Constant::Address(a) => a,
                            _ => panic!("Expected address, but found: {:?}", inner),
                        })
                        .collect::<Vec<BigUint>>();
                    Constant::AddressArray(b)
                }
                _ => {
                    let b = vs
                        .iter()
                        .map(|v| Self::translate_value(inner, v))
                        .collect::<Vec<Constant>>();
                    Constant::Vector(b)
                }
            },
            (Type::Primitive(PrimitiveType::Bool), MoveValue::Bool(b)) => Constant::Bool(*b),
            (Type::Primitive(PrimitiveType::U8), MoveValue::U8(b)) => Constant::U8(*b),
            (Type::Primitive(PrimitiveType::U16), MoveValue::U16(b)) => Constant::U16(*b),
            (Type::Primitive(PrimitiveType::U32), MoveValue::U32(b)) => Constant::U32(*b),
            (Type::Primitive(PrimitiveType::U64), MoveValue::U64(b)) => Constant::U64(*b),
            (Type::Primitive(PrimitiveType::U128), MoveValue::U128(b)) => Constant::U128(*b),
            (Type::Primitive(PrimitiveType::U256), MoveValue::U256(b)) => Constant::U256(b.into()),
            (Type::Primitive(PrimitiveType::Address), MoveValue::Address(a)) => {
                Constant::Address(move_model::addr_to_big_uint(a))
            }
            _ => panic!("Unexpected (and possibly invalid) constant type: {:?}", ty),
        }
    }

    pub fn get_local_type(
        &self,
        view: &FunctionDefinitionView<CompiledModule>,
        idx: usize,
    ) -> Type {
        let parameters = view.parameters();
        if idx < parameters.len() {
            self.globalize_signature(&parameters.0[idx])
        } else {
            self.globalize_signature(
                view.locals_signature()
                    .unwrap()
                    .token_at(idx as u8)
                    .signature_token(),
            )
        }
    }

    pub fn get_local_name(&self, func_def_idx: FunctionDefinitionIndex, idx: usize) -> Symbol {
        let func_id = self.module_data.function_idx_to_id[&func_def_idx];
        let func_data = &self.module_data.function_data[&func_id];
        if idx < func_data.arg_names.len() {
            return func_data.arg_names[idx as usize];
        }
        // Try to obtain name from source map.
        if let Ok(fmap) = self
            .module_data
            .source_map
            .get_function_source_map(func_def_idx)
        {
            if let Some((ident, _)) = fmap.get_parameter_or_local_name(idx as u64) {
                // The Move compiler produces temporary names of the form `<foo>%#<num>`,
                // where <num> seems to be generated non-deterministically.
                // Substitute this by a deterministic name which the backend accepts.
                let clean_ident = if ident.contains("%#") {
                    format!("tmp#${}", idx)
                } else {
                    ident
                };
                return self.symbol_pool.make(clean_ident.as_str());
            }
        }
        self.symbol_pool.make(&format!("$t{}", idx))
    }

    pub fn get_fun_id_by_idx(&self, idx: &FunctionHandleIndex) -> FunId {
        let h = self.module.function_handle_at(*idx);
        let name = self.module.identifier_at(h.name);
        let symbol = self.symbol_pool.make(name.as_str());
        let fun_id = FunId::new(symbol);
        fun_id
    }

    pub fn get_struct_id_by_idx(&self, idx: &StructHandleIndex) -> StructId {
        let h = self.module.struct_handle_at(*idx);
        let name = self.module.identifier_at(h.name);
        let symbol = self.symbol_pool.make(name.as_str());
        let struct_id = StructId::new(symbol);
        struct_id
    }

    pub fn get_type(&self, data: &FieldData) -> Type {
        match &data.info {
            FieldInfo::Declared { def_idx } => {
                let struct_def = self.module.struct_def_at(*def_idx);
                let field = match &struct_def.field_information {
                    StructFieldInformation::Declared(fields) => &fields[data.offset],
                    StructFieldInformation::Native => unreachable!(),
                };
                self.globalize_signature(&field.signature.0)
            }
            FieldInfo::Generated { type_ } => type_.clone(),
        }
    }

    pub fn get_module_handle_index_of_struct(&self, struct_handle_index: &StructHandleIndex) -> usize {
        let struct_handle = self.module.struct_handle_at(*struct_handle_index);
        struct_handle.module.into_index()
    }

    pub fn get_module_handle_index_of_func(&self, func_handle_index: &FunctionHandleIndex) -> usize {
        let func_handle = self.module.function_handle_at(*func_handle_index);
        func_handle.module.into_index()
    }

    pub fn get_fname_by_qid(&self, qid: &QualifiedId<FunId>) -> String {
        let mid = qid.module_id;
        let msym = self.module_names[mid.to_usize()].name();
        let mname = msym.display(&self.symbol_pool).to_string();
        let fid = qid.id;
        let fname = fid.symbol().display(&self.symbol_pool).to_string();
        format!("{}::{}", mname, fname)
    }

    pub fn display(&self, display_function_body: bool, display_one_or_all: Option<usize>) -> String {
        let mut f = String::new();
        let mut idxs = vec![];
        if let Some(idx) = display_one_or_all {
            // display idx function
            idxs.push(FunctionDefinitionIndex::new(idx as u16));
        } else {
            // display all functions
            for idx in 0..self.functions.len() {
                idxs.push(FunctionDefinitionIndex::new(idx as u16));
            }
        }
        for idx in idxs.iter() {
            let func_id = self.module_data.function_idx_to_id[idx];
            let func_define = self.module.function_def_at(*idx);
            let func_data: &move_model::model::FunctionData = &self.module_data.function_data[&func_id];
            let function = &self.functions[idx.into_index()];
            let view = FunctionDefinitionView::new(self.module, func_define);
            let modifier = if func_define.is_native() {
                "native "
            } else if func_define.is_entry {
                "entry"
            } else {
                ""
            };
            write!(
                f,
                "{}{} fun {}::{}",
                utils::visibility_str(&func_define.visibility),
                modifier,
                self.module_data.name.display(&self.symbol_pool),
                func_data.name.display(&self.symbol_pool)
            ).unwrap();
            // ghost_type_param_count?
            let tparams_count_all = view.type_parameters().len() + 0;
            let tparams_count_defined = view.type_parameters().len();
            if tparams_count_all != 0 {
                write!(f, "<").unwrap();
                for i in 0..tparams_count_all {
                    if i > 0 {
                        write!(f, ", ").unwrap();
                    }
                    write!(f, "#{}", i).unwrap();
                    if i >= tparams_count_defined {
                        write!(f, "*").unwrap(); // denotes a ghost type parameter
                    }
                }
                write!(f, ">").unwrap();
            }
            let tctx = TypeDisplayContext::WithoutEnv {
                symbol_pool: &self.symbol_pool,
                reverse_struct_table: &self.reverse_struct_table,
            };
            let user_local_count = match view.locals_signature() {
                Some(locals_view) => locals_view.len(),
                None => view.parameters().len(),
            };
            let write_decl = |f: &mut String, i: TempIndex| {
                let ty_ = &function.local_types[i];
                let ty = ty_.display(&tctx);
                if i < user_local_count {
                    let param = if display_function_body {
                        format!(
                            "$t{}|{}: {}",
                            i,
                            self.get_local_name(*idx, i).display(&self.symbol_pool),
                            ty
                        )
                    } else {
                        format!("{}", ty)
                    };
                    write!(f, "{}", param).unwrap()
                } else {
                    let param = if display_function_body {
                        format!("$t{}: {}", i, ty)
                    } else {
                        format!("{}", ty)
                    };
                    write!(f, "{}", param).unwrap()
                }
            };
            write!(f, "(").unwrap();

            for i in 0..view.arg_tokens().count() {
                if i > 0 {
                    write!(f, ", ").unwrap();
                }
                write_decl(&mut f, i);
            }
            write!(f, ")").unwrap();
            if view.return_count() > 0 {
                write!(f, ": ").unwrap();
                if view.return_count() > 1 {
                    write!(f, "(").unwrap();
                }
                let returns = &view.return_().0;
                for i in 0..view.return_count() {
                    if i > 0 {
                        write!(f, ", ").unwrap();
                    }
                    write!(
                        f,
                        "{}",
                        self.globalize_signature(&returns[i]).display(&tctx)
                    ).unwrap();
                }
                if view.return_count() > 1 {
                    write!(f, ")").unwrap();
                }
            }
            if display_function_body {
                if func_define.is_native() {
                    writeln!(f, ";").unwrap();
                } else {
                    writeln!(f, " {{").unwrap();
                    for i in view.arg_tokens().count()..function.local_types.len() {
                        write!(f, "     var ").unwrap();
                        write_decl(&mut f, i);
                        writeln!(f).unwrap();
                    }

                    let bytecodes = self.functions[idx.into_index()].code.clone();
                    let label_offsets = Bytecode::label_offsets(&bytecodes);
                    for (offset, code) in bytecodes.iter().enumerate() {
                        writeln!(
                            f,
                            "{:>3}: {}",
                            offset,
                            bytecode_display::display(code, &label_offsets, &self)
                        ).unwrap();
                    }
                    writeln!(f, "}}").unwrap();
                }
            } else {
                writeln!(f).unwrap();
            }
        }
        f
    }

    pub fn print_func_signature(&self) {
        println!("**********Function Signatures For {} Module***********",self.module_data.name.display(&self.symbol_pool));
        let mut func_signatures: Vec<String> = vec![];
        let mut idxs = vec![];
        for idx in 0..self.functions.len() {
            idxs.push(FunctionDefinitionIndex::new(idx as u16));
        }
        for idx in idxs.iter() {
            let mut fs = String::new();
            let func_id = self.module_data.function_idx_to_id[idx];
            let func_define = self.module.function_def_at(*idx);
            let func_data: &move_model::model::FunctionData = &self.module_data.function_data[&func_id];
            let function = &self.functions[idx.into_index()];
            let view = FunctionDefinitionView::new(self.module, func_define);
            let modifier = if func_define.is_native() {
                "native"
            } else if func_define.is_entry {
                "entry"
            } else {
                ""
            };
            if utils::visibility_str(&func_define.visibility) != "" {
                write!(fs, "\x1B[34m{}\x1B[0m ", utils::visibility_str(&func_define.visibility)).unwrap();
            }
            if modifier != "" {
                write!(fs, "\x1B[34m{}\x1B[0m ", modifier).unwrap();
            }
            write!(
                fs,
                "\x1B[34mfun\x1B[0m \x1B[93m{}\x1B[0m::\x1B[93m{}\x1B[0m",
                self.module_data.name.display(&self.symbol_pool),
                func_data.name.display(&self.symbol_pool)
            ).unwrap();
            // ghost_type_param_count?
            let tparams_count_all = view.type_parameters().len();
            if tparams_count_all != 0 {
                write!(fs, "<").unwrap();
                for i in 0..tparams_count_all {
                    if i > 0 {
                        write!(fs, ", ").unwrap();
                    }
                    write!(fs, "#{}", i).unwrap();
                }
                write!(fs, ">").unwrap();
            }
            let tctx = TypeDisplayContext::WithoutEnv {
                symbol_pool: &self.symbol_pool,
                reverse_struct_table: &self.reverse_struct_table,
            };
            let write_decl = |f: &mut String, i: TempIndex| {
                let ty_ = &function.local_types[i];
                let ty = ty_.display(&tctx);
                let param = format!("{}", ty);
                    write!(f, "\x1B[32m{}\x1B[0m", param).unwrap()
            };
            write!(fs, "(").unwrap();

            for i in 0..view.arg_tokens().count() {
                if i > 0 {
                    write!(fs, ", ").unwrap();
                }
                write_decl(&mut fs, i);
            }
            write!(fs, ")").unwrap();
            if view.return_count() > 0 {
                write!(fs, ": ").unwrap();
                if view.return_count() > 1 {
                    write!(fs, "(").unwrap();
                }
                let returns = &view.return_().0;
                for i in 0..view.return_count() {
                    if i > 0 {
                        write!(fs, ", ").unwrap();
                    }
                    write!(
                        fs,
                        "\x1B[32m{}\x1B[0m",
                        self.globalize_signature(&returns[i]).display(&tctx)
                    ).unwrap();
                }
                if view.return_count() > 1 {
                    write!(fs, ")").unwrap();
                }
            }
            func_signatures.push(fs);
        }

        func_signatures.sort();
        
        for fs in func_signatures.iter() {
            println!("{}", fs);
        }
        println!();

    }

    
}



pub fn create_move_struct_data(
    symbol_pool: &SymbolPool,
    module: &CompiledModule,
    def_idx: StructDefinitionIndex,
    name: Symbol,
    loc: Loc,
    attributes: Vec<Attribute>,
) -> StructData {
    let handle_idx = module.struct_def_at(def_idx).struct_handle;
    let field_data = if let StructFieldInformation::Declared(fields) =
        &module.struct_def_at(def_idx).field_information
    {
        let mut map = BTreeMap::new();
        for (offset, field) in fields.iter().enumerate() {
            let name = symbol_pool.make(module.identifier_at(field.name).as_str());
            let info = FieldInfo::Declared { def_idx };
            map.insert(FieldId::new(name), FieldData { name, offset, info });
        }
        map
    } else {
        BTreeMap::new()
    };
    let info = StructInfo::Declared {
        def_idx,
        handle_idx,
    };
    StructData {
        name,
        loc,
        attributes,
        info,
        field_data,
        spec: Spec::default(),
    }
}

pub fn get_def_bytecode(function: &FunctionInfo, sid: usize, code_offset: usize) -> &Bytecode {
    let tid = &function.def_attrid[sid];
    if tid.len() == 1 {
        &function.code[tid[0]]
    } else {
        let closest = tid.iter()
        .filter(|&x| *x < code_offset)
        .min_by_key(|&x| code_offset - x);
        if let Some(id) = closest {
            &function.code[*id]
        } else {
            &function.code[tid[0]]
        }
    }
}

pub fn get_function_name(idx: usize, stbgr: &StacklessBytecodeGenerator) -> String {
    let func_name = stbgr
        .module
        .identifier_at(
            stbgr
                .module
                .function_handle_at(stbgr.module.function_defs[idx].function)
                .name,
        )
        .to_string();
    return func_name;
}

pub fn is_native(idx: usize, stbgr: &StacklessBytecodeGenerator) -> bool {
    stbgr
        .module
        .function_def_at(FunctionDefinitionIndex::new(idx as u16))
        .is_native()
}