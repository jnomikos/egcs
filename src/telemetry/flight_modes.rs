use mavlink::dialects::common::MavStandardMode;

/// Standard modes use `MAV_CMD_DO_SET_STANDARD_MODE`,
/// custom (flight-stack-specific) modes use `MAV_CMD_DO_SET_MODE`.
#[derive(Clone, Copy, PartialEq, Eq)]
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
    match mode {
        MavStandardMode::MAV_STANDARD_MODE_NON_STANDARD => "Custom",
        MavStandardMode::MAV_STANDARD_MODE_POSITION_HOLD => "Position",
        MavStandardMode::MAV_STANDARD_MODE_ORBIT => "Orbit",
        MavStandardMode::MAV_STANDARD_MODE_CRUISE => "Cruise",
        MavStandardMode::MAV_STANDARD_MODE_ALTITUDE_HOLD => "Altitude",
        MavStandardMode::MAV_STANDARD_MODE_SAFE_RECOVERY => "Return",
        MavStandardMode::MAV_STANDARD_MODE_MISSION => "Mission",
        MavStandardMode::MAV_STANDARD_MODE_LAND => "Land",
        MavStandardMode::MAV_STANDARD_MODE_TAKEOFF => "Takeoff",
    }
}

pub(super) fn mode_selector(standard_mode: MavStandardMode, custom_mode: u32) -> ModeSelector {
    if standard_mode == MavStandardMode::MAV_STANDARD_MODE_NON_STANDARD {
        ModeSelector::Custom(custom_mode)
    } else {
        ModeSelector::Standard(standard_mode as u8)
    }
}
