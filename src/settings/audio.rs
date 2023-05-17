#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AudioSource {
    NONE,
    APPLICATION,
    SYSTEM,
}
