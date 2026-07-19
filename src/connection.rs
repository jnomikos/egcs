use mavlink::dialects::common::MavMessage;
use mavlink::MavHeader;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::Sender;
use std::sync::Arc;
use crate::flight_mode::*;

type Conn = Arc<Box<dyn mavlink::AsyncMavConnection<MavMessage> + Send + Sync>>;

const GCS_HEADER: MavHeader = MavHeader { system_id: 255, component_id: 190, sequence: 0 };


pub enum Command {
    Connect(String),
    Disconnect,
    Vehicle(VehicleCommand),
}

pub enum VehicleCommand {
    Arm,
    Disarm,
    Takeoff { altitude: f32 },
    Land,
    SetMode(FlightMode),
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
    pub system_id: Option<u8>,
    pub component_id: Option<u8>,
}

impl Telemetry {
    pub fn update(&mut self, header: &MavHeader, msg: &MavMessage) {
        match msg {
            MavMessage::ATTITUDE(data) => self.attitude = Some(data.clone()),
            MavMessage::GLOBAL_POSITION_INT(data) => self.global_position_int = Some(data.clone()),
            MavMessage::HEARTBEAT(data) => self.heartbeat = Some(data.clone()),
            MavMessage::SYS_STATUS(data) => self.sys_status = Some(data.clone()),
            MavMessage::COMMAND_ACK(data) => {
                println!("Command ACK: {:?}", data);
            }
            _ => {}
        }

        if let MavMessage::HEARTBEAT(_) = msg {
            self.system_id = Some(header.system_id);
            self.component_id = Some(header.component_id);
        }
    }

    pub fn altitude_m(&self) -> Option<f32> {
        self.global_position_int.as_ref().map(|p| p.alt as f32 / 1000.0)
    }

    pub fn relative_altitude_m(&self) -> Option<f32> {
        self.global_position_int.as_ref().map(|p| p.relative_alt as f32 / 1000.0)
    }

    pub fn armed(&self) -> Option<bool> {
        use mavlink::dialects::common::MavModeFlag;
        self.heartbeat.as_ref().map(|hb| hb.base_mode.contains(MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED))
    }

    pub fn flight_mode(&self) -> Option<FlightMode> {
        self.heartbeat.as_ref().and_then(|hb| FlightMode::from_custom_mode(hb.custom_mode))
    }

    pub fn status(&self) -> Option<&'static str> {
        use mavlink::dialects::common::MavState::*;
        self.heartbeat.as_ref().map(|hb| match hb.system_status {
            MAV_STATE_UNINIT => "Uninitialized",
            MAV_STATE_BOOT => "Booting",
            MAV_STATE_CALIBRATING => "Calibrating",
            MAV_STATE_STANDBY => "Ready",
            MAV_STATE_ACTIVE => "Active",
            MAV_STATE_CRITICAL => "Critical",
            MAV_STATE_EMERGENCY => "Emergency",
            MAV_STATE_POWEROFF => "Powering Off",
            MAV_STATE_FLIGHT_TERMINATION => "Flight Termination",
        })
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

async fn connected(conn: Conn, cmd_rx: &mut UnboundedReceiver<Command>, app_handle: &AppHandle) {
    let _ = conn.send(&GCS_HEADER, &request_parameters()).await;
    let _ = conn.send(&GCS_HEADER, &request_stream()).await;
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        tokio::select! {
            incoming = conn.recv() => match incoming {
                Ok((header, msg)) => {
                    app_handle.set(ConnStatus::Connected);
                    app_handle.telemetry_tx.send_modify(|t| t.update(&header, &msg));
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
                    let target_system = app_handle.telemetry_tx.borrow().system_id.unwrap_or(1);
                    if let Err(e) = handle_vehicle_command(&conn, target_system, vehicle_cmd).await {
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

async fn handle_vehicle_command(
    conn: &Conn,
    target_system: u8,
    cmd: VehicleCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    use mavlink::dialects::common::{MavCmd, MavModeFlag};

    let msg = match cmd {
        VehicleCommand::Arm => {
            command_long(target_system, MavCmd::MAV_CMD_COMPONENT_ARM_DISARM, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
        }
        VehicleCommand::Disarm => {
            command_long(target_system, MavCmd::MAV_CMD_COMPONENT_ARM_DISARM, [0.0; 7])
        }
        VehicleCommand::Takeoff { altitude } => {
            command_long(target_system, MavCmd::MAV_CMD_NAV_TAKEOFF, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, altitude])
        }
        VehicleCommand::Land => {
            command_long(target_system, MavCmd::MAV_CMD_NAV_LAND, [0.0; 7])
        }
        VehicleCommand::SetMode(mode) => {
            let (main, sub) = mode.main_sub();
            let base = MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED.bits() as f32;
            command_long(target_system, MavCmd::MAV_CMD_DO_SET_MODE, [base, main as f32, sub as f32, 0.0, 0.0, 0.0, 0.0])
        }
    };

    conn.send(&GCS_HEADER, &msg).await?;
    Ok(())
}

fn command_long(target_system: u8, command: mavlink::dialects::common::MavCmd, params: [f32; 7]) -> MavMessage {
    use mavlink::dialects::common::{COMMAND_LONG_DATA, MavComponent};
    MavMessage::COMMAND_LONG(COMMAND_LONG_DATA {
        param1: params[0],
        param2: params[1],
        param3: params[2],
        param4: params[3],
        param5: params[4],
        param6: params[5],
        param7: params[6],
        command,
        target_system,
        target_component: MavComponent::MAV_COMP_ID_AUTOPILOT1 as u8,
        confirmation: 0,
    })
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

fn gcs_heartbeat() -> MavMessage {
    use mavlink::dialects::common::{HEARTBEAT_DATA, MavType, MavAutopilot, MavModeFlag, MavState};
    MavMessage::HEARTBEAT(HEARTBEAT_DATA {
        custom_mode: 0,
        mavtype: MavType::MAV_TYPE_GCS,
        autopilot: MavAutopilot::MAV_AUTOPILOT_INVALID,   // a GCS has no autopilot
        base_mode: MavModeFlag::empty(),
        system_status: MavState::MAV_STATE_ACTIVE,
        mavlink_version: 3,
    })
}
