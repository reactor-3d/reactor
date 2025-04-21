use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Tab {
    Viewport(String),
    Settings(String),
}

impl Tab {
    pub fn new_viwport() -> Self {
        Self::Viewport("Viewport".into())
    }

    pub fn new_settings() -> Self {
        Self::Settings("Settings".into())
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Viewport(_) => "Viewport",
            Self::Settings(_) => "Settings",
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Viewport(title) | Self::Settings(title) => title.as_str(),
        }
    }

    pub fn increment_title(&mut self) {
        let name = self.name();

        match self {
            Self::Viewport(title) | Self::Settings(title) => {
                let mut num = title.trim_start_matches(name).trim().parse::<usize>().unwrap_or(0);
                num += 1;
                *title = format!("{name} {num}");
            },
        }
    }
}
