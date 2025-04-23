use eframe::wgpu::naga::FastHashMap;
use serde::{Deserialize, Serialize};

use crate::app::IdKey;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Tab {
    Viewport(ViewportTab),
    Settings(String),
}

impl Tab {
    pub fn new_viwport() -> Self {
        Self::Viewport(ViewportTab::default())
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
            Self::Viewport(ViewportTab { title, .. }) | Self::Settings(title) => title.as_str(),
        }
    }

    pub fn increment_title(&mut self) {
        let name = self.name();

        match self {
            Self::Viewport(ViewportTab { title, .. }) | Self::Settings(title) => {
                let mut num = title.trim_start_matches(name).trim().parse::<usize>().unwrap_or(0);
                num += 1;
                *title = format!("{name} {num}");
            },
        }

        if let Self::Viewport(tab) = self {
            tab.cached_ids.clear();
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ViewportTab {
    title: String,
    cached_ids: FastHashMap<IdKey, egui::Id>,
}

impl Default for ViewportTab {
    fn default() -> Self {
        Self {
            title: "Viewport".into(),
            cached_ids: Default::default(),
        }
    }
}

impl ViewportTab {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn id(&mut self, key: IdKey) -> egui::Id {
        if let Some(id) = self.cached_ids.get(&key) {
            *id
        } else {
            let id = egui::Id::new(format!("{}-{}", self.title(), key as u32));
            self.cached_ids.insert(key, id);
            id
        }
    }
}

impl PartialEq for ViewportTab {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

impl Eq for ViewportTab {}
