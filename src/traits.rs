use gstreamer as gst;

pub trait Pipeline {
    fn link(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()>;
    fn unlink(&self, pipeline: &gst::Pipeline) -> anyhow::Result<()>;
}

pub trait PipelineSrc {
    fn source(&self) -> gst::Element;
}

pub trait PipelineSink {
    fn sink(&self) -> gst::Element;
    //fn link_src(&self, src: &gst::Element) -> anyhow::Result<()>;
}
