#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    name: String,
    class: Option<String>,
    process: Option<String>,
}

impl Window {
    pub fn new<S: Into<String>>(name: S, class: Option<String>, process: Option<String>) -> Self {
        Self {
            name: name.into(),
            class,
            process,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn class(&self) -> Option<&String> {
        self.class.as_ref()
    }
    pub fn process(&self) -> Option<&String> {
        self.process.as_ref()
    }

    pub fn get_libobs_window_id(&self) -> String {
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
        return window_id;
    }
}
