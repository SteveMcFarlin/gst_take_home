#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::encoder::Config as EncoderConfig;
use crate::recorder::errors::RecorderError;
use crate::traits::*;
use crate::util::{gst_create_element, gst_create_video_encoder};
use gst::prelude::*;
use gstreamer as gst;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Encoder {
    config: EncoderConfig,

    video_convert: gst::Element,
    encoder: gst::Element,
    h265parse: gst::Element,
}

impl Encoder {
    pub fn new(config: EncoderConfig) -> anyhow::Result<Self> {
        let video_convert = gst_create_element("videoconvert", "videoconvert0")?;
        let encoder = gst_create_video_encoder(&config.variant)?;
        let h265parse =
            gst_create_element("h265parse", &format!("output_{}_h265parse", &config.name))?;
        Ok(Encoder {
            config,
            video_convert,
            encoder,
            h265parse,
        })
    }
}

impl Pipeline for Encoder {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline.add_many(&[&self.video_convert, &self.encoder, &self.h265parse])?;
        gst::Element::link_many(&[&self.video_convert, &self.encoder, &self.h265parse])?;
        Ok(())
    }

    fn unlink(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline.remove_many(&[&self.video_convert, &self.encoder, &self.h265parse])?;
        Ok(())
    }
}

impl PipelineSrc for Encoder {
    fn source(&self) -> gst::Element {
        self.h265parse.clone()
    }
}

impl PipelineSink for Encoder {
    fn sink(&self) -> gst::Element {
        self.video_convert.clone()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct Config {
    //Use gst-inspect-1.0 x264enc to see all options
    pub bitrate: u32,
    pub key_int_max: i32,
    //pub min_force_key_unit_interval: u64,
    pub option_string: String,
    pub name: String,
    pub speed_preset: String,
    pub tune: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bitrate: 5000,
            key_int_max: 250,
            //min_force_key_unit_interval: 0,
            option_string: "".to_string(),
            name: "x265enc".to_string(),
            speed_preset: "ultrafast".to_string(),
            tune: "zerolatency".to_string(),
        }
    }
}
