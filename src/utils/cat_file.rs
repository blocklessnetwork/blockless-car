use std::io::{Write, self};

use cid::Cid;

use crate::{reader::CarReader, Ipld, unixfs::UnixFs, error::CarError};


/// write ipld to output
/// `file_cid` is the file cid to write
/// `output` is the out the file write to.
pub fn write_ipld(
    reader: &mut impl CarReader, 
    file_cid: Cid, 
    output: &mut impl Write
) -> Result<(), CarError> 
{
    let file_ipld: Ipld = reader.ipld(&file_cid).unwrap();
    match file_ipld {
        Ipld::Bytes(b) => {
            output.write_all(&b[..])?;
        }
        m @ Ipld::Map(_) => {
            let unix_fs: Result<UnixFs, CarError> = (file_cid, m).try_into();
            let ufs = unix_fs?;
            for cufs in ufs.children().iter(){
                write_ipld(reader, cufs.cid().unwrap(), output)?;
            }
        }
        _ => {}
    };
    Ok(())
}

pub fn cat_ipld(
    reader: &mut impl CarReader, 
    file_cid: Cid
) -> Result<(), CarError>
{
    let mut stdout = io::stdout();
    write_ipld(reader, file_cid, &mut stdout)
}