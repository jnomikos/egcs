mod flight_modes;
mod position;
mod vehicle_state;

pub use flight_modes::{AvailableMode, ModeSelector};
use flight_modes::{mode_selector, standard_mode_label};

use mavlink::MavHeader;
use mavlink::dialects::common::{CURRENT_MODE_DATA, MavMessage, MavModeProperty};
use std::collections::BTreeMap;

#[derive(Default, Clone)]
pub struct Telemetry {
    pub extended_sys_state: Option<mavlink::dialects::common::EXTENDED_SYS_STATE_DATA>,
    pub attitude: Option<mavlink::dialects::common::ATTITUDE_DATA>,
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
                self.available_modes.insert(
                    data.mode_index,
                    AvailableMode {
                        name,
                        user_selectable: !data
                            .properties
                            .contains(MavModeProperty::MAV_MODE_PROPERTY_NOT_USER_SELECTABLE),
                        selector: mode_selector(data.standard_mode, data.custom_mode),
                    },
                );
            }
            MavMessage::CURRENT_MODE(data) => self.current_mode = Some(data.clone()),
            MavMessage::COMMAND_ACK(data) => {
                log::debug!("Command ACK: {data:?}");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_id_populates_on_heartbeat() {
        let mut telemetry = Telemetry::default();
        assert_eq!(telemetry.system_id, None);
        assert_eq!(telemetry.component_id, None);

        let header = mavlink::MavHeader {
            system_id: 1,
            component_id: 2,
            sequence: 0,
        };
        let msg = mavlink::dialects::common::MavMessage::HEARTBEAT(
            mavlink::dialects::common::HEARTBEAT_DATA {
                custom_mode: 0,
                mavtype: mavlink::dialects::common::MavType::MAV_TYPE_GENERIC,
                autopilot: mavlink::dialects::common::MavAutopilot::MAV_AUTOPILOT_GENERIC,
                base_mode: mavlink::dialects::common::MavModeFlag::empty(),
                system_status: mavlink::dialects::common::MavState::MAV_STATE_UNINIT,
                mavlink_version: 3,
            },
        );
        telemetry.update(&header, &msg);
        assert_eq!(telemetry.system_id, Some(1));
        assert_eq!(telemetry.component_id, Some(2));
    }
}
