pub(crate) trait Reader {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn is(bytes: &[u8]) -> bool;
}
