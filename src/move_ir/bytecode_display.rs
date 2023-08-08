// =================================================================================================
// Formatting
use core::fmt;
use move_binary_format::file_format::CodeOffset;
use move_model::{
    ast::TempIndex,
    model::{ModuleId, StructId},
    ty::{Type, TypeDisplayContext},
};
use std::{collections::BTreeMap, fmt::Formatter};

use move_stackless_bytecode::stackless_bytecode::{
    AbortAction, AssignKind, BorrowEdge, BorrowNode,
    Bytecode::{self}, HavocKind, Label, Operation,
};

use super::generate_bytecode::StacklessBytecodeGenerator;

pub fn display<'env>(
    bytecode: &'env Bytecode,
    label_offsets: &'env BTreeMap<Label, CodeOffset>,
    stbgr: &'env StacklessBytecodeGenerator,
) -> BytecodeDisplay<'env> {
    BytecodeDisplay {
        bytecode,
        label_offsets,
        stbgr,
    }
}

pub fn oper_display<'env>(
    oper: &'env Operation,
    stbgr: &'env StacklessBytecodeGenerator<'env>,
) -> OperationDisplay<'env> {
    OperationDisplay { oper, stbgr }
}

/// Creates a format object for a borrow node in context of a function target.
pub fn borrow_node_display<'env>(
    node: &'env BorrowNode,
    stbgr: &'env StacklessBytecodeGenerator<'env>,
) -> BorrowNodeDisplay<'env> {
    BorrowNodeDisplay { node, stbgr }
}

pub fn borrow_edge_display<'env>(
    edge: &'env BorrowEdge,
    stbgr: &'env StacklessBytecodeGenerator<'env>,
) -> BorrowEdgeDisplay<'env> {
    BorrowEdgeDisplay { stbgr, edge }
}

pub struct BytecodeDisplay<'env> {
    bytecode: &'env Bytecode,
    label_offsets: &'env BTreeMap<Label, CodeOffset>,
    stbgr: &'env StacklessBytecodeGenerator<'env>,
}

impl<'env> fmt::Display for BytecodeDisplay<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Bytecode::*;
        match &self.bytecode {
            Assign(_, dst, src, AssignKind::Copy) => {
                write!(f, "{} := copy({})", self.lstr(*dst), self.lstr(*src))?
            }
            Assign(_, dst, src, AssignKind::Move) => {
                write!(f, "{} := move({})", self.lstr(*dst), self.lstr(*src))?
            }
            Assign(_, dst, src, AssignKind::Store) => {
                write!(f, "{} := {}", self.lstr(*dst), self.lstr(*src))?
            }
            Call(_, dsts, oper, args, aa) => {
                if !dsts.is_empty() {
                    self.fmt_locals(f, dsts, false)?;
                    write!(f, " := ")?;
                }
                write!(f, "{}", oper_display(oper, self.stbgr))?;
                self.fmt_locals(f, args, true)?;
                if let Some(AbortAction(label, code)) = aa {
                    write!(
                        f,
                        " on_abort goto {} with {}",
                        self.label_str(*label),
                        self.lstr(*code)
                    )?;
                }
            }
            Ret(_, srcs) => {
                write!(f, "return ")?;
                self.fmt_locals(f, srcs, false)?;
            }
            Load(_, dst, cons) => {
                write!(f, "{} := {}", self.lstr(*dst), cons)?;
            }
            Branch(_, then_label, else_label, src) => {
                write!(
                    f,
                    "if ({}) goto {} else goto {}",
                    self.lstr(*src),
                    self.label_str(*then_label),
                    self.label_str(*else_label),
                )?;
            }
            Jump(_, label) => {
                write!(f, "goto {}", self.label_str(*label))?;
            }
            Label(_, label) => {
                write!(f, "label L{}", label.as_usize())?;
            }
            Abort(_, src) => {
                write!(f, "abort({})", self.lstr(*src))?;
            }
            Nop(_) => {
                write!(f, "nop")?;
            }
            SaveMem(_, _label, _qid) => {
                // TODO
                // let env = self.func_target.global_env();
                // write!(f, "@{} := save_mem({})", label.as_usize(), env.display(qid))?;
            }
            SaveSpecVar(_, _label, _qid) => {
                // TODO skip it
                // let env = self.func_target.global_env();
                // let module_env = env.get_module(qid.module_id);
                // let spec_var = module_env.get_spec_var(qid.id);
                // write!(
                //     f,
                //     "@{} := save_spec_var({}::{})",
                //     label.as_usize(),
                //     module_env.get_name().display(env.symbol_pool()),
                //     spec_var.name.display(env.symbol_pool())
                // )?;
            }
            Prop(_, _kind, _exp) => {
                // TODO
                // let exp_display = exp.display(self.func_target.func_env.module_env.env);
                // match kind {
                //     PropKind::Assume => write!(f, "assume {}", exp_display)?,
                //     PropKind::Assert => write!(f, "assert {}", exp_display)?,
                //     PropKind::Modifies => write!(f, "modifies {}", exp_display)?,
                // }
            }
        }
        Ok(())
    }
}

impl<'env> BytecodeDisplay<'env> {
    fn fmt_locals(
        &self,
        f: &mut Formatter<'_>,
        locals: &[TempIndex],
        always_brace: bool,
    ) -> fmt::Result {
        if !always_brace && locals.len() == 1 {
            write!(f, "{}", self.lstr(locals[0]))?
        } else {
            write!(f, "(")?;
            for (i, l) in locals.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.lstr(*l))?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }

    fn lstr(&self, idx: TempIndex) -> String {
        format!("$t{}", idx)
    }

    fn label_str(&self, label: Label) -> String {
        self.label_offsets
            .get(&label)
            .map(|offs| offs.to_string())
            .unwrap_or_else(|| format!("L{}", label.as_usize()))
    }
}

/// A display object for an operation.
pub struct OperationDisplay<'env> {
    oper: &'env Operation,
    stbgr: &'env StacklessBytecodeGenerator<'env>,
}

impl<'env> fmt::Display for OperationDisplay<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Operation::*;
        match self.oper {
            // User function
            Function(mid, fid, targs)
            | OpaqueCallBegin(mid, fid, targs)
            | OpaqueCallEnd(mid, fid, targs) => {
                write!(
                    f,
                    "{}",
                    match self.oper {
                        OpaqueCallBegin(_, _, _) => "opaque begin: ",
                        OpaqueCallEnd(_, _, _) => "opaque end: ",
                        _ => "",
                    }
                )?;
                write!(
                    f,
                    "{}::{}",
                    self.stbgr.module_names[mid.to_usize()].display(&self.stbgr.symbol_pool),
                    fid.symbol().display(&self.stbgr.symbol_pool),
                )?;
                self.fmt_type_args(f, targs)?;
            }

            // Pack/Unpack
            Pack(mid, sid, targs) => {
                write!(f, "pack {}", self.struct_str(*mid, *sid, targs))?;
            }
            Unpack(mid, sid, targs) => {
                write!(f, "unpack {}", self.struct_str(*mid, *sid, targs))?;
            }

            // Borrow
            BorrowLoc => {
                write!(f, "borrow_local")?;
            }
            BorrowField(mid, sid, targs, offset) => {
                write!(f, "borrow_field<{}>", self.struct_str(*mid, *sid, targs))?;
                let struct_data = self.stbgr.module_data.struct_data.get(sid).unwrap();
                for data in struct_data.field_data.values() {
                    if data.offset == *offset {
                        write!(f, ".{}", data.name.display(&self.stbgr.symbol_pool))?;
                        break;
                    }
                }
            }
            BorrowGlobal(mid, sid, targs) => {
                write!(f, "borrow_global<{}>", self.struct_str(*mid, *sid, targs))?;
            }
            GetField(mid, sid, targs, offset) => {
                write!(f, "get_field<{}>", self.struct_str(*mid, *sid, targs))?;
                let struct_data = self.stbgr.module_data.struct_data.get(sid).unwrap();
                for data in struct_data.field_data.values() {
                    if data.offset == *offset {
                        write!(f, ".{}", data.name.display(&self.stbgr.symbol_pool))?;
                        break;
                    }
                }
            }
            GetGlobal(mid, sid, targs) => {
                write!(f, "get_global<{}>", self.struct_str(*mid, *sid, targs))?;
            }

            // Resources
            MoveTo(mid, sid, targs) => {
                write!(f, "move_to<{}>", self.struct_str(*mid, *sid, targs))?;
            }
            MoveFrom(mid, sid, targs) => {
                write!(f, "move_from<{}>", self.struct_str(*mid, *sid, targs))?;
            }
            Exists(mid, sid, targs) => {
                write!(f, "exists<{}>", self.struct_str(*mid, *sid, targs))?;
            }

            // Builtins
            Uninit => {
                write!(f, "uninit")?;
            }
            Destroy => {
                write!(f, "destroy")?;
            }
            ReadRef => {
                write!(f, "read_ref")?;
            }
            WriteRef => {
                write!(f, "write_ref")?;
            }
            FreezeRef => {
                write!(f, "freeze_ref")?;
            }

            // Memory model
            UnpackRef => {
                write!(f, "unpack_ref")?;
            }
            PackRef => {
                write!(f, "pack_ref")?;
            }
            PackRefDeep => {
                write!(f, "pack_ref_deep")?;
            }
            UnpackRefDeep => {
                write!(f, "unpack_ref_deep")?;
            }
            WriteBack(_node, _edge) => {
                // TODO
                // write!(
                //     f,
                //     "write_back[{}{}]",
                //     node.display(self.func_target),
                //     edge.display(self.func_target.global_env())
                // )?;
            }
            IsParent(_node, _edge) => {
                // TODO
                // write!(
                //     f,
                //     "is_parent[{}{}]",
                //     node.display(self.func_target),
                //     edge.display(self.func_target.global_env())
                // )?;
            }
            Havoc(kind) => {
                write!(
                    f,
                    "havoc[{}]",
                    match kind {
                        HavocKind::Value => "val",
                        HavocKind::MutationValue => "mut",
                        HavocKind::MutationAll => "mut_all",
                    }
                )?;
            }
            Stop => {
                write!(f, "stop")?;
            }
            // Unary
            CastU8 => write!(f, "(u8)")?,
            CastU16 => write!(f, "(u16)")?,
            CastU32 => write!(f, "(u32)")?,
            CastU64 => write!(f, "(u64)")?,
            CastU128 => write!(f, "(u128)")?,
            CastU256 => write!(f, "(u256)")?,
            Not => write!(f, "!")?,

            // Binary
            Add => write!(f, "+")?,
            Sub => write!(f, "-")?,
            Mul => write!(f, "*")?,
            Div => write!(f, "/")?,
            Mod => write!(f, "%")?,
            BitOr => write!(f, "|")?,
            BitAnd => write!(f, "&")?,
            Xor => write!(f, "^")?,
            Shl => write!(f, "<<")?,
            Shr => write!(f, ">>")?,
            Lt => write!(f, "<")?,
            Gt => write!(f, ">")?,
            Le => write!(f, "<=")?,
            Ge => write!(f, ">=")?,
            Or => write!(f, "||")?,
            And => write!(f, "&&")?,
            Eq => write!(f, "==")?,
            Neq => write!(f, "!=")?,

            // Debugging
            TraceLocal(_l) => {
                // TODO
                // let name = self.func_target.get_local_name(*l);
                // write!(
                //     f,
                //     "trace_local[{}]",
                //     name.display(self.stbgr.symbol_pool())
                // )?
            }
            TraceAbort => write!(f, "trace_abort")?,
            TraceReturn(r) => write!(f, "trace_return[{}]", r)?,
            TraceExp(_kind, _node_id) => {
                // TODO
                // let loc = self.func_target.global_env().get_node_loc(*node_id);
                // write!(
                //     f,
                //     "trace_exp[{}, {}]",
                //     kind,
                //     loc.display(self.func_target.global_env())
                // )?
            }
            EmitEvent => write!(f, "emit_event")?,
            EventStoreDiverge => write!(f, "event_store_diverge")?,
            TraceGlobalMem(_) => write!(f, "trace_global_mem")?,
        }
        Ok(())
    }
}

impl<'env> OperationDisplay<'env> {
    fn fmt_type_args(&self, f: &mut Formatter<'_>, targs: &[Type]) -> fmt::Result {
        if !targs.is_empty() {
            let tctx = TypeDisplayContext::WithoutEnv {
                symbol_pool: &self.stbgr.symbol_pool,
                reverse_struct_table: &self.stbgr.reverse_struct_table,
            };
            write!(f, "<")?;
            for (i, ty) in targs.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", ty.display(&tctx))?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }

    fn struct_str(&self, mid: ModuleId, sid: StructId, targs: &[Type]) -> String {
        let ty = Type::Struct(mid, sid, targs.to_vec());
        let tctx = TypeDisplayContext::WithoutEnv {
            symbol_pool: &self.stbgr.symbol_pool,
            reverse_struct_table: &self.stbgr.reverse_struct_table,
        };
        format!("{}", ty.display(&tctx))
    }
}

/// A display object for a borrow node.
#[allow(unused)]
pub struct BorrowNodeDisplay<'env> {
    node: &'env BorrowNode,
    stbgr: &'env StacklessBytecodeGenerator<'env>,
}

impl<'env> fmt::Display for BorrowNodeDisplay<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use BorrowNode::*;
        match self.node {
            GlobalRoot(_s) => {
                // TODO
                // let ty = Type::Struct(s.module_id, s.id, s.inst.to_owned());
                // let tctx = TypeDisplayContext::WithEnv {
                //     env: self.func_target.global_env(),
                //     type_param_names: None,
                // };
                // write!(f, "{}", ty.display(&tctx))?;
            }
            LocalRoot(idx) => {
                write!(f, "LocalRoot($t{})", idx)?;
            }
            Reference(idx) => {
                write!(f, "Reference($t{})", idx)?;
            }
            ReturnPlaceholder(idx) => {
                write!(f, "Return({})", idx)?;
            }
        }
        Ok(())
    }
}

#[allow(unused)]
pub struct BorrowEdgeDisplay<'a> {
    stbgr: &'a StacklessBytecodeGenerator<'a>,
    edge: &'a BorrowEdge,
}

impl<'a> std::fmt::Display for BorrowEdgeDisplay<'a> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // use BorrowEdge::*;
        // TODO
        // let tctx = TypeDisplayContext::WithEnv {
        //     env: self.env,
        //     type_param_names: None,
        // };
        // match self.edge {
        //     Field(qid, field) => {
        //         let struct_env = self.env.get_struct(qid.to_qualified_id());
        //         let field_env = struct_env.get_field_by_offset(*field);
        //         let field_type = field_env.get_type().instantiate(&qid.inst);
        //         write!(
        //             f,
        //             ".{} ({})",
        //             field_env.get_name().display(self.env.symbol_pool()),
        //             field_type.display(&tctx),
        //         )
        //     }
        //     Index(_) => write!(f, "[]"),
        //     Direct => write!(f, "@"),
        //     Hyper(es) => {
        //         write!(
        //             f,
        //             "{}",
        //             es.iter()
        //                 .map(|e| format!("{}", e.display(self.env)))
        //                 .join("/")
        //         )
        //     }
        // }
        Ok(())
    }
}
