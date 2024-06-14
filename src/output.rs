#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
mod fakesink;
mod filesink;
mod muxer;
use crate::traits::{Pipeline, PipelineSink};

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
            name: "output".to_string(),
            variant: Variant::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Variant {
    FakeSink(fakesink::Config),
    FileSink(crate::output::filesink::Config),
}

impl Default for Variant {
    fn default() -> Self {
        Variant::FileSink(crate::output::filesink::Config::default())
    }
}

#[derive(Debug)]
pub enum Output {
    FakeSink(fakesink::FakeSink),
    FileSink(filesink::FileSink),
}

impl Output {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        match config.variant {
            Variant::FakeSink(c) => Ok(Output::FakeSink(fakesink::FakeSink::new(
                config.name.clone(),
                c,
            )?)),
            Variant::FileSink(c) => Ok(Output::FileSink(filesink::FileSink::new(
                config.name.clone(),
                c,
            )?)),
        }
    }
}

impl Pipeline for Output {
    fn link(&self, pipeline: &gst::Pipeline) -> Result<()> {
        match self {
            Output::FakeSink(f) => f.link(pipeline),
            Output::FileSink(fs) => fs.link(pipeline),
        }
    }

    fn unlink(&self, pipeline: &gst::Pipeline) -> Result<()> {
        match self {
            Output::FakeSink(sink) => sink.unlink(&pipeline),
            Output::FileSink(sink) => sink.unlink(&pipeline),
        }
    }
}

impl PipelineSink for Output {
    fn sink(&self) -> gst::Element {
        match self {
            Output::FakeSink(sink) => sink.sink(),
            Output::FileSink(sink) => sink.sink(),
        }
    }
}
