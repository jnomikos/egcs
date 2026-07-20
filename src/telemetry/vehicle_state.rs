use super::{AvailableMode, ModeSelector, Telemetry, mode_selector};
use mavlink::dialects::common::{MavLandedState, MavModeFlag};

impl Telemetry {
    pub fn armed(&self) -> Option<bool> {
        self.heartbeat.as_ref().map(|hb| {
            hb.base_mode
                .contains(MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED)
        })
    }

    pub fn is_flying(&self) -> Option<bool> {
        use MavLandedState::*;
        self.extended_sys_state
            .as_ref()
            .and_then(|s| match s.landed_state {
                MAV_LANDED_STATE_ON_GROUND => Some(false),
                MAV_LANDED_STATE_TAKEOFF | MAV_LANDED_STATE_IN_AIR | MAV_LANDED_STATE_LANDING => {
                    Some(true)
                }
                MAV_LANDED_STATE_UNDEFINED => None,
            })
    }

    pub fn is_landing(&self) -> Option<bool> {
        use MavLandedState::*;
        self.extended_sys_state
            .as_ref()
            .and_then(|s| match s.landed_state {
                MAV_LANDED_STATE_LANDING => Some(true),
                MAV_LANDED_STATE_ON_GROUND | MAV_LANDED_STATE_TAKEOFF | MAV_LANDED_STATE_IN_AIR => {
                    Some(false)
                }
                MAV_LANDED_STATE_UNDEFINED => None,
            })
    }

    pub fn vtol_in_forward_flight(&self) -> Option<bool> {
        use mavlink::dialects::common::MavVtolState::*;
        self.extended_sys_state
            .as_ref()
            .and_then(|s| match s.vtol_state {
                MAV_VTOL_STATE_UNDEFINED => None,
                MAV_VTOL_STATE_FW => Some(true),
                MAV_VTOL_STATE_TRANSITION_TO_FW
                | MAV_VTOL_STATE_TRANSITION_TO_MC
                | MAV_VTOL_STATE_MC => Some(false),
            })
    }

    pub fn selectable_modes(&self) -> impl Iterator<Item = &AvailableMode> + '_ {
        self.available_modes.values().filter(|m| m.user_selectable)
    }

    pub fn current_selector(&self) -> Option<ModeSelector> {
        self.current_mode
            .as_ref()
            .map(|c| mode_selector(c.standard_mode, c.custom_mode))
    }

    pub fn current_mode_name(&self) -> Option<&str> {
        let current = self.current_selector()?;
        self.available_modes
            .values()
            .find(|m| m.selector == current)
            .map(|m| m.name.as_str())
    }

    /*
    /// True when the vehicle could not enter (or fell out of) the last commanded mode.
    pub fn mode_change_rejected(&self) -> bool {
        self.current_mode.as_ref().is_some_and(|c| {
            c.intended_custom_mode != 0 && c.intended_custom_mode != c.custom_mode
        })
    }*/
}
