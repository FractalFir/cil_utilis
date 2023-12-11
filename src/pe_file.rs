use super::ReadHelper;
use std::io::{Read, SeekFrom, Seek};
#[derive(Debug,Clone, Copy)]
pub struct RVA(pub u64);
#[derive(Debug,Clone, Copy)]
pub struct VA(u64);
#[derive(Debug)]
struct PESection{
    section_type:SectionType,
    virtual_adress:u32,
    characteristics:u32,
    data:Vec<u8>
}
impl PESection{
    fn from_file(file:&mut (impl Read + Seek),section_header:&SectionHeader,header_end:u64)-> Result<Self, PEFileReadError>{
        let section_type = section_header.section_type;
        let mut data = vec![0;section_header.virtual_size as usize];
        //println!("section_header.offset_of_raw_data:{}",section_header.offset_of_raw_data);
        file.seek(SeekFrom::Start(section_header.offset_of_raw_data as u64))?;
        file.read_exact(&mut data)?;
        Ok(Self{ section_type, virtual_adress:section_header.virtual_adress, characteristics:section_header.characteristics, data })
    }
    fn slice_at_rva(&self,rva:RVA,length:u64)->Option<&[u8]>{
        println!("Trying to get slice of length {length} starting at {rva:?}. self.virtual_adress is {virtual_adress}. length:{length}",virtual_adress = self.virtual_adress,length = self.data.len());
        let start = self.virtual_adress as u64;
        let end = self.virtual_adress as u64 + self.data.len() as u64;
        if start <= rva.0 && (rva.0 + length) < end{
            let data_start = rva.0 - self.virtual_adress as u64;
            let data_end = data_start + length;
            Some(&self.data[data_start as usize .. data_end as usize])
        }
        else{
            None
        }
        
    }
}
#[derive(Debug,Clone, Copy)]
enum SectionType{
    Text,
    Reloc,
}
#[derive(Debug)]
struct SectionHeader{
    section_type:SectionType,
    virtual_size:u32,
    virtual_adress:u32,
    size_of_raw_data:u32,
    offset_of_raw_data:u32,
    characteristics:u32,
}
enum Subsystem{
    //None,
    CUI,
    GUI,
}
impl TryFrom<u16> for Subsystem{
    type Error = PEFileReadError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value{
            //0=>Ok(Subsystem::None),
            2=>Ok(Subsystem::CUI),
            3=>Ok(Subsystem::GUI),
            _=>Err(PEFileReadError::WrongSubsystem(value)),
        }
    }
} 
#[derive(Debug)]
pub struct PEFile {
    header:PEHeader,
    sections:Vec<PESection>,
}
#[derive(Debug)]
struct PEFileHeader {
    machine: u16,
    section_count: u16,
    timestamp: u32,
    symbol_table_offset: u32,
    symbol_table_size: u32,
    optional_header_size: u16,
    characteristics: u16,
}
#[derive(Debug)]
pub struct PEHeader {
    file_header: PEFileHeader,
    code_size: u32,
    init_data_size: u32,
    uninit_data_size: u32,
    entrypoint_rva: u32,
    code_rva: u32,
    data_rva: u32,
    nt_header:NTHeader,
    sections:Vec<SectionHeader>,
}
#[derive(Debug)]
pub struct NTHeader {
    image_base: u32,
    section_algiement: u32,
    file_aligement: u32,
    os_major: u16,
    os_minor: u16,
    user_major: u16,
    user_minor: u16,
    subsys_major: u16,
    subsys_minor: u16,
    cil_header:u32,
    cil_header_size:u32,
}
impl TryFrom<&str> for SectionType{
    type Error = PEFileReadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value{
            ".text"=>Ok(Self::Text),
            ".reloc"=>Ok(Self::Reloc),
            _=>Err(PEFileReadError::UnknownSectionName(value.into())),
        }
    }
}
impl SectionHeader{
    fn from_file(file:&mut impl Read) -> Result<Self, PEFileReadError>{
        let mut name = [0;8];
        file.read_exact(&mut name)?;
        let null = name.iter().position(|c| *c == 0).unwrap_or(8);
        let name = std::str::from_utf8(&name[..null]).expect("Not utf8 section name!");
        let section_type = SectionType::try_from(name)?;
        let virtual_size = file.read_u32()?;
        println!("virtual_size:{virtual_size}");
        let virtual_adress = file.read_u32()?;
        println!("virtual_adress:{virtual_adress}");
        let size_of_raw_data = file.read_u32()?;
        println!("size_of_raw_data:{size_of_raw_data}");
        let offset_of_raw_data = file.read_u32()?;
        println!("offset_of_raw_data:{offset_of_raw_data}");
        let reallcations = file.read_u32()?;
        assert_eq!(reallcations,0);
        let line_numbers = file.read_u32()?;
        assert_eq!(line_numbers,0);
        let reallcation_count = file.read_u16()?;
        assert_eq!(reallcation_count,0);
        let line_number_count = file.read_u16()?;
        assert_eq!(line_number_count,0);
        let characteristics = file.read_u32()?;
        Ok(Self{ section_type, virtual_size, virtual_adress, size_of_raw_data, offset_of_raw_data, characteristics })
    }
}
impl NTHeader {
    pub fn cil_header(&self)->RVA{
        RVA(self.cil_header as u64)
    }
    pub fn cil_header_size(&self)->u32{
        self.cil_header_size
    }
    fn from_file(file: &mut impl Read) -> Result<Self, PEFileReadError> {
        let image_base = file.read_u32()?;
        let section_algiement = file.read_u32()?;
        let file_aligement = file.read_u32()?;
        if file_aligement != 512{
            return Err(PEFileReadError::WrongFileAligement);
        }
        println!("file_aligement:{file_aligement}");
        let os_major = file.read_u16()?;
        
        let os_minor = file.read_u16()?;
        
        let user_major = file.read_u16()?;
        let user_minor = file.read_u16()?;
        let subsys_major = file.read_u16()?;
        println!("subsys_major:{subsys_major}");
        let subsys_minor = file.read_u16()?;
        let reserved = file.read_u32()?;
        if reserved != 0{
            return Err(PEFileReadError::WrongMagic);
        }
        let _image_size = file.read_u32()?;
        let _header_size  = file.read_u32()?;
        let checksum =  file.read_u32()?;
        if checksum != 0{
            return Err(PEFileReadError::WrongChecksum);
        }
        let _subsystem =  Subsystem::try_from(file.read_u16()?)?;
        let _dll_flags =  file.read_u16()?;
        let stack_reserve_size = file.read_u32()?;
        if stack_reserve_size != 0x100000{
            return Err(PEFileReadError::WrongStackReserve);
        }
        let stack_commit_size = file.read_u32()?;
        if stack_commit_size != 0x1000{
            return Err(PEFileReadError::WrongStackCommit);
        }
        let heap_reserve_size = file.read_u32()?;
        if heap_reserve_size != 0x100000{
            return Err(PEFileReadError::WrongHeapReserve);
        }
        let heap_commit_size = file.read_u32()?;
        if heap_commit_size != 0x1000{
            return Err(PEFileReadError::WrongHeapCommit);
        }
        let loader_flags = file.read_u32()?;
        if loader_flags != 0{
            return Err(PEFileReadError::WrongLoaderFlags);
        }
        let data_dir_count = file.read_u32()?;
        if data_dir_count != 0x10{
            return Err(PEFileReadError::WrongDirectoryCount);
        }
        let export_table = file.read_u64()?;
        if export_table != 0{
            return Err(PEFileReadError::ExportTablePresent);
        }
        let _import_table = file.read_u64()?;
        let resource_table = file.read_u64()?;
        if resource_table != 0{
            return Err(PEFileReadError::ResourceTablePresent);
        }
        let exception_table = file.read_u64()?;
        if exception_table != 0{
            return Err(PEFileReadError::ExceptionTablePresent);
        }
        let certificate_table = file.read_u64()?;
        if certificate_table != 0{
            return Err(PEFileReadError::CertificateTablePresent);
        }
        let _base_reallocation_table = file.read_u64()?;
        let _debug = file.read_u64()?;
        let _copyright = file.read_u64()?;
        let _global_ptr = file.read_u64()?;
        let _tls_table = file.read_u64()?;
        let _load_config_table = file.read_u64()?;
        let _bound_import = file.read_u64()?;
        let _iat: u64 = file.read_u64()?;
        let delay_import_descr = file.read_u64()?;
        assert_eq!(delay_import_descr,0);
        let cil_header = file.read_u32()?;
        let cil_header_size = file.read_u32()?;
        let reserved = file.read_u64()?;
        assert_eq!(reserved,0);
        Ok(Self{ image_base, section_algiement, file_aligement, os_major, os_minor, user_major, user_minor, subsys_major, subsys_minor, cil_header,cil_header_size })
    }
}
impl PEHeader {
    pub fn nt_header(&self)->&NTHeader{
        &self.nt_header
    }
    fn from_file(file: &mut (impl Read + Seek)) -> Result<(Self,u64), PEFileReadError> {
        let file_header = PEFileHeader::from_file(file)?;
        let magic = file.read_u16()?;
        if magic != 0x10B {
            return Err(PEFileReadError::WrongMagic);
        }
        let lmajor = file.read_u8()?;
        if lmajor != 6 {
            return Err(PEFileReadError::WrongMagic);
        }
        let lminor = file.read_u8()?;
        if lminor != 0 {
            return Err(PEFileReadError::WrongMagic);
        }
        let code_size = file.read_u32()?;
        let init_data_size = file.read_u32()?;
        let uninit_data_size = file.read_u32()?;
        let entrypoint_rva = file.read_u32()?;
        let code_rva = file.read_u32()?;
        let data_rva = file.read_u32()?;
        let nt_header = NTHeader::from_file(file)?;
        let header_end = file.stream_position()?;
        let mut sections = Vec::with_capacity(file_header.section_count as usize);
        for _ in 0..(file_header.section_count){
            sections.push(SectionHeader::from_file(file)?);
        }
        Ok((Self {
            file_header,
            code_size,
            init_data_size,
            uninit_data_size,
            entrypoint_rva,
            code_rva,
            data_rva,
            nt_header,
            sections,
        },header_end))
    }
}
impl PEFileHeader {
    fn from_file(file: &mut (impl Read + Seek)) -> Result<Self, PEFileReadError> {
        let machine = file.read_u16()?;
        let section_count = file.read_u16()?;
        let timestamp = file.read_u32()?;
        let symbol_table_offset = file.read_u32()?;
        let symbol_table_size = file.read_u32()?;
        let optional_header_size = file.read_u16()?;
        if optional_header_size != 224{
            todo!("Can't yet handle PE files with headers with size that differes form 224.");
        }
        println!("optional_header_size:{optional_header_size}");
        let characteristics = file.read_u16()?;
        Ok(Self {
            machine,
            section_count,
            timestamp,
            symbol_table_offset,
            symbol_table_size,
            optional_header_size,
            characteristics,
        })
    }
}
impl PEFile {
    pub fn slice_at_rva(&self,rva:RVA,length:u64)->Option<&[u8]>{
        for section in &self.sections{
            let slice = section.slice_at_rva(rva,length);
            if slice.is_some(){
                return slice;
            }
        }
        None
    }
    pub fn pe_header(&self)->&PEHeader{
        &self.header
    }
    pub fn from_file(file: &mut (impl Read + Seek)) -> Result<Self, PEFileReadError> {
        let mut file_stub = [0; MSDOS_STUB_FIRST.len()];
        // Read first part of the DOS stub
        file.read_exact(&mut file_stub)?;
        if file_stub != MSDOS_STUB_FIRST {
            return Err(PEFileReadError::InavlidDOSStub);
        };
        let lfanew = file.read_u32()?;
        let mut file_stub = [0; MSDOS_STUB_SECOND.len()];
        // Read first part of the DOS stub
        file.read_exact(&mut file_stub)?;
        if file_stub != MSDOS_STUB_SECOND {
            return Err(PEFileReadError::InavlidDOSStub);
        };
        println!("lfanew:{lfanew}");

        let mut data_after_stub = vec![0; (lfanew - 128) as usize];
        file.read_exact(&mut data_after_stub)?;
        let pe = file.read_u32()?;
        eprintln!("{data_after_stub:?} pe:{pe:8x}");
        if pe != 0x00004550 {
            return Err(PEFileReadError::NotPEFile);
        }
        let (header,header_end) = PEHeader::from_file(file)?;
        let sections = header.sections.iter().map(|section_header|PESection::from_file(file, section_header, header_end)).collect::<Result<_,_>>()?;
        Ok(Self{ header, sections })

    }
}
#[derive(Debug)]
pub enum PEFileReadError {
    IOError(std::io::Error),
    InavlidDOSStub,
    NotPEFile,
    WrongMagic,
    WrongChecksum,
    WrongSubsystem(u16),
    WrongFileAligement,
    WrongStackReserve,
    WrongStackCommit,
    WrongHeapReserve,
    WrongHeapCommit,
    WrongLoaderFlags,
    WrongDirectoryCount,
    ExportTablePresent,
    ResourceTablePresent,
    ExceptionTablePresent,
    CertificateTablePresent,
    UnknownSectionName(Box<str>),
}
impl From<std::io::Error> for PEFileReadError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
const MSDOS_STUB_FIRST: &[u8] = &[
    0x4d, 0x5a, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
    0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const MSDOS_STUB_SECOND: &[u8] = &[
    0x0e, 0x1f, 0xba, 0x0e, 0x00, 0xb4, 0x09, 0xcd, 0x21, 0xb8, 0x01, 0x4c, 0xcd, 0x21, 0x54, 0x68,
    0x69, 0x73, 0x20, 0x70, 0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e, 0x6e, 0x6f,
    0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6e, 0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f, 0x53, 0x20,
    0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x0d, 0x0d, 0x0a, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
