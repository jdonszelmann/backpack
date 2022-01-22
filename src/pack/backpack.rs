use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::RwLock;
use crate::RawFile;
use crate::pack::in_memory::InMemoryFile;
use crate::pack::{error, PACK_HEADER_SIZE, PACK_MAGIC, PACK_VERSION, TOC_SIZE};
use crate::pack::error::PackError;
use crate::pack::error::PackError::{Closed, NoName};
use crate::pack::slice::PackSlice;

pub struct PartialData {
    start: u64,
    end: u64,

    data: Vec<u8>,
}

pub enum BackPack<'f: 'backpack, 'backpack> {
    PartiallyParsed {
        file: Option<RawFile<'f, 'backpack>>,
        total_pack_size: u64,
        max_allowed_in_memory: usize,

        offsets: HashMap<String, (u64, u64)>,
        toc_blocks: Vec<u64>,

        data: Vec<PartialData>,

        last_toc_offset: u64,

        closed: bool,
    },
    Parsed {
        file: Option<RawFile<'f, 'backpack>>,

        offsets: HashMap<String, (u64, u64)>,
        data: HashMap<(u64, u64), RwLock<Vec<u8>>>,

        total_size: AtomicU64,

        closed: bool,
    },
}

impl<'f: 'backpack, 'backpack> BackPack<'f, 'backpack> {
    pub fn open<E: Into<PackError>>(backing: impl TryInto<RawFile<'f, 'backpack>, Error=E>) -> error::Result<Self> {
        if false {
            Self::open_partial(backing)
        } else {
            Self::open_complete(backing)
        }
    }

    pub fn retrieve_slice(&self, s: &PackSlice) -> &RwLock<Vec<u8>> {
        match self {
            BackPack::PartiallyParsed { .. } => todo!(),
            BackPack::Parsed { data, .. } => {
                data.get(&s.identifier())
                    .expect("no such file (only packslices obtained from a pack should be used in as_slice)")
            }
        }
    }

    fn convert_offset(sorted_toc_block_locations: &Vec<u64>, mut offset: u64) -> u64 {
        offset += PACK_HEADER_SIZE;

        for i in sorted_toc_block_locations {
            if offset <= *i {
                offset += TOC_SIZE as u64;
            }
        }

        offset
    }

    fn create_toc(offsets: &HashMap<String, (u64, u64)>) -> error::Result<Vec<Vec<u8>>> {
        if offsets.is_empty() {
            return Ok(Vec::new());
        }

        let mut res = Vec::new();
        let mut curr = Cursor::new(Vec::new());
        let ten_zeros: [u8; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        curr.write(&ten_zeros)?;

        let mut offsets = offsets.into_iter().collect::<Vec<_>>();
        offsets.sort_by_key(|(_, (i, _))| i);

        for (s, (offset, length)) in offsets {
            let entry_size = 8 + 2 + 2 + s.bytes().len();
            let filled = curr.seek(SeekFrom::Current(0))?;

            if filled + entry_size as u64 > TOC_SIZE as u64 {
                curr.seek(SeekFrom::Start(0))?;
                curr.write(&(filled as u16).to_le_bytes())?;
                curr.write(&(TOC_SIZE as u64).to_le_bytes())?;

                let mut buf = curr.into_inner();
                buf.resize(TOC_SIZE as usize, 0);
                res.push(buf);
                curr = Cursor::new(Vec::new());
                curr.write(&ten_zeros)?;
            } else {
                curr.write(&(s.as_bytes().len() as u16).to_le_bytes())?;
                curr.write(s.as_bytes())?;
                curr.write(&offset.to_le_bytes())?;
                curr.write(&length.to_le_bytes())?;
            }
        }

        let filled = curr.seek(SeekFrom::Current(0))?;
        curr.seek(SeekFrom::Start(0))?;
        curr.write(&(filled as u16).to_le_bytes())?;

        let mut buf = curr.into_inner();
        buf.resize(TOC_SIZE as usize, 0);
        res.push(buf);


        Ok(res)
    }

    fn write_headers(f: &'f mut RawFile, size: u64, offsets: &HashMap<String, (u64, u64)>) -> error::Result<()> {
        let toc_blocks = Self::create_toc(offsets)?;

        f.write(PACK_MAGIC)?;
        f.write(&PACK_VERSION.to_le_bytes())?;
        f.write(&size.to_le_bytes())?;
        if toc_blocks.is_empty() {
            f.write(&0u64.to_le_bytes())?;
        } else {
            f.write(&(PACK_HEADER_SIZE as u64).to_le_bytes())?;
        }
        for i in toc_blocks {
            f.write(&i)?;
        }

        Ok(())
    }

    fn parse_toc_block(filled: u16, block: &[u8], offsets: &mut HashMap<String, (u64, u64)>) -> error::Result<()> {
        let mut curr: usize = 0;
        while (curr as u16) < filled {
            let mut strlen_bytes = [0u8; 2];
            strlen_bytes.copy_from_slice(&block[curr..curr+2]);
            curr += 2;
            let strlen = u16::from_le_bytes(strlen_bytes);

            let mut string = Vec::new();
            string.extend_from_slice(&block[curr..curr + strlen as usize]);
            curr += strlen as usize;

            let mut offset_bytes = [0u8; 8];
            offset_bytes.copy_from_slice(&block[curr..curr+8]);
            curr += 8;
            let offset = u64::from_le_bytes(offset_bytes);

            let mut length_bytes = [0u8; 8];
            length_bytes.copy_from_slice(&block[curr..curr+8]);
            curr += 8;
            let length = u64::from_le_bytes(length_bytes);

            let string = String::from_utf8(string)?;
            offsets.insert(string, (offset, length));
        }

        Ok(())
    }

    fn parse_backwards_compatible(file: &mut RawFile, version: u16) -> error::Result<(HashMap<String, (u64, u64)>, Vec<u64>)>{
        match version {
            _ => Err(PackError::Incompatible(version))
        }
    }

    fn parse_headers(file: &mut RawFile) -> error::Result<(HashMap<String, (u64, u64)>, Vec<u64>)> {
        let mut magic_bytes = [0u8; PACK_MAGIC.len()];
        file.read_exact(&mut magic_bytes)?;
        if magic_bytes != PACK_MAGIC {
            return Err(PackError::BadMagic);
        }

        let mut version_bytes = [0u8; 2];
        file.read_exact(&mut version_bytes)?;
        let version = u16::from_le_bytes(version_bytes);
        if version != PACK_VERSION {
            return Self::parse_backwards_compatible(file, version);
        }


        let mut size_bytes = [0u8; 8];
        file.read_exact(&mut size_bytes)?;
        let pack_size = u64::from_le_bytes(size_bytes);

        let mut first_toc_offset_bytes = [0u8; 8];
        file.read_exact(&mut first_toc_offset_bytes)?;
        let first_toc_offset = u64::from_le_bytes(first_toc_offset_bytes);

        assert_eq!(file.current_offset()?, PACK_HEADER_SIZE);

        let mut offsets = HashMap::new();
        let mut toc_blocks = Vec::new();

        let mut next_toc_offset = first_toc_offset;

        while next_toc_offset != 0 {
            toc_blocks.push(next_toc_offset);

            file.seek(SeekFrom::Start(next_toc_offset))?;

            let mut toc_filled_bytes = [0u8; 2];
            file.read_exact(&mut toc_filled_bytes)?;
            let toc_filled = u16::from_le_bytes(toc_filled_bytes);

            let mut next_toc_bytes = [0u8; 8];
            file.read_exact(&mut next_toc_bytes)?;
            next_toc_offset = u64::from_le_bytes(next_toc_bytes);

            let mut toc_block_bytes = [0u8; TOC_SIZE as usize - 10];
            file.read_exact(&mut toc_block_bytes)?;
            Self::parse_toc_block(toc_filled - 10, &toc_block_bytes, &mut offsets)?;
        }

        Ok((offsets, toc_blocks))
    }

    pub fn open_complete<E: Into<PackError>>(file: impl TryInto<RawFile<'f, 'backpack>, Error=E>) -> error::Result<Self> {
        let mut file = file.try_into().map_err(Into::into)?;

        let (offsets, mut toc_blocks) = Self::parse_headers(&mut file)?;
        toc_blocks.sort();

        let mut data = HashMap::new();
        let mut total_size = 0;

        for (_name, (offset, length)) in &offsets {
            let new_offset = Self::convert_offset(&toc_blocks, *offset);
            file.seek(SeekFrom::Start(new_offset))?;

            let mut buf = vec![0; *length as usize];
            file.read_exact(&mut buf)?;

            total_size += buf.len() as u64;
            data.insert((*offset, *length), RwLock::new(buf));
        }


        Ok(Self::Parsed {
            file: Some(file),
            offsets,
            data,

            // not closed
            total_size: AtomicU64::new(total_size),
            closed: false
        })
    }

    pub fn open_partial<E: Into<PackError>>(backing: impl TryInto<RawFile<'f, 'backpack>, Error=E>) -> error::Result<Self> {
        todo!()
    }

    /// Create a new pack in a file. Usually called after File::create().
    /// Existing contents of the file are deleted.
    ///
    /// ```rust
    /// # use backpack::RawFile;
    /// # use backpack::BackPack;
    /// # use backpack::PackError;
    ///
    /// # fn main() -> Result<(), PackError> {
    ///     let file = RawFile::in_memory("test.bp");
    ///     let bp = BackPack::create(file)?;
    ///
    ///     // when done with it
    ///     bp.close();
    /// #   Ok(())
    /// # }
    /// ```
    ///
    pub fn create<E: Into<PackError>>(backing: impl TryInto<RawFile<'f, 'backpack>, Error=E>) -> error::Result<Self> {
        let mut file = backing.try_into().map_err(Into::into)?;
        file.seek(SeekFrom::Start(0))?;

        Ok(Self::Parsed {
            file: Some(file),
            offsets: Default::default(),
            data: HashMap::new(),
            // not closed
            total_size: AtomicU64::new(0),
            closed: false,
        })
    }

    /// Gets the number of bytes used to store files currently.
    /// If packs get really large (contain lots of files) you
    /// might want to flush
    pub fn memory_bytes(&self) -> usize {
        match self {
            BackPack::PartiallyParsed { .. } => todo!(),
            BackPack::Parsed { total_size, .. } => {
                total_size.load(Ordering::SeqCst) as usize
            },
        }
    }

    /// Write all changes since the last flush to the file
    ///
    /// ```rust
    /// # use backpack::RawFile;
    /// # use backpack::pack::BackPack;
    /// # use backpack::pack::PackError;
    /// # use std::io::Read;
    ///
    /// # fn main() -> Result<(), PackError> {
    ///     let file = RawFile::in_memory("test.bp");
    ///     let bp = BackPack::create(file)?;
    ///
    ///     // when done with it
    ///     let mut file = bp.close()?;
    ///     // now there's stuff in the file, which can be read again with `open`
    ///     let bp = BackPack::open(file)?;
    ///
    /// #   Ok(())
    /// # }
    ///
    /// ```
    pub fn flush(&mut self) -> error::Result<()> {
        match self {
            BackPack::PartiallyParsed { .. } => { todo!() }
            BackPack::Parsed {
                file,
                offsets,
                data,
                ..
            } => {
                let mut new_data = Vec::new();
                let mut new_offsets = HashMap::new();
                for (name, (start, length)) in offsets {
                    let contents = data.get(&(*start, *length))
                        .ok_or(PackError::InvalidEntry)?;

                    new_offsets.insert(name.clone(), (new_data.len() as u64, *length));
                    new_data.extend(contents.read().deref());
                }


                if let Some(file) = file {
                    file.seek(SeekFrom::Start(0))?;
                    BackPack::write_headers(file, data.len() as u64, &new_offsets)?;

                    file.write_all(&new_data)?;
                    Ok(())
                } else {
                    Err(Closed)
                }
            }
        }
    }

    pub fn add_file<E: Into<PackError>>(&mut self, f: impl TryInto<RawFile<'f, 'backpack>, Error=E>) -> error::Result<()> {
        let mut f = f.try_into().map_err(Into::<PackError>::into)?;

        match self {
            BackPack::PartiallyParsed { .. } => todo!(),
            BackPack::Parsed {
                offsets,
                data,
                total_size,
                .. } => {

                let mut f_data = Vec::new();
                f.read_to_end(&mut f_data)?;
                let prev = total_size.fetch_add(f_data.len() as u64, Ordering::SeqCst);
                let key = (prev, f_data.len() as u64);

                offsets.insert(f.name().ok_or(NoName)?.to_string(), key);
                data.insert(key, RwLock::new(f_data));
            }
        }

        Ok(())
    }

    pub fn add_file_named<E: Into<PackError>>(&mut self, f: impl TryInto<RawFile<'f, 'backpack>, Error=E>, name: impl AsRef<Path>) -> error::Result<()> {
        self.add_file(f.try_into().map_err(Into::<PackError>::into)?.with_name(name))
    }

    pub fn remove_file(&mut self, name: impl AsRef<Path>) {

    }

    pub fn get_file(&mut self, name: impl AsRef<Path>) -> error::Result<InMemoryFile> {
        match self {
            BackPack::PartiallyParsed { .. } => todo!(),
            BackPack::Parsed { offsets, data, .. } => {
                let name = name.as_ref().to_string_lossy().into_owned();
                let (offset, length) = offsets.get(&name)
                    .ok_or_else(|| PackError::FileNotFound(name.clone()))?;

                Ok(InMemoryFile::Packed {
                    name,
                    data: PackSlice::new(*offset, *length, self)
                })
            }
        }
    }

    /// Close a backpack, saving unsaved additions.
    /// WARNING: dropping a backpack without closing it may panic.
    /// Dropping makes a best-effort attempt to write unsaved changes
    /// in the backpack to disk. Write errors are converted into a panic.
    ///
    /// The only way to get rid of a backpack safely is by closing
    /// it with [`close`] or [`close_drop_unwritten_changes`].
    ///
    /// ```rust
    /// # use backpack::RawFile;
    /// # use backpack::BackPack;
    /// # use backpack::PackError;
    ///
    /// # fn main() -> Result<(), PackError> {
    ///     let file = RawFile::in_memory("test.bp");
    ///     let bp = BackPack::create(file)?;
    ///
    ///     // may panic!
    ///     drop(bp);
    ///
    /// #   Ok(())
    /// # }
    /// ```
    pub fn close(mut self) -> error::Result<RawFile<'f, 'backpack>> {
        self.flush()?;
        self.close_internal()
    }

    /// Same as [`close`]. If changes have occurred since the last flush, they are not saved.
    pub fn close_drop_unwritten_changes(self) -> error::Result<RawFile<'f, 'backpack>> {
        self.close_internal()
    }

    fn close_internal(mut self) -> error::Result<RawFile<'f, 'backpack>> {
        // make sure closing doesn't panic
        match &mut self {
            BackPack::PartiallyParsed { closed, file, .. } |
            BackPack::Parsed { closed, file, .. } => {
                *closed = true;
                let mut file = file.take().ok_or(Closed)?;
                file.seek(SeekFrom::Start(0))?;
                Ok(file)
            }
        }
    }

    fn best_effort_flush(&mut self) {
        if let Err(e) = self.flush() {
            panic!("failed to flush to disk. backpack likely corrupted. {}", e);
        }
    }
}

impl<'f, 'backpack> Drop for BackPack<'f, 'backpack> {
    fn drop(&mut self) {
        match &self {
            BackPack::PartiallyParsed {closed,  .. } |
            BackPack::Parsed { closed, .. } => {
                if !closed {
                    log::warn!("dropping unsaved backpack may panic. Attempting best-effort cleanup.");
                    self.best_effort_flush();
                    log::debug!("successfully flushed")
                }
            }
        }
    }
}
