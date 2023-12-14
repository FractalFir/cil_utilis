use crate::{
    assembly::{u16_from_slice_at, u32_from_slice_at, AssemblyRefIndex},
    bitvec::BitVec64,
};

#[derive(Copy, Clone, Debug)]
pub(crate) enum ResolutionScope {
    Module,
    //ModuleRef(ModuleRef),
    AssemblyRef(AssemblyRefIndex),
    //TypeRef(TypeRef),
}
impl ResolutionScope {
    pub(crate) fn decode(table_slice: &mut &[u8], tables_rows: &[u32], tables: BitVec64) -> Self {
        //let typedef_num = table_rows(tables_rows, tables, 0x2).unwrap_or_default();
        let max = 256; //typedef_num;
        let encoded = if max > (1 << 14) {
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
            0x00 => {
                assert_eq!(index, 0);
                ResolutionScope::Module
            }
            0x01 => todo!("ModuleRef:{index}"),
            /// ResolutionScope::ModuleRef(ModuleRef(index)),
            0x02 => ResolutionScope::AssemblyRef(AssemblyRefIndex(index)),
            _ => panic!("Invalid ResolutionScope tag:{tag}"),
        }
    }
}
