use std::path::{Path, PathBuf};
use std::io::{Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
use parking_lot::RwLockReadGuard;
use crate::error;
use crate::pack::maybe_ref::MaybeRef;
use crate::pack::slice::PackSlice;

impl InMemoryFile<'_, '_> {
    pub fn new(name: impl AsRef<Path>) -> Self {
        Self::Named {
            name: name.as_ref().to_path_buf(),
            data: Cursor::new(vec![]),
        }
    }

    pub fn unnamed() -> Self {
        Self::Unnamed {
            data: Default::default()
        }
    }

    pub fn name(&self) -> Option<&Path> {
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
            InMemoryFile::Packed { data, ..} => {
                data.resize(size);
                Ok(())
            }
        }
    }

    pub fn get_bytes(&self) -> MaybeRef<[u8]> {
        match self {
            InMemoryFile::Named { data, .. } => data.get_ref().as_slice().into(),
            InMemoryFile::Packed { data, .. } => RwLockReadGuard::map(data.get_bytes().read(), |i| i.as_slice()).into(),
            InMemoryFile::Unnamed { data, .. } => data.get_ref().as_slice().into(),
        }
    }

    pub fn with_name(self, s: impl AsRef<Path>) -> Self {
        match self {
            InMemoryFile::Named { data, .. } => {
                InMemoryFile::Named {
                    name: s.as_ref().to_path_buf(),
                    data
                }
            }
            InMemoryFile::Packed { data, .. } => {
                InMemoryFile::Packed {
                    name: s.as_ref().to_path_buf(),
                    data
                }
            }
            InMemoryFile::Unnamed { data } => {
                InMemoryFile::Named {
                    name: s.as_ref().to_path_buf(),
                    data
                }
            }
        }
    }

    pub fn try_clone(&self) -> error::Result<Self> {
        match self {
            InMemoryFile::Named { .. } => todo!(),
            InMemoryFile::Packed { data, name } => {
                Ok(InMemoryFile::Packed {
                    name: name.clone(),
                    data: data.clone(),
                })
            }
            InMemoryFile::Unnamed { .. } => todo!(),
        }
    }
}

pub enum InMemoryFile<'f, 'backpack> {
    Named {
        name: PathBuf,
        data: Cursor<Vec<u8>>,
    },
    Packed {
        name: PathBuf,
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
