#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum RateControl {
    CBR(u32),
    VBR(u32),
    CQP(u32),
    CRF(u32),
    ICQ(u32),
}

impl Default for RateControl {
    fn default() -> Self {
        Self::CQP(20)
    }
}
