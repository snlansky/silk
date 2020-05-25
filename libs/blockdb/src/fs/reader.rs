use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt};
use error::*;
use silk_proto::*;
use crate::fs::index::FilePointer;
use crate::fs::writer::block_path;
use std::collections::btree_map::BTreeMap;

pub struct BlockStoreReader {
    path: Arc<PathBuf>,
    readers: RefCell<BTreeMap<u64, BufReaderWithPos<File>>>,
}

impl BlockStoreReader {
    pub fn new(path: Arc<PathBuf>) -> BlockStoreReader {
        BlockStoreReader {
            path,
            readers: RefCell::new(BTreeMap::new()),
        }
    }

    fn read_and<F, R>(&self, fp: FilePointer, f: F) -> Result<R>
    where
        F: FnOnce(io::Take<&mut BufReaderWithPos<File>>) -> Result<R>,
    {
        let mut readers = self.readers.borrow_mut();

        if !readers.contains_key(&fp.suffix) {
            let reader = BufReaderWithPos::new(File::open(block_path(&self.path, fp.suffix))?)?;
            readers.insert(fp.suffix, reader);
        }
        let reader = readers.get_mut(&fp.suffix).unwrap();
        reader.seek(SeekFrom::Start(fp.pos))?;
        let blk_reader = reader.take(fp.len);
        f(blk_reader)
    }

    pub fn read_blk(&self, fp: FilePointer) -> Result<Block> {
        self.read_and(fp, |mut blk_reader| {
            let len = blk_reader.read_u32::<BigEndian>()?;
            let mut bytes = Vec::with_capacity(len as usize);
            blk_reader.read_to_end(&mut bytes)?;
            let blk = utils::proto::unmarshal::<Block>(&bytes)?;
            Ok(blk)
        })
    }
}

impl Clone for BlockStoreReader {
    fn clone(&self) -> BlockStoreReader {
        BlockStoreReader {
            path: Arc::clone(&self.path),
            readers: RefCell::new(BTreeMap::new()),
        }
    }
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}
