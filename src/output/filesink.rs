#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
// use super::media::encoder_config::EncoderConfig;
// use super::media::muxer::Muxer;
use crate::output::muxer::{
    Config as MuxerConfig, FlvConfig, MatroskaConfig, Mp4Config, MpegTsConfig, WebmConfig,
};
use crate::recorder::errors::RecorderError;
use crate::traits::*;
use crate::util::gst_create_element;

use gstreamer as gst;
use gstreamer::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub muxer_config: MuxerConfig,
    pub location: String,
    pub append: bool,
    pub async_to_pause: bool,
    pub blocksize: u32,
    pub buffer_mode: String,
    pub buffer_size: u32,
    pub max_bitrate: u64,
    pub max_lateness: i64,
    pub max_transient_error_timeout: i32,
    pub o_sync: bool,
    pub processing_deadline: u64,
    pub qos: bool,
    pub render_delay: u64,
    pub sync: bool,
    pub throttle_time: u64,
    pub ts_offset: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            muxer_config: MuxerConfig::default(),
            append: false,
            async_to_pause: true,
            blocksize: 4096,
            buffer_mode: "default".to_string(),
            buffer_size: 65535,
            location: "/tmp/output.mkv".to_string(),
            max_bitrate: 0,
            max_lateness: -1,
            max_transient_error_timeout: 0,
            o_sync: false,
            processing_deadline: 20000000,
            qos: false,
            render_delay: 0,
            sync: false,
            throttle_time: 0,
            ts_offset: 0,
        }
    }
}

#[derive(Debug)]
pub struct FileSink {
    name: String,
    config: Config,

    video_queue: gst::Element,
    muxer: gst::Element,
    filesink: gst::Element,
}

impl FileSink {
    pub fn new(name: String, config: Config) -> anyhow::Result<Self> {
        tracing::info!("Creating FileSink output {}", &name);

        let video_queue = gst_create_element("queue", &format!("output_{}_video_queue", &name))?;
        video_queue.connect("overrun", false, move |_| {
            tracing::warn!("FileSink Video queue overrun");
            None
        });

        let muxer = match config.muxer_config {
            MuxerConfig::Mpeg4(_) => gst_create_element(
                Mp4Config::name(),
                &format!("{}_output_muxer", Mp4Config::name()),
            )?,
            MuxerConfig::Flv(_) => gst_create_element(
                FlvConfig::name(),
                &format!("{}_output_muxer", FlvConfig::name()),
            )?,
            MuxerConfig::MpegTs(_) => gst_create_element(
                MpegTsConfig::name(),
                &format!("{}_output_muxer", MpegTsConfig::name()),
            )?,
            MuxerConfig::Matroska(_) => gst_create_element(
                MatroskaConfig::name(),
                &format!("{}_output_muxer", MatroskaConfig::name()),
            )?,
            MuxerConfig::Webm(_) => gst_create_element(
                WebmConfig::name(),
                &format!("{}_output_muxer", &WebmConfig::name()),
            )?,
        };

        let filesink = gst_create_element("filesink", &format!("{}_output_filesink", &name))?;
        filesink.set_property("location", &config.location);

        Ok(FileSink {
            name,
            config,
            video_queue,
            muxer,
            filesink,
        })
    }
}

impl Drop for FileSink {
    fn drop(&mut self) {
        tracing::info!("Dropping FileSink {}", self.name);
    }
}

impl Pipeline for FileSink {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        tracing::info!("Linking {} output elements", self.name);
        pipeline
            .add_many(&[&self.video_queue, &self.muxer, &self.filesink])
            .map_err(|e| {
                RecorderError::ElementError(format!(
                    "Failed to add elements to pipeline: {}",
                    e.to_string()
                ))
            })?;

        gst::Element::link_many(&[&self.video_queue, &self.muxer, &self.filesink]).map_err(
            |e| {
                RecorderError::ElementError(format!(
                    "Failed to link elements in pipeline: {}",
                    e.to_string()
                ))
            },
        )?;

        Ok(())
    }

    fn unlink(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        tracing::info!("Unlinking {} output elements", self.name);
        pipeline
            .remove_many(&[&self.video_queue, &self.muxer, &self.filesink])
            .map_err(|e| {
                RecorderError::ElementError(format!(
                    "Failed to remove elements from pipeline: {}",
                    e.to_string()
                ))
            })?;

        Ok(())
    }
}

impl PipelineSink for FileSink {
    fn sink(&self) -> gst::Element {
        self.video_queue.clone()
    }
}
