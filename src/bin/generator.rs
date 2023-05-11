use chrono::{Timelike, Utc};
use ground_station::telemetry::*;
use rand::{
    distributions::{Open01, Slice, Uniform},
    prelude::*,
};
use std::io::ErrorKind;
use std::ops::AddAssign;
use std::{
    io::{self, Write},
    net::TcpStream,
    thread, time,
};
use tracing::Level;

fn main() -> anyhow::Result<()> {
    // real team number
    const TEAM_ID: u16 = 1047;

    // made up sea level constant
    const SEA_LEVEL: f64 = 1600.0;

    // failure rate of packet sending
    const ARTIFICIAL_FAILURE_RATE: f64 = 0.001;

    // define the distributions of various variables
    let modes = [Mode::Flight, Mode::Simulation];
    let alt_dist = Uniform::new(0.0, 750.0);
    let mode_dist = Slice::new(&modes)?;
    let temp_dist = Uniform::new(12.0, 70.0);
    let volt_dist = Uniform::new(4.8, 5.6);
    let press_dist = Uniform::new(80.0, 101.325);
    let lat_dist = Uniform::new(37.0, 37.4);
    let long_dist = Uniform::new(-80.6, -80.2);
    let sat_dist = Uniform::new(8, 35);
    let tilt_dist = Uniform::new(-45.0, 45.0);
    let delay_dist = Uniform::new(0.5, 1.5);

    // define the mutable state of the system
    let mut rng = thread_rng();
    let mut packet_count = 0;
    let mut error_count = 0;

    // setup logging
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(Level::DEBUG)
        .with_writer(io::stderr)
        .init();

    // connect to the frontend, retry after 1 second
    let address = "127.0.0.1:10470";
    let mut stream = loop {
        match TcpStream::connect(address) {
            Ok(s) => break s,
            Err(e) => {
                tracing::warn!("Failed to connect to frontend on {address} - {e}");
                thread::sleep(time::Duration::from_millis(200));
            }
        }
    };

    let real_time = false;
    let max_packet_count = 1000;

    // send packets until we are disconnected
    let mut now = Utc::now();
    loop {
        // seperate the time from Utc::now() so that we can run the clock fast
        let delay = rng.sample(delay_dist);
        now.add_assign(chrono::Duration::milliseconds((delay * 1000.0) as i64));
        let altitude = rng.sample(alt_dist);
        let telem = Telemetry {
            team_id: TEAM_ID,
            mission_time: MissionTime {
                h: now.hour() as u8,
                m: now.minute() as u8,
                s: now.second() as u8,
                cs: (now.timestamp_millis().rem_euclid(1000) / 10) as u8,
            },
            packet_count,
            mode: *rng.sample(mode_dist),
            state: State::Yeeted,
            altitude,
            hs_deployed: HsDeployed::Deployed,
            pc_deployed: PcDeployed::Deployed,
            mast_raised: MastRaised::Raised,
            temperature: rng.sample(temp_dist),
            voltage: rng.sample(volt_dist),
            pressure: rng.sample(press_dist),
            gps_time: GpsTime {
                h: now.hour() as u8,
                m: now.minute() as u8,
                s: now.second() as u8,
            },
            gps_altitude: SEA_LEVEL + altitude,
            gps_latitude: rng.sample(lat_dist),
            gps_longitude: rng.sample(long_dist),
            gps_sats: rng.sample(sat_dist),
            tilt_x: rng.sample(tilt_dist),
            tilt_y: rng.sample(tilt_dist),
            cmd_echo: "CXON".to_string(),
        };
        tracing::trace!("Generated telem = {telem}");

        // artificially fail some packets
        let fail_packet: f64 = rng.sample(Open01);
        if max_packet_count != 0 && fail_packet < ARTIFICIAL_FAILURE_RATE {
            tracing::info!("Artificially failed a packet: {telem}");
        } else if let Err(e) = writeln!(stream, "{telem}") {
            if matches!(e.kind(), ErrorKind::BrokenPipe | ErrorKind::ConnectionReset) {
                tracing::info!("Client has disconnected, exiting.");
                break Ok(());
            }

            tracing::warn!("Failed to send telemetry packet: {e} - {error_count} errors so far");
            error_count += 1;
        }

        packet_count += 1;

        // wait to send the next packet
        if real_time {
            thread::sleep(time::Duration::from_secs_f32(delay));
        } else if max_packet_count <= packet_count {
            break Ok(());
        }
    }
}
