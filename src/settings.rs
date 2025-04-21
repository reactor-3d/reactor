use egui_dock::AllowedSplits;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, egui_probe::EguiProbe)]
pub enum EditMode {
    Editing,
    View,
}

impl EditMode {
    pub fn switch(&mut self) -> Self {
        match self {
            Self::Editing => *self = Self::View,
            Self::View => *self = Self::Editing,
        }
        *self
    }
}

#[derive(Debug, Deserialize, Serialize, egui_probe::EguiProbe)]
pub struct TabsSettings {
    pub draggable_tabs: bool,
    pub show_tab_name_on_hover: bool,

    #[serde(skip)]
    #[egui_probe(skip)]
    pub allowed_splits: AllowedSplits,

    pub show_close_buttons: bool,
    pub show_add_buttons: bool,
    pub show_leaf_close_all_buttons: bool,
    pub show_leaf_collapse_buttons: bool,
    pub show_secondary_button_hint: bool,
    pub secondary_button_on_modifier: bool,
    pub secondary_button_context_menu: bool,

    #[egui_probe(skip)]
    pub style: Option<egui_dock::Style>,
}

impl Default for TabsSettings {
    fn default() -> Self {
        Self {
            draggable_tabs: true,
            show_tab_name_on_hover: true,
            allowed_splits: AllowedSplits::All,
            show_close_buttons: true,
            show_add_buttons: true,
            show_leaf_close_all_buttons: true,
            show_leaf_collapse_buttons: true,
            show_secondary_button_hint: true,
            secondary_button_on_modifier: false,
            secondary_button_context_menu: false,
            style: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, egui_probe::EguiProbe)]
pub struct AppSettings {
    pub edit_mode: EditMode,
    pub editing_nodes_opacity: f32,
    pub viewing_nodes_opacity: f32,
    pub show_nodes: bool,
    pub animation_time: f32,
    pub tabs: TabsSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            edit_mode: EditMode::Editing,
            editing_nodes_opacity: 1.0,
            viewing_nodes_opacity: 0.5,
            show_nodes: true,
            animation_time: 0.2,
            tabs: Default::default(),
        }
    }
}
