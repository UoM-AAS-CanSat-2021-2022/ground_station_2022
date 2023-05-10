use std::ops::Mul;

const EARTH_RADIUS_KM: f64 = 6_371.009;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct WorldPosition {
    gps_altitude: f64,
    gps_latitude: f64,
    gps_longitude: f64,
}

impl WorldPosition {
    fn approx_linear_distance(&self, other: &Self) -> f64 {
        // get approx geographic distance using episoidal earth to plane projection
        // formula from https://en.wikipedia.org/wiki/Geographical_distance
        let (phi1, phi2) = (self.gps_latitude, other.gps_latitude);
        let (lam1, lam2) = (self.gps_longitude, other.gps_longitude);
        let phi_m = ((phi1 + phi1) / 2.0).to_radians();
        let del_phi = phi2 - phi1;
        let del_lam = lam2 - lam1;

        let k1 = 111.13209 - 0.56605 * f64::cos(phi_m * 2) + 0.00120 * f64::cos(phi_m * 4);
        let k2 = 111.41513 * f64::cos(phi_m) - 0.09455 * f64::cos(phi_m * 3)
            + 0.00012 * f64::cos(phi_m * 5);

        let geo_distance = f64::sqrt((k1 * del_phi).powi(2) + (k2 * del_lam).powi(2));

        // get linear distance using euclidean distance
        let del_height = other.gps_altitude - self.gps_altitude;

        f64::sqrt(geo_distance.powi(2) + del_height.powi(2))
    }
}
