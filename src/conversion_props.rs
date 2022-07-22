use core::fmt;

use crate::media_file::Codec;

pub enum OpusVbrOptions {
    Off,
    On,
    Constrained,
}

impl fmt::Display for OpusVbrOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpusVbrOptions::Off => write!(f, "off"),
            OpusVbrOptions::On => write!(f, "on"),
            OpusVbrOptions::Constrained => write!(f, "constrained"),
        }
    }
}

pub enum VbrOptions {
    Opus(OpusVbrOptions),
    // TODO: validate that this is in the range of 1 to 5.
    AacLibfdk(u8),
}

impl fmt::Display for VbrOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VbrOptions::Opus(opt) => write!(f, "{}", opt),
            VbrOptions::AacLibfdk(opt) => write!(f, "{}", opt),
        }
    }
}

#[derive(Clone)]
pub enum AudioCodec {
    Aac,
    AacLibfdk,
    Ac3,
    Flac,
    Mp2,
    Mp3Lame,
    Mp3Shine,
    Opus,
    Vorbis,
    WavPack,
}

impl Into<Codec> for AudioCodec {
    fn into(self) -> Codec {
        match self {
            AudioCodec::Aac => Codec::Aac,
            AudioCodec::AacLibfdk => Codec::Aac,
            AudioCodec::Ac3 => Codec::Ac3,
            AudioCodec::Flac => Codec::Flac,
            AudioCodec::Mp2 => Codec::Mp2,
            AudioCodec::Mp3Lame => Codec::Mp3,
            AudioCodec::Mp3Shine => Codec::Mp3,
            AudioCodec::Opus => Codec::Opus,
            AudioCodec::Vorbis => Codec::Opus,
            AudioCodec::WavPack => Codec::WavPack4,
        }
    }
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AudioCodec::Aac => write!(f, "aac"),
            AudioCodec::AacLibfdk => write!(f, "libfdk_aac"),
            AudioCodec::Ac3 => write!(f, "ac3"),
            AudioCodec::Flac => write!(f, "flac"),
            AudioCodec::Mp2 => write!(f, "libtwolame"),
            AudioCodec::Mp3Lame => write!(f, "libmp3lame"),
            AudioCodec::Mp3Shine => write!(f, "libshine"),
            AudioCodec::Opus => write!(f, "libopus"),
            AudioCodec::Vorbis => write!(f, "libvorbis"),
            AudioCodec::WavPack => write!(f, "wavpack"),
        }
    }
}

pub struct AudioProperties {
    /// The audio codec to be used for the conversion.
    pub codec: Option<AudioCodec>,
    /// The number of channels to be used for the conversion. If None, the number will be the same as the source.
    pub channels: Option<u16>,
    /// The bitrate for the audio conversion, in kilobits per second.
    pub bitrate: Option<u32>,
    /// The variable bitrate (VBR) options to be used.
    /// Only applicable when the codec is [`AudioCodec::Opus`] or [`AudioCodec::AacLibfdk`].
    pub vbr: Option<VbrOptions>,
    /// The encoding algorithm complexity.
    /// Only applicable when the codec is [`AudioCodec::Opus`].
    pub compression_level: Option<u8>,
    /// The number of threads to be used for the conversion.
    pub threads: Option<u8>,
}

impl AudioProperties {
    pub fn validate(&self) -> bool {
        let codec = if let Some(c) = &self.codec {
            c
        } else {
            return true;
        };

        match codec {
            AudioCodec::Opus => {
                let mut valid = true;

                if let Some(v) = &self.vbr {
                    valid &= matches!(v, VbrOptions::Opus(_));
                }

                if let Some(c) = &self.compression_level {
                    valid &= *c <= 10;
                }

                valid
            }
            _ => {
                todo!("Handle these cases.");
            }
        }
    }

    pub fn as_ffmpeg_argument_list(&self, file_in: &str, file_out: &str) -> Option<Vec<String>> {
        if !self.validate() {
            return None;
        }

        let mut args = Vec::with_capacity(100);

        // If we do not have an output codec, no conversion will be performed.
        let codec = if let Some(c) = &self.codec {
            c
        } else {
            return Some(vec![String::from("-c:a"), String::from("copy")]);
        };

        if let Some(threads) = self.threads {
            args.push(String::from("-threads"));
            args.push(threads.to_string());
        }

        args.push(String::from("-i"));
        args.push(file_in.to_string());

        args.push(String::from("-c:a"));
        args.push(format!("{}", codec));

        if let Some(bitrate) = self.bitrate {
            args.push(String::from("-b:a"));
            args.push(format!("{}k", bitrate));
        }

        if let Some(level) = self.compression_level {
            args.push(String::from("-compression_level"));
            args.push(level.to_string());
        }

        if let Some(vbr) = &self.vbr {
            args.push(String::from("-vbr"));
            args.push(format!("{}", vbr));
        }

        // The output file path should always go last.
        args.push(file_out.to_string());

        Some(args)
    }
}
