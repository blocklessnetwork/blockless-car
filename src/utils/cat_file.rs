use std::{io::{Write, self}, collections::VecDeque};

use cid::Cid;

use crate::{reader::CarReader, Ipld, unixfs::UnixFs, error::CarError};


/// write ipld to output
/// `file_cid` is the file cid to write
/// `output` is the out the file write to.
pub fn ipld_write(
    reader: &mut impl CarReader, 
    vecq: &mut VecDeque<Cid>, 
    output: &mut impl Write
) -> Result<(), CarError> 
{
    while let Some(file_cid) = vecq.pop_front() {
        let file_ipld: Ipld = reader.ipld(&file_cid).unwrap();
        
        match file_ipld {
            Ipld::Bytes(b) => {
                output.write_all(&b[..])?;
            }
            m @ Ipld::Map(_) => {
                let unix_fs: Result<UnixFs, CarError> = (file_cid, m).try_into();
                let ufs = unix_fs?;
                for cufs in ufs.children().iter() {
                    vecq.push_back(cufs.cid.unwrap());
                }
            }
            _ => {}
        };
    }
    Ok(())
}

pub fn cat_ipld(
    reader: &mut impl CarReader, 
    file_cid: Cid
) -> Result<(), CarError>
{
    let mut stdout = io::stdout();
    let mut vecq = VecDeque::new();
    vecq.push_back(file_cid);
    ipld_write(reader, &mut vecq, &mut stdout)
}