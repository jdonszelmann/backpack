use std::fs::Metadata;
use std::io::{Read, Seek, SeekFrom, Write, Result, Error as IoError};
use std::path::Path;
use crate::dropin::config::OpenPolicy;
use crate::{InMemoryFile, pack};
use crate::dropin::scope::{get_backpack, with_config};

pub struct File<'f, 'backpack> {
    inner: pack::RawFile<'f, 'backpack>,
}

impl<'f> File<'f, 'static> {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        with_config(|config| {
            match config.open_policy {
                OpenPolicy::OnDisk => Ok(Self {
                    inner: std::fs::File::create(path)?.into()
                }),
                OpenPolicy::InMemory => Ok(Self {
                    inner: InMemoryFile::new(path).into()
                }),
                OpenPolicy::ThreadLocalBackpack => {
                    let bp = get_backpack();
                    Ok(Self {
                        inner: bp.add_empty_file(path)
                            .map_err(Into::<IoError>::into)?
                            .into()
                    })
                }
            }
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        with_config(|config| {
            match config.open_policy {
                OpenPolicy::OnDisk => Ok(Self {
                    inner: std::fs::File::open(path)?.into()
                }),
                OpenPolicy::InMemory => Ok(Self {
                    inner: InMemoryFile::new(path).into()
                }),
                OpenPolicy::ThreadLocalBackpack => {
                    let bp = get_backpack();
                    Ok(Self {
                        inner: bp.get_file(path)
                            .map_err(Into::<IoError>::into)?
                            .into()
                    })
                }
            }
        })
    }

    pub fn sync_all(&self) -> Result<()> {
        self.inner.sync_all().map_err(Into::<IoError>::into)
    }

    pub fn sync_data(&self) -> Result<()> {
        self.inner.sync_data().map_err(Into::<IoError>::into)
    }

    pub fn set_len(&mut self, size: u64) -> Result<()> {
        self.inner.set_len(size).map_err(Into::<IoError>::into)
    }

    pub fn metadata(&self) -> Result<Metadata> {
        self.inner.metadata().map_err(Into::<IoError>::into)
    }

    pub fn try_clone(&self) -> Result<Self> {
        let ri = self.inner.try_clone().map_err(Into::<IoError>::into)?;
        Ok(Self {
            inner: ri,
        })
    }
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


