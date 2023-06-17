use crate::reader::Reader;
use crate::utils::{crc32, debug_vec, ByteReader};
use crate::vorbis_comment::VorbisComment;
use std::fmt::{Debug, Formatter};

const OGG_SIGNATURE: [u8; 4] = [0x4f, 0x67, 0x67, 0x53];

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct Ogg {
    pages: Vec<OggParsedPage>,
}

impl Ogg {
    fn load_fulldata(segments: &[Segment], start: usize) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        let mut cur = start;
        loop {
            let segment = &segments[cur];
            // not continued from previous segment
            if cur > start && segment.flags != 0x01 {
                break;
            }
            buf.extend_from_slice(&segment.data);
            cur += 1;
            // last segment
            if segment.flags == 0x04 || cur >= segments.len() {
                break;
            }
        }
        buf
    }
}

impl Reader for Ogg {
    #[allow(unused)]
    fn from_bytes(bytes: &[u8]) -> Self {
        if !Ogg::is(bytes) {
            panic!("Invalid ogg audio format.");
        }
        let mut segments: Vec<Segment> = Vec::new();
        let mut reader = ByteReader::new(bytes);
        let mut full_page = 0;
        loop {
            let segment = Segment::new(&mut reader);
            let flags = segment.flags;
            let size = segment.size;
            if segment.flags == 0x00 {
                full_page += 1;
            }
            segments.push(segment);
            if full_page >= 2 || reader.is_end() || flags == 0x4 {
                break;
            }
        }
        let mut pages = Vec::new();
        match Ogg::load_fulldata(&segments, 0) {
            bytes if OpusIdentification::is_opus_format(&bytes) => pages.push(
                OggParsedPage::OpusIdentification(OpusIdentification::new(&bytes)),
            ),
            bytes if VorbisIdentification::is_vorbis_format(&bytes) => pages.push(
                OggParsedPage::VorbisIdentification(VorbisIdentification::new(&bytes)),
            ),
            _ => (),
        };
        if pages.is_empty() {
            return Self { pages };
        }
        if let Some(comments) = Comments::new(&Ogg::load_fulldata(&segments, 1)) {
            pages.push(OggParsedPage::Comments(comments.inner));
        }
        Self { pages }
    }
    fn is(bytes: &[u8]) -> bool {
        bytes[0..4] == OGG_SIGNATURE
    }
}

pub(crate) struct Segment {
    signature: String,
    version: u8,
    /// This is an 8 bit field of flags, which indicates the type of page that follows.
    /// - 0x00 Continuation(Continuation of the previous packet)
    /// - 0x02 BOS(Begin of Stream)
    /// - 0x04 EOS(End of Stream)
    flags: u8,
    granule_position: usize,
    serial_number: u32,
    sequence_number: u32,
    checksum: u32,
    total_segments: u8,
    size: usize,
    data: Vec<u8>,
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Segment")
            .field("signature", &self.signature)
            .field("version", &self.version)
            .field("flags", &self.flags)
            .field("granule_position", &self.granule_position)
            .field("serial_number", &self.serial_number)
            .field(
                "maybe",
                &String::from_utf8_lossy(&self.serial_number.to_be_bytes()).to_string(),
            )
            .field("sequence_number", &self.sequence_number)
            .field("checksum", &self.checksum)
            .field("total_segments", &self.total_segments)
            .field("data", &debug_vec(&self.data))
            .finish()
    }
}

impl Segment {
    pub(crate) fn new(reader: &mut ByteReader) -> Self {
        if reader.peek(4) != OGG_SIGNATURE {
            panic!("Invalid ogg segment format")
        }
        let page_start = reader.offset();
        let signature = reader.read_uft8_string(4);
        let version = reader.read_next_u8();
        let flags = reader.read_next_u8();
        let granule_position = reader.read_next_u64(true) as usize;
        let serial_number = reader.read_next_u32(true);
        let sequence_number = reader.read_next_u32(true);
        let checksum_pos = reader.offset();
        let checksum = reader.read_next_u32(true);
        let total_segments = reader.read_next_u8();
        let segment_size = reader
            .read(total_segments as usize)
            .iter()
            .fold(0, |a, b| a + (*b as usize));
        let data = reader.read(segment_size).to_vec();
        // validate crc32
        {
            let mut view: Vec<u8> = reader.peek_range(page_start, checksum_pos).to_vec();
            view.push(0);
            view.push(total_segments);
            view.extend_from_slice(data.as_slice());
            if crc32(&view) == checksum {
                eprintln!("The packet is corrupted");
            }
        }
        Self {
            signature,
            version,
            flags,
            granule_position,
            serial_number,
            sequence_number,
            checksum,
            total_segments,
            size: segment_size,
            data,
        }
    }
}

#[derive(Debug)]
enum OggParsedPage {
    VorbisIdentification(VorbisIdentification),
    OpusIdentification(OpusIdentification),
    Comments(VorbisComment),
}

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct VorbisIdentification {
    vorbis_version: u32,
    audio_channels: u8,
    audio_sample_rate: u32,
    bitrate_maximum: i32,
    bitrate_nominal: i32,
    bitrate_minimum: i32,
    blocksize_0: u8, // 2 exponent, should less or equal blocksize_1
    blocksize_1: u8, // 2 exponent
    framing_flag: u8,
}

impl VorbisIdentification {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        reader.skip(7);
        let vorbis_version = reader.read_next_u32(false);
        let audio_channels = reader.read_next_u8();
        let audio_sample_rate = reader.read_next_u32(false);
        let bitrate_maximum = reader.read_next_i32(false);
        let bitrate_nominal = reader.read_next_i32(false);
        let bitrate_minimum = reader.read_next_i32(false);
        let blocksize = reader.read_next_u8();
        let framing_flag = reader.read_next_u8() & 0x1;
        Self {
            vorbis_version,
            audio_channels,
            audio_sample_rate,
            bitrate_maximum,
            bitrate_nominal,
            bitrate_minimum,
            blocksize_0: blocksize & 0xf0 >> 4, // front 4 bits
            blocksize_1: blocksize & 0x0f,      // back 4 bits
            framing_flag,
        }
    }
    pub(crate) fn is_vorbis_format(bytes: &[u8]) -> bool {
        bytes[0] == 0x1 && bytes[1..7] == [0x76, 0x6F, 0x72, 0x62, 0x69, 0x73]
    }
}

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct OpusIdentification {
    version: u8,
    channel_output_count: u8,
    pre_skip: u16,
    input_sample_rate: u32,
    output_gain: u16,
    channel_mapping_family: u8,
    channel_mapping_table: Option<Vec<u8>>,
}

impl OpusIdentification {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        reader.skip(8);
        let version = reader.read_next_u8();
        let channel_output_count = reader.read_next_u8();
        let pre_skip = reader.read_next_u16(false);
        let input_sample_rate = reader.read_next_u32(false);
        let output_gain = reader.read_next_u16(false);
        let channel_mapping_family = reader.read_next_u8();
        let channel_mapping_table = if channel_mapping_family == 0x00 {
            None
        } else {
            Some(reader.read(channel_mapping_family as usize).to_vec())
        };
        Self {
            version,
            channel_output_count,
            pre_skip,
            input_sample_rate,
            output_gain,
            channel_mapping_family,
            channel_mapping_table,
        }
    }
    pub(crate) fn is_opus_format(bytes: &[u8]) -> bool {
        bytes[0..8] == [0x4F, 0x70, 0x75, 0x73, 0x54, 0x61, 0x67, 0x73]
    }
}

#[derive(Debug)]
pub(crate) struct Comments {
    inner: VorbisComment,
}

impl Comments {
    pub(crate) fn new(bytes: &[u8]) -> Option<Self> {
        let mut reader = ByteReader::new(bytes);
        match reader.peek(8) {
            head if Comments::is_opus_format(head) => {
                reader.skip(8);
                Some(Self {
                    inner: VorbisComment::with_byte_reader(&mut reader),
                })
            }
            head if Comments::is_vorbis_format(head) => {
                reader.skip(7);
                Some(Self {
                    inner: VorbisComment::with_byte_reader(&mut reader),
                })
            }
            _ => None,
        }
    }
    pub(crate) fn is_opus_format(bytes: &[u8]) -> bool {
        // OpusTags
        bytes[0..8] == [0x4F, 0x70, 0x75, 0x73, 0x54, 0x61, 0x67, 0x73]
    }
    pub(crate) fn is_vorbis_format(bytes: &[u8]) -> bool {
        // Vorbis
        bytes[0] == 0x03 && bytes[1..7] == [0x76, 0x6F, 0x72, 0x62, 0x69, 0x73]
    }
}
