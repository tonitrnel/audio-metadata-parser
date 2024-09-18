mod flac;
mod id3;
mod ogg;
mod reader;
mod utils;
mod vorbis_comment;
mod base64;

pub use id3::*;
pub use flac::{Flac, FlacParsedBlock};
pub use ogg::*;
pub use reader::Reader;

mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test() {
        let path = Path::new("data/2i301c2x2v1v.mp3");
        let bytes = fs::read(path).unwrap();
        match bytes {
            bytes if Flac::is(&bytes) => {
                println!("{:#?}", Flac::from_bytes(&bytes))
            }
            bytes if Ogg::is(&bytes) => {
                println!("{:#?}", Ogg::from_bytes(&bytes));
            }
            bytes if ID3::is(&bytes) => {
                println!("{:#?}", ID3::from_bytes(&bytes));
            }
            _ => {
                println!("Not supported audio format")
            }
        }
    }
}
