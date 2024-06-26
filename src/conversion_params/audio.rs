use crate::{logger, media_file::MediaFileTrack};

use core::fmt;
use serde_derive::Deserialize;

use super::params_trait::ConversionParams;

/// Features supported by the audio codec.
enum CodecFeatures {
    /// Compression.
    Compression,
    /// Variable bitrate.
    Vbr,
}

/// Variable bitrate options applicable to the Opus codec.
#[derive(Clone, Deserialize)]
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

#[derive(Clone, Deserialize)]
pub enum VbrOptions {
    Opus(OpusVbrOptions),
    // TODO: validate that this is in the range of 1 to 5.
    AacLibfdk(u8),
}

impl fmt::Display for VbrOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VbrOptions::Opus(opt) => write!(f, "{opt}"),
            VbrOptions::AacLibfdk(opt) => write!(f, "{opt}"),
        }
    }
}

#[derive(Clone, Deserialize)]
pub enum AudioCodec {
    Aac,
    AacLibfdk,
    Ac3,
    Flac,
    Mp3Lame,
    Mp3Shine,
    Opus,
    Vorbis,
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AudioCodec::Aac => write!(f, "aac"),
            AudioCodec::AacLibfdk => write!(f, "libfdk_aac"),
            AudioCodec::Ac3 => write!(f, "ac3"),
            AudioCodec::Flac => write!(f, "flac"),
            AudioCodec::Mp3Lame => write!(f, "libmp3lame"),
            AudioCodec::Mp3Shine => write!(f, "libshine"),
            AudioCodec::Opus => write!(f, "libopus"),
            AudioCodec::Vorbis => write!(f, "libvorbis"),
        }
    }
}

#[derive(Deserialize)]
pub struct AudioConvertParams {
    /// The audio codec to be used for the conversion.
    pub codec: Option<AudioCodec>,
    /// The number of channels to be used for the conversion. If None, the number will be the same as the source.
    pub channels: Option<u32>,
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
    /// Should an adjustment be applied to the audio volume?
    pub volume_adjustment: Option<String>,
    /// Any custom filers to be applied.
    pub filters: Option<String>,
}

impl ConversionParams for AudioConvertParams {
    /// Validate the specified codec parameters.
    fn validate(&self) -> bool {
        let codec = if let Some(c) = &self.codec {
            c
        } else {
            return true;
        };

        let mut valid = true;

        match codec {
            // Validate the Opus audio conversion parameters.
            AudioCodec::Opus => {
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

    fn as_ffmpeg_argument_list(
        &self,
        track: &MediaFileTrack,
        file_in: &str,
        file_out: &str,
    ) -> Option<Vec<String>> {
        if !self.validate() {
            return None;
        }

        let mut args = Vec::with_capacity(100);

        // We always want to overwrite old files, if they exist.
        args.push("-y".to_string());

        // Number of threads to use when encoding.
        if let Some(threads) = self.threads {
            args.push("-threads".to_string());
            args.push(threads.to_string());
        }

        // If we do not have an output codec, no conversion will be performed.
        let codec = if let Some(c) = &self.codec {
            c
        } else {
            args.push("-c:a".to_string());
            args.push("copy".to_string());
            return Some(args);
        };

        // Input file.
        args.push("-i".to_string());
        args.push(file_in.to_string());

        // Volume adjustment, if specified.
        if let Some(vol) = &self.volume_adjustment {
            args.push("-filter:a".to_string());
            args.push(format!("volume={vol}"));
        }

        // Codec type.
        args.push("-c:a".to_string());
        args.push(format!("{codec}"));

        // Bitrate.
        if let Some(bitrate) = self.bitrate {
            args.push("-b:a".to_string());
            args.push(format!("{bitrate}k"));
        }

        // Any filters that may need to be applied.
        if let Some(filters) = &self.filters {
            args.push("-filter:a".to_string());
            args.push(filters.to_string());
        }

        // Variable bitrate (VBR).
        if let Some(vbr) = &self.vbr {
            if codec.supports_feature(CodecFeatures::Vbr) {
                // Opus defaults to a variable bitrate, so this parameter will be ignored
                // if set to on.
                args.push("-vbr".to_string());
                args.push(format!("{vbr}"));
            } else {
                logger::log(format!("The codec {codec} does not support VBR."), true);
            }
        }

        // Compression level.
        if let Some(level) = self.compression_level {
            if codec.supports_feature(CodecFeatures::Compression) {
                args.push("-compression_level".to_string());
                args.push(level.to_string());
            } else {
                logger::log(
                    format!("The codec {codec} does not support compression."),
                    true,
                );
            }
        }

        // The number of audio channels.
        if let Some(channels) = self.channels {
            if track.channels != channels {
                args.push("-ac".to_string());
                args.push(channels.to_string());
            }
        }

        // The output file path should always go last.
        args.push(file_out.to_string());

        Some(args)
    }
}

impl AudioCodec {
    /// Indicates whether the codec supports a specific codec feature.
    ///
    /// # Arguments
    ///
    /// * `feature` - The specific codec feature that we want to know about.
    fn supports_feature(&self, feature: CodecFeatures) -> bool {
        match &feature {
            CodecFeatures::Compression => matches!(
                self,
                AudioCodec::Flac | AudioCodec::Mp3Lame | AudioCodec::Opus
            ),
            CodecFeatures::Vbr => matches!(self, AudioCodec::AacLibfdk | AudioCodec::Opus),
        }
    }
}
