use serde::{Deserialize, Serialize};

pub type AdapterId = u32;

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Adapter {
    id: AdapterId,
    name: String,
    adapter_type: AdapterType,
}

impl Adapter {
    pub(crate) fn new(id: AdapterId, name: String) -> Self {
        let adapter_type = name.as_str().into();

        Self { id, name, adapter_type }
    }

    pub fn id(&self) -> AdapterId {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn adapter_type(&self) -> AdapterType {
        self.adapter_type
    }
}

impl PartialEq for Adapter {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterType {
    Intel,
    Amd,
    Nvidia,
    Unknown,
}

impl From<&str> for AdapterType {
    fn from(value: &str) -> Self {
        let lowercase = value.to_lowercase();
        if lowercase.contains("intel") {
            AdapterType::Intel
        } else if lowercase.contains("amd") {
            AdapterType::Amd
        } else if lowercase.contains("nvidia") {
            AdapterType::Nvidia
        } else {
            AdapterType::Unknown
        }
    }
}
