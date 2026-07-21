use super::Telemetry;

impl Telemetry {
    pub fn altitude_m(&self) -> Option<f32> {
        self.global_position_int
            .as_ref()
            .map(|p| p.alt as f32 / 1000.0)
    }

    pub fn relative_altitude_m(&self) -> Option<f32> {
        self.global_position_int
            .as_ref()
            .map(|p| p.relative_alt as f32 / 1000.0)
    }

    pub fn ground_speed_mps(&self) -> Option<f32> {
        self.global_position_int.as_ref().map(|p| {
            let vx = p.vx as f32 / 100.0;
            let vy = p.vy as f32 / 100.0;
            vx.hypot(vy)
        })
    }

    pub fn vertical_speed_mps(&self) -> Option<f32> {
        self.global_position_int
            .as_ref()
            .map(|p| -(p.vz as f32) / 100.0)
    }

    pub fn latitude_deg(&self) -> Option<f64> {
        self.global_position_int
            .as_ref()
            .map(|p| p.lat as f64 / 1e7)
    }

    pub fn longitude_deg(&self) -> Option<f64> {
        self.global_position_int
            .as_ref()
            .map(|p| p.lon as f64 / 1e7)
    }

    pub fn heading_deg(&self) -> Option<f32> {
        self.global_position_int
            .as_ref()
            .filter(|p| p.hdg != u16::MAX)
            .map(|p| p.hdg as f32 / 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_position_int_altitude_conversions() {
        let telemetry = Telemetry {
            global_position_int: Some(mavlink::dialects::common::GLOBAL_POSITION_INT_DATA {
                time_boot_ms: 0,
                lat: 0,
                lon: 0,
                alt: 12345,
                relative_alt: 6789,
                vx: 0,
                vy: 0,
                vz: 0,
                hdg: u16::MAX,
            }),
            ..Default::default()
        };

        let altitude_m = telemetry.altitude_m().unwrap_or(0.0);
        let relative_altitude_m = telemetry.relative_altitude_m().unwrap_or(0.0);
        assert!(
            (altitude_m - 12.345).abs() < 1e-5,
            "got {altitude_m}, expected 12.345"
        );
        assert!(
            (relative_altitude_m - 6.789).abs() < 1e-5,
            "got {relative_altitude_m}, expected 6.789"
        );
    }

    #[test]
    fn global_position_int_speed_conversions() {
        let telemetry = Telemetry {
            global_position_int: Some(mavlink::dialects::common::GLOBAL_POSITION_INT_DATA {
                time_boot_ms: 0,
                lat: 0,
                lon: 0,
                alt: 0,
                relative_alt: 0,
                vx: 100,
                vy: 200,
                vz: -300,
                hdg: u16::MAX,
            }),
            ..Default::default()
        };

        let speed = telemetry.ground_speed_mps().unwrap_or(0.0);
        let vertical_speed = telemetry.vertical_speed_mps().unwrap_or(0.0);
        assert!(
            (speed - 2.236_068).abs() < 1e-5,
            "got {speed}, expected 2.236068"
        );
        assert!(
            (vertical_speed - 3.0).abs() < 1e-5,
            "got {vertical_speed}, expected 3.0"
        );
    }

    #[test]
    /// `UINT16_MAX` hdg means unknown
    fn unknown_heading_deg_returns_none() {
        let telemetry = Telemetry {
            global_position_int: Some(mavlink::dialects::common::GLOBAL_POSITION_INT_DATA {
                time_boot_ms: 0,
                lat: 0,
                lon: 0,
                alt: 0,
                relative_alt: 0,
                vx: 0,
                vy: 0,
                vz: 0,
                hdg: u16::MAX,
            }),
            ..Default::default()
        };

        assert_eq!(telemetry.heading_deg(), None);
    }

    #[test]
    fn valid_heading_deg_returns_some() {
        let telemetry = Telemetry {
            global_position_int: Some(mavlink::dialects::common::GLOBAL_POSITION_INT_DATA {
                time_boot_ms: 0,
                lat: 0,
                lon: 0,
                alt: 0,
                relative_alt: 0,
                vx: 0,
                vy: 0,
                vz: 0,
                hdg: 9000,
            }),
            ..Default::default()
        };

        assert_eq!(telemetry.heading_deg(), Some(90.0));
    }
}
