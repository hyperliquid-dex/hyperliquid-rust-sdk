// For synchronous signing.
// Needed to duplicate our own copy because it wasn't possible to import from ethers-signers.
use alloy_primitives::B256;
use k256::{
    ecdsa::signature::digest::{
        generic_array::GenericArray,
        {Digest, FixedOutput, FixedOutputReset, HashMarker, Output, OutputSizeUser, Reset, Update},
    },
    sha2,
};

pub(crate) type Sha256Proxy = ProxyDigest<sha2::Sha256>;

#[derive(Clone)]
pub(crate) enum ProxyDigest<D: Digest> {
    Proxy(Output<D>),
    Digest(D),
}

impl<D: Digest + Clone> From<B256> for ProxyDigest<D>
where
    GenericArray<u8, <D as OutputSizeUser>::OutputSize>: Copy,
{
    fn from(src: B256) -> Self {
        ProxyDigest::Proxy(*GenericArray::from_slice(src.as_slice()))
    }
}

impl<D: Digest> Default for ProxyDigest<D> {
    fn default() -> Self {
        ProxyDigest::Digest(D::new())
    }
}

impl<D: Digest> Update for ProxyDigest<D> {
    // we update only if we are digest
    fn update(&mut self, data: &[u8]) {
        match self {
            ProxyDigest::Digest(ref mut d) => {
                d.update(data);
            }
            ProxyDigest::Proxy(..) => {
                unreachable!("can not update if we are proxy");
            }
        }
    }
}

impl<D: Digest> HashMarker for ProxyDigest<D> {}

impl<D: Digest> Reset for ProxyDigest<D> {
    // make new one
    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl<D: Digest> OutputSizeUser for ProxyDigest<D> {
    // we default to the output of the original digest
    type OutputSize = <D as OutputSizeUser>::OutputSize;
}

impl<D: Digest> FixedOutput for ProxyDigest<D> {
    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        match self {
            ProxyDigest::Digest(d) => {
                *out = d.finalize();
            }
            ProxyDigest::Proxy(p) => {
                *out = p;
            }
        }
    }
}

impl<D: Digest> FixedOutputReset for ProxyDigest<D> {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        let s = std::mem::take(self);
        Digest::finalize_into(s, out)
    }
}
