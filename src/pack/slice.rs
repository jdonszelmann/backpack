use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};
use crate::{BackPack, Result};
use crate::pack::guarded_bytes::GuardedBytes;

pub struct PackSlice<'f, 'backpack> {
    start: u64,
    end: u64,

    pos: u64,

    pub(crate) pack: &'backpack BackPack<'f, 'backpack>
}

impl<'f, 'backpack> PackSlice<'f, 'backpack> {
    pub fn new(start: u64, end: u64, pack: &'backpack BackPack<'f, 'backpack>) -> Self {
        Self {
            start,
            end,
            pos: 0,
            pack
        }
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    pub fn identifier(&self) -> (u64, u64) {
        (self.start, self.end)
    }

    pub fn as_slice(&self) -> GuardedBytes {
        self.pack.retrieve_slice(self).into()
    }

    pub fn resize(&mut self, size: u64, name: &str) -> Result<()> {

        todo!()
        // self.pack.resize_slice(self, size, name)
    }
}

impl Read for PackSlice<'_, '_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let g = self.pack.retrieve_slice(self)
            .read();

        let mut c = Cursor::new(g.deref());
        c.set_position(self.pos);
        let res = c.read(buf)?;
        self.pos = c.position();

        Ok(res)
    }
}

impl Write for PackSlice<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut g = self.pack.retrieve_slice(self)
            .write();

        let mut c = Cursor::new(g.deref_mut());
        c.set_position(self.pos);
        let res = c.write(buf)?;
        self.pos = c.position();

        Ok(res)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for PackSlice<'_, '_> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let mut g = self.pack.retrieve_slice(self)
            .read();

        let mut c = Cursor::new(g.deref());
        c.set_position(self.pos);
        let res = c.seek(pos)?;
        self.pos = c.position();

        Ok(res)
    }
}