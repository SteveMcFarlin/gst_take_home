#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
mod test;
mod v4l2;
use crate::traits::{Pipeline, PipelineSrc};
use anyhow::Result;
use gstreamer as gst;
use gstreamer::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub variant: Variant,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: "input".to_string(),
            // variant: Variant::V4l2(v4l2::Config::default()),
            variant: Variant::default(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum InputStats {
    Test(test::Stats),
    V4l2(v4l2::Stats),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Variant {
    Test(test::Config),
    V4l2(v4l2::Config),
}

impl Default for Variant {
    fn default() -> Self {
        Variant::V4l2(v4l2::Config::default())
        // Variant::Test(test::Config::default())
    }
}

#[derive(Debug)]
pub enum Input {
    Test(test::Test),
    V4l2(v4l2::V4l2),
}

impl Input {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        match config.variant {
            Variant::Test(c) => Ok(Input::Test(test::Test::new(config.name, c)?)),
            Variant::V4l2(c) => Ok(Input::V4l2(v4l2::V4l2::new("v4l2src".to_string(), c)?)),
        }
    }
}

impl Pipeline for Input {
    fn link(&self, pipeline: &gst::Pipeline) -> Result<()> {
        match self {
            Input::Test(input) => input.link(pipeline),
            Input::V4l2(input) => input.link(pipeline),
        }
    }

    fn unlink(&self, pipeline: &gstreamer::Pipeline) -> anyhow::Result<()> {
        match self {
            Input::Test(input) => input.unlink(pipeline),
            Input::V4l2(input) => input.unlink(pipeline),
        }
    }
}

impl PipelineSrc for Input {
    fn source(&self) -> gst::Element {
        match self {
            Input::Test(input) => input.source(),
            Input::V4l2(input) => input.source(),
        }
    }
}
