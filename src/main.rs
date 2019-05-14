use byteorder::{ByteOrder, NativeEndian};
use goblin::error::Result;
use goblin::Object;
use memmap::MmapOptions;
use std::env;
use std::fs::{self, OpenOptions};
use std::ops::Range;

#[link_section = "count"]
#[used]
static mut RUN_COUNT: u32 = 0;

fn get_section(obj: &Object, name: &str) -> Option<Range<usize>> {
    if let Object::Elf(elf) = obj {
        for section in &elf.section_headers {
            if &elf.shdr_strtab[section.sh_name] == name {
                return Some(section.file_range());
            }
        }
    }
    None
}

fn main() -> Result<()> {
    let run_count = unsafe { RUN_COUNT };
    println!("Previous run count: {}", run_count);
    let exe = env::current_exe()?;
    let tmp = exe.with_extension("tmp");
    fs::copy(&exe, &tmp)?;

    let file = OpenOptions::new().read(true).write(true).open(&tmp)?;
    let mut buf = unsafe { MmapOptions::new().map_mut(&file) }?;

    let obj = Object::parse(&buf)?;
    if let Some(range) = get_section(&obj, "count") {
        NativeEndian::write_u32(&mut buf[range], run_count + 1);

        let perms = fs::metadata(&exe)?.permissions();
        fs::rename(&tmp, &exe)?;
        fs::set_permissions(&exe, perms)?;
    }

    Ok(())
}
