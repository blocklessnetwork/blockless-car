use libipld::Cid;


#[derive(Debug, Clone, PartialEq, Eq, Default, libipld::DagCbor)]
pub struct CarHeaderV1 {
    #[ipld]
    pub roots: Vec<Cid>,
    #[ipld]
    pub version: u64,
}

impl CarHeaderV1 {
    pub fn new(roots: Vec<Cid>, version: u64) -> Self {
        Self { roots, version }
    }
}

impl From<Vec<Cid>> for CarHeaderV1 {

    fn from(roots: Vec<Cid>) -> Self {
        CarHeaderV1 {
            roots,
            version: 1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_head_v1() {
        
    }

}
