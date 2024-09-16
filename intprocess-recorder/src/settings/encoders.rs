use crate::{recorder::obs_data::ObsData, settings::RateControl};

use super::{adapter::AdapterType, Adapter};

// the encoders are sorted by their priority
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoder {
    /// NVIDIA h264 encoder
    JIM_NVENC,
    /// fallback NVIDIA h264 encoder
    FFMPEG_NVENC,
    /// NVIDIA AV1 encoder
    JIM_AV1,
    /// AMD h264 encoder
    AMD_AMF_H264,
    /// AMD AV1 encoder
    AMD_AMF_AV1,
    /// Intel h264 encoder
    OBS_QSV11_H264,
    /// Intel AV1 encoder
    OBS_QSV11_AV1,
    /// Software h264 encoder
    OBS_X264,
}

impl Encoder {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::JIM_NVENC => "jim_nvenc",
            Self::FFMPEG_NVENC => "ffmpeg_nvenc",
            Self::JIM_AV1 => "jim_av1_nvenc",
            Self::AMD_AMF_H264 => "h264_texture_amf",
            Self::AMD_AMF_AV1 => "av1_texture_amf",
            Self::OBS_QSV11_H264 => "obs_qsv11_v2",
            Self::OBS_QSV11_AV1 => "obs_qsv11_av1",
            Self::OBS_X264 => "obs_x264",
        }
    }

    #[must_use]
    pub(crate) fn settings(self, rate_control: RateControl) -> ObsData {
        match self {
            Self::JIM_NVENC | Self::FFMPEG_NVENC => nvidia_h264_settings(rate_control),
            Self::JIM_AV1 => nvidia_av1_settings(rate_control),
            Self::AMD_AMF_H264 | Self::AMD_AMF_AV1 => amd_amf_settings(rate_control),
            Self::OBS_QSV11_H264 => intel_quicksync_h264_settings(rate_control),
            Self::OBS_QSV11_AV1 => intel_quicksync_av1_settings(rate_control),
            Self::OBS_X264 => obs_x264_settings(rate_control),
        }
    }

    pub(crate) fn matches_adapter(&self, adapter: &Adapter) -> bool {
        match self {
            Self::OBS_X264 => true,
            Self::JIM_NVENC | Self::FFMPEG_NVENC | Self::JIM_AV1 => adapter.adapter_type() == AdapterType::Nvidia,
            Self::AMD_AMF_H264 | Self::AMD_AMF_AV1 => adapter.adapter_type() == AdapterType::Amd,
            Self::OBS_QSV11_H264 | Self::OBS_QSV11_AV1 => adapter.adapter_type() == AdapterType::Intel,
        }
    }
}

impl TryFrom<&str> for Encoder {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "jim_nvenc" => Ok(Self::JIM_NVENC),
            "ffmpeg_nvenc" => Ok(Self::FFMPEG_NVENC),
            "h264_texture_amf" => Ok(Self::AMD_AMF_H264),
            "obs_qsv11" => Ok(Self::OBS_QSV11_H264),
            "obs_x264" => Ok(Self::OBS_X264),
            _ => Err(()),
        }
    }
}

fn nvidia_h264_settings(settings: RateControl) -> ObsData {
    let mut data = ObsData::new();

    data.set_int("bf", 2);
    data.set_bool("psycho_aq", true);
    data.set_bool("lookahead", true);

    data.set_string("profile", "high");
    data.set_string("preset", "hq");

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

fn nvidia_av1_settings(settings: RateControl) -> ObsData {
    let mut data = ObsData::new();

    data.set_string("rate_control", "CQP");
    data.set_string("profile", "main");

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

fn amd_amf_settings(rate_control: RateControl) -> ObsData {
    let mut data = ObsData::new();

    // Picture Control Properties
    data.set_int("bf", 1);
    data.set_int("keyint_sec", 2);
    data.set_string("ffmpeg_opts", "MaxNumRefFrames=4 BReferenceEnable=1 BPicturesPattern=1 MaxConsecutiveBPictures=1 HighMotionQualityBoostEnable=1");

    data.set_string("preset", "quality");
    data.set_string("profile", "high");

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
        RateControl::CRF(_) => { /* invalid -> do nothing */ }
    };
    data
}

fn intel_quicksync_h264_settings(settings: RateControl) -> ObsData {
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

fn intel_quicksync_av1_settings(settings: RateControl) -> ObsData {
    let mut data = ObsData::new();

    data.set_string("profile", "high");

    match settings {
        RateControl::CQP(cqp) | RateControl::CRF(cqp) => {
            let cqp = cqp.clamp(0, 51);
            data.set_string("rate_control", "CQP");
            data.set_int("qpi", cqp);
            data.set_int("qpp", cqp);
            data.set_int("qpb", cqp);
        }
        _ => {}
    };
    data
}

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
        RateControl::ICQ(_) => { /* invalid -> do nothing */ }
    };
    data
}
