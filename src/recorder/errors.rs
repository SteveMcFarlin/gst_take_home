use serde::Serialize;

// use super::gst_pipeline::PipelineState;
use gstreamer as gst;

#[derive(Debug, Serialize, Clone)]
pub struct RecorderErrorLog {
    // pub state: PipelineState,
    pub error_message: Option<String>,
    pub error_pipeline_graph: Option<String>,
}

impl RecorderErrorLog {
    pub fn new(
        // state: PipelineState,
        error_message: Option<String>,
        error_pipeline_graph: Option<String>,
    ) -> Self {
        Self {
            // state,
            error_message,
            error_pipeline_graph,
        }
    }
}

impl From<RecorderError> for RecorderErrorLog {
    fn from(err: RecorderError) -> Self {
        Self {
            // state: PipelineState::Error,
            error_message: Some(err.to_string()),
            error_pipeline_graph: None,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum RecorderError {
    #[error("Failed to set {0} pipeline state to: {1:?}")]
    PipelineStateChangeError(String, gst::State),
    #[error("Pipeline is in error state")]
    PipelineInErrorState,
    #[error("Error locking pipeline")]
    PipelineLockError,
    #[error("Element error: {}", .0)]
    ElementError(String),
    #[error("Input Error: {}", .0)]
    InputError(String),
    #[error("Output Error: {}", .0)]
    OutputError(String),
    #[error("State error: {0}")]
    StateError(String),
    #[error("Application Error: {0}")]
    AppError(String),
}

impl RecorderError {
    pub fn with_trace(self) -> Self {
        match self.clone() {
            Self::OutputError(msg)
            | Self::ElementError(msg)
            | Self::StateError(msg)
            | Self::AppError(msg)
            | Self::InputError(msg) => {
                tracing::error!(msg);
            }
            Self::PipelineStateChangeError(name, state) => {
                tracing::error!("Failed to set {} pipeline state to: {:?}", name, state);
            }
            Self::PipelineInErrorState => {
                tracing::error!("Pipeline is in error state");
            }
            Self::PipelineLockError => {
                tracing::error!("Error locking pipeline");
            }
        }
        self
    }
}
