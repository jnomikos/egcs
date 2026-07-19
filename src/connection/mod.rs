mod link;
mod protocol;

pub use link::run;

use mavlink::dialects::common::MavMessage;
use mavlink::MavHeader;
use std::sync::Arc;

use crate::telemetry::{Telemetry, ModeSelector};

type Conn = Arc<Box<dyn mavlink::AsyncMavConnection<MavMessage> + Send + Sync>>;

const GCS_HEADER: MavHeader = MavHeader { system_id: 255, component_id: 190, sequence: 0 };


pub enum Command {
    Connect(String),
    Disconnect,
    Vehicle(VehicleCommand),
}

pub enum VehicleCommand {
    Arm,
    Disarm,
    Takeoff { altitude: f32 },
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