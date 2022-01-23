mod slice;
mod backpack;
mod file;
mod in_memory;
mod maybe_ref;

pub use file::RawFile;
pub use in_memory::InMemoryFile;
pub use crate::pack::backpack::BackPack;
pub use crate::error::{PackError, Result};

pub const fn parse_int(s: &'static [u8]) -> u16 {
    match s {
        [] => 0,
        [rest@.., h] => {
            let rest_int = parse_int(rest);

            let val = match h {
                b'0' => 0,
                b'1' => 1,
                b'2' => 2,
                b'3' => 3,
                b'4' => 4,
                b'5' => 5,
                b'6' => 6,
                b'7' => 7,
                b'8' => 8,
                b'9' => 9,
                _ => panic!("couldn't parse to integer; unknown digit in string"),
            };

            rest_int * 10 + val
        }
    }
}

pub const PACK_MAGIC: &[u8] = b"BACKPACK";
pub const PACK_VERSION: u16 = parse_int(env!("CARGO_PKG_VERSION_MAJOR").as_bytes());
pub const TOC_SIZE: u16 = 4096;
pub const PACK_HEADER_SIZE: u64 = 26;

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use crate::RawFile;
    use crate::pack::in_memory::InMemoryFile;
    use crate::pack::PACK_VERSION;
    use crate::pack::backpack::BackPack;
    use crate::error::PackError;

    #[test]
    pub fn test_version() {
        assert_eq!(PACK_VERSION, env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap())
    }

    #[test]
    pub fn parse_int() {
        assert_eq!(super::parse_int(b"10"), 10);
        assert_eq!(super::parse_int(b"15"), 15);
        assert_eq!(super::parse_int(b"150"), 150);
    }

    #[test]
    fn test_reopen() -> Result<(), PackError> {
        let file = RawFile::in_memory("test.bp");
        let bp = BackPack::create(file)?;

        // when done with it
        let file = bp.close()?;
        // now there's stuff in the file, which can be read again with `open`
        let bp = BackPack::open(file)?;
        bp.close()?;

      Ok(())
    }

    #[test]
    fn test_add_file() -> Result<(), PackError> {
        let file = RawFile::in_memory("test.bp");
        let mut bp = BackPack::create(file)?;

        let f: InMemoryFile = "test".into();
        bp.add_file(f.with_name("test.txt"))?;
        // when done with it
        let file = bp.close()?;

        // now there's stuff in the file, which can be read again with `open`
        let mut bp = BackPack::open(file)?;
        let f = bp.get_file("test.txt")?;

        assert_eq!(&*f.get_bytes(), b"test");

        bp.close()?;

        Ok(())
    }
}
