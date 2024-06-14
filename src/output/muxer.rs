use serde::{Deserialize, Serialize};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Config {
    Flv(FlvConfig),
    Mpeg4(Mp4Config),
    MpegTs(MpegTsConfig),
    Matroska(MatroskaConfig),
    Webm(WebmConfig),
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Config::*;

        let s = match self {
            Flv(_) => "flvmux",
            Mpeg4(_) => "mp4mux",
            MpegTs(_) => "mpegtsmux",
            Matroska(_) => "matroskamux",
            Webm(_) => "webmmux",
        };

        f.write_str(s)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::Matroska(MatroskaConfig::default())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Mp4Config {
    pub faststart: bool,
    pub faststart_file: String,
    pub force_chunks: bool,
    pub force_create_timecode_track: bool,
    pub fragment_duration: u32,
    pub fragment_mode: String,
    pub interleave_bytes: u64,
    pub interleave_time: u64,
    pub latency: u64,
    pub max_raw_audio_drift: u64,
    pub min_upstream_latency: u64,
    pub moov_recovery_file: String,
    pub movie_timescale: u32,
    pub presentation_time: bool,
    pub reserved_bytes_per_sec: u32,
    pub reserved_max_duration: u64,
    pub reserved_moov_update_period: u64,
    pub reserved_prefill: bool,
    pub start_gap_threshold: u64,
    pub start_time: u64,
    pub stgart_time_selection: String,
    pub streamable: bool,
    pub track_timescale: u32,
}

impl Default for Mp4Config {
    fn default() -> Self {
        Self {
            faststart: false,
            faststart_file: "/tmp/faststartfile".to_string(),
            force_chunks: false,
            force_create_timecode_track: false,
            fragment_duration: 0,
            fragment_mode: "dash-or-mss".to_string(),
            interleave_bytes: 0,
            interleave_time: 250000000,
            latency: 0,
            max_raw_audio_drift: 40000000,
            min_upstream_latency: 0,
            moov_recovery_file: "".to_string(),
            movie_timescale: 0,
            presentation_time: true,
            reserved_bytes_per_sec: 550,
            reserved_max_duration: 18446744073709551615,
            reserved_moov_update_period: 18446744073709551615,
            reserved_prefill: false,
            start_gap_threshold: 0,
            start_time: 18446744073709551615,
            stgart_time_selection: "zero".to_string(),
            streamable: false,
            track_timescale: 0,
        }
    }
}

impl Mp4Config {
    pub fn name() -> &'static str {
        "mp4mux"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct FlvConfig {
    pub latency: u64,
    pub start_time: u64,
    pub streamable: bool,
}

impl Default for FlvConfig {
    fn default() -> Self {
        Self {
            latency: 0,
            start_time: 0,
            streamable: true,
        }
    }
}

impl FlvConfig {
    pub fn name() -> &'static str {
        "flvmux"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MpegTsConfig {
    pub alignment: i32,
    pub bitrate: u64,
    pub latency: u64,
    pub min_upstream_latency: u64,
}

impl Default for MpegTsConfig {
    fn default() -> Self {
        Self {
            alignment: -1,
            bitrate: 0,
            latency: 0,
            min_upstream_latency: 0,
        }
    }
}

impl MpegTsConfig {
    pub fn name() -> &'static str {
        "mpegtsmux"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MatroskaConfig {
    pub streamable: bool,
}

impl Default for MatroskaConfig {
    fn default() -> Self {
        Self { streamable: true }
    }
}

impl MatroskaConfig {
    pub fn name() -> &'static str {
        "matroskamux"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct WebmConfig {
    pub streamable: bool,
}

impl Default for WebmConfig {
    fn default() -> Self {
        Self { streamable: true }
    }
}

impl WebmConfig {
    pub fn name() -> &'static str {
        "webmmux"
    }
}
