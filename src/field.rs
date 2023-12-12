use crate::{
    assembly::{table_rows, u16_from_slice_at, u32_from_slice_at},
    bitvec::BitVec64,
};
#[derive(Copy, Clone, Debug)]
pub struct FieldIndex(pub u32);
impl FieldIndex {
    pub(crate) fn decode(table_slice: &mut &[u8], tables_rows: &[u32], tables: BitVec64) -> Self {
        let typedef_num = table_rows(tables_rows, tables, 0x4).unwrap_or_default();
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
        FieldIndex(index)
    }
}
