use eframe::{App, CreationContext};
use egui::{Id, Key, LayerId, Order, Sense, UiBuilder};
use egui_dock::{AllowedSplits, DockArea, DockState, SurfaceIndex, TabViewer};
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

pub struct AppContext {
    settings: AppSettings,
}

impl TabViewer for AppContext {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Viewport(_) => {
                ui.input(|i| {
                    if i.key_pressed(Key::Tab) {
                        self.settings.edit_mode.switch();
                    }
                    if i.key_pressed(Key::H) {
                        self.settings.show_nodes = !self.settings.show_nodes;
                    }
                });

                let last_panel_rect = ui.min_rect();

                // Render area in the background
                let render_area_ui = ui.new_child(
                    UiBuilder::new()
                        .layer_id(LayerId::new(Order::Background, Id::new("render_area")))
                        .max_rect(last_panel_rect)
                        .sense(Sense::empty()),
                );

                if let EditMode::View = self.settings.edit_mode {
                    // Overlay mouse blocker in the foreground
                    let overlay_area_ui = ui.new_child(
                        UiBuilder::new()
                            .layer_id(LayerId::new(Order::Foreground, Id::new("overlay_area")))
                            .max_rect(last_panel_rect)
                            .sense(Sense::empty()),
                    );
                    let overlay_response =
                        overlay_area_ui.interact(last_panel_rect, Id::new("overlay_blocker"), Sense::click_and_drag());
                }
            },
            Tab::Settings(_) => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui_probe::Probe::new(&mut self.settings).show(ui);
                });
            },
        }
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

        Self {
            ctx: AppContext { settings },
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
        let settings = serde_json::to_string(&self.ctx.settings).unwrap();
        storage.set_string("settings", settings);

        let tabs_tree = serde_json::to_string(&self.tabs_tree).unwrap();
        storage.set_string("tabs_tree", tabs_tree);
    }
}
