use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::index::{BlockIndexInfo, CheckPoint, Index};
use error::*;
use std::ops::Range;

use silk_proto::*;
use byteorder::{WriteBytesExt, BigEndian};

const BLOCK_FILE_THRESHOLD: u64 = 1024 * 1024 * 64;

pub struct BlockStoreWriter {
    writer: BufWriterWithPos<File>,
    current_suffix: u64,
    current_offset: u64,
    path: Arc<PathBuf>,
}

impl BlockStoreWriter {
    pub fn new(path: Arc<PathBuf>, cp: Option<CheckPoint>) -> Result<BlockStoreWriter> {
        let writer = match cp {
            Some(cp) => BlockStoreWriter {
                writer: new_blk_file(&path, cp.suffix)?,
                current_suffix: cp.suffix,
                current_offset: cp.offset,
                path,
            },
            None => BlockStoreWriter {
                writer: new_blk_file(&path, 0)?,
                current_suffix: 0,
                current_offset: 0,
                path,
            },
        };
        Ok(writer)
    }

    pub fn save(&mut self, block: &Block) -> Result<(u64, Range<u64>)> {
        let bytes = utils::proto::marshal_with_length(block)?;
        let pos = self.writer.pos;
        let len = self.writer.write(&bytes)?;
        self.writer.flush()?;
        self.current_offset += len as u64;

        let info = (self.current_suffix, pos..self.writer.pos);

        if self.current_offset >= BLOCK_FILE_THRESHOLD {
            self.move_next_file()?;
        }
        Ok(info)
    }

    fn move_next_file(&mut self) -> Result<()> {
        self.current_suffix += 1;
        self.writer = new_blk_file(&self.path.clone(), self.current_suffix)?;
        self.current_offset = 0;
        Ok(())
    }
}

fn new_blk_file(path: &Path, suffix: u64) -> Result<BufWriterWithPos<File>> {
    let path = block_path(&path, suffix);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;
    Ok(writer)
}

pub fn block_path(dir: &Path, suffix: u64) -> PathBuf {
    dir.join(format!("blockfile_{:06}", suffix))
}

pub struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
