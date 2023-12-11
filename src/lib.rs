use std::{
    io::Read,
    path::{Path, PathBuf},
};




pub(crate) mod pe_file;
pub(crate) mod assembly;
fn build_ilasm(path: impl AsRef<Path>, is_dll: bool) -> PathBuf {
    let path = path.as_ref();
    let asm_type = if is_dll { "-dll" } else { "-exe" };
    let extension = if is_dll { "dll" } else { "exe" };
    let mut out_path = path.to_owned();
    out_path.set_extension(extension);
    let target = format!(
        "-output:{out_path}",
        out_path = out_path.clone().to_string_lossy()
    );
    let args: [String; 3] = [asm_type.into(), target, path.to_string_lossy().to_string()];
    let out = std::process::Command::new("ilasm")
        .args(args)
        .output()
        .expect("failed run ilasm process");
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !stdout.contains("\nOperation completed successfully\n") {
        panic!(
            "stdout:{} stderr:{}",
            stdout,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    out_path
}
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
#[cfg(test)]
use {std::fs::File, assembly::EncodedAssembly};
#[test]
fn deser_add_i32() {
    build_ilasm("test/add_i32.il", true);
    let mut file = File::open("test/add_i32.dll").unwrap();
    let asm = EncodedAssembly::from_file(&mut file).unwrap();
}
trait ReadHelper {
    fn read_u8(self) -> std::io::Result<u8>;
    fn read_u16(self) -> std::io::Result<u16>;
    fn read_u32(self) -> std::io::Result<u32>;
    fn read_u64(self) -> std::io::Result<u64>;
}
impl<R: Read> ReadHelper for R {
    fn read_u8(mut self) -> std::io::Result<u8> {
        let mut tmp = [0];
        self.read_exact(&mut tmp)?;
        Ok(tmp[0])
    }
    fn read_u16(mut self) -> std::io::Result<u16> {
        let mut tmp = [0; std::mem::size_of::<u16>()];
        self.read_exact(&mut tmp)?;
        Ok(u16::from_le_bytes(tmp))
    }
    fn read_u32(mut self) -> std::io::Result<u32> {
        let mut tmp = [0; std::mem::size_of::<u32>()];
        self.read_exact(&mut tmp)?;
        Ok(u32::from_le_bytes(tmp))
    }
    fn read_u64(mut self) -> std::io::Result<u64> {
        let mut tmp = [0; std::mem::size_of::<u64>()];
        self.read_exact(&mut tmp)?;
        Ok(u64::from_le_bytes(tmp))
    }
}
