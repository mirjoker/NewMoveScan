// // overflow

// use crate::move_ir::{generate_bytecode::{FunctionInfo, StacklessBytecodeGenerator}, utils::get_def_bytecode, data_dependency::data_dependency};
// use move_stackless_bytecode::stackless_bytecode::{
//     Bytecode, Operation, Constant
// };
// use ethnum::U256;
// use move_model::{ty::{PrimitiveType, Type}};

// pub fn detect_overflow(stbgr: &StacklessBytecodeGenerator, idx: usize) -> bool {
//     let function = &stbgr.functions[idx];
//     let mut ret_flag = false;
//     let local_types = &function.local_types;
//     // let dd = data_dependency(stbgr, idx);
//     for (code_offset, bytecode) in function.code.iter().enumerate() {
//         match &bytecode {
//             Bytecode::Call(_, _ , Operation::Shl, srcs, _) => {
//                 let mut oprand1 = get_def_bytecode(&function, srcs[0], code_offset);
//                 let mut oprand2 = get_def_bytecode(&function, srcs[1], code_offset);
//                 // let dd0 = dd.get(srcs[0]);
//                 // let dd1 = dd.get(srcs[1]);
//                 // if dd0.max.is_some() && dd1.max.is_some() {
//                 //     let a0 = dd0.max.unwrap();
//                 //     let a1 = dd1.max.unwrap();
//                 //     // println!("{} << {}", a0, a1);
//                 // }
//                 // 先判断oprand2的情况(是否是常量，是否有类型转换，是否有mod)
//                 if is_ldconst(oprand2) {
//                     // 如果oprand2是常量，意味着移位的位数是固定的
//                     let shl_bit = get_const_u8(oprand2).unwrap();
//                     // 再判断oprand1的情况，是否是常量，是否存在类型转换，是否存在mod
//                     if is_ldconst(oprand1) {
//                         // oprand1也是常量，那么可以直接算出是否会溢出
//                         match oprand1 {
//                             Bytecode::Load(_, _, Constant::U8(c)) => {
//                                 let num = *c;
//                                 if num.leading_zeros() < (shl_bit as u32) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Load(_, _, Constant::U16(c)) => {
//                                 let num = *c;
//                                 if num.leading_zeros() < (shl_bit as u32) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Load(_, _, Constant::U32(c)) => {
//                                 let num = *c;
//                                 if num.leading_zeros() < (shl_bit as u32) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Load(_, _, Constant::U64(c)) => {
//                                 let num = *c;
//                                 if num.leading_zeros() < (shl_bit as u32) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Load(_, _, Constant::U128(c)) => {
//                                 let num = *c;
//                                 if num.leading_zeros() < (shl_bit as u32) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Load(_, _, Constant::U256(c)) => {
//                                 let num = *c;
//                                 if num.leading_zeros() < (shl_bit as u32) {
//                                     return true;
//                                 }
//                             },
//                             _ => {
//                                 return true;
//                             }
//                         }
//                     } else if is_assign(oprand1) {
//                         // 如果oprand1是变量，那么认为会溢出
//                         return true;
//                     } else if is_call(oprand1) {
//                         match &oprand1 {
//                             // oprand1有mod操作
//                             Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                 let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                 if is_ldconst(mod_num) {
//                                     let modnum = get_const(mod_num).unwrap()-1;
//                                     if modnum.leading_zeros() < (shl_bit as u32) {
//                                         return true;
//                                     }
//                                 } else {
//                                     return true;
//                                 }
//                             },
//                             // oprand1有类型转换，src[0]就是原始数据的类型(也就是转换前变量的类型，可以反应转换前变量的范围)
//                             Bytecode::Call(_, _, Operation::CastU8, src, _) => {
//                                 // 这里可能在类型转换之前有mod操作,判断一下
//                                 let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                 match &tmp_oprand {
//                                     Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                         let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                         if is_ldconst(mod_num) {
//                                             let modnum = get_const(mod_num).unwrap()-1;
//                                             if modnum.leading_zeros() < (shl_bit as u32) {
//                                                 return true;
//                                             }
//                                         }
//                                     },
//                                     _ => {
//                                     }
//                                 }
//                                 if 8 < get_ubits(src[0], local_types) + (shl_bit as u16) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Call(_, _, Operation::CastU16, src, _) => {
//                                 // 这里可能在类型转换之前有mod操作,判断一下
//                                 let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                 match &tmp_oprand {
//                                     Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                         let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                         if is_ldconst(mod_num) {
//                                             let modnum = get_const(mod_num).unwrap()-1;
//                                             if modnum.leading_zeros() < (shl_bit as u32) {
//                                                 return true;
//                                             }
//                                         }
//                                     },
//                                     _ => {
//                                     }
//                                 }
//                                 if 16 < get_ubits(src[0], local_types) + (shl_bit as u16) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Call(_, _, Operation::CastU32, src, _) => {
//                                 // 这里可能在类型转换之前有mod操作,判断一下
//                                 let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                 match &tmp_oprand {
//                                     Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                         let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                         if is_ldconst(mod_num) {
//                                             let modnum = get_const(mod_num).unwrap()-1;
//                                             if modnum.leading_zeros() < (shl_bit as u32) {
//                                                 return true;
//                                             }
//                                         }
//                                     },
//                                     _ => {
//                                     }
//                                 }
//                                 if 32 < get_ubits(src[0], local_types) + (shl_bit as u16) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Call(_, _, Operation::CastU64, src, _) => {
//                                 // 这里可能在类型转换之前有mod操作,判断一下
//                                 let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                 match &tmp_oprand {
//                                     Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                         let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                         if is_ldconst(mod_num) {
//                                             let modnum = get_const(mod_num).unwrap()-1;
//                                             if modnum.leading_zeros() < (shl_bit as u32) {
//                                                 return true;
//                                             }
//                                         }
//                                     },
//                                     _ => {
//                                     }
//                                 }
//                                 if 64 < get_ubits(src[0], local_types) + (shl_bit as u16) {
//                                     return true;
//                                 }
//                             },
//                             Bytecode::Call(_, _, Operation::CastU128, src, _) => {
//                                 // 这里可能在类型转换之前有mod操作,判断一下
//                                 let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                 match &tmp_oprand {
//                                     Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                         let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                         if is_ldconst(mod_num) {
//                                             let modnum = get_const(mod_num).unwrap()-1;
//                                             if modnum.leading_zeros() < (shl_bit as u32) {
//                                                 return true;
//                                             }
//                                         }
//                                     },
//                                     _ => {
//                                     }
//                                 }
//                                 if 128 < get_ubits(src[0], local_types) + (shl_bit as u16) {
//                                     return true;
//                                 }
//                             }, 
//                             Bytecode::Call(_, _, Operation::CastU256, src, _) => {
//                                 // 这里可能在类型转换之前有mod操作,判断一下
//                                 let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                 match &tmp_oprand {
//                                     Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                         let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                         if is_ldconst(mod_num) {
//                                             let modnum = get_const(mod_num).unwrap()-1;
//                                             if modnum.leading_zeros() < (shl_bit as u32) {
//                                                 return true;
//                                             }
//                                         }
//                                     },
//                                     _ => {
//                                     }
//                                 }
//                                 if 256 < get_ubits(src[0], local_types) + (shl_bit as u16) {
//                                     return true;
//                                 }
//                             },
//                             _ => {
//                                 return true;
//                             }
//                         }
//                     }
//                 } else if is_call(oprand2) {
//                     // oprand2不是常量，这里可能有类型转换，或者mod
//                     match &oprand2 {
//                         // 先排除一个类型转换
//                         Bytecode::Call(_, _, Operation::CastU8, srcs, _) => {
//                             oprand2 = get_def_bytecode(&function, srcs[0], code_offset);
//                         },
//                         _ => {
//                         }
//                     }
//                     // 类型转换完后可能存在mod
//                     match &oprand2 {
//                         Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                             let op2_mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                             if is_ldconst(op2_mod_num) {
//                                 let op2modnum = get_const_u64(op2_mod_num).unwrap()-1;
//                                 if is_ldconst(oprand1) {
//                                     // oprand1是常量
//                                     match oprand1 {
//                                         Bytecode::Load(_, _, Constant::U8(c)) => {
//                                             let num = *c;
//                                             if num.leading_zeros() < (op2modnum as u32) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Load(_, _, Constant::U16(c)) => {
//                                             let num = *c;
//                                             if num.leading_zeros() < (op2modnum as u32) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Load(_, _, Constant::U32(c)) => {
//                                             let num = *c;
//                                             if num.leading_zeros() < (op2modnum as u32) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Load(_, _, Constant::U64(c)) => {
//                                             let num = *c;
//                                             if num.leading_zeros() < (op2modnum as u32) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Load(_, _, Constant::U128(c)) => {
//                                             let num = *c;
//                                             if num.leading_zeros() < (op2modnum as u32) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Load(_, _, Constant::U256(c)) => {
//                                             let num = *c;
//                                             if num.leading_zeros() < (op2modnum as u32) {
//                                                 return true;
//                                             }
//                                         },
//                                         _ => {
//                                             return true;
//                                         }
//                                     }
//                                 } else if is_assign(oprand1) {
//                                     // 如果oprand1是变量，那么认为会溢出
//                                     return true;
//                                 } else if is_call(oprand1) {
//                                     match &oprand1 {
//                                         // oprand1有mod操作
//                                         Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                             let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                             if is_ldconst(mod_num) {
//                                                 let modnum = get_const(mod_num).unwrap()-1;
//                                                 if modnum.leading_zeros() < (op2modnum as u32) {
//                                                     return true;
//                                                 }
//                                             } else {
//                                                 return true;
//                                             }
//                                         },
//                                         // oprand1有类型转换，src[0]就是原始数据的类型(也就是转换前变量的类型，可以反应转换前变量的范围)
//                                         Bytecode::Call(_, _, Operation::CastU8, src, _) => {
//                                             // 这里可能在类型转换之前有mod操作,判断一下
//                                             let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                             match &tmp_oprand {
//                                                 Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                                     let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                                     if is_ldconst(mod_num) {
//                                                         let modnum = get_const(mod_num).unwrap()-1;
//                                                         if modnum.leading_zeros() < (op2modnum as u32) {
//                                                             return true;
//                                                         }
//                                                     }
//                                                 },
//                                                 _ => {
//                                                 }
//                                             }
//                                             if 8 < get_ubits(src[0], local_types) + (op2modnum as u16) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Call(_, _, Operation::CastU16, src, _) => {
//                                             // 这里可能在类型转换之前有mod操作,判断一下
//                                             let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                             match &tmp_oprand {
//                                                 Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                                     let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                                     if is_ldconst(mod_num) {
//                                                         let modnum = get_const(mod_num).unwrap()-1;
//                                                         if modnum.leading_zeros() < (op2modnum as u32) {
//                                                             return true;
//                                                         }
//                                                     }
//                                                 },
//                                                 _ => {
//                                                 }
//                                             }
//                                             if 16 < get_ubits(src[0], local_types) + (op2modnum as u16) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Call(_, _, Operation::CastU32, src, _) => {
//                                             // 这里可能在类型转换之前有mod操作,判断一下
//                                             let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                             match &tmp_oprand {
//                                                 Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                                     let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                                     if is_ldconst(mod_num) {
//                                                         let modnum = get_const(mod_num).unwrap()-1;
//                                                         if modnum.leading_zeros() < (op2modnum as u32) {
//                                                             return true;
//                                                         }
//                                                     }
//                                                 },
//                                                 _ => {
//                                                 }
//                                             }
//                                             if 32 < get_ubits(src[0], local_types) + (op2modnum as u16) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Call(_, _, Operation::CastU64, src, _) => {
//                                             // 这里可能在类型转换之前有mod操作,判断一下
//                                             let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                             match &tmp_oprand {
//                                                 Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                                     let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                                     if is_ldconst(mod_num) {
//                                                         let modnum = get_const(mod_num).unwrap()-1;
//                                                         if modnum.leading_zeros() < (op2modnum as u32) {
//                                                             return true;
//                                                         }
//                                                     }
//                                                 },
//                                                 _ => {
//                                                 }
//                                             }
//                                             if 64 < get_ubits(src[0], local_types) + (op2modnum as u16) {
//                                                 return true;
//                                             }
//                                         },
//                                         Bytecode::Call(_, _, Operation::CastU128, src, _) => {
//                                             // 这里可能在类型转换之前有mod操作,判断一下
//                                             let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                             match &tmp_oprand {
//                                                 Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                                     let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                                     if is_ldconst(mod_num) {
//                                                         let modnum = get_const(mod_num).unwrap()-1;
//                                                         if modnum.leading_zeros() < (op2modnum as u32) {
//                                                             return true;
//                                                         }
//                                                     }
//                                                 },
//                                                 _ => {
//                                                 }
//                                             }
//                                             if 128 < get_ubits(src[0], local_types) + (op2modnum as u16) {
//                                                 return true;
//                                             }
//                                         }, 
//                                         Bytecode::Call(_, _, Operation::CastU256, src, _) => {
//                                             // 这里可能在类型转换之前有mod操作,判断一下
//                                             let tmp_oprand = get_def_bytecode(&function, srcs[0], code_offset);
//                                             match &tmp_oprand {
//                                                 Bytecode::Call(_, _, Operation::Mod, srcs, _) => {
//                                                     let mod_num = get_def_bytecode(&function, srcs[1], code_offset);
//                                                     if is_ldconst(mod_num) {
//                                                         let modnum = get_const(mod_num).unwrap()-1;
//                                                         if modnum.leading_zeros() < (op2modnum as u32) {
//                                                             return true;
//                                                         }
//                                                     }
//                                                 },
//                                                 _ => {
//                                                 }
//                                             }
//                                             if 256 < get_ubits(src[0], local_types) + (op2modnum as u16) {
//                                                 return true;
//                                             }
//                                         },
//                                         _ => {
//                                             return true;
//                                         }
//                                     }
//                                 }
//                             } else {
//                                 return true;
//                             }
//                         },
//                         _ => {
//                             return true;
//                         }
//                     }
//                 } else {
//                     return true;
//                 }
//             },
//             _ => {
//                 continue;
//             }
//         }
//     }
//     ret_flag
// }

// fn is_ldconst(bytecode: &Bytecode) -> bool {
//     match bytecode {
//         Bytecode::Load(_, _, _) => {
//             return  true;
//         },
//         _ => {
//             return false;
//         }
//     }
// }

// fn is_assign(bytecode: &Bytecode) -> bool {
//     match bytecode {
//         Bytecode::Assign(_, _, _, _)=> {
//             return  true;
//         },
//         _ => {
//             return false;
//         }
//     }
// }

// fn is_call(bytecode: &Bytecode) -> bool {
//     match bytecode {
//         Bytecode::Call(_, _, _, _, _)=> {
//             return  true;
//         },
//         _ => {
//             return false;
//         }
//     }
// }

// fn get_const(bytecode: &Bytecode) -> Option<U256> {
//     match bytecode {
//         Bytecode::Load(_, _, Constant::U8(c)) => {
//             return Some(ethnum::U256::from(*c));
//         },
//         Bytecode::Load(_, _, Constant::U16(c)) => {
//             return Some(ethnum::U256::from(*c));
//         },
//         Bytecode::Load(_, _, Constant::U32(c)) => {
//             return Some(ethnum::U256::from(*c));
//         },
//         Bytecode::Load(_, _, Constant::U64(c)) => {
//             return Some(ethnum::U256::from(*c));
//         },
//         Bytecode::Load(_, _, Constant::U128(c)) => {
//             return Some(ethnum::U256::from(*c));
//         },
//         Bytecode::Load(_, _, Constant::U256(c)) => {
//             return Some(ethnum::U256::from(*c));
//         },
//         _ => {
//             return Option::None;
//         }
//     }
// }

// fn get_const_u8(bytecode: &Bytecode) -> Option<u8> {
//     match bytecode {
//         Bytecode::Load(_, _, Constant::U8(c)) => {
//             return Some(*c);
//         },
//         _ => {
//             return Option::None;
//         }
//     }
// }

// fn get_const_u16(bytecode: &Bytecode) -> Option<u16> {
//     match bytecode {
//         Bytecode::Load(_, _, Constant::U16(c)) => {
//             return Some(*c);
//         },
//         _ => {
//             return Option::None;
//         }
//     }
// }

// fn get_const_u32(bytecode: &Bytecode) -> Option<u32> {
//     match bytecode {
//         Bytecode::Load(_, _, Constant::U32(c)) => {
//             return Some(*c);
//         },
//         _ => {
//             return Option::None;
//         }
//     }
// }

// fn get_const_u64(bytecode: &Bytecode) -> Option<u64> {
//     match bytecode {
//         Bytecode::Load(_, _, Constant::U64(c)) => {
//             return Some(*c);
//         },
//         _ => {
//             return Option::None;
//         }
//     }
// }

// fn get_const_u128(bytecode: &Bytecode) -> Option<u128> {
//     match bytecode {
//         Bytecode::Load(_, _, Constant::U128(c)) => {
//             return Some(*c);
//         },
//         _ => {
//             return Option::None;
//         }
//     }
// }

// fn get_const_u256(bytecode: &Bytecode) -> Option<U256> {
//     match bytecode {
//         Bytecode::Load(_, _, Constant::U256(c)) => {
//             return Some(*c);
//         },
//         _ => {
//             return Option::None;
//         }
//     }
// }

// fn pass_assign(bytecodes: &Vec<Bytecode>, code_offset: usize, src_idx: usize) -> &Bytecode {
//     let mut tmp_index = code_offset - 1;
//     while tmp_index!=0 {
//         match &bytecodes[tmp_index] {
//             Bytecode::Call(_, dst, _, _, _) => {
//                 if dst[0] == src_idx  {
//                     return &bytecodes[tmp_index];
//                 } else {
//                     tmp_index = tmp_index - 1;
//                     continue;
//                 }
//             },
//             Bytecode::Assign(_, dst, _, _) => {
//                 if *dst == src_idx {
//                     return &bytecodes[tmp_index];
//                 } else {
//                     tmp_index = tmp_index - 1;
//                     continue;
//                 }
//             },
//             Bytecode::Load(_, dst, _) => {
//                 if *dst == src_idx {
//                     return &bytecodes[tmp_index];
//                 } else {
//                     tmp_index = tmp_index - 1;
//                     continue;
//                 }
//             },
//             _ => {
//                 tmp_index = tmp_index - 1;
//                 continue;
//             }
//         }
//     }
//     return &bytecodes[tmp_index];
// }

// fn get_ubits(src: usize, local_types: &Vec<Type>) -> u16 {
//     match local_types[src] {
//         move_model::ty::Type::Primitive(PrimitiveType::U8) => {
//             return 8;
//         },
//         move_model::ty::Type::Primitive(PrimitiveType::U16) => {
//             return 16;
//         },
//         move_model::ty::Type::Primitive(PrimitiveType::U32) => {
//             return 32;
//         },
//         move_model::ty::Type::Primitive(PrimitiveType::U64) => {
//             return 64;
//         },
//         move_model::ty::Type::Primitive(PrimitiveType::U128) => {
//             return 128;
//         },
//         move_model::ty::Type::Primitive(PrimitiveType::U256) => {
//             return 256;
//         },
//         _ => {
//             return 0;
//         }
//     }
// }
