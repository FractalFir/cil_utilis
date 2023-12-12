use crate::{
    assembly::{table_rows, u16_from_slice_at, u32_from_slice_at, StringIndex},
    bitvec::BitVec64,
    param::ParamIndex, type_def::TypeDefOrRef,
};
#[derive(Copy, Clone, Debug)]
pub struct MethodIndex(u32);
impl MethodIndex {
    pub(crate) fn decode(table_slice: &mut &[u8], tables_rows: &[u32], tables: BitVec64) -> Self {
        let typedef_num = table_rows(tables_rows, tables, 0x6).unwrap_or_default();
        let max = typedef_num;
        let index = if max > 1 << 16 {
            let index = u32_from_slice_at(table_slice, 0);
            *table_slice = &table_slice[4..];
            index
        } else {
            let index = u16_from_slice_at(table_slice, 0);
            *table_slice = &table_slice[2..];
            index as u32
        };
        MethodIndex(index)
    }
}
#[derive(Copy,Clone, Debug)]
pub struct MethodDef {
    rva: u32,
    rva_size: u32,
    impl_flags: u16,
    flags: u16,
    name: StringIndex,
    param_index: ParamIndex,
}
impl MethodDef {
    pub(crate) fn from_vecs(
        rvas: &[(u32,u32)],
        impl_flags: &[u16],
        flags: &[u16],
        names: &[StringIndex],
        param_indices: &[ParamIndex],
    ) -> Vec<Self> {
        let mut res = Vec::with_capacity(flags.len());
        for index in 0..flags.len() {
            let flags = flags[index];
            let impl_flags = impl_flags[index];
            let name = names[index];
            let (rva,rva_size) = rvas[index];
            let param_index = param_indices[index];
            res.push(Self {
                flags,
                impl_flags,
                rva,
                rva_size,
                name,
                param_index,
            })
        }
        res
    }
}