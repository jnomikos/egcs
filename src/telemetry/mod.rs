mod position;
mod vehicle_state;

use mavlink::dialects::common::MavMessage;
use mavlink::MavHeader;

#[derive(Default, Clone)]
pub struct Telemetry {
    pub extended_sys_state: Option<mavlink::dialects::common::EXTENDED_SYS_STATE_DATA>,
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
            MavMessage::HEARTBEAT(data) => {
                self.heartbeat = Some(data.clone());
                self.system_id = Some(header.system_id);
                self.component_id = Some(header.component_id);
            }
            MavMessage::SYS_STATUS(data) => self.sys_status = Some(data.clone()),
            MavMessage::EXTENDED_SYS_STATE(data) => self.extended_sys_state = Some(data.clone()),
            MavMessage::COMMAND_ACK(data) => {
                println!("Command ACK: {:?}", data);
            }
            _ => {}
        }
    }
}