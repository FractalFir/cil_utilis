use std::io::{Read, Seek};

use crate::{
    bitvec::BitVec64,
    field::{Field, FieldIndex},
    method::{MemberRef, MemberRefParent, MethodDef, MethodIndex},
    param::{Param, ParamIndex},
    pe_file::{PEFile, PEFileReadError, RVA},
    resolution_scope::ResolutionScope,
    table::TypeRef,
    type_def::{TypeDef, TypeDefOrRef},
};
#[derive(Copy, Clone, Debug)]
pub(crate) struct AssemblyRefIndex(pub u32);
#[derive(Clone, Copy)]
struct HeapSizes {
    bitvec: BitVec64,
}
impl From<u8> for HeapSizes {
    fn from(value: u8) -> Self {
        Self {
            bitvec: (value as u64).into(),
        }
    }
}
impl HeapSizes {
    pub fn read_string_index(&self, slice: &[u8]) -> StringIndex {
        if self.bitvec.bit(0) {
            StringIndex(u32_from_slice_at(slice, 0))
        } else {
            StringIndex(u16_from_slice_at(slice, 0) as u32)
        }
    }
    pub fn string_index_size(&self) -> usize {
        if self.bitvec.bit(0) {
            4
        } else {
            2
        }
    }
    pub fn read_guid_index(&self, slice: &[u8]) -> GUIDIndex {
        if self.bitvec.bit(1) {
            GUIDIndex(u32_from_slice_at(slice, 0))
        } else {
            GUIDIndex(u16_from_slice_at(slice, 0) as u32)
        }
    }
    pub fn guid_index_size(&self) -> usize {
        if self.bitvec.bit(1) {
            4
        } else {
            2
        }
    }
    pub fn blob_index_size(&self) -> usize {
        if self.bitvec.bit(2) {
            4
        } else {
            2
        }
    }
    pub fn read_blob_index(&self, slice: &[u8]) -> BlobIndex {
        if self.bitvec.bit(2) {
            BlobIndex(u32_from_slice_at(slice, 0))
        } else {
            BlobIndex(u16_from_slice_at(slice, 0) as u32)
        }
    }
}
#[derive(Debug)]
enum MetadataStream {
    LogicalMetadataTable(Box<[u32]>, BitVec64, Box<[Table]>),
    Strings(Box<[u8]>),
    US(Box<[u8]>),
    Blob(Box<[u8]>),
    GUID(Box<[u128]>),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StringIndex(pub u32);
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GUIDIndex(pub u32);
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BlobIndex(pub u32);
#[derive(Clone, Debug)]
pub(crate) enum Table {
    Module {
        name: StringIndex,
        mvid: GUIDIndex,
    },
    TypeRefTable(Box<[TypeRef]>),
    TypeDefTable(Box<[TypeDef]>),
    Fields(Box<[Field]>),
    MethodDefTable(Box<[MethodDef]>),
    Param(Vec<Param>),
    MemberRef(Box<[MemberRef]>),
    StandAloneSig(Box<[BlobIndex]>),
    Assembly {
        hash_alg_id: u32,
        major: u16,
        minor: u16,
        build_number: u16,
        revision_number: u16,
        flags: u32,
        public_key: BlobIndex,
        name: StringIndex,
        culture: StringIndex,
    },
    AssemblyRefs(Box<[AssemblyRef]>),
}
#[derive(Clone, Debug)]
pub(crate) struct AssemblyRef {
    hash_alg_id: u32,
    major: u16,
    minor: u16,
    build_number: u16,
    revision_number: u16,
    flags: u32,
    public_key: BlobIndex,
    name: StringIndex,
    culture: StringIndex,
}

impl AssemblyRef {
    pub(crate) fn name(&self) -> StringIndex {
        self.name
    }
}
pub(crate) fn table_rows(tables_rows: &[u32], tables: BitVec64, table: u8) -> Option<u32> {
    match tables.into_iter().position(|v| v == table) {
        Some(type_def_row) => Some(tables_rows[type_def_row as usize]),
        None => None,
    }
}
impl Table {
    fn from(
        table: u8,
        table_slice: &mut &[u8],
        rows: u32,
        sizes: HeapSizes,
        tables_rows: &[u32],
        tables: BitVec64,
    ) -> Table {
        match table {
            0x00 => {
                assert_eq!(rows, 1, "There can be one and only one module table!");
                let generation = u16_from_slice_at(table_slice, 0);
                assert_eq!(generation, 0);
                *table_slice = &table_slice[2..];
                let name = sizes.read_string_index(table_slice);
                *table_slice = &table_slice[sizes.string_index_size()..];
                let mvid = sizes.read_guid_index(table_slice);
                *table_slice = &table_slice[sizes.guid_index_size()..];
                let enc_id = sizes.read_guid_index(table_slice);
                *table_slice = &table_slice[sizes.guid_index_size()..];
                assert_eq!(enc_id, GUIDIndex(0));
                let enc_base_id = sizes.read_guid_index(table_slice);
                *table_slice = &table_slice[sizes.guid_index_size()..];
                assert_eq!(enc_base_id, GUIDIndex(0));
                Self::Module { name, mvid }
            }
            0x1 => {
                let mut type_refs = Vec::with_capacity(rows as usize);
                for _ in 0..rows {
                    let scope = ResolutionScope::decode(table_slice, tables_rows, tables);
                    let name = sizes.read_string_index(table_slice);
                    assert_ne!(name.0,0);
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    let namespace = sizes.read_string_index(table_slice);
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    println!("scope:{scope:?} name:{name:?} namespace:{namespace:?}");
                    type_refs.push(TypeRef::new(scope, name, namespace));
                }
                Table::TypeRefTable(type_refs.into())
            }
            0x2 => {
                let mut flags = Vec::with_capacity(rows as usize);
                let mut type_names = Vec::with_capacity(rows as usize);
                let mut type_namespaces = Vec::with_capacity(rows as usize);
                let mut derived_from = Vec::with_capacity(rows as usize);
                let mut field_idx = Vec::with_capacity(rows as usize);
                let mut method_idx = Vec::with_capacity(rows as usize);
                for _ in 0..rows {
                    flags.push(u32_from_slice_at(table_slice, 0));
                    *table_slice = &table_slice[4..];
                    type_names.push(sizes.read_string_index(table_slice));
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    type_namespaces.push(sizes.read_string_index(table_slice));
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    derived_from.push(TypeDefOrRef::decode(table_slice, tables_rows, tables));
                    field_idx.push(FieldIndex::decode(table_slice, tables_rows, tables));
                    //let idx = u32_from_slice_at(table_slice, 0);
                    
                    let idx = MethodIndex::decode(table_slice, tables_rows, tables);

                    method_idx.push(idx);
                }

                let type_defs = TypeDef::from_vecs(
                    &flags,
                    &type_names,
                    &type_namespaces,
                    &derived_from,
                    &field_idx,
                    &method_idx,
                );
                println!("type_defs:{type_defs:?}");
                let rva = u32_from_slice_at(table_slice, 0);
                println!("after typedef:0x{rva:x}");
                Self::TypeDefTable(type_defs.into())
            }
            0x4 => {
                let mut fields = Vec::with_capacity(rows as usize);
                for _ in 0..rows {
                    let flags = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let name = sizes.read_string_index(table_slice);
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    println!("Field name:{name:?}");
                    let signature = sizes.read_blob_index(table_slice);
                    *table_slice = &table_slice[sizes.blob_index_size()..];
                    fields.push(Field::new(flags, name, signature));
                }
                Table::Fields(fields.into())
            }
            0x6 => {
                let mut method_defs = Vec::with_capacity(rows as usize);
                for _ in 0..rows {
                    let rva = u32_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[4..];
                    let impl_flags = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let flags = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let name = sizes.read_string_index(table_slice);
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    let signature = sizes.read_blob_index(table_slice);
                    *table_slice = &table_slice[sizes.blob_index_size()..];
                    let param_index = ParamIndex::decode(table_slice, tables_rows, tables);
                    method_defs.push(MethodDef::new(
                        rva,
                        impl_flags,
                        flags,
                        name,
                        signature,
                        param_index,
                    ));
                }
                Self::MethodDefTable(method_defs.into())
            }
            0x8 => {
                let mut params = Vec::with_capacity(rows as usize);
                for _ in 0..rows {
                    let flags = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let sequence = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let name = sizes.read_string_index(table_slice);
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    params.push(Param::new(flags, sequence, name))
                }
                Self::Param(params)
            }
            0xa => {
                let mut member_refs = Vec::with_capacity(rows as usize);
                for _ in 0..rows {
                    let class = MemberRefParent::decode(table_slice, tables_rows, tables);
                    let name = sizes.read_string_index(table_slice);
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    let signature = sizes.read_blob_index(table_slice);
                    *table_slice = &table_slice[sizes.blob_index_size()..];

                    member_refs.push(MemberRef::new(class, name, signature))
                }
                Self::MemberRef(member_refs.into())
            }
            0x11 => {
                let mut sigs = Vec::with_capacity(rows as usize);
                for _ in 0..rows {
                    let signature = sizes.read_blob_index(table_slice);
                    *table_slice = &table_slice[sizes.blob_index_size()..];

                    sigs.push(signature);
                }
                Self::StandAloneSig(sigs.into())
            }
            0x20 => {
                let hash_alg_id = u32_from_slice_at(table_slice, 0);
                *table_slice = &table_slice[4..];
                let major = u16_from_slice_at(table_slice, 0);
                *table_slice = &table_slice[2..];
                let minor = u16_from_slice_at(table_slice, 0);
                *table_slice = &table_slice[2..];
                let build_number = u16_from_slice_at(table_slice, 0);
                *table_slice = &table_slice[2..];

                let revision_number = u16_from_slice_at(table_slice, 0);
                *table_slice = &table_slice[2..];
                let flags = u32_from_slice_at(table_slice, 0);
                *table_slice = &table_slice[4..];
                let public_key = sizes.read_blob_index(table_slice);
                *table_slice = &table_slice[sizes.blob_index_size()..];
                let name = sizes.read_string_index(table_slice);
                    println!("asm_name:{name:?} minor:{minor:x} major:{major:x} build_number:{build_number:x} revision_number:{revision_number:x}");
                *table_slice = &table_slice[sizes.string_index_size()..];
                assert_ne!(name.0,0);
                let culture = sizes.read_string_index(table_slice);
                    *table_slice = &table_slice[sizes.string_index_size()..];
                    println!("asm_culture:{culture:?}");
                Self::Assembly {
                    hash_alg_id,
                    major,
                    minor,
                    build_number,
                    revision_number,
                    flags,
                    public_key,
                    name,
                    culture
                }
            }
            
            0x23 => {
                let mut refs = Vec::with_capacity(rows as usize);
                for _ in 0..rows{
                    let hash_alg_id = u32_from_slice_at(table_slice, 0);
                    println!("hash_alg_id:{hash_alg_id:x}");
                    *table_slice = &table_slice[4..];
                    let major = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let minor = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let build_number = u16_from_slice_at(table_slice, 0);
                    println!("build_number:{build_number}");
                    *table_slice = &table_slice[2..];
                    let revision_number = u16_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[2..];
                    let flags = u32_from_slice_at(table_slice, 0);
                    *table_slice = &table_slice[4..];
                    let public_key = sizes.read_blob_index(table_slice);
                    *table_slice = &table_slice[sizes.blob_index_size()..];
                    let name = sizes.read_string_index(table_slice);
                    //assert_ne!(name.0,0);
                    println!("asm_ref_name:{name:?}");
                    //*table_slice = &table_slice[sizes.string_index_size()..];
                    //let culture = sizes.read_string_index(table_slice);
                    //*table_slice = &table_slice[sizes.string_index_size()..];
                    //println!("asm_ref_culture:{culture:?}");
                    refs.push(AssemblyRef {
                        hash_alg_id,
                        major,
                        minor,
                        build_number,
                        revision_number,
                        flags,
                        public_key,
                        name,
                        culture:StringIndex(0),
                    });
                }
                let refs = [];
                Self::AssemblyRefs(refs.into())
            }
            _ => todo!("Unknown table 0x{table:x}",),
        }
    }
}
fn next_table(mut table_list: u64) -> Option<(u64, u64)> {
    let leading_zeros = table_list.leading_zeros();
    if leading_zeros == 64 {
        return None;
    }
    let mask = 1_u64 << leading_zeros;
    table_list &= !mask;
    Some((table_list, mask))
}
impl MetadataStream {
    fn logical_metadata_table(stream: &[u8]) -> Self {
        let reserved = u32_from_slice_at(stream, 0);
        assert_eq!(reserved, 0);
        let major = stream[4];
        assert_eq!(major, 2);
        let minor = stream[5];
        assert_eq!(minor, 0);
        let heap_sizes = stream[6];
        let reserved = stream[7];
        assert_eq!(reserved, 1);
        let present_tables = u64_from_slice_at(stream, 8);
        assert!(present_tables < 1 << 44);
        let sorted_tables = u64_from_slice_at(stream, 16);

        let table_count = present_tables.count_ones();
        let mut rows = Vec::with_capacity(table_count as usize);
        for idx in 0..table_count {
            rows.push(u32_from_slice_at(stream, (24 + idx * 4) as usize));
        }
        let mut table_slice: &[u8] = &stream[(24 + table_count as usize * 4)..];
        let tables: BitVec64 = present_tables.into();
        let mut encoded_tables = Vec::with_capacity(table_count as usize);
        for (table, curr_rows) in tables.into_iter().zip(rows.iter()) {
            encoded_tables.push(Table::from(
                table,
                &mut table_slice,
                *curr_rows,
                heap_sizes.into(),
                &rows,
                tables,
            ));
        }
        Self::LogicalMetadataTable(rows.into(), tables, encoded_tables.into())
    }
    fn string_stream(stream: &[u8]) -> Self {
        Self::Strings(stream.to_owned().into())
    }
    fn us_stream(stream: &[u8]) -> Self {
        Self::US(stream.to_owned().into())
    }
    fn blob_stream(stream: &[u8]) -> Self {
        println!("blob len:{}", stream.len());
        Self::Blob(stream.to_owned().into())
    }
    fn guid_stream(stream: &[u8]) -> Self {
        let guid_count = stream.len() / std::mem::size_of::<u128>();
        let mut guids = Vec::with_capacity(guid_count);
        for index in 0..guid_count {
            guids.push(u128_from_slice_at(stream, index * 16));
        }
        Self::GUID(guids.to_owned().into())
    }
    fn from_slice(metadata: &[u8], curr_offset: &mut usize) -> Self {
        let stream_offset = u32_from_slice_at(metadata, *curr_offset);

        let stream_size = u32_from_slice_at(metadata, *curr_offset + 4);
        println!("stream_offset:{stream_offset} stream_size:{stream_size}");
        *curr_offset += 8;
        let name_start = *curr_offset;
        while metadata[*curr_offset] != 0 {
            *curr_offset += 1;
        }
        let name_end = *curr_offset;
        if *curr_offset | 0b11 != 0 {
            *curr_offset &= !0b11;
            *curr_offset += 0b100;
        }

        let name =
            std::str::from_utf8(&metadata[name_start..name_end]).expect("Not utf8 stream name!");
        println!("name:{name} curr_offset:{curr_offset}");
        let stream =
            &metadata[(stream_offset as usize)..(stream_offset as usize + stream_size as usize)];

        match name {
            "#~" => Self::logical_metadata_table(stream),
            "#Strings" => Self::string_stream(stream),
            "#US" => Self::us_stream(stream),
            "#Blob" => Self::blob_stream(stream),
            "#GUID" => Self::guid_stream(stream),
            _ => todo!("Unknown stream \"{name}\"."),
        }
    }
}
#[derive(Debug)]
struct RawMetadata {
    major: u32,
    minor: u32,
    version: Box<str>,
    streams: Vec<MetadataStream>,
}
impl RawMetadata {
    fn from_slice(metadata: &[u8]) -> Self {
        let magic = u32_from_slice_at(metadata, 0);
        assert_eq!(magic, 0x424A5342);
        let major = u32_from_slice_at(metadata, 4);
        let minor = u32_from_slice_at(metadata, 6);
        let reserved = u32_from_slice_at(metadata, 8);
        assert_eq!(reserved, 0);
        let length = u32_from_slice_at(metadata, 12);
        let rounded_length = (length / 4 + (length % 4)) * 4;
        let version = &metadata[16..(16 + rounded_length as usize)];
        let null = version.iter().position(|c| *c == 0).unwrap_or(8);
        let version = std::str::from_utf8(&version[..null]).expect("Not utf8 version name!");
        println!("version:{version}");
        let flags = u16_from_slice_at(metadata, 16 + rounded_length as usize);
        assert_eq!(flags, 0);
        let stream_count = u16_from_slice_at(metadata, 16 + rounded_length as usize + 2);
        let mut curr_offset = 16 + rounded_length as usize + 4;
        println!("curr_offset:{curr_offset}");
        let mut streams = Vec::with_capacity(curr_offset);
        for _ in 0..stream_count {
            streams.push(MetadataStream::from_slice(metadata, &mut curr_offset));
        }
        RawMetadata {
            major,
            minor,
            version: version.into(),
            streams,
        }
    }
}
#[derive(Debug)]
pub struct CILHeader {
    flags: u32,
    entrypoint: u32,
    native_resource_rva: u32,
    native_resource_size: u32,
    strong_name_rva: u32,
    strong_name_size: u32,
    vtable_fixups: u64,
    raw_metadata: RawMetadata,
}
fn u128_from_slice_at(slice: &[u8], offset: usize) -> u128 {
    let value: [u8; 16] = slice[offset..(offset + 16)].try_into().unwrap();
    u128::from_le_bytes(value)
}
pub fn u64_from_slice_at(slice: &[u8], offset: usize) -> u64 {
    let value: [u8; 8] = slice[offset..(offset + 8)].try_into().unwrap();
    u64::from_le_bytes(value)
}
pub fn u32_from_slice_at(slice: &[u8], offset: usize) -> u32 {
    let value: [u8; 4] = slice[offset..(offset + 4)].try_into().unwrap();
    u32::from_le_bytes(value)
}
pub fn u16_from_slice_at(slice: &[u8], offset: usize) -> u16 {
    let value: [u8; 2] = slice[offset..(offset + 2)].try_into().unwrap();
    u16::from_le_bytes(value)
}
impl CILHeader {
    fn read_from_pe(pe_file: &PEFile) -> Self {
        let cli_header_rva: RVA = pe_file.pe_header().nt_header().cil_header();
        let cil_header_size = pe_file.pe_header().nt_header().cil_header_size();
        //let cli_header_rva = RVA(8192);
        let cli_header = pe_file
            .slice_at_rva(cli_header_rva, cil_header_size as u64)
            .expect("CIL header has invalid RVA");
        println!("cli_header:{cli_header:?}");
        let cb = u32_from_slice_at(cli_header, 0);
        let major_runtime = u16_from_slice_at(cli_header, 4);
        let minor_runtime = u16_from_slice_at(cli_header, 6);
        let metadata_rva = u32_from_slice_at(cli_header, 8);
        //println!("metadata_rva:{metadata_rva}");
        let metadata_size = u32_from_slice_at(cli_header, 8 + 4);
        let flags = u32_from_slice_at(cli_header, 16);
        let metadata = pe_file
            .slice_at_rva(RVA(metadata_rva as u64), metadata_size as u64)
            .expect("Metadata header has invalid RVA");
        let entrypoint = u32_from_slice_at(cli_header, 20);
        let native_resource_rva = u32_from_slice_at(cli_header, 24);
        let native_resource_size = u32_from_slice_at(cli_header, 28);
        let strong_name_rva = u32_from_slice_at(cli_header, 24);
        let strong_name_size = u32_from_slice_at(cli_header, 28);
        let code_manager_table = u64_from_slice_at(cli_header, 40);
        assert_eq!(code_manager_table, 0);
        let vtable_fixups = u64_from_slice_at(cli_header, 48);
        let export_address_table_jumps = u64_from_slice_at(cli_header, 56);
        assert_eq!(export_address_table_jumps, 0);
        let managed_native_header = u64_from_slice_at(cli_header, 56);
        assert_eq!(managed_native_header, 0);
        let raw_metadata = RawMetadata::from_slice(metadata);
        Self {
            flags,
            entrypoint,
            native_resource_rva,
            native_resource_size,
            strong_name_rva,
            strong_name_size,
            vtable_fixups,
            raw_metadata,
        }
    }

    pub fn raw_metadata(&self) -> &RawMetadata {
        &self.raw_metadata
    }
}
pub struct EncodedAssembly {
    pe_file: PEFile,
    header: CILHeader,
}
pub fn get_blob(blob_heap: &[u8], pos: BlobIndex) -> &[u8] {
    let ptr: &[u8] = &blob_heap[pos.0 as usize..];
    if ptr[0] & 0x80 == 0 {
        let size = (ptr[0] & 0x7f) as usize;
        &ptr[1..(1 + size)]
    } else if ptr[0] & 0x40 == 0 {
        let size = ((ptr[0] & 0x3f) as usize) << 8 + ptr[1] as usize;
        &ptr[2..(2 + size)]
    } else {
        let size = ((ptr[0] & 0x1f) as usize)
            << 24 + ((ptr[1] as usize) << 16) + ((ptr[2] as usize) << 8) + ptr[3] as usize;
        &ptr[4..(4 + size)]
    }
}
pub fn decode_blob_compressed_value(blob_heap: &mut &[u8]) -> u32 {
    if blob_heap[0] & 0x80 == 0 {
        let value = (blob_heap[0] & 0x7f) as u32;
        *blob_heap = &blob_heap[1..];
        value
    } else if blob_heap[0] & 0x40 == 0 {
        let value = ((blob_heap[0] & 0x3f) as u32) << 8 + blob_heap[1] as u32;
        *blob_heap = &blob_heap[2..];
        value
    } else {
        let value = ((blob_heap[0] & 0x1f) as u32)
            << 24
                + ((blob_heap[1] as u32) << 16)
                + ((blob_heap[2] as u32) << 8)
                + blob_heap[3] as u32;
        *blob_heap = &blob_heap[4..];
        value
    }
}
impl EncodedAssembly {
    pub fn pe_file(&self) -> &PEFile {
        &self.pe_file
    }
    pub fn from_file(file: &mut (impl Read + Seek)) -> Result<Self, AssemblyReadError> {
        let pe_file = PEFile::from_file(file)?;
        let header = CILHeader::read_from_pe(&pe_file);
        Ok(Self { pe_file, header })
    }
    pub fn str_at(&self, string_index: StringIndex) -> &str {
        let slice = &self.string_stream()[(string_index.0 as usize)..];
        let mut null = 0;
        while slice[null] != 0 {
            null += 1;
        }
        std::str::from_utf8(&slice[..null]).expect("Invalid asm string!")
    }
    pub fn string_stream(&self) -> &[u8] {
        for stream in &self.header.raw_metadata.streams {
            match stream {
                MetadataStream::Strings(strings) => return &strings,
                _ => (),
            }
        }
        panic!("No String stream!")
    }
    pub fn guid_stream(&self) -> &[u128] {
        for stream in &self.header.raw_metadata.streams {
            match stream {
                MetadataStream::GUID(guid) => return &guid,
                _ => (),
            }
        }
        panic!("No GUID stream!")
    }
    pub fn blob_stream(&self) -> &[u8] {
        for stream in &self.header.raw_metadata.streams {
            match stream {
                MetadataStream::Blob(blob) => return &blob,
                _ => (),
            }
        }
        panic!("No Blob stream!")
    }
    pub fn blob_at(&self, blob_index: BlobIndex) -> &[u8] {
        get_blob(self.blob_stream(), blob_index)
    }
    pub fn table_stream(&self) -> &[Table] {
        for stream in &self.header.raw_metadata.streams {
            match stream {
                MetadataStream::LogicalMetadataTable(_, _, tables) => return &tables,
                _ => (),
            }
        }
        panic!("No LogicalMetadataTable stream!")
    }
    pub fn tables_rows(&self) -> (&[u32], BitVec64) {
        for stream in &self.header.raw_metadata.streams {
            match stream {
                MetadataStream::LogicalMetadataTable(rows, tables, _) => return (&rows, *tables),
                _ => (),
            }
        }
        panic!("No LogicalMetadataTable stream!")
    }
    pub fn methods(&self) -> &[MethodDef] {
        for table in self.table_stream() {
            match table {
                Table::MethodDefTable(defs) => return &defs,
                _ => (),
            }
        }
        &[]
    }
    pub fn field_len(&self) -> usize {
        for table in self.table_stream() {
            match table {
                //Table::Fields(defs)=>return &defs,
                _ => (),
            }
        }
        0
    }
    pub fn params_len(&self) -> usize {
        for table in self.table_stream() {
            match table {
                Table::Param(params) => return params.len(),
                _ => (),
            }
        }
        0
    }

    pub fn cil_header(&self) -> &CILHeader {
        &self.header
    }
}
#[derive(Debug)]
pub enum AssemblyReadError {
    PEError(PEFileReadError),
}
impl From<PEFileReadError> for AssemblyReadError {
    fn from(value: PEFileReadError) -> Self {
        Self::PEError(value)
    }
}
