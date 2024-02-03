#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum AudioSource {
    /// no audio
    NONE,
    /// only the audio of the window that is being captured
    APPLICATION,
    /// the default audio output of the pc
    SYSTEM,
    /// the default audio input and output of the pc
    ALL,
}
