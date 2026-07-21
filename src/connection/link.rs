use mavlink::dialects::common::MavMessage;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::watch::Sender;

use super::{Command, Conn, ConnStatus, GCS_HEADER};
use crate::telemetry::Telemetry;

use super::protocol::{
    gcs_heartbeat, handle_vehicle_command, request_available_modes, request_parameters,
    request_stream,
};

pub async fn run(
    mut cmd_rx: UnboundedReceiver<Command>,
    status_tx: Sender<ConnStatus>,
    telemetry_tx: Sender<Telemetry>,
    ctx: eframe::egui::Context,
) {
    let app_handle = AppHandle {
        status_tx,
        telemetry_tx,
        ctx,
    };

    while let Some(url) = wait_for_connect(&mut cmd_rx).await {
        match connect(&url).await {
            Ok(conn) => {
                app_handle.set(ConnStatus::Connecting);
                connected(conn, &mut cmd_rx, &app_handle).await;
                app_handle.set(ConnStatus::Disconnected);
            }
            Err(e) => app_handle.set(ConnStatus::Failed(e.to_string())),
        }
    }
}

struct AppHandle {
    status_tx: Sender<ConnStatus>,
    telemetry_tx: Sender<Telemetry>,
    ctx: eframe::egui::Context,
}

impl AppHandle {
    fn set(&self, status: ConnStatus) {
        self.status_tx.send_replace(status);
        self.ctx.request_repaint();
    }
}

async fn connect(url: &str) -> Result<Conn, Box<dyn std::error::Error>> {
    let mut mavconn = mavlink::connect_async::<MavMessage>(url).await?;
    mavconn.set_protocol_version(mavlink::MavlinkVersion::V2);
    Ok(Arc::new(mavconn))
}

async fn connected(conn: Conn, cmd_rx: &mut UnboundedReceiver<Command>, app_handle: &AppHandle) {
    conn.send(&GCS_HEADER, &request_parameters()).await.ok();
    conn.send(&GCS_HEADER, &request_stream()).await.ok();
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut modes_requested = false;
    loop {
        tokio::select! {
            incoming = conn.recv() => match incoming {
                Ok((header, msg)) => {
                    use mavlink::dialects::common::MavComponent;
                    if header.component_id == MavComponent::MAV_COMP_ID_AUTOPILOT1 as u8 {
                        app_handle.set(ConnStatus::Connected);
                        app_handle.telemetry_tx.send_modify(|t| t.update(&header, &msg));

                        // Fetch the mode list once we know the target system, and again
                        // whenever the vehicle signals the set has changed.
                        let target_system = app_handle.telemetry_tx.borrow().system_id;
                        let modes_changed = matches!(msg, MavMessage::AVAILABLE_MODES_MONITOR(_));
                        if let Some(sys) = target_system
                            && (!modes_requested || modes_changed)
                        {
                            conn.send(&GCS_HEADER, &request_available_modes(sys))
                                .await
                                .ok();
                            modes_requested = true;
                        }
                    }
                }
                Err(e) => {
                    log::error!("Link error, disconnecting: {e}");
                    break; // link dropped
                }
            },
            cmd = cmd_rx.recv() => match cmd {
                Some(Command::Disconnect) | None => break, // disconnect or channel closed
                Some(Command::Connect(_)) => {} // ignore, already connected
                Some(Command::Vehicle(vehicle_cmd)) => {
                    let telemetry = app_handle.telemetry_tx.borrow().clone();
                    if let Err(e) = handle_vehicle_command(&conn, &telemetry, vehicle_cmd).await {
                        log::error!("Command failed: {e}");
                    }
                }
            },
            _ = heartbeat.tick() => {
                conn.send(&GCS_HEADER, &gcs_heartbeat()).await.ok();
            }
        }
    }
}

async fn wait_for_connect(cmd_rx: &mut UnboundedReceiver<Command>) -> Option<String> {
    while let Some(cmd) = cmd_rx.recv().await {
        if let Command::Connect(url) = cmd {
            return Some(url);
        }
    }
    None // channel closed
}
