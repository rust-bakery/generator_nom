use flavors;
use flavors::parser::{complete_tag,CodecId,FrameType,SoundFormat,SoundRate,SoundSize,SoundType,TagHeader};
use nom::be_u32;

named!(pub flv_tag<flavors::parser::Tag>,
  terminated!(complete_tag, be_u32)
);

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Tag {
  pub header: TagHeader,
  pub data:   TagData,
}

impl Tag {
  pub fn new(t: flavors::parser::Tag) -> Tag {
    let flavors::parser::Tag { header, data } = t;
    let d = match data {
      flavors::parser::TagData::Script   => TagData::Script,
      flavors::parser::TagData::Audio(a) => {
        TagData::Audio(
          AudioData {
            sound_format: a.sound_format,
            sound_rate:   a.sound_rate,
            sound_size:   a.sound_size,
            sound_type:   a.sound_type,
            sound_data:   Vec::from(a.sound_data),
          }
        )
      },
      flavors::parser::TagData::Video(v) => {
        TagData::Video(
          VideoData {
            frame_type: v.frame_type,
            codec_id:   v.codec_id,
            video_data: Vec::from(v.video_data),
          }
        )
      },
    };

    Tag {
      header: header,
      data:   d,
    }
  }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum TagData {
  Audio(AudioData),
  Video(VideoData),
  Script,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct AudioData {
  pub sound_format: SoundFormat,
  pub sound_rate:   SoundRate,
  pub sound_size:   SoundSize,
  pub sound_type:   SoundType,
  pub sound_data:   Vec<u8>,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct VideoData {
  pub frame_type: FrameType,
  pub codec_id:   CodecId,
  pub video_data: Vec<u8>,
}

