use eframe::{App, CreationContext};
use egui::{Key, LayerId, Order, Sense, UiBuilder};
use egui_dock::{DockArea, DockState, SurfaceIndex, TabViewer};
use egui_snarl::Snarl;
use egui_snarl::ui::SnarlWidget;
use serde::{Deserialize, Serialize};

use crate::node::Node;
use crate::node::viewer::NodeViewer;
use crate::settings::{AppSettings, EditMode};
use crate::tabs::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[repr(u32)]
pub enum UiIdKey {
    Nodes = 0,
    RenderArea,
    EditingArea,
    OverlayArea,
    OverlayBlocker,
}

pub struct AppContext {
    settings: AppSettings,
    snarl: Snarl<Node>,
    viewer: NodeViewer,
}

impl AppContext {
    pub fn open_tab(&mut self, tab: &Tab) {
        self.settings
            .edit_modes
            .insert(tab.title().to_string(), EditMode::default());
        self.viewer.open_tab(tab, &mut self.snarl);
    }

    pub fn close_tab(&mut self, tab: &Tab) {
        self.settings.edit_modes.remove(tab.title());
        self.viewer.close_tab(tab, &mut self.snarl);
    }
}

impl TabViewer for AppContext {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Viewport(tab) => {
                ui.input(|i| {
                    if i.modifiers.ctrl && i.key_pressed(Key::Tab) {
                        self.settings.edit_modes.get_mut(tab.title()).map(|mode| mode.switch());
                    }
                    if i.key_pressed(Key::H) {
                        self.settings.show_nodes = !self.settings.show_nodes;
                    }
                });

                let last_panel_rect = ui.min_rect();

                // Render area in the background
                let render_area_ui = ui.new_child(
                    UiBuilder::new()
                        .layer_id(LayerId::new(Order::Background, tab.id(UiIdKey::RenderArea)))
                        .max_rect(last_panel_rect)
                        .sense(Sense::empty()),
                );
                self.viewer
                    .draw(tab, &last_panel_rect, render_area_ui.painter(), &mut self.snarl);

                if self.settings.show_nodes {
                    // Editing area with nodes in the middle
                    let mut editing_area_ui = ui.new_child(
                        UiBuilder::new()
                            .layer_id(LayerId::new(Order::Middle, tab.id(UiIdKey::EditingArea)))
                            .max_rect(last_panel_rect)
                            .sense(Sense::empty()),
                    );

                    editing_area_ui.set_max_size(last_panel_rect.size());

                    let opacity = match self.settings.edit_modes.get(tab.title()) {
                        Some(EditMode::Editing) => self.settings.editing_nodes_opacity,
                        Some(EditMode::View) => self.settings.viewing_nodes_opacity,
                        None => 1.0,
                    };
                    editing_area_ui.set_opacity(opacity);

                    SnarlWidget::new()
                        .id(tab.id(UiIdKey::Nodes))
                        .style(self.settings.snarl_style)
                        .show(&mut self.snarl, &mut self.viewer, &mut editing_area_ui);
                }

                if let Some(EditMode::View) = self.settings.edit_modes.get(tab.title()) {
                    // Overlay mouse blocker in the foreground
                    let mut render_area_ui = ui.new_child(
                        UiBuilder::new()
                            .layer_id(LayerId::new(Order::Foreground, tab.id(UiIdKey::OverlayArea)))
                            .max_rect(last_panel_rect)
                            .sense(Sense::empty()),
                    );

                    render_area_ui.set_max_size(last_panel_rect.size());

                    let overlay_response = render_area_ui.interact(
                        last_panel_rect,
                        tab.id(UiIdKey::OverlayBlocker),
                        Sense::click_and_drag(),
                    );

                    self.viewer.after_show(tab, ui, &overlay_response, &mut self.snarl);
                }
            },
            Tab::Settings(_) => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui_probe::Probe::new(&mut self.settings).show(ui);
                });
            },
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.close_tab(tab);
        true
    }
}

pub struct Reactor3dApp {
    ctx: AppContext,
    tabs_tree: DockState<Tab>,
}

impl Reactor3dApp {
    pub fn new(cx: &CreationContext) -> Self {
        egui_extras::install_image_loaders(&cx.egui_ctx);

        cx.egui_ctx.style_mut(|style| style.animation_time *= 10.0);

        let mut snarl = cx.storage.map_or_else(Snarl::new, |storage| {
            storage
                .get_string("snarl")
                .and_then(|snarl| serde_json::from_str(&snarl).ok())
                .unwrap_or_default()
        });

        let settings = cx.storage.map_or_else(AppSettings::default, |storage| {
            storage
                .get_string("settings")
                .and_then(|settings| serde_json::from_str(&settings).ok())
                .unwrap_or_default()
        });

        let tabs_tree = cx
            .storage
            .and_then(|storage| {
                storage
                    .get_string("tabs_tree")
                    .and_then(|tabs_tree| serde_json::from_str(&tabs_tree).ok())
            })
            .unwrap_or_else(|| DockState::<Tab>::new(Default::default()));

        let screen_rect = cx.egui_ctx.input(|i| i.screen_rect());
        let max_viewport_resolution = (screen_rect.width() * screen_rect.height() / 10.0) as u32;
        println!("Max resolution: {max_viewport_resolution}");

        let viewer = NodeViewer::new(
            cx.wgpu_render_state.clone().expect("WGPU must be enabled"),
            max_viewport_resolution,
            tabs_tree.iter_all_tabs().map(|((..), tab)| tab),
            &mut snarl,
        );

        Self {
            ctx: AppContext {
                settings,
                snarl,
                viewer,
            },
            tabs_tree,
        }
    }
}

impl App for Reactor3dApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                ui.menu_button("View", |ui| {
                    for mut tab in [Tab::new_viwport(), Tab::new_settings()] {
                        if ui.button(tab.title()).clicked() {
                            while self.tabs_tree.find_tab(&tab).is_some() {
                                tab.increment_title();
                            }
                            self.ctx.open_tab(&tab);
                            self.tabs_tree[SurfaceIndex::main()].push_to_focused_leaf(tab);

                            ui.close_menu();
                        }
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_switch(ui);
            });
        });

        ctx.style_mut(|style| style.animation_time = self.ctx.settings.animation_time);

        egui::CentralPanel::default().show(ctx, |ui| {
            let style = self
                .ctx
                .settings
                .tabs
                .style
                .get_or_insert(egui_dock::Style::from_egui(ui.style()))
                .clone();

            DockArea::new(&mut self.tabs_tree)
                .style(style)
                .show_close_buttons(self.ctx.settings.tabs.show_close_buttons)
                .show_add_buttons(self.ctx.settings.tabs.show_add_buttons)
                .draggable_tabs(self.ctx.settings.tabs.draggable_tabs)
                .show_tab_name_on_hover(self.ctx.settings.tabs.show_tab_name_on_hover)
                .allowed_splits(self.ctx.settings.tabs.allowed_splits)
                .show_leaf_close_all_buttons(self.ctx.settings.tabs.show_leaf_close_all_buttons)
                .show_leaf_collapse_buttons(self.ctx.settings.tabs.show_leaf_collapse_buttons)
                .show_secondary_button_hint(self.ctx.settings.tabs.show_secondary_button_hint)
                .secondary_button_on_modifier(self.ctx.settings.tabs.secondary_button_on_modifier)
                .secondary_button_context_menu(self.ctx.settings.tabs.secondary_button_context_menu)
                .show_inside(ui, &mut self.ctx);
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let snarl = serde_json::to_string(&self.ctx.snarl).unwrap();
        storage.set_string("snarl", snarl);

        let settings = serde_json::to_string(&self.ctx.settings).unwrap();
        storage.set_string("settings", settings);

        let tabs_tree = serde_json::to_string(&self.tabs_tree).unwrap();
        storage.set_string("tabs_tree", tabs_tree);
    }
}
