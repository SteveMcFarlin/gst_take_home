use crate::recorder::errors::RecorderError;
use crate::traits::*;
use crate::util::gst_create_element;
use gstreamer as gst;
use gstreamer::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct FakeStats {
    pub stats: String,
}

impl Default for FakeStats {
    fn default() -> Self {
        Self {
            stats: "Not implemented for Fake".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct FakeSink {
    pub name: String,
    video: gst::Element,
}

impl FakeSink {
    pub fn _name(&self) -> &str {
        "Fake"
    }

    pub fn new(name: String, config: Config) -> anyhow::Result<Self> {
        tracing::info!("Creating Fake input {}", &name);

        let video = gst_create_element("fakesink", &format!("output_{}_fakesink_video", &name))
            .map_err(|_| {
                RecorderError::ElementError(format!("Error creating {} video fakesink", &name))
            })?;

        Ok(FakeSink { name, video })
    }

    pub fn _get_stats(&self) -> FakeStats {
        FakeStats::default()
    }
}

impl Drop for FakeSink {
    fn drop(&mut self) {
        tracing::info!("Dropping Fake input {}", &self.name);
    }
}

impl Pipeline for FakeSink {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        tracing::info!("Linking Fake input {}", &self.name);
        pipeline.add_many(&[&self.video]).map_err(|_| {
            RecorderError::ElementError(format!(
                "Error adding {} audio and video fakesink elements",
                self.name
            ))
        })?;

        Ok(())
    }

    fn unlink(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline.remove_many(&[&self.video]).map_err(|_| {
            RecorderError::ElementError(format!(
                "Error removing {} audio and video fakesink elements",
                self.name
            ))
        })?;
        Ok(())
    }
}

impl PipelineSink for FakeSink {
    fn sink(&self) -> gst::Element {
        self.video.clone()
    }
}
