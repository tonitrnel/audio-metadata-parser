use crate::utils::ByteReader;
use std::fmt::Debug;

#[allow(unused)]
#[derive(Debug)]
pub struct VorbisComment {
    pub(crate) vendor: String,
    pub(crate) comments: Vec<(String, String)>,
}

impl VorbisComment {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        VorbisComment::with_byte_reader(&mut reader)
    }
    pub(crate) fn with_byte_reader(reader: &mut ByteReader) -> Self {
        let vendor = {
            // little-endian
            let len = reader.read_next_u32(false) as usize;
            reader.read_uft8_string(len)
        };
        // comment list length
        let length = reader.read_next_u32(false) as usize;
        let mut comments: Vec<(String, String)> = Vec::with_capacity(length);
        loop {
            // comment value len
            let len = reader.read_next_u32(false) as usize;
            let str = reader.read_uft8_string(len);
            let part = str.splitn(2, '=').collect::<Vec<_>>();
            comments.push((part[0].to_string(), part[1].to_string()));
            if reader.is_end() || comments.len() == length {
                break;
            }
        }
        Self { vendor, comments }
    }
    pub fn vendor(&self)->&str{
        &self.vendor
    }
    pub fn comments(&self) -> &[(String, String)]{
        &self.comments
    }
}
