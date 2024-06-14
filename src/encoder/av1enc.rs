#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::encoder::Config as EncoderConfig;
use crate::recorder::errors::RecorderError;
use crate::traits::*;
use crate::util::{gst_create_element, gst_create_video_encoder};
use gst::prelude::*;
use gstreamer as gst;
use serde::{Deserialize, Serialize};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct Config {
    pub name: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: "av1enc".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Encoder {
    config: EncoderConfig,

    pub video_convert: gst::Element,
    pub encoder: gst::Element,
    pub av1parse: gst::Element,
}

impl Encoder {
    pub fn new(config: EncoderConfig) -> anyhow::Result<Self> {
        let video_convert = gst_create_element("videoconvert", "videoconvert0")?;
        let encoder = gst_create_video_encoder(&config.variant)?;
        let av1parse =
            gst_create_element("av1parse", &format!("output_{}_av1parse", &config.name))?;

        Ok(Encoder {
            config,
            video_convert,
            encoder,
            av1parse,
        })
    }
}

impl Pipeline for Encoder {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline.add_many(&[&self.video_convert, &self.encoder, &self.av1parse])?;
        gst::Element::link_many(&[&self.video_convert, &self.encoder, &self.av1parse])?;

        Ok(())
    }

    fn unlink(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline.remove_many(&[&self.video_convert, &self.encoder, &self.av1parse])?;
        Ok(())
    }
}

impl PipelineSrc for Encoder {
    fn source(&self) -> gst::Element {
        self.av1parse.clone()
    }
}

impl PipelineSink for Encoder {
    fn sink(&self) -> gst::Element {
        self.video_convert.clone()
    }
}
