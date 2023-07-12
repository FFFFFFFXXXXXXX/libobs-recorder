#[cfg(feature = "full")]
use crate::{recorder::obs_data::ObsData, RateControl};

#[cfg(feature = "full")]
mod consts {
    pub const ENABLE: i64 = 1;

    pub const AMD_AMF_CQP: i64 = 0;
    pub const AMD_AMF_VBR: i64 = 2;
    pub const AMD_AMF_CBR: i64 = 3;
    pub const AMD_AMF_QUALITY_PRESET: i64 = 1;
}

// the encoders are sorted by their default preference
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoder {
    JIM_NVENC,
    FFMPEG_NVENC,
    AMD_NEW_H264,
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
            Self::AMD_NEW_H264 => "h264_texture_amf",
            Self::OBS_QSV11 => "obs_qsv11",
            Self::OBS_X264 => "obs_x264",
        }
    }

    #[cfg(feature = "full")]
    pub(crate) fn settings(&self, rate_control: RateControl) -> ObsData {
        match *self {
            Self::JIM_NVENC | Self::FFMPEG_NVENC => nvidia_nvenc_settings(rate_control),
            Self::AMD_AMF_H264 => amd_amf_h264_settings(rate_control),
            Self::AMD_NEW_H264 => amd_new_h264_settings(rate_control),
            Self::OBS_QSV11 => intel_quicksync_settings(rate_control),
            Self::OBS_X264 => obs_x264_settings(rate_control),
        }
    }
}

impl TryFrom<&str> for Encoder {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "jim_nvenc" => Ok(Self::JIM_NVENC),
            "ffmpeg_nvenc" => Ok(Self::FFMPEG_NVENC),
            "amd_amf_h264" => Ok(Self::AMD_AMF_H264),
            "h264_texture_amf" => Ok(Self::AMD_NEW_H264),
            "obs_qsv11" => Ok(Self::OBS_QSV11),
            "obs_x264" => Ok(Self::OBS_X264),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "full")]
fn amd_amf_h264_settings(rate_control: RateControl) -> ObsData {
    let mut data = ObsData::new();
    // Picture Control Properties
    data.set_double("Interval.Keyframe", 2.0);
    data.set_int("HighMotionQualityBoost", consts::ENABLE);
    data.set_int("BFrame.Pattern", 1);
    data.set_int("BFrame.Reference", consts::ENABLE);
    data.set_int("QualityPreset", consts::AMD_AMF_QUALITY_PRESET);
    data.set_string("preset", "quality");
    data.set_string("profile", "high");
    match rate_control {
        RateControl::CBR(cbr) => {
            data.set_int("RateControlMethod", consts::AMD_AMF_CBR);
            data.set_int("bitrate", cbr);
        }
        RateControl::VBR(vbr) => {
            data.set_int("RateControlMethod", consts::AMD_AMF_VBR);
            data.set_int("bitrate", vbr);
        }
        RateControl::CQP(cqp) | RateControl::ICQ(cqp) => {
            let cqp = cqp.clamp(0, 51);
            data.set_int("RateControlMethod", consts::AMD_AMF_CQP);
            data.set_int("QP.IFrame", cqp);
            data.set_int("QP.PFrame", cqp);
        }
        _ => {}
    };
    data
}

#[cfg(feature = "full")]
fn amd_new_h264_settings(rate_control: RateControl) -> ObsData {
    let mut data = ObsData::new();
    // Picture Control Properties
    data.set_int("bf", 1);
    data.set_int("keyint_sec", 2);
    data.set_string("preset", "quality");
    data.set_string("profile", "high");
    data.set_string("ffmpeg_opts", "MaxNumRefFrames=4 BReferenceEnable=1 BPicturesPattern=1 MaxConsecutiveBPictures=1 HighMotionQualityBoostEnable=1");
    match rate_control {
        RateControl::CBR(cbr) => {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", cbr);
        }
        RateControl::VBR(vbr) => {
            data.set_string("rate_control", "VBR");
            data.set_int("bitrate", vbr);
        }
        RateControl::CQP(cqp) | RateControl::ICQ(cqp) => {
            let cqp = cqp.clamp(0, 51);
            data.set_string("rate_control", "CQP");
            data.set_int("cqp", cqp);
        }
        _ => {}
    };
    data
}

#[cfg(feature = "full")]
fn nvidia_nvenc_settings(settings: RateControl) -> ObsData {
    let mut data = ObsData::new();
    data.set_string("profile", "high");
    data.set_string("preset", "hq");
    data.set_int("bf", 2);
    data.set_bool("psycho_aq", true);
    data.set_bool("lookahead", true);
    match settings {
        RateControl::CBR(cbr) => {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", cbr);
        }
        RateControl::VBR(vbr) => {
            data.set_string("rate_control", "VBR");
            data.set_int("bitrate", vbr);
            data.set_int("max_bitrate", vbr + vbr / 2);
        }
        RateControl::CQP(cqp) => {
            data.set_string("rate_control", "CQP");
            data.set_int("cqp", cqp);
            data.set_int("bitrate", 40000);
            data.set_int("max_bitrate", 60000);
        }
        _ => {}
    };
    data
}

#[cfg(feature = "full")]
fn intel_quicksync_settings(settings: RateControl) -> ObsData {
    let mut data = ObsData::new();
    data.set_string("profile", "high");
    match settings {
        RateControl::CBR(cbr) => {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", cbr);
            data.set_int("max_bitrate", cbr + cbr / 2);
        }
        RateControl::VBR(vbr) => {
            data.set_string("rate_control", "VBR");
            data.set_int("bitrate", vbr);
            data.set_int("max_bitrate", vbr + vbr / 2);
        }
        RateControl::CQP(cqp) | RateControl::CRF(cqp) => {
            let cqp = cqp.clamp(0, 51);
            data.set_string("rate_control", "CQP");
            data.set_int("qpi", cqp);
            data.set_int("qpp", cqp);
            data.set_int("qpb", cqp);
        }
        RateControl::ICQ(icq) => {
            let icq = icq.clamp(0, 51);
            data.set_string("rate_control", "ICQ");
            data.set_int("icq_quality", icq);
        }
    };
    data
}

#[cfg(feature = "full")]
fn obs_x264_settings(rate_control: RateControl) -> ObsData {
    let mut data = ObsData::new();
    data.set_bool("use_bufsize", true);
    data.set_string("profile", "high");
    data.set_string("preset", "veryfast");

    match rate_control {
        RateControl::CBR(cbr) => {
            data.set_string("rate_control", "CBR");
            data.set_int("bitrate", cbr);
        }
        RateControl::VBR(vbr) => {
            data.set_string("rate_control", "VBR");
            data.set_int("bitrate", vbr);
        }
        RateControl::CRF(crf) | RateControl::CQP(crf) => {
            let crf = crf.clamp(0, 51);
            data.set_string("rate_control", "CRF");
            data.set_int("crf", crf);
        }
        _ => {}
    };
    data
}
