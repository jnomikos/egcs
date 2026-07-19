use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::Sender;
use mavlink::dialects::common::MavMessage;

use crate::telemetry::Telemetry;
use super::{
    Conn, ConnStatus, Command, GCS_HEADER
};

use super::protocol::{handle_vehicle_command, request_parameters, request_stream, gcs_heartbeat};

pub async fn run(
    mut cmd_rx: UnboundedReceiver<Command>,
    status_tx: UnboundedSender<ConnStatus>,
    telemetry_tx: Sender<Telemetry>,
    ctx: eframe::egui::Context,
) {
    let app_handle = AppHandle { status_tx, telemetry_tx, ctx };

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
    status_tx: UnboundedSender<ConnStatus>,
    telemetry_tx: Sender<Telemetry>,
    ctx: eframe::egui::Context,
}

impl AppHandle {
    fn set(&self, status: ConnStatus) {
        let _ = self.status_tx.send(status);
        self.ctx.request_repaint();
    }
}

async fn connect(url: &str) -> Result<Conn, Box<dyn std::error::Error>> {
    let mut mavconn = mavlink::connect_async::<MavMessage>(url).await?;
    mavconn.set_protocol_version(mavlink::MavlinkVersion::V2);
    Ok(Arc::new(mavconn))
}

async fn connected(conn: Conn, cmd_rx: &mut UnboundedReceiver<Command>, app_handle: &AppHandle) {
    let _ = conn.send(&GCS_HEADER, &request_parameters()).await;
    let _ = conn.send(&GCS_HEADER, &request_stream()).await;
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        tokio::select! {
            incoming = conn.recv() => match incoming {
                Ok((header, msg)) => {
                    use mavlink::dialects::common::MavComponent;
                    if header.component_id == MavComponent::MAV_COMP_ID_AUTOPILOT1 as u8 {
                        app_handle.set(ConnStatus::Connected);
                        app_handle.telemetry_tx.send_modify(|t| t.update(&header, &msg));
                    }
                }
                Err(e) => {
                    println!("Link error, disconnecting: {}", e);
                    break; // link dropped
                }
            },
            cmd = cmd_rx.recv() => match cmd {
                Some(Command::Disconnect) | None => break, // disconnect or channel closed
                Some(Command::Connect(_)) => {} // ignore, already connected
                Some(Command::Vehicle(vehicle_cmd)) => {
                    let telemetry = app_handle.telemetry_tx.borrow().clone();
                    if let Err(e) = handle_vehicle_command(&conn, &telemetry, vehicle_cmd).await {
                        println!("Command failed: {e}");
                    }
                }
            },
            _ = heartbeat.tick() => {
                let _ = conn.send(&GCS_HEADER, &gcs_heartbeat()).await;
            }
        }
    }
}

async fn wait_for_connect(cmd_rx: &mut UnboundedReceiver<Command>) -> Option<String> {
    while let Some(cmd) = cmd_rx.recv().await {
        if let Command::Connect(url) = cmd { return Some(url); }
    }
    None // channel closed
}