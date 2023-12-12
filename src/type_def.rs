use crate::{
    assembly::{table_rows, u16_from_slice_at, u32_from_slice_at, StringIndex},
    bitvec::BitVec64,
    field::FieldIndex,
    method::MethodIndex,
};
#[derive(Copy, Clone, Debug)]
pub struct TypeDefIndex(pub u32);
#[derive(Copy, Clone, Debug)]
pub struct TypeRefIndex(u32);
#[derive(Copy, Clone, Debug)]
pub struct TypeSpecIndex(u32);
#[derive(Copy, Clone, Debug)]

pub(crate) struct TypeDef {
    flags: u32,
    name: StringIndex,
    namespace: StringIndex,
    derived_from: TypeDefOrRef,
    field_index: FieldIndex,
    method_index: MethodIndex,
}
impl TypeDef {
    pub(crate) fn flags(&self)->u32{
        self.flags
    }
    pub(crate) fn name(&self)->StringIndex{
        self.name
    }
    pub(crate) fn namespace(&self)->StringIndex{
        self.namespace
    }
    pub(crate) fn derived_from(&self)->TypeDefOrRef{
        self.derived_from
    }
    pub(crate) fn field_index(&self)->FieldIndex{
        self.field_index
    }
    pub(crate) fn from_vecs(
        flags: &[u32],
        type_names: &[StringIndex],
        namespaces: &[StringIndex],
        derived_from: &[TypeDefOrRef],
        fields: &[FieldIndex],
        methods: &[MethodIndex],
    ) -> Vec<Self> {
        let mut res = Vec::with_capacity(flags.len());
        for index in 0..flags.len() {
            let flags = flags[index];
            let name = type_names[index];
            let namespace = namespaces[index];
            let derived_from = derived_from[index];
            let field_index = fields[index];
            let method_index = methods[index];
            res.push(Self {
                flags,
                name,
                namespace,
                derived_from,
                field_index,
                method_index,
            })
        }
        res
    }
}
#[derive(Copy, Clone, Debug)]
pub(crate) enum TypeDefOrRef {
    TypeDef(TypeDefIndex),
    TypeRef(TypeRefIndex),
    TypeSpec(TypeSpecIndex),
}
impl TypeDefOrRef {
    pub(crate) fn decode(table_slice: &mut &[u8], tables_rows: &[u32], tables: BitVec64) -> Self {
        let typedef_num = table_rows(tables_rows, tables, 0x2).unwrap_or_default();
        let max = typedef_num;
        let encoded = if max > 1 << 14 {
            let encoded = u32_from_slice_at(table_slice, 0);
            *table_slice = &table_slice[4..];
            encoded
        } else {
            let encoded = u16_from_slice_at(table_slice, 0);
            *table_slice = &table_slice[2..];
            encoded as u32
        };
        let tag = encoded & 0b11;
        let index = tag & (!0b11);
        match tag {
            0x00 => TypeDefOrRef::TypeDef(TypeDefIndex(index)),
            0x01 => TypeDefOrRef::TypeRef(TypeRefIndex(index)),
            0x02 => TypeDefOrRef::TypeSpec(TypeSpecIndex(index)),
            _ => panic!("Invalid TypeDefOrRef tag:{tag}"),
        }
    }
}
