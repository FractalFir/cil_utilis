use crate::{assembly::{decode_blob_compressed_value, EncodedAssembly}, type_def::TypeDefOrRef};
// II.23.1.16
pub fn decode_type(signature: &mut &[u8],asm:&EncodedAssembly) -> Option<Type> {
    let tpe = decode_blob_compressed_value(signature);
    match tpe {
        0x0=>None,
        0x1=>Some(Type::Void),
        0x2=>Some(Type::Bool),
        0x3=>Some(Type::Char),
        0x4 => Some(Type::I8),
        0x5 => Some(Type::U8),
        0x6 => Some(Type::I16),
        0x7 => Some(Type::U16),
        0x8 => Some(Type::I32),
        0x9 => Some(Type::U32),
        0xa => Some(Type::I64),
        0xb => Some(Type::U64),
        0xc => Some(Type::F32),
        0xd => Some(Type::F64),
        0xe => Some(Type::String),
        0xf=>Some(Type::Ptr(decode_type(signature,asm).unwrap().into())),
        0x10=>Some(Type::Ref(decode_type(signature,asm).unwrap().into())),
        0x11=>{
            let (rows,tables) = asm.tables_rows(); 
            Some(Type::ValueType(TypeDefOrRef::decode(signature, rows, tables)))
        },
        0x12=>{
            let (rows,tables) = asm.tables_rows(); 
            Some(Type::ValueType(TypeDefOrRef::decode(signature, rows, tables)))
        },
        0x13=>Some(Type::Generic(decode_blob_compressed_value(signature))),
        0x14=>{
            let element = decode_type(signature,asm).unwrap().into();
            let rank = decode_blob_compressed_value(signature);
            let bound_count = decode_blob_compressed_value(signature);
            assert_eq!(bound_count,0,"Array bounds not supported!");
            Some(Type::Array(element,rank))
        },
        0x1d=>{
            let element = decode_type(signature,asm).unwrap().into();
            Some(Type::Array(element,1))
        },
        _ => todo!("Unknown type {tpe:x}"),
    }
}
#[derive(Debug)]
pub enum Type {
    Void,
    Bool,
    Char,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    String,
    Ptr(Box<Self>),
    Ref(Box<Self>),
    ValueType(TypeDefOrRef),
    ClassType(TypeDefOrRef),
    Generic(u32),
    Array(Box<Type>,u32),
}
