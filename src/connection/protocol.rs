use super::{Conn, Telemetry, VehicleCommand, GCS_HEADER};
use mavlink::dialects::common::MavMessage;

pub async fn handle_vehicle_command(
    conn: &Conn,
    telemetry: &Telemetry,
    cmd: VehicleCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    use mavlink::dialects::common::{MavCmd, MavModeFlag};

    let target_system = telemetry.system_id.unwrap_or(1);

    let msg = match cmd {
        VehicleCommand::Arm => {
            command_long(target_system, MavCmd::MAV_CMD_COMPONENT_ARM_DISARM, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
        }
        VehicleCommand::Disarm => {
            command_long(target_system, MavCmd::MAV_CMD_COMPONENT_ARM_DISARM, [0.0; 7])
        }
        VehicleCommand::Takeoff { altitude } => {
            let target_amsl = telemetry.altitude_m().unwrap_or(0.0) + altitude;
            command_long(target_system, MavCmd::MAV_CMD_NAV_TAKEOFF, [0.0, 0.0, 0.0, f32::NAN, f32::NAN, f32::NAN, target_amsl])
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

pub fn gcs_heartbeat() -> MavMessage {
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
