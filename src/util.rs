#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::encoder::x264enc::{self, Config};
use crate::encoder::{x265enc, VideoEncoder as VideoEncoderConfig};
use crate::recorder::errors::RecorderError;
use gstreamer as gst;
use gstreamer::glib::error;
use gstreamer::prelude::*;

pub fn gst_create_element(element_type: &str, name: &str) -> anyhow::Result<gst::Element> {
    gst::ElementFactory::make(element_type)
        .name(name)
        .build()
        .map_err(|_| {
            anyhow::Error::new(RecorderError::ElementError(format!(
                "Error creating {name} {element_type}"
            )))
        })
}

pub fn _gst_link_elements(src: &gst::Element, sink: &gst::Element) -> anyhow::Result<()> {
    src.link(sink).map_err(|_| {
        anyhow::Error::new(RecorderError::ElementError(format!(
            "Error linking elements {:?} and {:?}",
            src, sink
        )))
    })
}

pub fn gst_create_video_encoder(config: &VideoEncoderConfig) -> anyhow::Result<gst::Element> {
    match config {
        VideoEncoderConfig::X264(config) => {
            let encoder = match gst_create_element("x264enc", "video_encoder") {
                Ok(element) => element,
                Err(_) => anyhow::bail!("Error creating x264enc"),
            };
            set_x264_props(&encoder, config);
            Ok(encoder)
        }
        VideoEncoderConfig::X265(config) => {
            let encoder = match gst_create_element("x265enc", "video_encoder") {
                Ok(element) => element,
                Err(_) => anyhow::bail!("Error creating x265enc"),
            };
            set_x265_props(&encoder, config);
            Ok(encoder)
        }
        VideoEncoderConfig::AV1(config) => {
            let encoder = match gst_create_element("av1enc", "video_encoder") {
                Ok(element) => element,
                Err(_) => anyhow::bail!("Error creating av1enc"),
            };
            Ok(encoder)
        }
        _ => anyhow::bail!("Encoder not implemented"),
    }
}

// https://gstreamer.freedesktop.org/documentation/x265/index.html?gi-language=c
// More info: 'gst-inspect-1.0 x265enc'
pub fn set_x265_props(x265: &gst::Element, config: &x265enc::Config) {
    x265.set_property("bitrate", config.bitrate);
    x265.set_property("key-int-max", config.key_int_max);
    x265.set_property("option-string", &config.option_string);
    x265.set_property_from_str("speed-preset", &config.speed_preset);
    x265.set_property_from_str("tune", &config.tune);
}

// https://gstreamer.freedesktop.org/documentation/x264/index.html?gi-language=c
// More info: 'gst-inspect-1.0 x264enc'
pub fn set_x264_props(x264: &gst::Element, config: &x264enc::Config) {
    x264.set_property_from_str("analyse", &config.analyse);
    x264.set_property("aud", config.aud);
    x264.set_property("bitrate", config.bitrate);
    x264.set_property("b-adapt", config.b_adapt);
    x264.set_property("bframes", config.bframes);
    x264.set_property("b-pyramid", config.b_pyramid);
    x264.set_property("byte-stream", config.byte_stream);
    x264.set_property("cabac", config.cabac);
    x264.set_property("dct8x8", config.dct8x8);
    x264.set_property_from_str("frame-packing", &config.frame_packing);
    x264.set_property("insert-vui", config.insert_vui);
    x264.set_property("interlaced", config.interlaced);
    x264.set_property("intra-refresh", config.intra_refresh);
    x264.set_property("ip-factor", config.ip_factor);
    x264.set_property("key-int-max", config.key_int_max);
    x264.set_property("mb-tree", config.mb_tree);
    x264.set_property_from_str("me", &config.me);
    x264.set_property(
        "min-force-key-unit-interval",
        config.min_force_key_unit_interval,
    );
    x264.set_property("multipass-cache-file", &config.multipass_cache_file);
    x264.set_property("name", &config.name);
    x264.set_property("noise-reduction", config.noise_reduction);
    x264.set_property("option-string", &config.option_string);
    x264.set_property_from_str("pass", &config.pass);
    x264.set_property("pb-factor", config.pb_factor);
    x264.set_property_from_str("psy-tune", &config.psy_tune);
    x264.set_property("qos", config.qos);
    x264.set_property("qp-max", config.qp_max);
    x264.set_property("qp-min", config.qp_min);
    x264.set_property("qp-step", config.qp_step);
    x264.set_property("quantizer", config.quantizer);
    x264.set_property("rc-lookahead", config.rc_lookahead);
    x264.set_property("ref", config.ref_frames);
    x264.set_property("sliced-threads", config.sliced_threads);
    x264.set_property_from_str("speed-preset", &config.speed_preset);
    x264.set_property("sps-id", config.sps_id);
    x264.set_property("subme", config.subme);
    x264.set_property("sync-lookahead", config.sync_lookahead);
    x264.set_property("threads", config.threads);
    x264.set_property("trellis", config.trellis);
    x264.set_property_from_str("tune", &config.tune);
    x264.set_property("vbv-buf-capacity", config.vbv_buffer_capacity);
    x264.set_property("weightb", config.weightb);
}
