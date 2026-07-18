use mavlink::dialects::common::MavMessage;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::{Receiver, Sender};
use std::sync::Arc;

type Conn = Arc<Box<dyn mavlink::AsyncMavConnection<MavMessage> + Send + Sync>>;

pub enum Command {
    Connect(String),
    Disconnect,
}

#[derive(Default, PartialEq, Clone, Debug)]
pub enum ConnStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

#[derive(Default, Clone)]
pub struct Telemetry {
    pub attitude: Option<mavlink::dialects::common::ATTITUDE_DATA>,
    pub gps_raw_int: Option<mavlink::dialects::common::GPS_RAW_INT_DATA>,
    pub global_position_int: Option<mavlink::dialects::common::GLOBAL_POSITION_INT_DATA>,
    pub heartbeat: Option<mavlink::dialects::common::HEARTBEAT_DATA>,
    pub sys_status: Option<mavlink::dialects::common::SYS_STATUS_DATA>,
}

impl Telemetry {
    pub fn update(msg: &MavMessage) -> Self {
        let mut telemetry = Telemetry::default();
        match msg {
            MavMessage::ATTITUDE(data) => telemetry.attitude = Some(data.clone()),
            MavMessage::GPS_RAW_INT(data) => telemetry.gps_raw_int = Some(data.clone()),
            MavMessage::GLOBAL_POSITION_INT(data) => telemetry.global_position_int = Some(data.clone()),
            MavMessage::HEARTBEAT(data) => telemetry.heartbeat = Some(data.clone()),
            MavMessage::SYS_STATUS(data) => telemetry.sys_status = Some(data.clone()),
            _ => {}
        }
        telemetry
    }
}

struct Ui {
    status_tx: UnboundedSender<ConnStatus>,
    telemetry_tx: Sender<Telemetry>,
    ctx: eframe::egui::Context,
}

impl Ui {
    fn set(&self, status: ConnStatus) {
        let _ = self.status_tx.send(status);
        self.ctx.request_repaint();
    }
}

pub async fn run(
    mut cmd_rx: UnboundedReceiver<Command>,
    status_tx: UnboundedSender<ConnStatus>,
    telemetry_tx: Sender<Telemetry>,
    ctx: eframe::egui::Context,
) {
    let ui = Ui { status_tx, telemetry_tx, ctx };

    while let Some(url) = wait_for_connect(&mut cmd_rx).await {
        match connect(&url).await {
            Ok(conn) => {
                ui.set(ConnStatus::Connecting);
                connected(conn, &mut cmd_rx, &ui).await;
                ui.set(ConnStatus::Disconnected);
            }
            Err(e) => ui.set(ConnStatus::Failed(e.to_string())),
        }
    }
}

async fn wait_for_connect(cmd_rx: &mut UnboundedReceiver<Command>) -> Option<String> {
    while let Some(cmd) = cmd_rx.recv().await {
        if let Command::Connect(url) = cmd { return Some(url); }
    }
    None // channel closed
}

/// Open the link and hand back a shareable handle. `set_protocol_version` is the
/// only `&mut` call, so we do it here, before wrapping in `Arc`.
async fn connect(url: &str) -> Result<Conn, Box<dyn std::error::Error>> {
    let mut mavconn = mavlink::connect_async::<MavMessage>(url).await?;
    mavconn.set_protocol_version(mavlink::MavlinkVersion::V2);
    Ok(Arc::new(mavconn))
}

async fn connected(conn: Conn, cmd_rx: &mut UnboundedReceiver<Command>, ui: &Ui) {
    let _ = conn.send_default(&request_parameters()).await;
    let _ = conn.send_default(&request_stream()).await;

    loop {
        tokio::select! {
            incoming = conn.recv() => match incoming {
                Ok((_header, msg)) => {
                    ui.set(ConnStatus::Connected);
                    let telemetry = Telemetry::update(&msg);
                    let _ = ui.telemetry_tx.send(telemetry);
                    println!("Received message: {:?}", msg);
                }
                Err(e) => {
                    println!("Link error, disconnecting: {}", e);
                    break; // link dropped
                }
            },
            cmd = cmd_rx.recv() => match cmd {
                Some(Command::Disconnect) | None => break, // disconnect or channel closed
                Some(Command::Connect(_)) => {} // ignore, already connected
            }
        }
    }
}
/// Create a message requesting the parameters list
pub fn request_parameters() -> MavMessage {
    MavMessage::PARAM_REQUEST_LIST(
        mavlink::dialects::common::PARAM_REQUEST_LIST_DATA {
            target_system: 0,
            target_component: 0,
        },
    )
}

/// Create a message enabling data streaming
pub fn request_stream() -> MavMessage {
    #[expect(deprecated)]
    MavMessage::REQUEST_DATA_STREAM(
        mavlink::dialects::common::REQUEST_DATA_STREAM_DATA {
            target_system: 0,
            target_component: 0,
            req_stream_id: mavlink::dialects::common::MavDataStream::MAV_DATA_STREAM_ALL as u8,
            req_message_rate: 10,
            start_stop: 1,
        },
    )
}
