use egui::{ComboBox, Ui};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, OutPin};
use reactor_derives::Noded;
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelectedTab, SelfNodeMut};
use crate::node::viewer::ui::input;
use crate::node::{NodeFlags, Noded};
use crate::tabs::Tab;

#[derive(Clone, Default, Serialize, Deserialize, Noded)]
pub struct OutputNode {
    tab_titles: Vec<String>,
    selected_title: Option<String>,
}

impl OutputNode {
    pub const NAME: &str = "Output";
    pub const INPUTS: [u64; 1] = [NodeFlags::RENDERS.bits()];
    pub const OUTPUTS: [u64; 0] = [];

    pub fn new(tab_titles: Vec<String>, selected_title: Option<String>) -> Self {
        Self {
            tab_titles,
            selected_title,
        }
    }

    pub fn selected_title(&self) -> Option<&String> {
        self.selected_title.as_ref()
    }

    pub fn set_open_tabs<'a>(&mut self, tabs: impl Iterator<Item = &'a Tab>) {
        self.set_open_tab_titles(tabs.filter_map(|tab| {
            if let Tab::Viewport(tab) = tab {
                Some(tab.title().to_string())
            } else {
                None
            }
        }));
    }

    pub fn set_open_tab_titles(&mut self, tab_titles: impl IntoIterator<Item = String>) {
        self.tab_titles = tab_titles.into_iter().collect();
        if let Some(title) = &self.selected_title {
            if !self.contains_tab(title) {
                self.selected_title = None;
            }
        }
    }

    pub fn contains_tab(&self, title: &str) -> bool {
        self.tab_titles.iter().any(|t| t == title)
    }

    pub fn remove_tab(&mut self, title: &str) {
        self.tab_titles.retain(|t| t != title);
        if let Some(selected) = &mut self.selected_title {
            if selected == title {
                self.selected_title = None;
            }
        }
    }

    fn into_selected_tab(self_node: SelfNodeMut) -> Option<SelectedTab> {
        let SelfNodeMut { id: node_id, snarl } = self_node;

        snarl[node_id]
            .output_ref()
            .map(|output_node| output_node.selected_title.as_deref().unwrap_or_default())
            .map(|title| SelectedTab { title, node_id })
    }
}

impl MessageHandling for OutputNode {
    fn handle_display_input(_self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        Some(match pin.id.input {
            0 => input::empty_view(ui, "Output"),
            _ => unreachable!(),
        })
    }

    fn handle_display_body<'a>(
        mut self_node: SelfNodeMut<'a>,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
    ) -> Option<SelectedTab<'a>> {
        let output = self_node.node_mut().as_output_mut();

        let mut target = ComboBox::from_id_salt("Target");
        if let Some(title) = &output.selected_title {
            target = target.selected_text(title);
        }

        let mut changed_selected = false;
        ui.horizontal(|ui| {
            ui.label("Target");
            target.show_ui(ui, |ui| {
                let is_selected = output.selected_title.is_none();
                let mut response = ui.selectable_label(is_selected, "");
                if response.clicked() && !is_selected {
                    output.selected_title = None;
                    response.mark_changed();
                    changed_selected = true;
                }

                for title in &output.tab_titles {
                    let is_selected = output.selected_title.as_ref() == Some(title);
                    let mut response = ui.selectable_label(is_selected, title);
                    if response.clicked() && !is_selected {
                        output.selected_title = Some(title.clone());
                        response.mark_changed();
                        changed_selected = true;
                    }
                }
            });
        });

        if changed_selected {
            Self::into_selected_tab(self_node)
        } else {
            None
        }
    }

    fn handle_interface_open_tab(mut self_node: SelfNodeMut, tab: &Tab) {
        if let Tab::Viewport(viewport) = tab {
            let output = self_node.node_mut().as_output_mut();
            if !output.contains_tab(viewport.title()) {
                output.tab_titles.push(viewport.title().to_string());
            }
        }
    }

    fn handle_interface_close_tab(mut self_node: SelfNodeMut, tab: &Tab) {
        if let Tab::Viewport(viewport) = tab {
            let output = self_node.node_mut().as_output_mut();
            output.remove_tab(viewport.title());
        }
    }
}
