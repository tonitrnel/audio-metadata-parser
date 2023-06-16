use crate::reader::Reader;
use crate::utils::{ByteReader, CharacterEncoding};
use std::fmt::{Debug, Formatter};
use std::io::SeekFrom;

const ID3_SIGNATURE: [u8; 3] = [0x49, 0x44, 0x33];

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct ID3 {
    version: u8,
    revision: u8,
    flags: u8,
    frames_size: usize,
    tags: Vec<ID3ParsedTag>,
}

impl Reader for ID3 {
    #[allow(unused)]
    fn from_bytes(bytes: &[u8]) -> Self {
        if !ID3::is(bytes) {
            panic!("Invalid id3 audio format")
        }
        let mut bytes = ByteReader::with_offset(bytes, 3);
        let mut tags: Vec<ID3ParsedTag> = Vec::new();
        let version = bytes.read_next_u8();
        let revision = bytes.read_next_u8();
        let flags = bytes.read_next_u8();
        // total of 28 bits
        let frames_size = {
            let buf = bytes.read(4);
            (buf[3] as u32)
                | ((buf[2] as u32) << 7)
                | ((buf[1] as u32) << 14)
                | ((buf[0] as u32) << 21)
        } as usize;
        if flags == 0x40 {
            let extended_header_size = bytes.read_next_u32(true);
            bytes.skip(extended_header_size as usize);
        }
        let mut parsed_bytes = 0usize;
        // println!("{}", frames_size);
        loop {
            if bytes.peek(4) == [0x00, 0x00, 0x00, 0x00] {
                break;
            }
            if parsed_bytes == frames_size || bytes.is_end() {
                break;
            }
            let frame = Frame::new(&mut bytes);
            let size = frame.size;
            match frame {
                frame if Text::is_text_information(&frame) => {
                    tags.push(ID3ParsedTag::Text(Text::new(frame).0))
                }
                frame if Comments::is_comments(&frame) => {
                    tags.push(ID3ParsedTag::Comments(Comments::new(frame)))
                }
                frame if AttachedPicture::is_attached_picture(&frame) => {
                    tags.push(ID3ParsedTag::AttachedPicture(AttachedPicture::new(frame)))
                }
                _ => tags.push(ID3ParsedTag::Raw(frame)),
            }
            parsed_bytes += 10 + size; // header + payload
        }
        bytes.seek(SeekFrom::End(128));
        if bytes.read(3) == [0x54, 0x41, 0x47] && bytes.peek(1) != [0x00] {
            let reserved = bytes.peek_range(bytes.len() - 3, bytes.len() - 2)[0] == 0x00;
            tags.push(ID3ParsedTag::V1Tag(V1Tag {
                title: bytes
                    .read_uft8_string(30)
                    .trim_end_matches('\u{0000}')
                    .to_string(),
                artist: bytes
                    .read_uft8_string(30)
                    .trim_end_matches('\u{0000}')
                    .to_string(),
                album: bytes
                    .read_uft8_string(30)
                    .trim_end_matches('\u{0000}')
                    .to_string(),
                year: bytes.read_uft8_string(4).parse::<u32>().unwrap_or(0),
                comment: if reserved {
                    bytes
                        .read_uft8_string(28)
                        .trim_end_matches('\u{0000}')
                        .to_string()
                } else {
                    bytes
                        .read_uft8_string(30)
                        .trim_end_matches('\u{0000}')
                        .to_string()
                },
                track: if reserved {
                    bytes.skip(1);
                    Some(bytes.read_next_u8())
                } else {
                    None
                },
                genre: bytes.read_next_u8(),
            }))
        }
        Self {
            version,
            revision,
            flags,
            frames_size,
            tags,
        }
    }
    fn is(bytes: &[u8]) -> bool {
        bytes[0..3] == ID3_SIGNATURE
    }
}

#[derive(Debug)]
enum ID3ParsedTag {
    // id3 v1
    V1Tag(V1Tag),
    // id3 v2
    Text((String, String)),
    Comments(Comments),
    AttachedPicture(AttachedPicture),
    Raw(Frame),
}

#[allow(unused)]
/// ID3 V2
pub(crate) struct Frame {
    id: String,
    /// Data size
    size: usize,
    /// Flags
    ///
    /// - First:
    ///     - bit 7: tag alter preservation
    ///     - bit 6: filter alter preservation
    ///     - bit 5: readonly
    /// - Second:
    ///     - bit 7: compression
    ///     - bit 6: encryption
    ///     - bit 5: grouping identity
    flags: (u8, u8),
    /// Data encode
    ///
    /// - 0x00 ISO-8859-1
    /// - 0x01 UTF-16LE
    /// - 0x02 UTF-16BE
    /// - 0x03 UTF-8
    encoding: Option<FrameEncoding>,
    data: Vec<u8>,
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("flags", &self.flags)
            .field("encoding", &self.encoding)
            .field("data", &format!("[..]({})", self.data.len()))
            .finish()
    }
}

impl Frame {
    pub(crate) fn new(bytes: &mut ByteReader) -> Self {
        let id = bytes.read_uft8_string(4);
        // size excluded 1 byte of encoding
        let size = bytes.read_next_u32(true) as usize - 1;
        let flags = (bytes.read_next_u8(), bytes.read_next_u8());
        let encoding = match bytes.read_next_u8() {
            0x00 => Some(FrameEncoding::Iso8859_1),
            0x01 => Some(FrameEncoding::Utf16le),
            0x02 => Some(FrameEncoding::Utf16be),
            0x03 => Some(FrameEncoding::Utf8),
            _ => None,
        };
        Self {
            id,
            size,
            flags,
            encoding,
            data: bytes.read(size).to_vec(),
        }
    }
}

#[derive(Debug, Default)]
enum FrameEncoding {
    #[default]
    Iso8859_1 = 0x00,
    Utf16le = 0x01,
    Utf16be = 0x02,
    Utf8 = 0x03,
}

#[derive(Debug)]
#[allow(unused)]
/// ID3 V1
pub(crate) struct V1Tag {
    title: String,
    artist: String,
    album: String,
    year: u32,
    comment: String,
    track: Option<u8>,
    genre: u8,
}

/// Attached Picture
///
/// Structure
/// ```text
/// | ...M | 0x00 | ...D | T | 0x00 | ...B
/// ```
/// - M: Mime type string, Unknown length.
/// - D: Description string, Unknown length.
/// - T: Image type, 1 Byte.
/// - B: Image binary data.
pub(crate) struct AttachedPicture {
    r#type: u8,
    mime: String,
    description: String,
    data: Vec<u8>,
}
impl Debug for AttachedPicture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachedPicture")
            .field("type", &self.r#type)
            .field("mime", &self.mime)
            .field("description", &self.description)
            .field("data", &format!("[..]({})", self.data.len()))
            .finish()
    }
}
impl AttachedPicture {
    pub(crate) fn new(frame: Frame) -> Self {
        let mut bytes = ByteReader::new(&frame.data);
        let mime = bytes.read_uft8_variant_string();
        let r#type = bytes.read_next_u8();
        let description = bytes.read_uft8_variant_string();
        Self {
            r#type,
            mime,
            description,
            data: bytes.read_remaining().to_vec(),
        }
    }
    pub(crate) fn is_attached_picture(frame: &Frame) -> bool {
        frame.id == "APIC"
    }
}

#[derive(Debug)]
pub(crate) struct Text((String, String));

impl Text {
    pub(crate) fn new(frame: Frame) -> Self {
        let mut bytes = ByteReader::new(&frame.data);
        Self((
            frame.id,
            match frame.encoding.unwrap_or_default() {
                FrameEncoding::Utf16le => bytes.read_string(frame.size, CharacterEncoding::Utf16le),
                FrameEncoding::Utf16be => bytes.read_string(frame.size, CharacterEncoding::Utf16be),
                FrameEncoding::Iso8859_1 | FrameEncoding::Utf8 => bytes.read_uft8_variant_string(),
            },
        ))
    }
    pub(crate) fn is_text_information(frame: &Frame) -> bool {
        frame.id.starts_with('T') && frame.id != "TXXX"
    }
}

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct Comments {
    language: String,
    excerpt: String,
    content: String,
}

impl Comments {
    pub(crate) fn new(frame: Frame) -> Self {
        let mut bytes = ByteReader::new(&frame.data);
        let language = bytes.read_uft8_string(3);
        let encoding = frame.encoding.unwrap_or_default();
        let mut read_next_string = || match encoding {
            FrameEncoding::Utf16le => bytes.read_variant_string(CharacterEncoding::Utf16le),
            FrameEncoding::Utf16be => bytes.read_variant_string(CharacterEncoding::Utf16be),
            FrameEncoding::Iso8859_1 | FrameEncoding::Utf8 => bytes.read_uft8_variant_string(),
        };
        let excerpt = read_next_string();
        let content = read_next_string();
        Self {
            language,
            excerpt,
            content,
        }
    }
    pub(crate) fn is_comments(frame: &Frame) -> bool {
        frame.id == "COMM"
    }
}
