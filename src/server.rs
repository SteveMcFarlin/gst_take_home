#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::recorder::errors::RecorderErrorLog;
use crate::recorder::Config as RecorderConfig;
use crate::recorder::Recorder;
use async_std::task::sleep;
use axum::{
    body::{boxed, Body, BoxBody},
    extract::{Json, State},
    http::{HeaderValue, Request, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use gstreamer::tags::TrackCount;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    pub recorder: Arc<Mutex<Recorder>>,
    pub ip_addr: IpAddr,
    pub port: u16,
}

impl AppState {
    fn new(recorder: Arc<Mutex<Recorder>>, ip_addr: IpAddr, port: u16) -> Self {
        Self {
            recorder,
            ip_addr,
            port,
        }
    }
}

type BoxedFuture<'a> = Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>;

async fn run_timer<F>(duration: u64, closure: F)
where
    F: FnOnce() -> BoxedFuture<'static> + Send + 'static,
{
    tracing::info!("Running timer for {duration} seconds");
    tokio::time::sleep(tokio::time::Duration::from_secs(duration)).await;
    closure().await;
}

async fn start_timer(duration: u64, host: String, port: u16) {
    tracing::info!("Starting timer for {duration} seconds");
    run_timer(duration, move || {
        Box::pin(async move {
            tracing::info!("Timer expired. Stopping recorder");
            let client = reqwest::Client::new();
            let url = format!("http://{host}:{port}/stop");
            let res = client.post(url).body("duration timer").send().await;
            match res {
                Ok(res) => {
                    tracing::info!("{:?}", res);
                }
                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                }
            }
        })
    })
    .await;
}

impl IntoResponse for RecorderErrorLog {
    fn into_response(self) -> Response<BoxBody> {
        match serde_json::to_string(&self) {
            Ok(body) => (StatusCode::INTERNAL_SERVER_ERROR, body).into_response(),
            Err(e) => {
                tracing::error!("failed to serialize error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
                    .into_response()
            }
        }
    }
}

pub async fn start_server(
    host: IpAddr,
    port: u16,
    shutdown_signal: Option<tokio::sync::oneshot::Receiver<()>>,
) -> anyhow::Result<()> {
    tracing::info!("Starting server");
    let host_addr = std::net::SocketAddr::from((host, port));

    let recorder = match Recorder::new() {
        Ok(recorder) => Arc::new(Mutex::new(recorder)),
        Err(e) => {
            panic!("Failed to create recorder: {e:?}");
        }
    };

    let app_state = AppState::new(recorder, host, port);

    let app = Router::new()
        .route("/", get(root))
        .route("/start", post(start))
        .route("/stop", post(stop))
        .with_state(app_state);

    tracing::debug!("listening on {}", host_addr);

    axum::Server::bind(&host_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            if let Some(shutdown_signal) = shutdown_signal {
                tracing::info!("Waiting for shutdown signal");
                shutdown_signal.await.unwrap();
            } else {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install CTRL+C signal handler");
            }
            tracing::info!("Shutdown signal received");
            std::process::exit(0);
        })
        .await
        .map_err(|e| {
            tracing::error!("server error: {}", e);
            anyhow::Error::msg(format!("server error: {e}"))
        })
}

fn get_recorder(state: &AppState) -> std::sync::MutexGuard<'_, Recorder> {
    state
        .recorder
        .lock()
        //Should we really panic here? Not being able to acquire the lock is a pretty bad error.
        .map_err(|e| panic!("failed to lock recorder: {e}"))
        .unwrap()
}

async fn root(State(_): State<AppState>) -> String {
    tracing::info!("/");
    format!("Recorder {:?}", crate::VERSION)
}

async fn start(
    State(state): State<AppState>,
    Json(payload): Json<RecorderConfig>,
) -> Result<impl IntoResponse, RecorderErrorLog> {
    tracing::info!("/start: {:?}", payload);
    let duration = payload.duration.unwrap_or(0);
    let mut recorder = get_recorder(&state);
    recorder.start(payload)?;

    // There is probably a better way to do the duration. Look into GST for this.
    if duration > 0 {
        let _ = tokio::task::spawn(start_timer(
            duration,
            state.ip_addr.to_string(),
            state.port.clone(),
        ));
    }

    let response = serde_json::json!({"status": "OK"}).to_string();
    Ok((StatusCode::OK, response).into_response())
}

async fn stop(State(state): State<AppState>) -> Result<impl IntoResponse, RecorderErrorLog> {
    tracing::info!("/stop");
    let mut recorder = get_recorder(&state);

    recorder.stop()?;

    tracing::info!("recorder stopped");
    let response = serde_json::json!({"status": "OK"}).to_string();
    Ok((StatusCode::OK, response).into_response())
}
