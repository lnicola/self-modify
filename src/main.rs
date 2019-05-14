use byteorder::{NativeEndian, WriteBytesExt};
use goblin::error::Result;
use goblin::Object;
use std::env;
use std::fs;

#[link_section = "count"]
#[no_mangle]
static mut RUN_COUNT: u32 = 0;

fn main() -> Result<()> {
    let run_count = unsafe { RUN_COUNT };
    println!("Previous run count: {}", run_count);
    let exe = env::current_exe()?;
    let mut buffer = fs::read(&exe)?;

    let mut range = None;
    if let Object::Elf(elf) = Object::parse(&buffer)? {
        for section in elf.section_headers {
            let name = &elf.shdr_strtab[section.sh_name];
            if name == "count" {
                range = Some(section.file_range());
                break;
            }
        }
    }

    if let Some(range) = range {
        let tmp = exe.with_extension("tmp");
        let mut buf = &mut buffer[range];
        buf.write_u32::<NativeEndian>(run_count + 1)?;
        fs::write(&tmp, &buffer)?;
        let perms = fs::metadata(&exe)?.permissions();
        fs::rename(&tmp, &exe)?;
        fs::set_permissions(&exe, perms)?;
    }

    Ok(())
}
