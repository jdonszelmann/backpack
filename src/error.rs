use std::convert::Infallible;
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;
use std::string::FromUtf8Error;
use thiserror::Error;
use crate::pack::PACK_MAGIC;

#[derive(Error, Debug)]
pub enum PackError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("backpack magic number, expected {:?}", PACK_MAGIC)]
    BadMagic,

    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),

    #[error("version {0} found in backpack could not be read by this version of the backpack library")]
    Incompatible(u16),

    #[error("attempted operation on closed file")]
    Closed,

    #[error("file {0:?} not present in backpack")]
    FileNotFound(PathBuf),

    #[error("attempted to pack a file which has no name")]
    NoName,

    #[error("invalid table of content entry in the backpack. this is a bug")]
    InvalidEntry,
}

impl Into<std::io::Error> for PackError {
    fn into(self) -> IoError {
        match self {
            PackError::Io(e) => e,
            e@PackError::BadMagic |
            e@PackError::Utf8Error(_) => IoError::new(ErrorKind::InvalidData, e),
            e@PackError::Incompatible(_) => IoError::new(ErrorKind::Unsupported, e),
            e@PackError::Closed => IoError::new(ErrorKind::Other, e),
            e@PackError::FileNotFound(_) => IoError::new(ErrorKind::NotFound, e),
            e@PackError::NoName |
            e@PackError::InvalidEntry => IoError::new(ErrorKind::Other, e)
        }
    }
}

impl From<Infallible> for PackError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

pub type Result<T> = std::result::Result<T, PackError>;
