use crate::{assembly::{Table, EncodedAssembly}, type_def::{TypeDefOrRef, TypeDef, TypeDefIndex}, method::MethodIndex, field::FieldIndex};
pub(crate) enum DecodedTable{
    Module{
        name:Box<str>,
        mvid:u128,
    },
    TypeDefTable(Box<[DecodedTypeDef]>),
}
impl DecodedTable {
    pub(crate) fn decode(table:&Table,asm:&EncodedAssembly)->Self{
        match table{
            Table::Module { name, mvid }=>{
                let name = asm.str_at(*name).to_owned().into();
                let mvid = if mvid.0 == 0{
                    0
                }else{
                    asm.guid_stream()[(mvid.0 - 1) as usize]
                };
                Self::Module { name, mvid }
            }
            Table::TypeDefTable(type_defs)=>{
                let mut res = Vec::with_capacity(type_defs.len());
                for index in 0..type_defs.len(){
                    let type_def = type_defs[index];
                    let flags = type_def.flags();
                    let name:Box<str> = asm.str_at(type_def.name()).to_owned().into();
                    let namespace:Box<str> = asm.str_at(type_def.namespace()).to_owned().into();
                    let derived_from = DotnetTypeRef::new(type_def.derived_from(),asm);
                    let methods_start = type_def.method_index();
                    let methods_end = type_defs.get(index + 1).map(|td|td.method_index()).unwrap_or_else(||MethodIndex(asm.methods().len() as u32));
                    let fields_start = type_def.field_index();
                    let fields_end = type_defs.get(index + 1).map(|td|td.field_index()).unwrap_or_else(||FieldIndex(asm.field_len() as u32));
                    res.push(DecodedTypeDef{ flags, name, namespace, derived_from, fields:(fields_start..fields_end) ,methods:(methods_start..methods_end)})
                }
                Self::TypeDefTable(res.into())
            }
            Table::MethodDefTable(methods)=>{
                for method in methods.iter(){
                    let name:Box<str> = asm.str_at(method.name()).to_owned().into();
                    let signature = method.signature();
                    let method = crate::method::decode_method(asm.pe_file(), method.rva());
                    let signature = Signature::decode(asm.blob_at(signature));
                   
                    //let flags = 
                    println!("method name:{name} method:{method:?} signature:{signature:?}");
                }
                todo!()
            }
            _=>todo!("Table {table:?} is unsuported!"),
        }
        
    }
}
#[derive(Debug)]
struct Signature{

}
impl Signature{
    pub fn decode(mut signature:&[u8])->Self{
        let flags = signature[0];
        let argc = signature[1];
        println!("signature:{signature:?} flags:{flags:b} argc:{argc}");
        Self{}
    }
}
struct DecodedMethod{
    name:Box<str>,
}
struct DecodedTypeDef{
    flags: u32,
    name: Box<str>,
    namespace:Box<str>,
    derived_from: Option<DotnetTypeRef>,
    fields: std::ops::Range<FieldIndex>,
    methods: std::ops::Range<MethodIndex>,
}
struct DotnetTypeRef{

}
impl DotnetTypeRef{
    fn new(tdor:TypeDefOrRef,asm:&EncodedAssembly)->Option<Self>{
        match tdor{
            TypeDefOrRef::TypeDef(def_idx)=>{
                if def_idx.0 == 0{
                    return None;
                }
                todo!("Can't get name of type {def_idx:?}");
            }
            _=>todo!("Can't get name of type {tdor:?}")
        }
    }
}