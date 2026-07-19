use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::{Receiver, Sender};
use crate::connection::Telemetry;
use crate::connection::{self, ConnStatus};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EgcsApp {
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

impl Default for EgcsApp {
    fn default() -> Self {
        Self {
            connection_url: "udpin:0.0.0.0:14550".to_owned(),
            conn_status: ConnStatus::Disconnected,
            cmd_tx: None,
            status_rx: None,
            telemetry_rx: None,
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
        let (telemetry_tx, _telemetry_rx) = tokio::sync::watch::channel::<connection::Telemetry>(connection::Telemetry::default());
        
        let ctx = cc.egui_ctx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
            rt.block_on(connection::run(cmd_rx, status_tx, telemetry_tx, ctx));
        });

        let mut app: EgcsApp = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default();

        app.cmd_tx = Some(cmd_tx);
        app.status_rx = Some(status_rx);
        app.telemetry_rx = Some(_telemetry_rx);
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

        while let Some(status) = self.status_rx.as_mut().and_then(|rx| rx.try_recv().ok()) {
            self.conn_status = status;
        }

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.horizontal(|ui| {
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

                if let ConnStatus::Failed(e) = &self.conn_status {
                    ui.colored_label(egui::Color32::RED, format!("Failed: {e}"));
                }
            });
        });
    }
}
