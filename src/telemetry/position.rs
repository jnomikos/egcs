use super::Telemetry;

impl Telemetry {
    pub fn altitude_m(&self) -> Option<f32> {
        self.global_position_int.as_ref().map(|p| p.alt as f32 / 1000.0)
    }

    pub fn relative_altitude_m(&self) -> Option<f32> {
        self.global_position_int.as_ref().map(|p| p.relative_alt as f32 / 1000.0)
    }

    pub fn ground_speed_mps(&self) -> Option<f32> {
        self.global_position_int.as_ref().map(|p| {
            let vx = p.vx as f32 / 100.0;
            let vy = p.vy as f32 / 100.0;
            (vx * vx + vy * vy).sqrt()
        })
    }

    pub fn vertical_speed_mps(&self) -> Option<f32> {
        self.global_position_int.as_ref().map(|p| -(p.vz as f32) / 100.0)
    }

    pub fn latitude_deg(&self) -> Option<f64> {
        self.global_position_int.as_ref().map(|p| p.lat as f64 / 1e7)
    }

    pub fn longitude_deg(&self) -> Option<f64> {
        self.global_position_int.as_ref().map(|p| p.lon as f64 / 1e7)
    }

    pub fn heading_deg(&self) -> Option<f32> {
        self.global_position_int
            .as_ref()
            .filter(|p| p.hdg != u16::MAX)
            .map(|p| p.hdg as f32 / 100.0)
    }
}