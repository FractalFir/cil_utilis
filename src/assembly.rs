use std::io::{Read, Seek};

use crate::pe_file::{PEFile, PEFileReadError,RVA};
#[derive(Debug)]
enum MetadataStream{
    LogicalMetadataTable,
    Strings(Box<[u8]>),
    US(Box<[u8]>),
    Blob(Box<[u8]>),
    GUID(Box<[u128]>),
}
impl MetadataStream{
    fn logical_metadata_table(stream:&[u8])->Self{
        Self::LogicalMetadataTable
    }
    fn string_stream(stream:&[u8])->Self{
        Self::Strings(stream.to_owned().into())
    }
    fn us_stream(stream:&[u8])->Self{
        Self::US(stream.to_owned().into())
    }
    fn blob_stream(stream:&[u8])->Self{
        Self::Blob(stream.to_owned().into())
    }
    fn guid_stream(stream:&[u8])->Self{
        let guid_count = stream.len()/std::mem::size_of::<u128>();
        let mut guids = Vec::with_capacity(guid_count);
        for index in 0.. guid_count{
            guids.push(u128_from_slice_at(stream, index*16));
        }
        Self::GUID(guids.to_owned().into())
    }
    fn from_slice(metadata:&[u8],curr_offset:&mut usize)->Self{
        let stream_offset = u32_from_slice_at(metadata,*curr_offset);
        
        let stream_size = u32_from_slice_at(metadata,*curr_offset + 4);
        println!("stream_offset:{stream_offset} stream_size:{stream_size}");
        *curr_offset += 8;
        let name_start = *curr_offset;
        while metadata[*curr_offset] != 0{
            *curr_offset+=1;
        }
        let name_end = *curr_offset;
        if *curr_offset | 0b11 != 0{
            *curr_offset &= !0b11; 
            *curr_offset += 0b100;
        }
       
        let name = std::str::from_utf8(&metadata[name_start..name_end]).expect("Not utf8 stream name!");
        println!("name:{name} curr_offset:{curr_offset}");
        let stream = &metadata[(stream_offset as usize)..(stream_offset as usize + stream_size as usize)];
       
        match name{
            "#~"=>Self::logical_metadata_table(stream),
            "#Strings"=>Self::string_stream(stream),
            "#US"=>Self::us_stream(stream),
            "#Blob"=>Self::blob_stream(stream),
            "#GUID"=>Self::guid_stream(stream),
            _=>todo!("Unknown stream \"{name}\"."),
        }
    }
} 
#[derive(Debug)]
struct RawMetadata{
    major:u32,
    minor:u32,
    version:Box<str>,
    streams:Vec<MetadataStream>,
}
impl RawMetadata{
    fn from_slice(metadata:&[u8])->Self{
        let magic = u32_from_slice_at(metadata,0);
        assert_eq!(magic,0x424A5342);
        let major = u32_from_slice_at(metadata,4);
        let minor = u32_from_slice_at(metadata,6);
        let reserved = u32_from_slice_at(metadata,8);
        assert_eq!(reserved,0);
        let length = u32_from_slice_at(metadata,12);
        let rounded_length = (length / 4 + (length % 4))*4;
        let version = &metadata[16..(16 + rounded_length as usize)];
        let null = version.iter().position(|c| *c == 0).unwrap_or(8);
        let version = std::str::from_utf8(&version[..null]).expect("Not utf8 version name!");
        println!("version:{version}");
        let flags = u16_from_slice_at(metadata,16 + rounded_length as usize);
        assert_eq!(flags,0);
        let stream_count = u16_from_slice_at(metadata,16 + rounded_length as usize + 2 );
        let mut curr_offset = 16 + rounded_length as usize + 4;
        println!("curr_offset:{curr_offset}");
        let mut streams = Vec::with_capacity(curr_offset);
        for _ in 0..stream_count{
            streams.push(MetadataStream::from_slice(metadata, &mut curr_offset));
        }
        RawMetadata{ major, minor, version:version.into(), streams }
    }
}
#[derive(Debug)]
pub struct CILHeader{
    flags:u32,
    entrypoint:u32,
    native_resource_rva:u32,
    native_resource_size:u32,
    strong_name_rva:u32,
    strong_name_size:u32,
    vtable_fixups:u64,
    raw_metadata:RawMetadata,
}
fn u128_from_slice_at(slice:&[u8],offset:usize)->u128{
    let value:[u8;16]  = slice[offset..(offset+16)].try_into().unwrap();
    u128::from_le_bytes(value)
}
fn u64_from_slice_at(slice:&[u8],offset:usize)->u64{
    let value:[u8;8]  = slice[offset..(offset+8)].try_into().unwrap();
    u64::from_le_bytes(value)
}
fn u32_from_slice_at(slice:&[u8],offset:usize)->u32{
    let value:[u8;4]  = slice[offset..(offset+4)].try_into().unwrap();
    u32::from_le_bytes(value)
}
fn u16_from_slice_at(slice:&[u8],offset:usize)->u16{
    let value:[u8;2]  = slice[offset..(offset+2)].try_into().unwrap();
    u16::from_le_bytes(value)
}
impl CILHeader{
    fn read_from_pe(pe_file:&PEFile)->Self{
        let cli_header_rva: RVA = pe_file.pe_header().nt_header().cil_header();
        let cil_header_size =pe_file.pe_header().nt_header().cil_header_size();
        //let cli_header_rva = RVA(8192);
        let cli_header = pe_file.slice_at_rva(cli_header_rva, cil_header_size as u64).expect("CIL header has invalid RVA");
        println!("cli_header:{cli_header:?}");
        let cb = u32_from_slice_at(cli_header,0);
        let major_runtime = u16_from_slice_at(cli_header,4);
        let minor_runtime = u16_from_slice_at(cli_header,6);
        let metadata_rva = u32_from_slice_at(cli_header,8);
        //println!("metadata_rva:{metadata_rva}");
        let metadata_size = u32_from_slice_at(cli_header,8+4);
        let flags = u32_from_slice_at(cli_header,16);
        let metadata = pe_file.slice_at_rva(RVA(metadata_rva as u64), metadata_size as u64).expect("Metadata header has invalid RVA");
        let entrypoint = u32_from_slice_at(cli_header,20);
        let native_resource_rva = u32_from_slice_at(cli_header,24);
        let native_resource_size = u32_from_slice_at(cli_header,28);
        let strong_name_rva = u32_from_slice_at(cli_header,24);
        let strong_name_size = u32_from_slice_at(cli_header,28);
        let code_manager_table = u64_from_slice_at(cli_header,40);
        assert_eq!(code_manager_table,0);
        let vtable_fixups = u64_from_slice_at(cli_header,48);
        let export_address_table_jumps = u64_from_slice_at(cli_header,56);
        assert_eq!(export_address_table_jumps,0);
        let managed_native_header = u64_from_slice_at(cli_header,56);
        assert_eq!(managed_native_header,0);
        let raw_metadata = RawMetadata::from_slice(metadata);
        Self{
        flags,
        entrypoint,
        native_resource_rva,
        native_resource_size,
        strong_name_rva,
        strong_name_size,
        vtable_fixups,
            raw_metadata
        }
    }
}
pub struct EncodedAssembly{
    pe_file:PEFile,
    header:CILHeader,
}
impl EncodedAssembly {
    pub fn from_file(file: &mut (impl Read + Seek)) -> Result<Self, AssemblyReadError>{
        let pe_file = PEFile::from_file(file)?;
        let header = CILHeader::read_from_pe(&pe_file);
        Ok(Self{ pe_file, header })
    }
}
#[derive(Debug)]
pub enum AssemblyReadError{
    PEError(PEFileReadError),
}
impl From<PEFileReadError> for AssemblyReadError{
    fn from(value: PEFileReadError) -> Self {
       Self::PEError(value)
    }
}