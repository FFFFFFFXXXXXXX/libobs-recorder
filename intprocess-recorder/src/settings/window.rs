#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Window {
    name: String,
    class: Option<String>,
    process: Option<String>,
}

impl Window {
    pub fn new(name: impl Into<String>, class: Option<String>, process: Option<String>) -> Self {
        Self {
            name: name.into(),
            class,
            process,
        }
    }

    pub(crate) fn get_libobs_window_id(&self) -> String {
        let mut window_id = String::new();
        window_id.push_str(&self.name);
        window_id.push(':');
        if let Some(class) = &self.class {
            window_id.push_str(class);
        }
        window_id.push(':');
        if let Some(process) = &self.process {
            window_id.push_str(process);
        }
        window_id
    }
}
