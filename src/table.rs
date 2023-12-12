use crate::{assembly::{Table, EncodedAssembly}, type_def::{TypeDefOrRef, TypeDef, TypeDefIndex}};
pub(crate) enum DecodedTable{
    Module{
        name:Box<str>,
        mvid:u128,
    }
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
                for type_def in type_defs.iter(){
                    let flags = type_def.flags();
                    let name:Box<str> = asm.str_at(type_def.name()).to_owned().into();
                    let namespace:Box<str> = asm.str_at(type_def.namespace()).to_owned().into();
                    let derived_from = DotnetTypeRef::new(type_def.derived_from(),asm);
                    let fields = if type_def.field_index().0 == 0{
                        vec![]
                    }else{
                        todo!("fields:{:?}",type_def.field_index())
                    };
                    DecodedTypeDef{ flags, name, namespace, derived_from, fields };
                    //println!("type {namespace}::{name}");
                }
                todo!()
            }
            _=>todo!("Table {table:?} is unsuported!"),
        }
        
    }
}
struct DecodedTypeDef{
    flags: u32,
    name: Box<str>,
    namespace:Box<str>,
    derived_from: Option<DotnetTypeRef>,
    fields: Vec<()>,
    //method_index: MethodIndex,
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