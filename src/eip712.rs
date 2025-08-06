use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{keccak256, B256},
};

pub(crate) trait Eip712 {
    fn domain(&self) -> Eip712Domain;
    fn struct_hash(&self) -> B256;

    fn eip712_signing_hash(&self) -> B256 {
        let mut digest_input = [0u8; 2 + 32 + 32];
        digest_input[0] = 0x19;
        digest_input[1] = 0x01;
        digest_input[2..34].copy_from_slice(&self.domain().hash_struct()[..]);
        digest_input[34..66].copy_from_slice(&self.struct_hash()[..]);
        keccak256(digest_input)
    }
}
