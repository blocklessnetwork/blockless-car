#![allow(unused)]
use std::io::{Seek, Read, SeekFrom};

use cid::Cid;
use ipld::Block;

use crate::{error::CarError, Ipld};


#[derive(Debug, Clone)]
pub struct Section {
    cid: Cid, 
    pos: u64,
    len: usize,
}

impl Section {
    pub fn new(cid: Cid, pos: u64, len: usize) -> Self {
        Self {
            cid,
            pos,
            len,
        }
    }

    #[inline]
    pub fn read_data<T>(&self, mut seeker: T) -> Result<Vec<u8>, CarError>
    where
        T: Seek + Read
    {
        seeker.seek(SeekFrom::Start(self.pos))?;
        let mut buf = vec![0u8; self.len];
        seeker.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    pub fn ipld<T>(&mut self, mut seeker: T) -> Result<Ipld, CarError> 
    where
        T: Seek + Read
    {
        let data = self.read_data(&mut seeker)?;
        let block = Block::<ipld::DefaultParams>::new(self.cid, data).unwrap();
        block.ipld()
            .map_err(|e| CarError::Parsing(e.to_string()))
    }
    
    #[inline(always)]
    pub fn cid(&self) -> Cid {
        self.cid
    }

    #[inline(always)]
    pub fn pos(&self) -> u64 {
        self.pos
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }
}