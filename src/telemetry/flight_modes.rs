use mavlink::dialects::common::MavStandardMode;

/// Standard modes use MAV_CMD_DO_SET_STANDARD_MODE,
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

pub(super) fn standard_mode_label(mode: MavStandardMode) -> &'static str {
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

pub(super) fn mode_selector(standard_mode: MavStandardMode, custom_mode: u32) -> ModeSelector {
    if standard_mode == MavStandardMode::MAV_STANDARD_MODE_NON_STANDARD {
        ModeSelector::Custom(custom_mode)
    } else {
        ModeSelector::Standard(standard_mode as u8)
    }
}
