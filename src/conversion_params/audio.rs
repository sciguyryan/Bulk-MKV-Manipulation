use core::fmt;

use super::params::ConversionParams;

/// Variable bitrate options applicable to the Opus codec.
#[allow(unused)]
pub enum OpusVbrOptions {
    /// Disable variable bitrate, enabling constant bitrate.
    Off,
    /// Enable variable bitrate, on by default.
    On,
    /// Enable constrained variable bitrate.
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

#[allow(unused)]
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

#[allow(unused)]
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

pub struct AudioParams {
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

impl ConversionParams for AudioParams {
    /// Validate the specified codec parameters.
    fn validate(&self) -> bool {
        let codec = if let Some(c) = &self.codec {
            c
        } else {
            return true;
        };

        match codec {
            // Validate the Opus audio conversion parameters.
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

    fn as_ffmpeg_argument_list(&self, file_in: &str, file_out: &str) -> Option<Vec<String>> {
        if !self.validate() {
            return None;
        }

        let mut args = Vec::with_capacity(100);

        // If we do not have an output codec, no conversion will be performed.
        let codec = if let Some(c) = &self.codec {
            c
        } else {
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            return Some(args);
        };

        // Number of threads to use when encoding.
        if let Some(threads) = self.threads {
            args.push("-threads".to_string());
            args.push(threads.to_string());
        }

        // Input file.
        args.push("-i".to_string());
        args.push(file_in.to_string());

        // Codec type.
        args.push("-c:a".to_string());
        args.push(format!("{}", codec));

        // Bitrate.
        if let Some(bitrate) = self.bitrate {
            args.push("-b:a".to_string());
            args.push(format!("{}k", bitrate));
        }

        // Compression level. Only applied to audio tracks.
        if let Some(level) = self.compression_level {
            args.push("-compression_level".to_string());
            args.push(level.to_string());
        }

        if let Some(vbr) = &self.vbr {
            // Opus defaults to a variable bitrate, so this parameter will be ignored
            // if set to on.
            args.push("-vbr".to_string());
            args.push(format!("{}", vbr));
        }

        // The output file path should always go last.
        args.push(file_out.to_string());

        Some(args)
    }
}
