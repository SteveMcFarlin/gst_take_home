#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::encoder::Config as EncoderConfig;
use crate::recorder::errors::RecorderError;
use crate::traits::*;
use crate::util::{gst_create_element, gst_create_video_encoder};
use anyhow::Ok;
use gst::prelude::*;
use gstreamer as gst;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Encoder {
    config: EncoderConfig,

    video_convert: gst::Element,
    encoder: gst::Element,
    h264parse: gst::Element,
}

impl Encoder {
    pub fn new(config: EncoderConfig) -> anyhow::Result<Self> {
        let video_convert = gst_create_element("videoconvert", "videoconvert0")?;
        let encoder = gst_create_video_encoder(&config.variant)?;
        let h264parse =
            gst_create_element("h264parse", &format!("output_{}_h264parse", &config.name))?;

        Ok(Encoder {
            config,
            video_convert,
            encoder,
            h264parse,
        })
    }
}

impl Pipeline for Encoder {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline.add_many(&[&self.video_convert, &self.encoder, &self.h264parse])?;
        gst::Element::link_many(&[&self.video_convert, &self.encoder, &self.h264parse])?;

        Ok(())
    }

    fn unlink(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()> {
        pipeline.remove_many(&[&self.video_convert, &self.encoder, &self.h264parse])?;
        Ok(())
    }
}

impl PipelineSrc for Encoder {
    fn source(&self) -> gst::Element {
        self.h264parse.clone()
    }
}

impl PipelineSink for Encoder {
    fn sink(&self) -> gst::Element {
        self.video_convert.clone()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct Config {
    //Use gst-inspect-1.0 x264enc to see all options

    /* Settings for analyse
    Flags "GstX264EncAnalyse" Default: 0x00000000, "<empty_string>"
    (0x00000001): i4x4             - i4x4
    (0x00000002): i8x8             - i8x8
    (0x00000010): p8x8             - p8x8
    (0x00000020): p4x4             - p4x4
    (0x00000100): b8x8             - b8x8
     */
    pub analyse: String,
    pub aud: bool,
    pub b_adapt: bool,
    pub b_pyramid: bool,
    pub bframes: u32,
    pub bitrate: u32,
    pub byte_stream: bool,
    pub cabac: bool,
    pub dct8x8: bool,

    /* Settings for frame_packing
    (-1): auto             - Automatic (use incoming video information)
    (0): checkerboard     - checkerboard - Left and Right pixels alternate in a checkerboard pattern
    (1): column-interleaved - column interleaved - Alternating pixel columns represent Left and Right views
    (2): row-interleaved  - row interleaved - Alternating pixel rows represent Left and Right views
    (3): side-by-side     - side by side - The left half of the frame contains the Left eye view, the right half the Right eye view
    (4): top-bottom       - top bottom - L is on top, R on bottom
    (5): frame-interleaved - frame interleaved - Each frame contains either Left or Right view alternately
    */
    pub frame_packing: String,
    pub insert_vui: bool,
    pub interlaced: bool,
    pub intra_refresh: bool,
    pub ip_factor: f32,
    pub key_int_max: u32,
    pub mb_tree: bool,

    /* Settings for me
    Enum "GstX264EncMe" Default: 1, "hex"
    (0): dia              - dia
    (1): hex              - hex
    (2): umh              - umh
    (3): esa              - esa
    (4): tesa             - tesa
     */
    pub me: String,
    pub min_force_key_unit_interval: u64,
    pub multipass_cache_file: String,
    pub name: String,
    pub noise_reduction: u32,
    pub option_string: String, //These are key value options.

    /* Settings for pass
    Enum "GstX264EncPass" Default: 0, "cbr"
    (0): cbr              - Constant Bitrate Encoding
    (4): quant            - Constant Quantizer
    (5): qual             - Constant Quality
    (17): pass1            - VBR Encoding - Pass 1
    (18): pass2            - VBR Encoding - Pass 2
    (19): pass3            - VBR Encoding - Pass 3
     */
    pub pass: String,
    pub pb_factor: f32,

    /* Settings for psy_tune
    Enum "GstX264EncPsyTune" Default: 0, "none"
    (0): none             - No tuning
    (1): film             - Film
    (2): animation        - Animation
    (3): grain            - Grain
    (4): psnr             - PSNR
    (5): ssim             - SSIM
    */
    pub psy_tune: String,
    pub qos: bool,
    pub qp_max: u32,
    pub qp_min: u32,
    pub qp_step: u32,
    pub quantizer: u32,
    pub rc_lookahead: i32,
    pub ref_frames: u32,
    pub sliced_threads: bool,

    /* Settings for speed-preset
    Enum "GstX264EncPreset" Default: 6, "medium"
    (0): None             - No preset
    (1): ultrafast        - ultrafast
    (2): superfast        - superfast
    (3): veryfast         - veryfast
    (4): faster           - faster
    (5): fast             - fast
    (6): medium           - medium
    (7): slow             - slow
    (8): slower           - slower
    (9): veryslow         - veryslow
    (10): placebo          - placebo
     */
    pub speed_preset: String,
    pub sps_id: u32,
    pub subme: u32,
    pub sync_lookahead: i32,
    pub threads: u32,
    pub trellis: bool,

    /* Settings for tune
    Flags "GstX264EncTune" Default: 0x00000000, "<empty string>"
    (0x00000001): stillimage       - Still image
    (0x00000002): fastdecode       - Fast decode
    (0x00000004): zerolatency      - Zero latency
     */
    pub tune: String,
    pub vbv_buffer_capacity: u32,
    pub weightb: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            analyse: "".to_string(),
            aud: true,
            b_adapt: true,
            b_pyramid: false,
            bframes: 0,
            bitrate: 2048,
            byte_stream: false,
            cabac: true,
            dct8x8: false,
            frame_packing: "auto".to_string(),
            insert_vui: true,
            interlaced: false,
            intra_refresh: false,
            ip_factor: 1.4,
            key_int_max: 60,
            mb_tree: true,
            me: "hex".to_string(),
            min_force_key_unit_interval: 0,
            multipass_cache_file: "x264enc0".to_string(),
            name: "x264enc0".to_string(),
            noise_reduction: 0,
            option_string: "".to_string(),
            pass: "cbr".to_string(),
            pb_factor: 1.3,
            psy_tune: "none".to_string(),
            qos: false,
            qp_max: 51,
            qp_min: 10,
            qp_step: 4,
            quantizer: 21,
            rc_lookahead: 40,
            ref_frames: 3,
            sliced_threads: false,
            speed_preset: "medium".to_string(),
            sps_id: 0,
            subme: 1,
            sync_lookahead: -1,
            threads: 0,
            trellis: true,
            tune: "".to_string(),
            vbv_buffer_capacity: 600,
            weightb: false,
        }
    }
}
