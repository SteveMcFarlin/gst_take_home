pub mod errors;
mod gst_pipeline;

use crate::encoder::Config as EncoderConfig;
use crate::input::Config as InputConfig;
use crate::output::Config as OutputConfig;

use self::gst_pipeline::{GstPipeline, PipelineState};
use core::panic;
use crossbeam_channel::{bounded, Receiver, Sender};
use errors::RecorderError;
use gst::prelude::*;
use gstreamer as gst;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub duration: Option<u64>,
    pub input: InputConfig,
    pub output: OutputConfig,
    pub encoder: EncoderConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: "recorder".to_string(),
            duration: None,
            input: InputConfig::default(),
            output: OutputConfig::default(),
            encoder: EncoderConfig::default(),
        }
    }
}

impl Config {
    pub fn new(
        duration: u64,
        input: InputConfig,
        output: OutputConfig,
        encoder: EncoderConfig,
    ) -> Self {
        Config {
            name: "recorder".to_string(),
            duration: Some(duration),
            input,
            output,
            encoder,
        }
    }
}

pub struct Recorder {
    pipeline: Option<Arc<Mutex<GstPipeline>>>,
    // join_handle: Option<tokio::task::JoinHandle<()>>,
    join_handle: Option<std::thread::JoinHandle<()>>,
    state_rx: Mutex<Option<Receiver<PipelineState>>>,
}

impl Recorder {
    pub fn new() -> Result<Self, RecorderError> {
        Ok(Recorder {
            pipeline: None,
            join_handle: None,
            state_rx: Mutex::new(None),
        })
    }

    pub fn start(&mut self, config: Config) -> Result<(), RecorderError> {
        tracing::info!("Starting recorder: {:?}", &config);
        if self.pipeline.is_some() {
            return Err(RecorderError::AppError(
                "Recorder already started".to_string(),
            ))?;
        }

        let duration = config.duration.unwrap_or(0);

        let gst_pipeline = GstPipeline::new(config).map_err(|e| {
            tracing::error!("Failed to create GstPipeline: {e}");
            return RecorderError::AppError(format!("Failed to create GstPipeline: {e}"));
        })?;

        gst_pipeline.link_pipelines().map_err(|e| {
            tracing::error!("Failed to link pipeline: {e}");
            return RecorderError::AppError(format!("Failed to link pipeline: {e}"));
        })?;

        gst_pipeline.connect_pipelines().map_err(|e| {
            tracing::error!("Failed to connect pipeline: {e}");
            return RecorderError::AppError(format!("Failed to connect pipeline: {e}"));
        })?;

        self.pipeline = Some(Arc::new(Mutex::new(gst_pipeline)));
        self.maybe_start_thread();
        self.change_pipeline_state(gst::State::Playing)?;

        let result = self.state_rx.lock();
        if let Ok(rx) = result {
            if let Some(rx) = rx.as_ref() {
                match rx.recv() {
                    Ok(state) => {
                        tracing::info!("Received state: {:?}", state);
                    }
                    Err(e) => {
                        tracing::error!("Error during state transition: {:?}", e);
                        //TODO: Should we stop the pipeline here?
                        return Err(RecorderError::StateError(format!(
                            "Error during state transition: {:?}",
                            e
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), RecorderError> {
        tracing::info!("Stopping recorder");
        if self.pipeline.is_none() {
            return Err(RecorderError::AppError(
                "Recorder already stopped".to_string(),
            ));
        }

        let pipeline = self.lock_pipeline()?.pipeline.clone();
        pipeline.send_event(gst::event::Eos::new());

        //Wait for EOS to complete.
        let result = self.state_rx.lock();
        if let Ok(rx) = result {
            if let Some(rx) = rx.as_ref() {
                match rx.recv() {
                    Ok(state) => {
                        tracing::info!("Received state: {:?}", state);
                    }
                    Err(e) => {
                        tracing::error!("Error during state transition: {:?}", e);
                    }
                }
            }
        }

        match self.join_handle.take() {
            Some(handle) => match handle.join() {
                Ok(_) => {
                    tracing::info!("Join handle completed");
                }
                Err(e) => {
                    tracing::error!("Join handle failed: {:?}", e);
                }
            },
            None => {
                tracing::error!("Join handle is None");
            }
        }
        self.join_handle = None;
        self.pipeline = None;
        Ok(())
    }

    pub fn get_state(&self) -> Result<PipelineState, RecorderError> {
        Err(RecorderError::AppError("Not implemented".to_string()))
    }

    fn lock_pipeline(&self) -> Result<std::sync::MutexGuard<GstPipeline>, RecorderError> {
        if self.pipeline.is_none() {
            return Err(RecorderError::AppError("Pipeline is None".to_string()));
        }
        let pipeline = self.pipeline.as_ref().unwrap().lock().map_err(|e| {
            tracing::error!("Failed to lock pipeline: {e}");
            RecorderError::AppError(format!("Failed to lock pipeline: {e}"))
        })?;
        Ok(pipeline)
    }

    fn change_pipeline_state(&mut self, target_state: gst::State) -> Result<(), RecorderError> {
        let mut gst_pipeline = self.lock_pipeline()?;
        let result = gst_pipeline.set_state(target_state);
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("Failed to set pipeline state: {e}");
                Err(RecorderError::StateError(format!(
                    "Failed to set pipeline state: {e}"
                )))
            }
        }
    }

    fn _debug_pipeline(&self) -> Result<(), RecorderError> {
        let gst_pipeline = self.lock_pipeline()?;
        gst_pipeline
            .pipeline
            .debug_to_dot_file(gst::DebugGraphDetails::all(), "pipeline-playing");
        Ok(())
    }

    fn maybe_start_thread(&mut self) {
        if self.join_handle.is_some() {
            return;
        }

        tracing::info!("Starting EOS monitor thread");
        let (bus_tx, bus_rx) = oneshot::channel::<Result<(), RecorderError>>();

        if self.pipeline.is_none() {
            tracing::error!("Pipeline is None");
            return;
        }
        let pipe = self.pipeline.as_mut().unwrap().clone();

        tokio::spawn(async move {
            match bus_rx.await {
                Ok(result) => {
                    tracing::info!("EOS watcher received signal");
                    match result {
                        Ok(_) => {
                            tracing::info!("EOS Watcher received Ok signal");
                        }
                        Err(e) => {
                            tracing::error!("Failed to set pipeline state: {e}");
                            match pipe.lock() {
                                Ok(_) => {
                                    tracing::info!("Locked pipeline");
                                }
                                Err(e) => {
                                    tracing::error!("Failed to lock pipeline: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    tracing::info!("EOS Watcher received drop signal");
                }
            }
        });

        // :WARNING:
        // Given this is bounded size 1 there has to be a one to one
        // relationship between the sender and receiver. If the sender
        // fails to send the receiver will block forever.
        let (state_tx, state_rx) = bounded(1);
        let mutex = self.state_rx.lock();

        //If we can not lock the mutex then we are in a bad state.
        if mutex.is_err() {
            panic!("Failed to lock state_rx mutex");
        }

        *mutex.unwrap() = Some(state_rx);

        let pipe = self.pipeline.as_mut().unwrap().clone();
        self.join_handle = Some(std::thread::spawn(move || {
            Recorder::watch_bus(pipe, bus_tx, state_tx)
        }));
    }

    fn watch_bus(
        pipeline: Arc<Mutex<GstPipeline>>,
        bus_tx: oneshot::Sender<Result<(), RecorderError>>,
        state_tx: Sender<PipelineState>,
    ) {
        tracing::info!("Watching Recorder pipeline bus");

        let gst_pipeline = pipeline.lock().expect("Failed to lock pipeline");
        let bus = gst_pipeline.pipeline.bus().expect("Failed to get bus");
        std::mem::drop(gst_pipeline);

        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            //Note: If we have lots of messages this lock could become an issue.
            let gst_pipeline = pipeline.lock();
            if let Err(e) = gst_pipeline {
                tracing::error!("Failed to lock pipeline {e}");
                continue;
            }

            use gst::MessageView;
            match msg.view() {
                MessageView::Eos(..) => {
                    tracing::info!("GST Pipline received EOS");
                    let _ = state_tx.send(PipelineState::Stopped);
                    let _ = bus_tx.send(Ok(()));
                    let state = gst_pipeline.unwrap().pipeline.state(gst::ClockTime::NONE);
                    tracing::info!("Pipeline current state: {:?}", state);
                    break;
                }
                MessageView::Error(err) => {
                    let err_str = err.error().to_string();
                    tracing::error!("{}", err_str);

                    let _ = state_tx.send(PipelineState::Error);
                    let _ = bus_tx.send(Err(RecorderError::ElementError(err_str)));
                    break;
                }
                MessageView::StateChanged(state_changed) => {
                    let src = state_changed.src();

                    if let Some(obj) = src {
                        tracing::info!("OBJECT name {:?} type {:?}", obj.name(), obj.type_());
                        tracing::info!(
                            "STATE CHANGE from {:?} to {:?}",
                            state_changed.old(),
                            state_changed.current()
                        );

                        if obj.is::<gst::Pipeline>() {
                            let old = state_changed.old();
                            let pending = state_changed.pending();
                            match state_changed.current() {
                                gst::State::Ready => {
                                    tracing::info!(
                                        "GST Pipeline state changed from {:?} to {:?} pending {:?}",
                                        old,
                                        gst::State::Ready,
                                        pending
                                    );
                                }
                                gst::State::Playing => {
                                    tracing::info!(
                                        "GST Pipeline state changed from {:?} to {:?} pending {:?}",
                                        old,
                                        gst::State::Playing,
                                        pending
                                    );
                                    let _ = state_tx.send(PipelineState::Playing);
                                }
                                gst::State::Paused => {
                                    tracing::info!(
                                        "GST Pipeline state changed from {:?} to {:?} pending {:?}",
                                        old,
                                        gst::State::Paused,
                                        pending
                                    );
                                }
                                gst::State::Null => {
                                    tracing::info!(
                                        "GST Pipeline state changed from {:?} to {:?} pending {:?}",
                                        old,
                                        gst::State::Null,
                                        pending
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => (), //tracing::info!("Unhandled message: {:?}", msg),
            }
        }
        tracing::info!("Leaving watch_bus loop");
    }
}
