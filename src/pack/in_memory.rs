use std::path::Path;
use std::io::{Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
use crate::pack::error;
use crate::pack::guarded_bytes::GuardedBytes;
use crate::pack::slice::PackSlice;

impl InMemoryFile<'_, '_> {
    pub fn new(name: impl AsRef<Path>) -> Self {
        Self::Named {
            name: name.as_ref().to_string_lossy().into_owned(),
            data: Cursor::new(vec![]),
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            InMemoryFile::Named { name, .. } |
            InMemoryFile::Packed { name, .. } => Some(name),
            InMemoryFile::Unnamed { .. } => None
        }
    }

    pub fn current_offset(&self) -> u64 {
        match self {
            InMemoryFile::Named { data, .. } |
            InMemoryFile::Unnamed { data } => {
                data.position()
            }
            InMemoryFile::Packed { data, .. } => {
                data.position()
            }
        }
    }

    pub fn set_len(&mut self, size: u64) -> error::Result<()> {
        match self {
            InMemoryFile::Named { data, .. } |
            InMemoryFile::Unnamed { data } => {
                data.get_mut().resize(size as usize, 0);
                Ok(())
            }
            InMemoryFile::Packed { data, name } => {
                data.resize(size, name)
            }
        }
    }

    pub fn as_slice(&self) -> GuardedBytes {
        match self {
            InMemoryFile::Named { data, .. } => data.get_ref().as_slice().into(),
            InMemoryFile::Packed { data, .. } => data.as_slice(),
            InMemoryFile::Unnamed { data, .. } => data.get_ref().as_slice().into()
        }
    }

    pub fn with_name(self, s: impl AsRef<Path>) -> Self {
        match self {
            InMemoryFile::Named { data, .. } => {
                InMemoryFile::Named {
                    name: s.as_ref().to_string_lossy().into_owned(),
                    data
                }
            }
            InMemoryFile::Packed { data, .. } => {
                InMemoryFile::Packed {
                    name: s.as_ref().to_string_lossy().into_owned(),
                    data
                }
            }
            InMemoryFile::Unnamed { data } => {
                InMemoryFile::Named {
                    name: s.as_ref().to_string_lossy().into_owned(),
                    data
                }
            }
        }
    }
}

pub enum InMemoryFile<'f, 'backpack> {
    Named {
        name: String,
        data: Cursor<Vec<u8>>,
    },
    Packed {
        name: String,
        data: PackSlice<'f, 'backpack>,
    },
    Unnamed {
        data: Cursor<Vec<u8>>,
    },
}

impl From<String> for InMemoryFile<'_, '_> {
    fn from(s: String) -> Self {
        Self::Unnamed {
            data: Cursor::new(s.into_bytes())
        }
    }
}

impl From<Vec<u8>> for InMemoryFile<'_, '_> {
    fn from(data: Vec<u8>) -> Self {
        Self::Unnamed {
            data: Cursor::new(data)
        }
    }
}

impl From<&str> for InMemoryFile<'_, '_> {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl Read for InMemoryFile<'_, '_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InMemoryFile::Named { data, .. } |
            InMemoryFile::Unnamed { data } => data.read(buf),
            InMemoryFile::Packed { data, .. } => data.read(buf),
        }
    }
}

impl Write for InMemoryFile<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            InMemoryFile::Named { data, .. } |
            InMemoryFile::Unnamed { data } => data.write(buf),
            InMemoryFile::Packed { .. } => {
                Err(std::io::Error::new(ErrorKind::PermissionDenied, "can't write to file backed by backpack"))
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for InMemoryFile<'_, '_> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            InMemoryFile::Named { data, .. } |
            InMemoryFile::Unnamed { data } => data.seek(pos),
            InMemoryFile::Packed { data, .. } => data.seek(pos),
        }
    }
}
