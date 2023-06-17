mod flac;
mod id3;
mod ogg;
mod reader;
mod utils;
mod vorbis_comment;

mod tests {
    use crate::{flac::Flac, id3::ID3, ogg::Ogg, reader::Reader};
    use std::fs;
    use std::path::Path;

    #[test]
    fn test() {
        let path = Path::new("../../storage/f6ca0264-fd78-4576-b170-9988f7737221.ogg");
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
