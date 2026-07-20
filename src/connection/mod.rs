mod link;
mod protocol;

pub use link::run;

use mavlink::MavHeader;
use mavlink::dialects::common::MavMessage;
use std::sync::Arc;

use crate::telemetry::{ModeSelector, Telemetry};

type Conn = Arc<Box<dyn mavlink::AsyncMavConnection<MavMessage> + Send + Sync>>;

const GCS_HEADER: MavHeader = MavHeader {
    system_id: 255,
    component_id: 190,
    sequence: 0,
};

pub enum Command {
    Connect(String),
    Disconnect,
    Vehicle(VehicleCommand),
}

pub enum VehicleCommand {
    Arm,
    Disarm,
    Takeoff {
        altitude: f32,
    },
    DoReposition {
        latitude_deg: i32,
        longitude_deg: i32,
    },
    Land,
    SetMode(ModeSelector),
}

#[derive(Default, PartialEq, Clone, Debug)]
pub enum ConnStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}
