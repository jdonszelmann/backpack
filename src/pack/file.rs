use std::fs::Metadata;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use crate::pack::in_memory::InMemoryFile;
use crate::error::Result;

pub enum RawFile<'f, 'backpack> {
    InMemory(InMemoryFile<'f, 'backpack>),
    Disk {
        name: Option<PathBuf>,
        file: std::fs::File,
    },
}

impl<'f, 'backpack> RawFile<'f, 'backpack> {
    pub fn into_memory(self) -> std::result::Result<InMemoryFile<'f, 'backpack>, RawFile<'f, 'backpack>> {
        match self {
            RawFile::InMemory(f) => Ok(f),
            f @ RawFile::Disk { .. } => Err(f)
        }
    }

    pub fn convert_into_memory(self) -> Result<InMemoryFile<'f, 'backpack>> {
        match self {
            RawFile::InMemory(f) => Ok(f),
            RawFile::Disk { mut file, name, .. } => {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;

                Ok(if let Some(name) = name {
                    InMemoryFile::Named {
                        name,
                        data: Cursor::new(data),
                    }
                } else {
                    data.into()
                })
            }
        }
    }

    pub fn with_name(self, name: impl AsRef<Path>) -> Self {
        match self {
            RawFile::InMemory(f) => RawFile::InMemory(f.with_name(name)),
            RawFile::Disk { file, .. } => {
                RawFile::Disk {
                    name: Some(name.as_ref().to_path_buf()),
                    file,
                }
            }
        }
    }

    pub fn in_memory(name: impl AsRef<Path>) -> Self {
        Self::InMemory(InMemoryFile::new(name))
    }

    pub fn create(s: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::Disk {
            name: Some(s.as_ref().to_path_buf()),
            file: std::fs::File::create(s)?,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::Disk {
            name: Some(path.as_ref().to_path_buf()),
            file: std::fs::File::open(path)?,
        })
    }

    pub fn current_offset(&mut self) -> Result<u64> {
        match self {
            RawFile::Disk { file, .. } => file.seek(SeekFrom::Current(0)).map_err(Into::into),
            RawFile::InMemory(f, ..) => Ok(f.current_offset()),
        }
    }

    pub fn sync_all(&self) -> Result<()> {
        match self {
            RawFile::Disk { file, .. } => file.sync_all().map_err(Into::into),
            RawFile::InMemory(..) => Ok(()),
        }
    }

    pub fn sync_data(&self) -> Result<()> {
        match self {
            RawFile::InMemory(..) => Ok(()),
            RawFile::Disk { file, .. } => file.sync_data().map_err(Into::into),
        }
    }

    pub fn metadata(&self) -> Result<Metadata> {
        match self {
            RawFile::InMemory(..) => todo!(),
            RawFile::Disk { file, .. } => file.metadata().map_err(Into::into),
        }
    }

    pub fn try_clone(&self) -> Result<Self> {
        match self {
            RawFile::InMemory(f) => Ok(RawFile::InMemory(f.try_clone()?)),
            RawFile::Disk { file, name } => {
                let f =  file.try_clone()?;
                Ok(RawFile::Disk {
                    name: name.clone(),
                    file: f,
                })
            }
        }
    }

    pub fn set_len(&mut self, size: u64) -> Result<()> {
        match self {
            RawFile::InMemory(f, ..) => {
                f.set_len(size)?;
                Ok(())
            }
            RawFile::Disk { file, .. } => file.set_len(size).map_err(Into::into),
        }
    }

    pub fn name(&self) -> Option<&Path> {
        match self {
            RawFile::InMemory(f, ..) => f.name(),
            RawFile::Disk { name,  .. } => name.as_deref(),
        }
    }
}

impl Write for RawFile<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            RawFile::Disk { file, .. } => {
                file.write(buf)
            }
            RawFile::InMemory(f, ..) => f.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            RawFile::Disk { file, .. } => {
                file.flush()
            }
            RawFile::InMemory(f, ..) => f.flush()
        }
    }
}

impl Read for RawFile<'_, '_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            RawFile::Disk { file, .. } => {
                file.read(buf)
            }
            RawFile::InMemory(f, ..) => f.read(buf)
        }
    }
}

impl Seek for RawFile<'_, '_> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            RawFile::Disk { file, .. } => file.seek(pos),
            RawFile::InMemory(f, ..) => f.seek(pos),
        }
    }
}

impl From<std::fs::File> for RawFile<'_, '_> {
    fn from(f: std::fs::File) -> Self {
        Self::Disk {
            file: f,
            name: None,
        }
    }
}

impl<'f, 'backpack> From<InMemoryFile<'f, 'backpack>> for RawFile<'f, 'backpack> {
    fn from(f: InMemoryFile<'f, 'backpack>) -> Self {
        RawFile::InMemory(f)
    }
}

impl From<String> for RawFile<'_, '_> {
    fn from(s: String) -> Self {
        RawFile::InMemory(s.into())
    }
}

impl From<&str> for RawFile<'_, '_> {
    fn from(s: &str) -> Self {
        RawFile::InMemory(s.to_string().into())
    }
}

impl From<Vec<u8>> for RawFile<'_, '_> {
    fn from(s: Vec<u8>) -> Self {
        RawFile::InMemory(s.into())
    }
}
