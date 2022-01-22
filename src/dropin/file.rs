use std::io::{Read, Seek, SeekFrom, Write};
use crate::pack;

pub struct File<'f, 'backpack> {
    inner: pack::RawFile<'f, 'backpack>,
}

impl File<'_, '_> {

}

impl Read for File<'_, '_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for File<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for File<'_, '_> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}


