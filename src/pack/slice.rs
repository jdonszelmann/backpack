use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};
use parking_lot::RwLock;
use crate::BackPack;

pub struct PackSlice<'f, 'backpack> {
    start: u64,
    end: u64,

    pos: u64,

    pub(crate) pack: &'f BackPack<'f, 'backpack>
}

impl Clone for PackSlice<'_, '_> {
    fn clone(&self) -> Self {
        PackSlice {
            start: self.start,
            end: self.end,
            pos: self.pos,
            pack: self.pack
        }
    }
}

impl<'f, 'backpack> PackSlice<'f, 'backpack> {
    pub fn new(start: u64, end: u64, pack: &'f BackPack<'f, 'backpack>) -> Self {
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

    pub fn get_bytes(&self) -> &RwLock<Vec<u8>> {
        self.pack.retrieve_slice(self)
    }

    pub fn resize(&mut self, size: u64) {
        let v = self.pack.retrieve_slice(self);
        v.write().resize(size as usize, 0);
    }
}

impl Read for PackSlice<'_, '_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let g = self.pack.retrieve_slice(self).read();

        let mut c = Cursor::new(g.deref());
        c.set_position(self.pos);
        let res = c.read(buf)?;
        self.pos = c.position();

        Ok(res)
    }
}

impl Write for PackSlice<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut g = self.pack
            .retrieve_slice(self)
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
        let g = self.pack
            .retrieve_slice(self)
            .read();


        let mut c = Cursor::new(g.deref());
        c.set_position(self.pos);
        let res = c.seek(pos)?;

        self.pos = c.position();

        Ok(res)
    }
}