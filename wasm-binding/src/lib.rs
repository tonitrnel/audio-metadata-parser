use std::collections::HashMap;
use serde::Serialize;
use wasm_bindgen::{JsCast};
use wasm_bindgen::prelude::wasm_bindgen;
use audio_metadata_parser::{Flac, Ogg, ID3, Reader, ID3ParsedTag, FlacParsedBlock, OggParsedPage};


#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_TYPE_CONST: &'static str = r#"
export interface Image {
  data: ArrayBuffer;
  description: string;
  mime: string;
}

export interface Metadata {
  title?: string;
  artist?: string;
  album?: string;
  conver?: Image;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Metadata")]
    pub type TMetadata;
}

#[derive(Serialize)]
pub struct Image {
    data: Vec<u8>,
    description: String,
    mime: String,
}

#[derive(Serialize)]
pub struct Metadata {
    /// 音频标题
    title: Option<String>,
    /// 艺术家
    artist: Option<String>,
    /// 专辑
    album: Option<String>,
    /// 封面
    cover: Option<Image>,
}

#[wasm_bindgen]
pub fn parse(bytes: Vec<u8>) -> Option<TMetadata> {
    let metadata = match &bytes {
        bytes if Flac::is(bytes) => {
            let parser = Flac::from_bytes(bytes);
            let fields = parser.blocks().iter().fold(HashMap::new(), |mut map, it| {
                match it {
                    FlacParsedBlock::Comment(comment) => {
                        map.extend(comment.comments().iter().map(|(a, b)| (a.as_str(), b.as_str())));
                        map
                    }
                    _ => map
                }
            });
            let cover = parser.blocks().iter().find_map(|it| {
                match it {
                    FlacParsedBlock::Picture(picture) => Some(Image {
                        data: Vec::from(picture.data()),
                        description: String::from(picture.description()),
                        mime: String::from(picture.mime()),
                    }),
                    _ => None
                }
            });
            Some(Metadata {
                title: fields.get("TITLE").map(|&it| String::from(it)),
                artist: fields.get("ARTIST").map(|&it| String::from(it)),
                album: fields.get("ALBUM").map(|&it| String::from(it)),
                cover,
            })
        }
        bytes if Ogg::is(bytes) => {
            let parser = Ogg::from_bytes(&bytes);
            let fields = parser.pages().iter().fold(HashMap::new(), |mut map, it| {
                match it {
                    OggParsedPage::Comments(comment) => {
                        map.extend(comment.comments().iter().map(|(a, b)| (a.as_str(), b.as_str())));
                        map
                    }
                    _ => map
                }
            });
            Some(Metadata {
                title: fields.get("TITLE").map(|&it| String::from(it)),
                artist: fields.get("ARTIST").map(|&it| String::from(it)),
                album: fields.get("ALBUM").map(|&it| String::from(it)),
                cover: None,
            })
        }
        bytes if ID3::is(bytes) => {
            let parser = ID3::from_bytes(bytes);
            let fields = parser.tags().iter().filter_map(|it| match it {
                ID3ParsedTag::Text((key, value)) => Some((key.as_str(), value.as_str())),
                _ => None
            }).collect::<HashMap<&str, &str>>();
            let cover = parser.tags().iter().find_map(|it| match it {
                ID3ParsedTag::AttachedPicture(picture) => Some(Image {
                    data: Vec::from(picture.data()),
                    description: String::from(picture.description()),
                    mime: String::from(picture.mime()),
                }),
                _ => None
            });
            Some(Metadata {
                title: fields.get("TIT2").map(|&it| String::from(it)),
                artist: fields.get("TPE1").map(|&it| String::from(it)),
                album: fields.get("TALB").map(|&it| String::from(it)),
                cover,
            })
        }
        _ => {
            None
        }
    };
    metadata.map(|it| serde_wasm_bindgen::to_value(&it).unwrap().unchecked_into::<TMetadata>())
}