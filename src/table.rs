use std::ops::Range;

use crate::{
    assembly::{decode_blob_compressed_value, EncodedAssembly, Table},
    field::FieldIndex,
    method::{Method, MethodIndex},
    param::ParamIndex,
    r#type::{decode_type, Type},
    type_def::{TypeDef, TypeDefIndex, TypeDefOrRef},
};
pub(crate) enum DecodedTable {
    Module { name: Box<str>, mvid: u128 },
    TypeDefTable(Box<[DecodedTypeDef]>),
    MethodDef(Box<[(Box<str>, Method, Signature, Range<ParamIndex>)]>),
    Params(Box<[(u16, u16, Box<str>)]>),
}
impl DecodedTable {
    pub(crate) fn decode(table: &Table, asm: &EncodedAssembly) -> Self {
        match table {
            Table::Module { name, mvid } => {
                let name = asm.str_at(*name).to_owned().into();
                let mvid = if mvid.0 == 0 {
                    0
                } else {
                    asm.guid_stream()[(mvid.0 - 1) as usize]
                };
                Self::Module { name, mvid }
            }
            Table::TypeDefTable(type_defs) => {
                let mut res = Vec::with_capacity(type_defs.len());
                for index in 0..type_defs.len() {
                    let type_def = type_defs[index];
                    let flags = type_def.flags();
                    let name: Box<str> = asm.str_at(type_def.name()).to_owned().into();
                    let namespace: Box<str> = asm.str_at(type_def.namespace()).to_owned().into();
                    let derived_from = DotnetTypeRef::new(type_def.derived_from(), asm);
                    let methods_start = type_def.method_index();
                    let methods_end = type_defs
                        .get(index + 1)
                        .map(|td| td.method_index())
                        .unwrap_or_else(|| MethodIndex(asm.methods().len() as u32));
                    let fields_start = type_def.field_index();
                    let fields_end = type_defs
                        .get(index + 1)
                        .map(|td| td.field_index())
                        .unwrap_or_else(|| FieldIndex(asm.field_len() as u32));
                    res.push(DecodedTypeDef {
                        flags,
                        name,
                        namespace,
                        derived_from,
                        fields: (fields_start..fields_end),
                        methods: (methods_start..methods_end),
                    })
                }
                Self::TypeDefTable(res.into())
            }
            Table::MethodDefTable(methods) => {
                let mut new_methods = Vec::new();
                for index in 0..methods.len() {
                    let method = methods[index];
                    let name: Box<str> = asm.str_at(method.name()).to_owned().into();
                    let signature = method.signature();
                    let param_start = method.param_start();
                    let param_end = methods
                        .get(index + 1)
                        .map(|method| method.param_start())
                        .unwrap_or_else(|| ParamIndex(asm.params_len() as u32));
                    let method = crate::method::decode_method(asm.pe_file(), method.rva());
                    let signature = Signature::decode(asm.blob_at(signature),asm);
                    println!("{name}:{signature:?}");
                    new_methods.push((name, method, signature, param_start..param_end));
                }
                Self::MethodDef(new_methods.into())
            }
            Table::Param(params) => Self::Params(
                params
                    .iter()
                    .map(|param| {
                        let flags = param.flags();
                        let sequence = param.sequence();
                        let name: Box<str> = asm.str_at(param.name()).to_owned().into();
                        (flags, sequence, name)
                    })
                    .collect(),
            ),
            _ => todo!("Table {table:?} is unsuported!"),
        }
    }
}
//ECMA spec II.23.1.16
#[derive(Debug)]
struct Signature {
    flags:u8,
    args: Box<[Type]>,
    ret: Type,
}
impl Signature {
    pub fn decode(mut signature: &[u8],asm:&EncodedAssembly) -> Self {
        let flags = signature[0];
        signature = &signature[1..];
        let argc: u32 = decode_blob_compressed_value(&mut signature);
        let mut args = Vec::with_capacity(argc as usize);
        for _ in 0..argc {
            args.push(decode_type(&mut signature,asm).unwrap());
        }
        let ret = decode_type(&mut signature,asm).unwrap();
        //println!("signature:{signature:?} flags:{flags:b} argc:{argc}");
        Self {
            args: args.into(),
            ret,
            flags,
        }
    }
}
struct DecodedTypeDef {
    flags: u32,
    name: Box<str>,
    namespace: Box<str>,
    derived_from: Option<DotnetTypeRef>,
    fields: std::ops::Range<FieldIndex>,
    methods: std::ops::Range<MethodIndex>,
}
struct DotnetTypeRef {}
impl DotnetTypeRef {
    fn new(tdor: TypeDefOrRef, asm: &EncodedAssembly) -> Option<Self> {
        match tdor {
            TypeDefOrRef::TypeDef(def_idx) => {
                if def_idx.0 == 0 {
                    return None;
                }
                todo!("Can't get name of type {def_idx:?}");
            }
            _ => todo!("Can't get name of type {tdor:?}"),
        }
    }
}
