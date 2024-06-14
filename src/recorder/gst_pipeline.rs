use super::Config as RecorderConfig;
use crate::recorder::errors::RecorderError;
use crate::traits::Pipeline;
use crate::traits::{PipelineSink, PipelineSrc};
use crate::{encoder, input, output};
use gst::prelude::*;
use gstreamer as gst;
use serde::{Deserialize, Serialize};

/* */
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum PipelineState {
    Stopped,
    Playing,
    Error,
}

impl std::fmt::Display for PipelineState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PipelineState::Stopped => write!(f, "Stopped"),
            PipelineState::Playing => write!(f, "Playing"),
            PipelineState::Error => write!(f, "Error"),
        }
    }
}

impl PipelineState {
    pub fn is_valid_transition(
        &self,
        new_state: &PipelineState,
    ) -> anyhow::Result<PipelineState, RecorderError> {
        match (self, new_state) {
            (PipelineState::Stopped, PipelineState::Playing) => Ok(PipelineState::Playing),
            (PipelineState::Playing, PipelineState::Stopped) => Ok(PipelineState::Stopped),
            (_, PipelineState::Error) => Ok(PipelineState::Error),
            (PipelineState::Error, _) => Err(RecorderError::AppError(format!(
                "Cannot transition from Error state {self:?} to {new_state:?})"
            )))?,
            (_, _) => Err(RecorderError::AppError(format!(
                "Invalid state transition from {self:?} to {new_state:?}"
            )))?,
        }
    }

    pub fn set_state(&mut self, state: PipelineState) -> anyhow::Result<(), RecorderError> {
        let result = self.is_valid_transition(&state);
        match result {
            Ok(new_state) => {
                *self = new_state;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub struct GstPipeline {
    pub config: RecorderConfig,
    pub pipeline: gst::Pipeline,
    pub state: PipelineState,
    input: input::Input,
    encoder: encoder::Encoder,
    output: output::Output,
}

impl Drop for GstPipeline {
    fn drop(&mut self) {
        tracing::info!("Dropping pipeline {}", &self.config.name);
        if let Err(e) = self.pipeline.set_state(gst::State::Null) {
            tracing::error!("Failed to set pipeline to null state: {e}");
        }
        if let Err(e) = self.unlink_pipelines() {
            tracing::error!("Failed to unlink pipeline: {e}");
        }
    }
}

impl GstPipeline {
    pub fn new(config: RecorderConfig) -> anyhow::Result<Self> {
        let pipeline = gst::Pipeline::new(None);

        let input = input::Input::new(config.input.clone())?;
        let encoder = encoder::Encoder::new(config.encoder.clone())?;
        let output = output::Output::new(config.output.clone())?;

        Ok(Self {
            config,
            pipeline,
            state: PipelineState::Stopped,
            input,
            encoder,
            output,
        })
    }

    pub fn set_state(&mut self, state: gst::State) -> anyhow::Result<(), RecorderError> {
        let result = self.pipeline.set_state(state);
        match result {
            Ok(_) => {
                tracing::info!("Pipeline state transitioning to: {:?}", state);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Pipeline state transition failed: {e}");
                Err(RecorderError::PipelineStateChangeError(
                    format!("Failed to transition pipeline state: {e}"),
                    state,
                ))?
            }
        }
    }

    pub fn _get_state(&self) -> PipelineState {
        self.state
    }

    pub fn link_pipelines(&self) -> anyhow::Result<()> {
        self.input.link(&self.pipeline.clone())?;
        self.encoder.link(&self.pipeline.clone())?;
        self.output.link(&self.pipeline.clone())?;

        Ok(())
    }

    pub fn connect_pipelines(&self) -> anyhow::Result<()> {
        let input_src = self.input.source();
        let codec_sink = self.encoder.sink();
        let codec_src = self.encoder.source();
        let output_sink = self.output.sink();

        println!("Input src: {:?}", input_src);
        println!("Codec sink: {:?}", codec_sink);
        println!("Codec src: {:?}", codec_src);
        println!("Output sink: {:?}", output_sink);

        input_src.link(&codec_sink).map_err(|e| {
            RecorderError::ElementError(format!("Error linking Input to Codec: {:?}", e))
        })?;
        codec_src.link(&output_sink).map_err(|e| {
            RecorderError::ElementError(format!("Error linking Codec to Output: {:?}", e))
        })?;

        Ok(())
    }

    pub fn unlink_pipelines(&self) -> anyhow::Result<()> {
        self.input.unlink(&self.pipeline)?;
        self.encoder.unlink(&self.pipeline)?;
        self.output.unlink(&self.pipeline)?;

        Ok(())
    }
}
