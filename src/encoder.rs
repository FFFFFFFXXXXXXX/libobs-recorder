use crate::{
    obs_data::ObsData,
    rate_control::{Cqp, RateControl},
};

const DEFAULT_CQP: u32 = 20;
const AMF_CONSTANT_QP: u32 = 0;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Encoder {
    JIM_NVENC,
    FFMPEG_NVENC,
    AMD_AMF_H264,
    OBS_QSV11,
    OBS_X264,
}

impl Encoder {
    pub fn id(&self) -> &str {
        match *self {
            Self::JIM_NVENC => "jim_nvenc",
            Self::FFMPEG_NVENC => "ffmpeg_nvenc",
            Self::AMD_AMF_H264 => "amd_amf_h264",
            Self::OBS_QSV11 => "obs_qsv11",
            Self::OBS_X264 => "obs_x264",
        }
    }

    pub(crate) fn settings(&self, settings: &RateControl) -> ObsData {
        match *self {
            Self::JIM_NVENC | Self::FFMPEG_NVENC => nvenc_settings(settings),
            Self::AMD_AMF_H264 => amd_amf_h264_settings(settings),
            Self::OBS_QSV11 => quicksync_settings(settings),
            Self::OBS_X264 => obs_x264_settings(settings),
        }
    }
}

fn amd_amf_h264_settings(settings: &RateControl) -> ObsData {
    let mut data = ObsData::new();
    // Static Properties
    data.set_int("Usage", 0);
    data.set_int("Profile", 100);
    // Common Properties
    data.set_int("VBVBuffer", 1);
    // Picture Control Properties
    data.set_double("KeyframeInterval", 2.0);
    data.set_int("BFrame.Pattern", 0);
    if settings.cbr.is_set() {
        data.set_string("rate_control", "CBR");
        data.set_int("bitrate", settings.cbr);
        data.set_int("FillerData", 1);
        data.set_int("VBVBuffer.Size", settings.cbr);
    } else {
        let cqp = if settings.cqp.is_set() {
            settings.cqp
        } else {
            Cqp::new(DEFAULT_CQP)
        };
        data.set_int("RateControlMethod", AMF_CONSTANT_QP);
        data.set_int("QP.IFrame", cqp);
        data.set_int("QP.PFrame", cqp);
        data.set_int("QP.BFrame", cqp);
        data.set_int("VBVBuffer.Size", 100000);
    }
    return data;
}

fn nvenc_settings(settings: &RateControl) -> ObsData {
    let mut data = ObsData::new();
    data.set_string("profile", "high");
    data.set_string("preset", "hq");
    if settings.cbr.is_set() {
        data.set_string("rate_control", "CBR");
        data.set_int("bitrate", settings.cbr);
    } else {
        let cqp = if settings.cqp.is_set() {
            settings.cqp
        } else {
            Cqp::new(DEFAULT_CQP)
        };
        data.set_string("rate_control", "CQP");
        data.set_int("cqp", cqp);
    }
    return data;
}

fn quicksync_settings(settings: &RateControl) -> ObsData {
    let mut data = ObsData::new();
    data.set_string("profile", "high");
    if settings.icq.is_set() {
        data.set_string("rate_control", "ICQ");
        data.set_int("icq_quality", settings.icq);
    } else if settings.cbr.is_set() {
        data.set_string("rate_control", "CBR");
        data.set_int("bitrate", settings.cbr);
    } else {
        let cqp = if settings.cqp.is_set() {
            settings.cqp
        } else {
            Cqp::new(DEFAULT_CQP)
        };
        data.set_string("rate_control", "CQP");
        data.set_int("qpi", cqp);
        data.set_int("qpp", cqp);
        data.set_int("qpb", cqp);
    }
    return data;
}

fn obs_x264_settings(settings: &RateControl) -> ObsData {
    let mut data = ObsData::new();
    data.set_bool("use_bufsize", true);
    data.set_string("profile", "high");
    data.set_string("preset", "veryfast");
    if settings.cbr.is_set() {
        data.set_string("rate_control", "CBR");
        data.set_int("bitrate", settings.cbr);
    } else {
        let cqp = if settings.cqp.is_set() {
            settings.cqp
        } else {
            Cqp::new(DEFAULT_CQP)
        };
        data.set_string("rate_control", "CRF");
        data.set_int("crf", cqp);
    }
    return data;
}
