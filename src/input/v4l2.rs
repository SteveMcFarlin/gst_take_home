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
pub struct Config {
    pub device: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: "/dev/video0".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct V4l2 {
    pub name: String,
    config: Config,
    stats: Arc<Mutex<Stats>>,

    video: gst::Element,
    video_queue: gst::Element,
}

impl V4l2 {
    pub fn new(name: String, config: Config) -> anyhow::Result<Self> {
        tracing::info!("Creating Test input {}", &name);

        let stats = Arc::new(Mutex::new(Stats::default()));

        let video = gst_create_element("v4l2src", &format!("input_{}_v4l2src", &name))?;
        video.set_property("device", &config.device);

        let video_queue = gst_create_element("queue", &format!("input_{}_video_queue", &name))?;
        let stat = stats.clone();
        video_queue.connect("overrun", false, move |_| {
            tracing::warn!("V4L2 queue overrun: {:?}", &stat);
            stat.lock().unwrap().video_queue_overrun_count += 1;
            None
        });

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

    pub fn _get_stats(&self) -> Stats {
        let lock = self.stats.lock();
        if lock.is_err() {
            tracing::error!("Error locking stats");
            return Stats::default();
        }
        let mut stats = lock.unwrap().clone();

        stats.video_queue_current_level_buffers =
            self.video_queue.property::<u32>("current-level-buffers");
        stats.video_queue_current_level_bytes =
            self.video_queue.property::<u32>("current-level-bytes");
        stats.video_queue_current_level_time =
            self.video_queue.property::<u64>("current-level-time");

        stats
    }
}

impl Drop for V4l2 {
    fn drop(&mut self) {
        tracing::info!("Dropping V4l2 input {}", &self.name);
    }
}

impl Pipeline for V4l2 {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        tracing::info!("Linking V4l2 input {}", &self.name);
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

impl PipelineSrc for V4l2 {
    fn source(&self) -> gst::Element {
        self.video_queue.clone()
    }
}
