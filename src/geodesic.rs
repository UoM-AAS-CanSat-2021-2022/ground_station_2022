use crate::telemetry::Telemetry;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct WorldPosition {
    pub gps_altitude: f64,
    pub gps_latitude: f64,
    pub gps_longitude: f64,
}

impl From<Telemetry> for WorldPosition {
    fn from(
        Telemetry {
            gps_altitude,
            gps_latitude,
            gps_longitude,
            ..
        }: Telemetry,
    ) -> Self {
        Self {
            gps_altitude,
            gps_latitude,
            gps_longitude,
        }
    }
}

impl WorldPosition {
    pub fn approx_linear_distance(&self, other: &Self) -> f64 {
        // get approx geographic distance using episoidal earth to plane projection
        // formula from https://en.wikipedia.org/wiki/Geographical_distance
        let (phi1, phi2) = (self.gps_latitude, other.gps_latitude);
        let (lam1, lam2) = (self.gps_longitude, other.gps_longitude);
        let phi_m = ((phi1 + phi1) / 2.0).to_radians();
        let del_phi = phi2 - phi1;
        let del_lam = lam2 - lam1;

        let k1 = 111.13209 - 0.56605 * f64::cos(phi_m * 2.0) + 0.00120 * f64::cos(phi_m * 4.0);
        let k2 = 111.41513 * f64::cos(phi_m) - 0.09455 * f64::cos(phi_m * 3.0)
            + 0.00012 * f64::cos(phi_m * 5.0);

        let geo_distance_km = (k1 * del_phi).hypot(k2 * del_lam);

        // get linear distance using euclidean distance
        let del_height_m = other.gps_altitude - self.gps_altitude;

        (geo_distance_km * 1000.0).hypot(del_height_m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_linear_distance_real_life_data() {
        let tom = WorldPosition {
            gps_latitude: 53.369486,
            gps_longitude: -1.835693,
            gps_altitude: 502.0,
        };
        let sam1 = WorldPosition {
            gps_latitude: 53.367134,
            gps_longitude: -1.831956,
            gps_altitude: 359.0,
        };
        let sam2 = WorldPosition {
            gps_latitude: 53.364508,
            gps_longitude: -1.837413,
            gps_altitude: 310.0,
        };
        let sam3 = WorldPosition {
            gps_latitude: 53.361916,
            gps_longitude: -1.837548,
            gps_altitude: 274.0,
        };
        assert!(tom.approx_linear_distance(&tom) <= 1e-10);
        assert!((tom.approx_linear_distance(&sam1) - 388.4).abs() <= 1.0);
        assert!((tom.approx_linear_distance(&sam2) - 597.4).abs() <= 1.0);
        assert!((tom.approx_linear_distance(&sam3) - 881.5).abs() <= 1.0);
    }
}
