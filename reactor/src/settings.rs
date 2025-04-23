use eframe::wgpu::naga::FastHashMap;
use egui_dock::AllowedSplits;
use egui_snarl::ui::{NodeLayout, PinPlacement, SnarlStyle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Deserialize, Serialize, egui_probe::EguiProbe)]
pub enum EditMode {
    #[default]
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
            show_add_buttons: false,
            show_leaf_close_all_buttons: false,
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
    pub edit_modes: FastHashMap<String, EditMode>,
    pub editing_nodes_opacity: f32,
    pub viewing_nodes_opacity: f32,
    pub show_nodes: bool,
    pub animation_time: f32,
    pub tabs: TabsSettings,
    pub snarl_style: SnarlStyle,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            edit_modes: Default::default(),
            editing_nodes_opacity: 1.0,
            viewing_nodes_opacity: 0.5,
            show_nodes: true,
            animation_time: 0.2,
            tabs: Default::default(),
            snarl_style: default_snarl_style(),
        }
    }
}

const fn default_snarl_style() -> SnarlStyle {
    SnarlStyle {
        node_layout: Some(NodeLayout::FlippedSandwich),
        pin_placement: Some(PinPlacement::Edge),
        pin_size: Some(7.0),
        node_frame: Some(egui::Frame {
            inner_margin: egui::Margin::same(8),
            outer_margin: egui::Margin {
                left: 0,
                right: 0,
                top: 0,
                bottom: 4,
            },
            corner_radius: egui::CornerRadius::same(8),
            fill: egui::Color32::from_gray(30),
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        bg_frame: Some(egui::Frame {
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            corner_radius: egui::CornerRadius::ZERO,
            fill: egui::Color32::TRANSPARENT,
            stroke: egui::Stroke::NONE,
            shadow: egui::Shadow::NONE,
        }),
        crisp_magnified_text: Some(true),
        ..SnarlStyle::new()
    }
}
