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
        let state = self.extended_sys_state.as_ref()?;
        match state.landed_state {
            MavLandedState::MAV_LANDED_STATE_ON_GROUND => Some(false),
            MavLandedState::MAV_LANDED_STATE_TAKEOFF
            | MavLandedState::MAV_LANDED_STATE_IN_AIR
            | MavLandedState::MAV_LANDED_STATE_LANDING => Some(true),
            MavLandedState::MAV_LANDED_STATE_UNDEFINED => None,
        }
    }

    pub fn is_landing(&self) -> Option<bool> {
        let state = self.extended_sys_state.as_ref()?;
        match state.landed_state {
            MavLandedState::MAV_LANDED_STATE_LANDING => Some(true),
            MavLandedState::MAV_LANDED_STATE_ON_GROUND
            | MavLandedState::MAV_LANDED_STATE_TAKEOFF
            | MavLandedState::MAV_LANDED_STATE_IN_AIR => Some(false),
            MavLandedState::MAV_LANDED_STATE_UNDEFINED => None,
        }
    }

    pub fn vtol_in_forward_flight(&self) -> Option<bool> {
        use mavlink::dialects::common::MavVtolState;
        let state = self.extended_sys_state.as_ref()?;
        match state.vtol_state {
            MavVtolState::MAV_VTOL_STATE_UNDEFINED => None,
            MavVtolState::MAV_VTOL_STATE_FW => Some(true),
            MavVtolState::MAV_VTOL_STATE_TRANSITION_TO_FW
            | MavVtolState::MAV_VTOL_STATE_TRANSITION_TO_MC
            | MavVtolState::MAV_VTOL_STATE_MC => Some(false),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_modes_filtered_correctly() {
        let mut telemetry = Telemetry::default();
        telemetry.available_modes.insert(
            0,
            AvailableMode {
                name: "mode1".to_owned(),
                user_selectable: true,
                selector: ModeSelector::Standard(0),
            },
        );
        telemetry.available_modes.insert(
            1,
            AvailableMode {
                name: "mode2".to_owned(),
                user_selectable: false,
                selector: ModeSelector::Standard(1),
            },
        );

        let selectable: Vec<_> = telemetry.selectable_modes().collect();
        assert_eq!(selectable.len(), 1);
        assert_eq!(selectable[0].name, "mode1");
    }
}
