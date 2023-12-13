use crate::{
    assembly::{table_rows, u16_from_slice_at, u32_from_slice_at, StringIndex, BlobIndex},
    bitvec::BitVec64,
    param::ParamIndex, type_def::TypeDefOrRef, pe_file::{PEFile, RVA},
};
#[derive(Copy, Clone, Debug)]
pub struct MethodIndex(pub u32);
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
    impl_flags: u16,
    flags: u16,
    name: StringIndex,
    signature:BlobIndex,
    param_index: ParamIndex,
}
impl MethodDef {
    pub(crate) fn rva(&self)->RVA{
        RVA(self.rva as u64)
    }
    pub(crate) fn name(&self)->StringIndex{
        self.name
    }
    pub(crate) fn signature(&self)->BlobIndex{
        self.signature
    }
    pub(crate) fn from_vecs(
        rvas: &[u32],
        impl_flags: &[u16],
        flags: &[u16],
        names: &[StringIndex],
        signatures:&[BlobIndex],
        param_indices: &[ParamIndex],
    ) -> Vec<Self> {
        let mut res = Vec::with_capacity(flags.len());
        for index in 0..flags.len() {
            let flags = flags[index];
            let impl_flags = impl_flags[index];
            let name = names[index];
            let rva= rvas[index];
            let signature = signatures[index];
            let param_index = param_indices[index];
            res.push(Self {
                flags,
                impl_flags,
                rva,
                name,
                signature,
                param_index,
            })
        }
        res
    }
}
#[derive(Debug)]
pub(crate) struct Method{
    ops:Box<[CILOp]>,
}
pub(crate) fn decode_method(file:&PEFile,rva:RVA)->Method{
    let first_byte = file.slice_at_rva(rva, 1).expect("Can't find method header!")[0];
    let tag = first_byte & 0b11;
    match tag{
        0x2=>{
            let len = first_byte>>2;
            let mut slice = &file.slice_at_rva(rva, len as u64 + 1).unwrap()[1..];
            let mut ops = Vec::new();
            while slice.len() > 0{
                ops.push(decode_op(&mut slice));
            }
            Method{ops:ops.into()}
        }
        0x3=>todo!("Can't decode fat headers"),
        _=>todo!("Unsuported method header")
    }
}
fn decode_op(slice:&mut &[u8])->CILOp{
    let byte = slice[0];
    match byte{
        0x2=>{
            *slice = &slice[1..];
            CILOp::LDArg0.into()
        }
        0x3=>{
            *slice = &slice[1..];
            CILOp::LDArg1.into()
        }
        0x4=>{
            *slice = &slice[1..];
            CILOp::LDArg2.into()
        }
        0x5=>{
            *slice = &slice[1..];
            CILOp::LDArg3.into()
        }
        0x25=>{
            *slice = &slice[1..];
            CILOp::Dup
        }
        0x2a=>{
            *slice = &slice[1..];
            CILOp::Ret
        }
        0x58=>{
            *slice = &slice[1..];
            CILOp::Add
        }
        0x5a=>{
            *slice = &slice[1..];
            CILOp::Mul
        }
        _=>todo!("Unhandled op 0x{byte:x}!"),
    }
}
#[derive(Debug)]
enum CILOp{
    LDArg0,
    LDArg1,
    LDArg2,
    LDArg3,
    //LDArg(u16),
    Add,
    Mul,
    Ret,
    Dup,
}