use std::collections::BTreeMap;
use std::io::{Read, Result as IoResult, Seek, SeekFrom, Write};

pub const BLOCK_SIZE: u64 = 4096;

pub struct SparseDisk {
    pub blocks: BTreeMap<u64, Vec<u8>>,
    pos: u64,
    size: u64,
}

impl SparseDisk {
    pub fn new(size: u64) -> Self {
        Self {
            blocks: BTreeMap::new(),
            pos: 0,
            size,
        }
    }
}

impl Read for SparseDisk {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let block_idx = self.pos / BLOCK_SIZE;
        let offset = (self.pos % BLOCK_SIZE) as usize;

        // ensure the bounds
        let remaining_in_disk = self.size.saturating_sub(self.pos);
        if remaining_in_disk == 0 {
            return Ok(0);
        }

        let len = buf
            .len()
            .min((BLOCK_SIZE as usize) - offset)
            .min(remaining_in_disk as usize);

        if let Some(block) = self.blocks.get(&block_idx) {
            buf[..len].copy_from_slice(&block[offset..offset + len]);
        } else {
            buf[..len].fill(0);
        }

        self.pos += len as u64;
        Ok(len)
    }
}

impl Write for SparseDisk {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let block_idx = self.pos / BLOCK_SIZE;
        let offset = (self.pos % BLOCK_SIZE) as usize;
        let len = buf.len().min((BLOCK_SIZE as usize) - offset);

        let block = self
            .blocks
            .entry(block_idx)
            .or_insert_with(|| vec![0; BLOCK_SIZE as usize]);
        block[offset..offset + len].copy_from_slice(&buf[..len]);

        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

impl Seek for SparseDisk {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        self.pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => (self.size as i64 + p) as u64,
            SeekFrom::Current(p) => (self.pos as i64 + p) as u64,
        };
        Ok(self.pos)
    }
}
