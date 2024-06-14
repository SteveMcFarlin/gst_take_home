#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::recorder::errors::RecorderError;
use crate::traits::{Pipeline, PipelineSrc};
use crate::util::gst_create_element;
use gstreamer as gst;
use gstreamer::prelude::*;
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Clone, Default)]
pub struct Stats {
    video_queue_current_level_buffers: u32,
    video_queue_current_level_bytes: u32,
    video_queue_current_level_time: u64,
    video_queue_overrun_count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct Test {
    name: String,
    config: Config,
    stats: Arc<Mutex<Stats>>,

    video: gst::Element,
    video_queue: gst::Element,
}

impl Test {
    pub fn new(name: String, config: Config) -> anyhow::Result<Self> {
        tracing::info!("Creating Test input {}", &name);

        let stats = Arc::new(Mutex::new(Stats::default()));

        let video = gst_create_element("videotestsrc", &format!("input_{}_videotestsrc", &name))?;
        video.set_property("is-live", true);
        video.set_property_from_str("pattern", "smpte");

        let video_queue = gst_create_element("queue", &format!("input_{}_video_queue", &name))?;
        let stat = stats.clone();
        video_queue.connect("overrun", false, move |_| {
            tracing::warn!("Test Video queue overrun: {:?}", &stat);
            stat.lock().unwrap().video_queue_overrun_count += 1;
            None
        });
        let video_convert =
            gst_create_element("videoconvert", &format!("input_{}_video_convert", &name))?;

        Ok(Self {
            name,
            config,
            stats,
            video,
            video_queue,
        })
    }

    pub fn _name(&self) -> &str {
        &self.name
    }
}

impl Drop for Test {
    fn drop(&mut self) {
        tracing::info!("Dropping Test input {}", &self.name);
    }
}

impl Pipeline for Test {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        tracing::info!("Linking Test input {}", &self.name);
        pipeline
            .add_many(&[&self.video, &self.video_queue])
            .map_err(|_| {
                RecorderError::ElementError(format!(
                    "Error adding {} elements to pipeline",
                    self.name
                ))
            })?;

        gst::Element::link_many(&[&self.video, &self.video_queue]).map_err(|_| {
            RecorderError::ElementError(format!("Error linking {} video elements", self.name))
        })?;

        Ok(())
    }

    fn unlink(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline
            .remove_many(&[&self.video, &self.video_queue])
            .map_err(|_| {
                RecorderError::ElementError(format!("Error unlinking {} elements", self.name))
            })?;

        Ok(())
    }
}

impl PipelineSrc for Test {
    fn source(&self) -> gst::Element {
        self.video_queue.clone()
    }
}
