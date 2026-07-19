use super::Telemetry;
use crate::flight_mode::FlightMode;
use mavlink::dialects::common::{MavModeFlag, MavLandedState};

impl Telemetry {
    pub fn armed(&self) -> Option<bool> {
        self.heartbeat.as_ref().map(|hb| hb.base_mode.contains(MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED))
    }

    pub fn is_flying(&self) -> Option<bool> {
        use MavLandedState::*;
        self.extended_sys_state.as_ref().and_then(|s| match s.landed_state {
            MAV_LANDED_STATE_ON_GROUND => Some(false),
            MAV_LANDED_STATE_TAKEOFF | MAV_LANDED_STATE_IN_AIR | MAV_LANDED_STATE_LANDING => Some(true),
            MAV_LANDED_STATE_UNDEFINED => None,
        })
    }

    pub fn is_landing(&self) -> Option<bool> {
        use MavLandedState::*;
        self.extended_sys_state.as_ref().and_then(|s| match s.landed_state {
            MAV_LANDED_STATE_LANDING => Some(true),
            MAV_LANDED_STATE_ON_GROUND | MAV_LANDED_STATE_TAKEOFF | MAV_LANDED_STATE_IN_AIR => Some(false),
            MAV_LANDED_STATE_UNDEFINED => None,
        })
    }

    pub fn vtol_in_forward_flight(&self) -> Option<bool> {
        use mavlink::dialects::common::MavVtolState::*;
        self.extended_sys_state.as_ref().and_then(|s| match s.vtol_state {
            MAV_VTOL_STATE_UNDEFINED => None,
            MAV_VTOL_STATE_FW => Some(true),
            MAV_VTOL_STATE_TRANSITION_TO_FW | MAV_VTOL_STATE_TRANSITION_TO_MC | MAV_VTOL_STATE_MC => Some(false),
        })
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