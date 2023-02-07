#![allow(unused)]
use cid::Cid;

use crate::{error::CarError, header::CarHeader, reader::CarReader, section::Section, Ipld};
use std::{io::{Read, Seek}, collections::HashMap};

use super::read_section;

pub(crate) struct CarReaderV1<R> {
    inner: R,
    sections: HashMap<Cid, Section>,
    header: CarHeader,
}

impl<R> CarReaderV1<R>
where
    R: Read + Seek,
{
    pub fn new(mut inner: R) -> Result<Self, CarError> {
        let header = CarHeader::read_header(&mut inner)?;
        let mut sections = HashMap::new();
        while let Some(section) = read_section(&mut inner)? {
            sections.insert(section.cid(), section);
        }
        Ok(Self { inner, header, sections })
    }
}

impl<R> CarReader for CarReaderV1<R>
where
    R: Read + Seek,
{
    #[inline(always)]
    fn header(&self) -> &CarHeader {
        &self.header
    }

    #[inline(always)]
    fn sections(&self) -> Vec<Section> {
        self.sections.values().map(Section::clone).collect()
    }

    #[inline]
    fn read_section_data(&mut self, cid: &Cid) -> Result<Vec<u8>, CarError> {
        let s = self.sections.get(cid).ok_or(CarError::InvalidSection("cid not exist".into()))?;
        s.read_data(&mut self.inner)
    }

    #[inline]
    fn ipld(&mut self, cid: &Cid) -> Result<Ipld, CarError> {
        let s = self.sections.get_mut(cid).ok_or(CarError::InvalidSection("cid not exist".into()))?;
        s.ipld(&mut self.inner)
    }

}

#[cfg(test)]
mod test {

    use crate::unixfs::UnixFs;
    use super::*;

    #[test]
    fn test_read() {
        let file = std::path::Path::new("test");
        let file = file.join("carv1-basic.car");
        let file = std::fs::File::open(file).unwrap();
        let mut reader = CarReaderV1::new(file).unwrap();
        let roots = reader.header().roots();
        assert_eq!(reader.sections().len(), 6);
        for r in roots.iter() {
            let s_ipld = reader.ipld(r).unwrap();
            let unix_fs: Result<UnixFs, CarError> = s_ipld.try_into();
            assert!(unix_fs.is_ok());
            unix_fs.map(|fs| {
                assert_eq!(fs.children.len(), 3);
            });
        }
    }
}
