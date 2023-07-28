use crate::{recorder::obs_data::ObsData, settings::RateControl};

// the encoders are sorted by their default preference
#[allow(non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoder {
    JIM_NVENC,
    FFMPEG_NVENC,
    AMD_H264,
    OBS_QSV11,
    OBS_X264,
}

impl Encoder {
    #[must_use]
    pub fn id(&self) -> &str {
        match *self {
            Self::JIM_NVENC => "jim_nvenc",
            Self::FFMPEG_NVENC => "ffmpeg_nvenc",
            Self::AMD_H264 => "h264_texture_amf",
            Self::OBS_QSV11 => "obs_qsv11",
            Self::OBS_X264 => "obs_x264",
        }
    }

    #[must_use]
    pub(crate) fn settings(self, rate_control: RateControl) -> ObsData {
        match self {
            Self::JIM_NVENC | Self::FFMPEG_NVENC => nvidia_nvenc_settings(rate_control),
            Self::AMD_H264 => amd_new_h264_settings(rate_control),
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
            "h264_texture_amf" => Ok(Self::AMD_H264),
            "obs_qsv11" => Ok(Self::OBS_QSV11),
            "obs_x264" => Ok(Self::OBS_X264),
            _ => Err(()),
        }
    }
}

fn amd_new_h264_settings(rate_control: RateControl) -> ObsData {
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

fn nvidia_nvenc_settings(settings: RateControl) -> ObsData {
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
