use std::collections::HashSet;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::{Receiver};
use crate::telemetry::Telemetry;
use crate::connection::{self, ConnStatus};
use crate::theme::*;
use egui::{
    Ui, WidgetText
};
use egui_dock::{
    AllowedSplits, DockArea, DockState, NodeIndex, NodePath, OverlayType, Style, SurfaceIndex,
    TabInteractionStyle, TabViewer, tab_viewer::OnCloseResponse,
};

#[derive(serde::Deserialize, serde::Serialize)]
struct DockContext {
    pub style: Option<Style>,
    open_tabs: HashSet<String>,
    show_close_buttons: bool,
    show_add_buttons: bool,
    draggable_tabs: bool,
    show_tab_name_on_hover: bool,
    #[serde(skip)]
    allowed_splits: AllowedSplits,
    show_leaf_close_all: bool,
    show_leaf_collapse: bool,
    show_secondary_button_hint: bool,
    secondary_button_on_modifier: bool,
    secondary_button_context_menu: bool,

    connection_url: String,
    #[serde(skip)]
    conn_status: ConnStatus,
    #[serde(skip)]
    cmd_tx: Option<UnboundedSender<connection::Command>>,
    #[serde(skip)]
    status_rx: Option<UnboundedReceiver<ConnStatus>>,
    #[serde(skip)]
    telemetry_rx: Option<Receiver<Telemetry>>,
}

impl DockContext {
    fn actions(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let connected = self.conn_status == ConnStatus::Connected;
            if !connected {
                ui.label("Not connected to UAV.");
                return;
            }

            let Some(rx) = &mut self.telemetry_rx else {
                ui.label("No telemetry data available.");
                return;
            };

            let rx_borrowed = rx.borrow();

            let armed = rx_borrowed.armed().unwrap_or(false);
            let flying = rx_borrowed.is_flying().unwrap_or(false);
            let landing = rx_borrowed.is_landing().unwrap_or(false);

            if armed && !flying {
                if action_button(ui, "Disarm", RED) {
                    if let Some(tx) = &self.cmd_tx {
                        let _ = tx.send(connection::Command::Vehicle(connection::VehicleCommand::Disarm));
                    }
                }

                if action_button(ui, "Takeoff", BLUE) {
                    if let Some(tx) = &self.cmd_tx {
                        let _ = tx.send(connection::Command::Vehicle(connection::VehicleCommand::Takeoff { altitude: 20.0 }));
                    }
                }
            } else if armed && flying && !landing {
                if action_button(ui, "Land", AMBER) {
                    if let Some(tx) = &self.cmd_tx {
                        let _ = tx.send(connection::Command::Vehicle(connection::VehicleCommand::Land));
                    }
                }
            } else if !armed {
                if action_button(ui, "Arm", GREEN) {
                    if let Some(tx) = &self.cmd_tx {
                        let _ = tx.send(connection::Command::Vehicle(connection::VehicleCommand::Arm));
                    }
                }
            }
            
            Self::flight_mode_selector(ui, &rx_borrowed, &self.cmd_tx);
            
        });
    }

    fn flight_mode_selector(
        ui: &mut Ui,
        telemetry: &Telemetry,
        cmd_tx: &Option<UnboundedSender<connection::Command>>,
    ) {
        let current = telemetry.current_selector();

        egui::ComboBox::from_label("")
            .width(100.0)
            .selected_text(telemetry.current_mode_name().unwrap_or("Unknown"))
            .show_ui(ui, |ui| {
                for mode in telemetry.selectable_modes() {
                    if ui.selectable_label(current == Some(mode.selector), &mode.name).clicked() {
                        if let Some(tx) = cmd_tx {
                            let _ = tx.send(connection::Command::Vehicle(
                                connection::VehicleCommand::SetMode(mode.selector),
                            ));
                        }
                    }
                }
            });
    }

    fn comm_link(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let connected = self.conn_status == ConnStatus::Connected;
            let connecting = self.conn_status == ConnStatus::Connecting;

            ui.label("UAV URL:");
            ui.add_enabled(
                !connected && !connecting,
                egui::TextEdit::singleline(&mut self.connection_url),
            );

            let conn_label = match self.conn_status {
                ConnStatus::Connecting => "Connecting…",
                _ => "Connect",
            };
            if !connected {
                if ui.add_enabled(!connecting, egui::Button::new(conn_label)).clicked() {
                    if let Some(tx) = &self.cmd_tx {
                        let _ = tx.send(connection::Command::Connect(self.connection_url.clone()));
                    }
                    self.conn_status = ConnStatus::Connecting;
                }
            }
            if connected || connecting {
                if ui.button("Disconnect").clicked() {
                    if let Some(tx) = &self.cmd_tx {
                        let _ = tx.send(connection::Command::Disconnect);
                    }
                }
            }
        });
    }

    fn telemetry(&mut self, ui: &mut Ui) {
        let connected = self.conn_status == ConnStatus::Connected;
        if !connected {
            ui.label("Not connected to UAV.");
            return;
        }

        let Some(rx) = &mut self.telemetry_rx else {
            ui.label("No telemetry data available.");
            return;
        };
        let telemetry = rx.borrow();

        let armed = telemetry.armed().unwrap_or(false);
        let flying = telemetry.is_flying().unwrap_or(false);
        let landing = telemetry.is_landing().unwrap_or(false);
        let vtol_fw = telemetry.vtol_in_forward_flight().unwrap_or(false);

        let alt = telemetry.altitude_m().unwrap_or(0.0);
        let rel = telemetry.relative_altitude_m().unwrap_or(0.0);
        let gs = telemetry.ground_speed_mps().unwrap_or(0.0);
        let vs = telemetry.vertical_speed_mps().unwrap_or(0.0);
        let mode = telemetry.current_mode_name();

        egui::Grid::new("telemetry_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                stat_tile(ui, "ALTITUDE (m)", format!("{alt:.1}"), TEAL);
                stat_tile(ui, "REL ALT (m)", format!("{rel:.1}"), TEAL);
                ui.end_row();

                stat_tile(ui, "GROUNDSPEED (m/s)", format!("{gs:.1}"), BLUE);
                stat_tile(ui, "VERT SPEED (m/s)", format!("{vs:.1}"), BLUE);
                ui.end_row();

                stat_tile(
                    ui,
                    "FLIGHT MODE",
                    mode.unwrap_or("Unknown"),
                    AMBER,
                );
                stat_tile(
                    ui,
                    "STATUS",
                    if armed {
                        if flying {
                            "Flying"
                        } else if landing {
                            "Landing"
                        } else if vtol_fw {
                            "VTOL FW"
                        } else {
                            "Armed"
                        }
                    } else {
                        "Disarmed"
                    },
                    if armed { GREEN } else { RED },
                );
                ui.end_row();
            });
    }

    fn map(&mut self, ui: &mut Ui) {
        ui.label("Map coming soon!");
    }
}

impl TabViewer for DockContext {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Map" => self.map(ui),
            "Comm Link" => self.comm_link(ui),
            "Telemetry" => self.telemetry(ui),
            "Actions" => self.actions(ui),
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        self.open_tabs.contains(tab)
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> OnCloseResponse {
        self.open_tabs.remove(tab);
        OnCloseResponse::Close
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EgcsApp {
    #[serde(skip)]
    dock_context: DockContext,
    #[serde(skip)]
    tree: DockState<String>,
}

impl Default for EgcsApp {
    fn default() -> Self {
        let mut dock_state =
            DockState::new(vec!["Map".to_owned()]);
        "Undock".clone_into(&mut dock_state.translations.tab_context_menu.eject_button);
        let [a, b] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.5,
            vec!["Comm Link".to_owned(), "Actions".to_owned()],
        );
        let [_, _] = dock_state.main_surface_mut().split_below(
            b,
            0.4,
            vec!["Telemetry".to_owned()],
        );

        let mut open_tabs = HashSet::new();

        for node in dock_state[SurfaceIndex::main()].iter() {
            if let Some(tabs) = node.tabs() {
                for tab in tabs {
                    open_tabs.insert(tab.clone());
                }
            }
        }

        let context = DockContext {
            style: None,
            open_tabs,
            show_close_buttons: false,
            show_add_buttons: false,
            draggable_tabs: false,
            show_tab_name_on_hover: true,
            allowed_splits: AllowedSplits::All,
            show_leaf_close_all: false,
            show_leaf_collapse: true,
            show_secondary_button_hint: true,
            secondary_button_on_modifier: false,
            secondary_button_context_menu: false,
            connection_url: "udpin:0.0.0.0:14550".to_owned(),
            conn_status: ConnStatus::Disconnected,
            cmd_tx: None,
            status_rx: None,
            telemetry_rx: None,
        };
        Self {
            dock_context: context,
            tree: dock_state,
        }
    }
}

impl EgcsApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel::<connection::Command>();
        let (status_tx, status_rx) = tokio::sync::mpsc::unbounded_channel::<ConnStatus>();
        // Telemetry is receive only and thus should be a watch
        let (telemetry_tx, _telemetry_rx) = tokio::sync::watch::channel::<Telemetry>(Telemetry::default());
        
        let ctx = cc.egui_ctx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
            rt.block_on(connection::run(cmd_rx, status_tx, telemetry_tx, ctx));
        });

        let mut app: EgcsApp = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default();

        app.dock_context.cmd_tx = Some(cmd_tx);
        app.dock_context.status_rx = Some(status_rx);
        app.dock_context.telemetry_rx = Some(_telemetry_rx);
        app
    }
}

impl eframe::App for EgcsApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        while let Some(status) = self.dock_context.status_rx.as_mut().and_then(|rx| rx.try_recv().ok()) {
            self.dock_context.conn_status = status;
        }

        let style = self
            .dock_context
            .style
            .get_or_insert(Style::from_egui(ui.style()))
            .clone();

        DockArea::new(&mut self.tree)
            .style(style)
            .show_close_buttons(self.dock_context.show_close_buttons)
            .show_add_buttons(self.dock_context.show_add_buttons)
            .draggable_tabs(self.dock_context.draggable_tabs)
            .show_tab_name_on_hover(self.dock_context.show_tab_name_on_hover)
            .allowed_splits(self.dock_context.allowed_splits)
            .show_leaf_close_all_buttons(self.dock_context.show_leaf_close_all)
            .show_leaf_collapse_buttons(self.dock_context.show_leaf_collapse)
            .show_secondary_button_hint(self.dock_context.show_secondary_button_hint)
            .secondary_button_on_modifier(self.dock_context.secondary_button_on_modifier)
            .secondary_button_context_menu(self.dock_context.secondary_button_context_menu)
            .show_inside(ui, &mut self.dock_context);
    }
}
