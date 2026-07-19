//! PX4 flight modes, ported from QGC's `PX4/px4_custom_mode.h` + `PX4FirmwarePlugin.cc`.
#![allow(dead_code)]


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainMode {
    Manual = 1,
    Altctl,
    Posctl,
    Auto,
    Acro,
    Offboard,
    Stabilized,
    RattitudeDeprecated,
    Simple,
    Termination,
    AltitudeCruise,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoSubMode {
    Ready = 1,
    Takeoff,
    Loiter,
    Mission,
    Rtl,
    Land,
    ReservedDoNotUse,
    FollowTarget,
    Precland,
    VtolTakeoff,
    GuidedCourse,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PosctlSubMode {
    Posctl = 0,
    Orbit,
    Slow,
}

const fn pack(main: u8, sub: u8) -> u32 {
    ((main as u32) << 16) | ((sub as u32) << 24)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlightMode {
    Manual,
    Stabilized,
    Acro,
    Rattitude,
    Altctl,
    Offboard,
    Simple,
    Termination,
    AltitudeCruise,
    PosctlPosctl,
    PosctlOrbit,
    PosctlSlow,
    AutoLoiter,
    AutoMission,
    AutoRtl,
    AutoFollowTarget,
    AutoLand,
    AutoPrecland,
    AutoReady,
    AutoTakeoff,
    AutoVtolTakeoff,
    AutoGuidedCourse,
}

impl FlightMode {
    pub const ALL: [FlightMode; 22] = {
        use FlightMode::*;
        [
            Manual, Stabilized, Acro, Rattitude, Altctl, Offboard, Simple,
            Termination, AltitudeCruise, PosctlPosctl, PosctlOrbit, PosctlSlow,
            AutoLoiter, AutoMission, AutoRtl, AutoFollowTarget, AutoLand,
            AutoPrecland, AutoReady, AutoTakeoff, AutoVtolTakeoff, AutoGuidedCourse,
        ]
    };

    pub const fn custom_mode(self) -> u32 {
        use FlightMode::*;
        match self {
            Manual => pack(MainMode::Manual as u8, 0),
            Stabilized => pack(MainMode::Stabilized as u8, 0),
            Acro => pack(MainMode::Acro as u8, 0),
            Rattitude => pack(MainMode::RattitudeDeprecated as u8, 0),
            Altctl => pack(MainMode::Altctl as u8, 0),
            Offboard => pack(MainMode::Offboard as u8, 0),
            Simple => pack(MainMode::Simple as u8, 0),
            Termination => pack(MainMode::Termination as u8, 0),
            AltitudeCruise => pack(MainMode::AltitudeCruise as u8, 0),
            PosctlPosctl => pack(MainMode::Posctl as u8, PosctlSubMode::Posctl as u8),
            PosctlOrbit => pack(MainMode::Posctl as u8, PosctlSubMode::Orbit as u8),
            PosctlSlow => pack(MainMode::Posctl as u8, PosctlSubMode::Slow as u8),
            AutoLoiter => pack(MainMode::Auto as u8, AutoSubMode::Loiter as u8),
            AutoMission => pack(MainMode::Auto as u8, AutoSubMode::Mission as u8),
            AutoRtl => pack(MainMode::Auto as u8, AutoSubMode::Rtl as u8),
            AutoFollowTarget => pack(MainMode::Auto as u8, AutoSubMode::FollowTarget as u8),
            AutoLand => pack(MainMode::Auto as u8, AutoSubMode::Land as u8),
            AutoPrecland => pack(MainMode::Auto as u8, AutoSubMode::Precland as u8),
            AutoReady => pack(MainMode::Auto as u8, AutoSubMode::Ready as u8),
            AutoTakeoff => pack(MainMode::Auto as u8, AutoSubMode::Takeoff as u8),
            AutoVtolTakeoff => pack(MainMode::Auto as u8, AutoSubMode::VtolTakeoff as u8),
            AutoGuidedCourse => pack(MainMode::Auto as u8, AutoSubMode::GuidedCourse as u8),
        }
    }

    /// `(main, sub)` bytes for `MAV_CMD_DO_SET_MODE` param2/param3.
    pub const fn main_sub(self) -> (u8, u8) {
        let m = self.custom_mode();
        ((m >> 16) as u8, (m >> 24) as u8)
    }

    pub const fn label(self) -> &'static str {
        use FlightMode::*;
        match self {
            Manual => "Manual",
            Stabilized => "Stabilized",
            Acro => "Acro",
            Rattitude => "Rattitude",
            Altctl => "Altitude",
            Offboard => "Offboard",
            Simple => "Simple",
            Termination => "Termination",
            AltitudeCruise => "Altitude Cruise",
            PosctlPosctl => "Position",
            PosctlOrbit => "Orbit",
            PosctlSlow => "Position Slow",
            AutoLoiter => "Hold",
            AutoMission => "Mission",
            AutoRtl => "Return",
            AutoFollowTarget => "Follow Me",
            AutoLand => "Land",
            AutoPrecland => "Precision Land",
            AutoReady => "Ready",
            AutoTakeoff => "Takeoff",
            AutoVtolTakeoff => "VTOL Takeoff",
            AutoGuidedCourse => "Guided Course",
        }
    }

    pub fn from_custom_mode(custom_mode: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.custom_mode() == custom_mode)
    }
}
