mod position;
mod vehicle_state;

use std::collections::BTreeMap;
use mavlink::dialects::common::{MavMessage, MavModeProperty, MavStandardMode, CURRENT_MODE_DATA};
use mavlink::MavHeader;

/// How a mode is commanded: standard modes use MAV_CMD_DO_SET_STANDARD_MODE,
/// custom (flight-stack-specific) modes use MAV_CMD_DO_SET_MODE.
#[derive(Clone, Copy, PartialEq)]
pub enum ModeSelector {
    Standard(u8),
    Custom(u32),
}

#[derive(Clone)]
pub struct AvailableMode {
    pub name: String,
    pub user_selectable: bool,
    pub selector: ModeSelector,
}

fn standard_mode_label(mode: MavStandardMode) -> &'static str {
    use MavStandardMode::*;
    match mode {
        MAV_STANDARD_MODE_NON_STANDARD => "Custom",
        MAV_STANDARD_MODE_POSITION_HOLD => "Position",
        MAV_STANDARD_MODE_ORBIT => "Orbit",
        MAV_STANDARD_MODE_CRUISE => "Cruise",
        MAV_STANDARD_MODE_ALTITUDE_HOLD => "Altitude",
        MAV_STANDARD_MODE_SAFE_RECOVERY => "Return",
        MAV_STANDARD_MODE_MISSION => "Mission",
        MAV_STANDARD_MODE_LAND => "Land",
        MAV_STANDARD_MODE_TAKEOFF => "Takeoff",
    }
}

fn mode_selector(standard_mode: MavStandardMode, custom_mode: u32) -> ModeSelector {
    if standard_mode == MavStandardMode::MAV_STANDARD_MODE_NON_STANDARD {
        ModeSelector::Custom(custom_mode)
    } else {
        ModeSelector::Standard(standard_mode as u8)
    }
}

#[derive(Default, Clone)]
pub struct Telemetry {
    pub extended_sys_state: Option<mavlink::dialects::common::EXTENDED_SYS_STATE_DATA>,
    pub attitude: Option<mavlink::dialects::common::ATTITUDE_DATA>,
    pub gps_raw_int: Option<mavlink::dialects::common::GPS_RAW_INT_DATA>,
    pub global_position_int: Option<mavlink::dialects::common::GLOBAL_POSITION_INT_DATA>,
    pub heartbeat: Option<mavlink::dialects::common::HEARTBEAT_DATA>,
    pub sys_status: Option<mavlink::dialects::common::SYS_STATUS_DATA>,
    pub available_modes: BTreeMap<u8, AvailableMode>,
    pub current_mode: Option<CURRENT_MODE_DATA>,
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
            MavMessage::AVAILABLE_MODES(data) => {
                let name = match data.mode_name.to_str() {
                    Ok(s) if !s.is_empty() => s.to_owned(),
                    _ => standard_mode_label(data.standard_mode).to_owned(),
                };
                self.available_modes.insert(data.mode_index, AvailableMode {
                    name,
                    user_selectable: !data.properties.contains(MavModeProperty::MAV_MODE_PROPERTY_NOT_USER_SELECTABLE),
                    selector: mode_selector(data.standard_mode, data.custom_mode),
                });
            }
            MavMessage::CURRENT_MODE(data) => self.current_mode = Some(data.clone()),
            MavMessage::COMMAND_ACK(data) => {
                println!("Command ACK: {:?}", data);
            }
            _ => {}
        }
    }
}