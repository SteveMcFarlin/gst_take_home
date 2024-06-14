#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
pub mod av1enc;
pub mod nvh264enc;
pub mod nvh265enc;
pub mod x264enc;
pub mod x265enc;

use crate::traits::{Pipeline, PipelineSink, PipelineSrc};
use gstreamer as gst;
use gstreamer::prelude::*;
use serde::{Deserialize, Serialize};

use x264enc::{Config as X264Config, Encoder as X264Encoder};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub variant: VideoEncoder,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: "encoder".to_string(),
            variant: VideoEncoder::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum VideoEncoder {
    X264(x264enc::Config),
    NVH264,
    X265(x265enc::Config),
    NVH265,
    AV1(av1enc::Config),
}

impl Default for VideoEncoder {
    fn default() -> Self {
        VideoEncoder::X264(x264enc::Config::default())
    }
}

#[derive(Debug)]
pub enum Encoder {
    X264(x264enc::Encoder),
    NVH264,
    X265(x265enc::Encoder),
    NVH265,
    AV1(av1enc::Encoder),
}

impl Encoder {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        match config.variant {
            VideoEncoder::X264(_) => Ok(Encoder::X264(x264enc::Encoder::new(config)?)),
            VideoEncoder::NVH264 => Ok(Encoder::NVH264),
            VideoEncoder::X265(_) => Ok(Encoder::X265(x265enc::Encoder::new(config)?)),
            VideoEncoder::NVH265 => Ok(Encoder::NVH265),
            VideoEncoder::AV1(_) => Ok(Encoder::AV1(av1enc::Encoder::new(config)?)),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Encoder::X264(_) => "x264enc".to_string(),
            Encoder::NVH264 => "nvh264enc".to_string(),
            Encoder::X265(_) => "x265enc".to_string(),
            Encoder::NVH265 => "nvh265enc".to_string(),
            Encoder::AV1(_) => "av1enc".to_string(),
        }
    }
}

impl Pipeline for Encoder {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        match self {
            Encoder::X264(enc) => enc.link(&pipeline),
            // Encoder::NVH264 => Ok(()),
            Encoder::X265(enc) => enc.link(&pipeline),
            // Encoder::NVH265 => Ok(()),
            // Encoder::AV1 => Ok(()),
            _ => todo!("Implement the rest of the encoders"),
        }
    }

    fn unlink(&self, pipeline: &gstreamer::Pipeline) -> anyhow::Result<()> {
        match self {
            Encoder::X264(enc) => enc.unlink(&pipeline),
            // Encoder::NVH264 => Ok(()),
            Encoder::X265(enc) => enc.unlink(&pipeline),
            // Encoder::NVH265 => Ok(()),
            // Encoder::AV1 => Ok(()),
            _ => todo!("Implement the rest of the encoders"),
        }
    }
}

impl PipelineSink for Encoder {
    fn sink(&self) -> gst::Element {
        println!("Encoder::sink(): {:?}", self);
        match self {
            Encoder::X264(sink) => sink.sink(),
            // Encoder::NVH264 => gst::Element::new("nvh264enc", Some("encoder")),
            Encoder::X265(sink) => sink.sink(),
            // Encoder::NVH265 => gst::Element::new("nvh265enc", Some("encoder")),
            // Encoder::AV1 => gst::Element::new("av1enc", Some("encoder")),
            _ => todo!("Implement the rest of the encoders"),
        }
    }
}

impl PipelineSrc for Encoder {
    fn source(&self) -> gst::Element {
        match self {
            Encoder::X264(src) => src.source(),
            // Encoder::NVH264 => gst::Element::new("nvh264enc", Some("encoder")),
            Encoder::X265(src) => src.source(),
            // Encoder::NVH265 => gst::Element::new("nvh265enc", Some("encoder")),
            // Encoder::AV1 => gst::Element::new("av1enc", Some("encoder")),
            _ => todo!("Implement the rest of the encoders"),
        }
    }
}
