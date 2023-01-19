use cid::Cid;

#[derive(Debug, Clone, PartialEq, Eq, Default, ipld::DagCbor)]
pub struct CarHeaderV1 {
    #[ipld]
    pub roots: Vec<Cid>,
    #[ipld]
    pub version: u64,
}

impl CarHeaderV1 {
    pub fn new(roots: Vec<Cid>) -> Self {
        Self {
            roots,
            version: 1,
        }
    }
}

impl From<Vec<Cid>> for CarHeaderV1 {

    fn from(roots: Vec<Cid>) -> Self {
        Self::new(roots)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ipld::codec::{Decode, Encode};
    use ipld_cbor::DagCborCodec;
    use cid::Cid;
    use cid::multihash::{MultihashDigest, Code::Blake2b256};

    #[test]
    fn test_head_v1() {
        let digest = Blake2b256.digest(b"test");
        let cid = Cid::new_v1(DagCborCodec.into(), digest);
        let mut bytes = Vec::new();
        let header = CarHeaderV1::new(vec![cid]);
        header.encode(DagCborCodec, &mut bytes).unwrap();
        assert_eq!(
            CarHeaderV1::decode(DagCborCodec, &mut std::io::Cursor::new(&bytes)).unwrap(),
            header
        );
    }

}
