mod commands;
mod graphable;
mod received_packet;
pub use received_packet::ReceivedPacket;

use graphable::Graphable;

use crate::xbee::TxStatus;
use crate::{
    app::commands::CommandPanel,
    as_str::AsStr,
    constants::{BAUD_RATES, BROADCAST_ADDR, SEALEVEL_HPA, TEAM_ID, TEAM_ID_STR},
    telemetry::{Telemetry, TelemetryField},
    xbee::{DeliveryStatus, TxRequest, XbeePacket},
};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use eframe::{egui, emath::Align};
use egui::{
    plot::{Line, Plot, PlotPoint, PlotPoints},
    Color32, Grid, Layout, Sense, Ui, Vec2,
};
use egui_extras::{Column, TableBuilder};
use enum_iterator::{all, Sequence};
use parking_lot::FairMutex;
use serialport::{SerialPort, SerialPortType};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    fs::OpenOptions,
    io::{ErrorKind, Read, Write},
    path::PathBuf,
    sync::atomic::AtomicUsize,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

static CURR_RADIO: AtomicUsize = AtomicUsize::new(0);

// use the strongest ordering for all atomic operations
const ORDER: Ordering = Ordering::SeqCst;
const TELEMETRY_FILE: &str = "Flight_1047.csv";

// static atomic state for sharing with the sending thread
// have we started the sending thread? - prevent starting two threads
static SEND_THREAD_STARTED: AtomicBool = AtomicBool::new(false);
// have we paused sending the SIMP packets
static SEND_THREAD_PAUSED: AtomicBool = AtomicBool::new(false);
// are we cancelling the sending thread
static SEND_THREAD_CANCEL: AtomicBool = AtomicBool::new(false);
// how many pressure values have been sent?
static SENT_SIMPS: AtomicU32 = AtomicU32::new(0);

#[derive(Builder)]
#[builder(pattern = "owned", default)]
pub struct GroundStationGui {
    /// The collected telemetry from the current run
    telemetry: Vec<Telemetry>,

    /// The values for displaying in the graphs
    graph_values: HashMap<Graphable, Vec<PlotPoint>>,

    /// The number of missed telemetry packets
    missed_packets: u32,

    /// How many telemetry points does the one graph view show?
    one_graph_points: usize,

    /// How many telemetry points does the all graphs view show?
    all_graphs_points: usize,

    /// Show all the points in the one graph view?
    one_graph_shows_all: bool,

    /// Do we show all the points in the all graphg view?
    all_graphs_show_all: bool,

    /// What does the one graph view show?
    one_graph_shows: Graphable,

    /// What does the main view show?
    main_view: MainPanelView,

    // ===== show windows? =====
    /// Show the settings window?
    show_settings_window: bool,

    /// Show the command window?
    show_command_window: bool,

    /// Show the radio window?
    show_radio_window: bool,

    /// Show the simulation window?
    show_sim_window: bool,

    // ===== simulation mode values =====
    /// The simulation pressure values
    simp_values: Option<Vec<u32>>,

    /// The graph values for each SIMP value
    simp_graph_values: Option<Vec<PlotPoint>>,

    // ===== command and radio data =====
    /// The command center
    command_center: CommandPanel,

    /// The channel over which to send and receive commands
    cmd_sender: Sender<String>,
    cmd_receiver: Receiver<String>,

    /// A mapping from the time a command was state, to the command and it's status
    /// allows iterating in sent order due to BTreeMap's inherent ordering
    command_history: BTreeMap<DateTime<Utc>, (String, CommandStatus)>,

    /// The radio's serial port name
    radio_port: String,

    /// The radio's baud rate
    radio_baud: u32,

    /// The XBee radio serial port connection
    radio: Option<Arc<FairMutex<Box<dyn SerialPort>>>>,

    /// The channel down which to receive packets
    #[builder(setter(strip_option))]
    packet_rx: Option<Receiver<ReceivedPacket>>,

    /// The received packets from the radio
    packet_log: Vec<Packet>,
}

impl Default for GroundStationGui {
    fn default() -> Self {
        let (tx, rx) = channel();

        Self {
            telemetry: vec![],
            graph_values: Default::default(),
            missed_packets: 0,
            one_graph_points: 40,
            all_graphs_points: 40,
            one_graph_shows_all: false,
            all_graphs_show_all: false,
            one_graph_shows: Default::default(),
            main_view: Default::default(),
            show_settings_window: false,
            show_command_window: false,
            show_radio_window: false,
            show_sim_window: false,
            simp_values: None,
            simp_graph_values: None,
            command_center: Default::default(),
            cmd_sender: tx,
            cmd_receiver: rx,
            command_history: Default::default(),
            radio_port: "".to_string(),
            radio_baud: 230400,
            radio: None,
            packet_rx: None,
            packet_log: vec![],
        }
    }
}

#[derive(Sequence, Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum MainPanelView {
    #[default]
    AllGraphs,
    OneGraph,
    Table,
    Packets,
    Commands,
}

impl AsStr for MainPanelView {
    fn as_str(&self) -> &'static str {
        match self {
            MainPanelView::OneGraph => "One Graph",
            MainPanelView::AllGraphs => "All Graphs",
            MainPanelView::Table => "Data Table",
            MainPanelView::Packets => "Packets",
            MainPanelView::Commands => "Commands",
        }
    }
}

impl fmt::Display for MainPanelView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl GroundStationGui {
    /// Receive any telemetry that is waiting on the incoming channel
    fn recv_telem(&mut self) {
        // take ownership of the receiver so we can mutate self
        if let Some(rx) = self.packet_rx.take() {
            // receive anything sent down the channel
            loop {
                match rx.try_recv() {
                    Ok(packet) => {
                        self.packet_log.push(Packet::Received(packet.clone()));
                        if let ReceivedPacket::Telemetry { telem, .. } = &packet {
                            self.add_telem(telem.clone());
                        } else if let ReceivedPacket::Status { tx_status, .. } = &packet {
                            self.recv_ack(*tx_status);
                        } else {
                            for telem in Self::recover_telemetry(&packet) {
                                tracing::info!(
                                    "Recovered some telemetry from an invalid packet - {telem}"
                                );
                                self.add_telem(telem);
                            }
                        }
                    }

                    // don't replace the reader if the receiver is disconnected
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        tracing::warn!("Telemetry Receiver disconnected.");
                        break;
                    }

                    // if the receiver has no more telemetry then give
                    // ownership of the receiver back to self
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        self.packet_rx = Some(rx);
                        break;
                    }
                }
            }
        }
    }

    /// handles all the logic / state that must be kept in sync when adding telemetry
    fn add_telem(&mut self, telem: Telemetry) {
        // calculate how many packets we missed if any
        if let Some(prev) = self.telemetry.last() {
            self.missed_packets += telem.packet_count.saturating_sub(1 + prev.packet_count);
        }

        tracing::debug!("{:?}", telem);
        self.telemetry.push(telem.clone());

        let time = telem.mission_time.as_seconds();
        for field in all::<Graphable>() {
            self.graph_values
                .entry(field)
                .or_default()
                .push(PlotPoint::new(time, field.extract_telemetry_value(&telem)));
        }

        // save the telemetry out to the telemetry file
        let handle = OpenOptions::new()
            .append(true)
            .create(true)
            .open(TELEMETRY_FILE);

        let result = match handle {
            Ok(mut file) => writeln!(file, "{telem}"),
            Err(e) => {
                tracing::warn!("Failed to open `{TELEMETRY_FILE}` - {e}.");
                Ok(())
            }
        };

        if let Err(e) = result {
            tracing::warn!("Encountered error while writing to file: {e}");
        }
    }

    /// Handle an ack for a packet
    fn recv_ack(&mut self, tx_status: TxStatus) {
        // if the delivery was a success mark it as acknowledged
        if tx_status.status == DeliveryStatus::Success {
            // mark the command as acknowledged
            for (_, (cmd, status)) in self.command_history.iter_mut().rev() {
                match status {
                    CommandStatus::Sent { frame_id } if *frame_id == tx_status.frame_id => {
                        tracing::info!("Received acknowledgement for command - {cmd:?}");
                        *status = CommandStatus::Acked;
                        break;
                    }
                    _ => (),
                }
            }
        }
    }

    /// Sometimes invalid packets contain data that we can actually salvage
    fn recover_telemetry(packet: &ReceivedPacket) -> Vec<Telemetry> {
        let ReceivedPacket::Invalid(data) = packet else {
            return vec![];
        };

        // extract all the ASCII substrings of this data
        let mut ascii_substrings = vec![String::new()];
        for byte in data {
            if byte.is_ascii() {
                // if we hit ascii data just add it to the last string
                ascii_substrings.last_mut().unwrap().push(*byte as char);
            } else {
                // if we don't then add an empty String if the last one isn't empty
                if !ascii_substrings.last().unwrap().is_empty() {
                    ascii_substrings.push(String::new());
                }
            }
        }

        tracing::debug!("Found ascii substrings in invalid data: {ascii_substrings:?}");

        // collect any substrings which parse as telemetry
        ascii_substrings
            .into_iter()
            .filter_map(|s| {
                let start = s.find(TEAM_ID_STR)?;
                s[start..].parse().ok()
            })
            .collect()
    }

    /// Attempts to open a connection to the given radio
    fn open_radio_connection(&mut self) {
        // use an atomic to specify the current thread number
        // this means that we don't have to keep a handle to the thread
        // we can simply increment this atomic and old threads will stop

        // try to open the new radio
        match serialport::new(&self.radio_port, self.radio_baud).open() {
            Ok(port) => {
                let radio_num = CURR_RADIO.fetch_add(1, ORDER) + 1;
                let radio = Arc::new(FairMutex::new(port));
                let (tx, rx) = channel();
                self.radio = Some(radio.clone());
                self.packet_rx = Some(rx);

                // start a new thread :D
                if let Err(e) = thread::Builder::new()
                    .name(format!("radio_{radio_num}"))
                    .spawn(move || Self::radio_thread(radio_num, radio, tx))
                {
                    tracing::error!("Failed to start radio reader thread - {e:?}");
                }
                tracing::info!("Successfully opened port.");
            }
            Err(e) => {
                tracing::error!("Failed to open port - {e:?}");
            }
        }
    }

    // this thread handles receiving data from the radio and sending
    // received packets back to the main thread
    fn radio_thread(
        radio_num: usize,
        radio_mutex: Arc<FairMutex<Box<dyn SerialPort>>>,
        packet_tx: Sender<ReceivedPacket>,
    ) {
        // allocate a buffer for receiving packets
        let mut buf = [0u8; 4096];
        let mut write_idx = 0;

        // open the radio data log in append mode
        let mut log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("radio_data.raw");

        if let Err(e) = log_file.as_ref() {
            tracing::warn!("Failed to open radio data log, radio data will not be saved. - {e:?}");
        }

        // check we are the current radio - exiting cleanly if we aren't
        while radio_num == CURR_RADIO.load(ORDER) {
            // acquire a lock on the radio
            let mut radio = radio_mutex.lock();

            match radio.read(&mut buf[write_idx..]) {
                Ok(bytes_read) => {
                    tracing::debug!(
                        "Read {bytes_read} bytes from the radio - {:?} - {:02X?}",
                        String::from_utf8_lossy(&buf[..bytes_read]),
                        &buf[write_idx..write_idx + bytes_read]
                    );

                    // save any data we receive to a file
                    if let Ok(file) = log_file.as_mut() {
                        let save_data_res = file.write_all(&buf[write_idx..write_idx + bytes_read]);

                        // log any errors
                        if let Err(e) = save_data_res {
                            tracing::info!("Failed to save radio data to 'radio_data.raw' - {e:?}");
                        }
                    }

                    // bump the write index
                    write_idx += bytes_read;
                }

                Err(e) => {
                    match e.kind() {
                        // this kind of error happens when no data is there to be read
                        // we can safely ignore this kind of error
                        ErrorKind::TimedOut => {
                            // sleep for a bit then continue
                            thread::sleep(Duration::from_millis(1));
                            continue;
                        }
                        ErrorKind::BrokenPipe => {
                            tracing::info!("Radio disconnected - stopping receiver thread");
                            return;
                        }
                        _ => {
                            tracing::warn!("Received unrecognised error while reading from radio - {e:?} - stopping receiver thread");
                            return;
                        }
                    }
                }
            };

            // find packets in the sent data by looking for the start byte
            let candidates = buf[..write_idx]
                .iter()
                .enumerate()
                .filter_map(|(idx, b)| (*b == 0x7E).then_some(idx));

            // keep track of where we have parsed upto
            let mut parsed_upto = 0;
            for start in candidates {
                tracing::debug!("start = {start}, parsed_upto = {parsed_upto}");

                let potential_packet = &buf[start..write_idx];
                let received: ReceivedPacket = potential_packet.into();

                match &received {
                    ReceivedPacket::Telemetry { packet, .. }
                    | ReceivedPacket::Received { packet, .. }
                    | ReceivedPacket::Status { packet, .. }
                    | ReceivedPacket::InvalidFrame(packet)
                    | ReceivedPacket::Unrecognised(packet) => {
                        // as good as we're going to get from this one, so send it over
                        tracing::info!("Received: {received:02X?}");

                        // if our start is further than `parsed_upto` then output
                        // whatever came before as an invalid packet.
                        if start != parsed_upto {
                            // we don't really care if this fails
                            let _ = packet_tx
                                .send(ReceivedPacket::Invalid(buf[parsed_upto..start].to_vec()));
                        }

                        // calculate the packet length while we still borrow the packet
                        let packet_len = packet.data.len() + 5;

                        // if this fails then this thread should die
                        if let Err(e) = packet_tx.send(received) {
                            tracing::error!("Encountered error sending packet over channel - {e:?} - ending radio thread.");
                            return;
                        }

                        // now update parsed_upto
                        // packet_len = data_len + 1 (checksum) + 1 (frame type) + 2 (length) + 1 (start byte)
                        parsed_upto = start + packet_len;
                    }
                    // parse failed so try again later
                    ReceivedPacket::Invalid(_) => {}
                }
            }

            // if we are at the end of the buffer then attempt to find the start byte of the
            // last packet sent and make that the new start of the buffer
            if write_idx == buf.len() {
                // only search in the last 256 bytes because that is the maximum size of a packet
                match buf[buf.len() - 256..].iter().rposition(|x| *x == 0x7E) {
                    // simply set parsed_upto and let the later code handle the buffer logic
                    Some(back_pos) => parsed_upto = back_pos,
                    None => parsed_upto = write_idx,
                }
            }

            // if we have parsed any data then move unparsed data to the start
            if parsed_upto > 0 {
                buf.copy_within(parsed_upto..write_idx, 0);
                write_idx -= parsed_upto;
            }

            // we want to check the radio very often so only sleep for a millisecond
            thread::sleep(Duration::from_millis(1));
        }

        // if the write index is not zero, output whatever is in the buffer as Invalid([..]) before exiting
        if write_idx != 0 {
            packet_tx
                .send(ReceivedPacket::Invalid(buf[..write_idx].to_vec()))
                .ok();
        }
    }

    /// Close the current radio
    fn close_radio(&mut self) {
        self.radio = None;
        tracing::debug!("Closed connection - CURR_RADIO={}", CURR_RADIO.load(ORDER));
        CURR_RADIO.fetch_add(1, ORDER);

        // receive any packets remaining
        while let Some(packet) = self.packet_rx.as_mut().and_then(|rx| rx.try_recv().ok()) {
            self.packet_log.push(Packet::Received(packet.clone()));
            if let ReceivedPacket::Telemetry { telem, .. } = packet {
                self.add_telem(telem);
            } else {
                for telem in Self::recover_telemetry(&packet) {
                    tracing::info!("Recovered some telemetry from an invalid packet - {telem}");
                    self.add_telem(telem);
                }
            }
        }
    }

    /// Handle reading commands from the channel and sending them down the radio
    fn handle_commands(&mut self) {
        // read any waiting commands into the command history, marking then unsent
        while let Ok(cmd) = self.cmd_receiver.try_recv() {
            tracing::debug!("Received command from channel - cmd={cmd:?}");
            self.command_history
                .insert(Utc::now(), (cmd, CommandStatus::Unsent));
        }

        // wrapping counter for the frame IDs
        static FRAME_ID_COUNTER: AtomicU8 = AtomicU8::new(1);

        let Some(radio_mutex) = self.radio.as_mut() else { return };

        // attempt to send any unsent commands
        for (_, (ref cmd, status)) in self.command_history.iter_mut() {
            if *status != CommandStatus::Unsent {
                continue;
            }

            let mut frame_id = FRAME_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
            // frame ID == 0 means no ack :(
            while frame_id == 0 {
                frame_id = FRAME_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
            }
            let req = TxRequest::new(frame_id, BROADCAST_ADDR, cmd);
            let Ok(packet): std::io::Result<XbeePacket> = req.clone().try_into() else {
                tracing::error!("Failed to build a packet for cmd={cmd:?}");
                continue;
            };
            match packet.clone().serialise() {
                Ok(data) => {
                    let mut radio = radio_mutex.lock();
                    if let Err(e) = radio.write(&data) {
                        tracing::error!("Failure sending packet - {data:02X?} - {e:?}");
                    } else {
                        tracing::info!("Sent command {cmd:?} with frame_id={frame_id:02X}");
                        *status = CommandStatus::Sent { frame_id };
                        self.packet_log.push(Packet::Sent(req));
                    }
                }
                Err(e) => {
                    tracing::error!("Failure serialising packet with data - {cmd:?} - {e:?}")
                }
            }
        }
    }

    fn load_sim_file(&mut self, path: PathBuf) -> anyhow::Result<()> {
        // first read the lines of the file
        let file_data = std::fs::read_to_string(path)?;
        let lines: Vec<_> = file_data.split_ascii_whitespace().collect();

        // pre-allocate a vector with enough capacity to hold one pressure value for each line
        let mut pressure_data: Vec<u32> = Vec::with_capacity(lines.len());

        for line in lines {
            // try to parse the line as u32, log the error if it failed
            let s = line.trim();
            if let Ok(pressure) = s.parse::<u32>() {
                pressure_data.push(pressure);
            } else if let Ok(telem) = s.parse::<Telemetry>() {
                pressure_data.push(Self::altitude_to_pressure(telem.altitude));
            } else {
                tracing::warn!("Failed to parse line as pressure value - line={s:?}")
            }
        }

        // create the graph values
        let plot_points: Vec<PlotPoint> = pressure_data
            .iter()
            .enumerate()
            .map(|(i, simp)| PlotPoint::new(i as f64, Self::pressure_to_altitude(*simp)))
            .collect();

        self.simp_values = Some(pressure_data);

        self.simp_graph_values = Some(plot_points);
        Ok(())
    }

    fn pressure_to_altitude(pressure: u32) -> f64 {
        // Adapted from readAltitude
        // Equation taken from BMP180 datasheet (page 16):
        //  http://www.adafruit.com/datasheets/BST-BMP180-DS000-09.pdf

        // Note that using the equation from wikipedia can give bad results
        // at high altitude. See this thread for more information:
        //  http://forums.adafruit.com/viewtopic.php?f=22&t=58064
        let simp_hpa = (pressure as f64) / 100.0;
        44330.0 * (1.0 - (simp_hpa / SEALEVEL_HPA).powf(0.1903))
    }

    fn altitude_to_pressure(altitude: f64) -> u32 {
        // inverted form of pressure_to_altitude
        let presssure_hpa = SEALEVEL_HPA * (1.0 - altitude / 44330.0).powf(1.0 / 0.1903);
        (presssure_hpa * 100.0) as u32
    }
}

/// GUI components
impl GroundStationGui {
    fn graph(&mut self, ui: &mut Ui, id_source: &str, field: Graphable, to_show: usize) {
        let to_skip = self.telemetry.len().saturating_sub(to_show);
        let points: Vec<PlotPoint> = self
            .graph_values
            .entry(field)
            .or_default()
            .iter()
            .skip(to_skip)
            .copied()
            .collect();
        let line = Line::new(PlotPoints::Owned(points)).name(field.as_str());
        // TODO: add a CoordinatesFormatter here
        Plot::new(id_source).show(ui, |plot_ui| plot_ui.line(line));
    }

    fn one_graph_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Graph showing: ");
            egui::ComboBox::from_id_source("main_graph")
                .selected_text(self.one_graph_shows.as_str())
                .width(120.0)
                .wrap(false)
                .show_ui(ui, |ui| {
                    for e in all::<Graphable>() {
                        ui.selectable_value(&mut self.one_graph_shows, e, e.as_str());
                    }
                });

            ui.label("No. Points: ");
            ui.add_enabled_ui(!self.one_graph_shows_all, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.one_graph_points, 5..=100).clamp_to_range(false),
                );
            });

            ui.label("Show all: ");
            ui.add(egui::Checkbox::new(&mut self.one_graph_shows_all, ""));

            self.missed_packets_widget(ui);
        });

        let to_show = if self.one_graph_shows_all {
            usize::MAX
        } else {
            self.one_graph_points
        };
        self.graph(ui, "main_plot", self.one_graph_shows, to_show);
    }

    fn all_graphs_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("No. Points: ");
            ui.add_enabled_ui(!self.all_graphs_show_all, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.all_graphs_points, 5..=100).clamp_to_range(false),
                );
            });

            ui.label("Show all: ");
            ui.add(egui::Checkbox::new(&mut self.all_graphs_show_all, ""));

            // show the missed packets
            self.missed_packets_widget(ui);
        });

        let to_show = if self.all_graphs_show_all {
            usize::MAX
        } else {
            self.all_graphs_points
        };
        let width = ui.available_width() / 5.0;
        let height = ui.available_height() / 2.0;

        Grid::new("all_graphs")
            .min_col_width(width)
            .max_col_width(width)
            .min_row_height(height)
            .spacing([5.0, 5.0])
            .show(ui, |ui| {
                for (i, field) in all::<Graphable>().enumerate() {
                    ui.vertical_centered(|ui| {
                        ui.heading(field.as_str());
                        self.graph(ui, field.as_str(), field, to_show);
                    });
                    if i == 4 || i == 9 {
                        ui.end_row();
                    }
                }
            });
    }

    fn data_table_view(&self, ui: &mut Ui) {
        const ROW_HEIGHT: f32 = 20.0;
        const COL_WIDTH_MULT: f32 = 13.0;

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_height(f32::INFINITY)
            .show(ui, |ui| {
                let mut builder = TableBuilder::new(ui).striped(true).stick_to_bottom(true);

                for field in all::<TelemetryField>() {
                    let min_width = field.as_str().len() as f32 * COL_WIDTH_MULT;
                    builder = builder.column(
                        Column::initial(min_width)
                            .at_least(min_width)
                            .resizable(true),
                    );
                }

                builder
                    .auto_shrink([false, false])
                    .max_scroll_height(f32::INFINITY)
                    .header(ROW_HEIGHT + 5.0, |mut header| {
                        for field in all::<TelemetryField>() {
                            header.col(|ui| {
                                ui.heading(field.as_str());
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(ROW_HEIGHT, self.telemetry.len(), |row_index, mut row| {
                            let telem = &self.telemetry[row_index];

                            for field in all::<TelemetryField>() {
                                row.col(|ui| {
                                    ui.label(telem.get_field(field));
                                });
                            }
                        });
                    });
            });
    }

    fn packets_view(&self, ui: &mut Ui) {
        const ROW_HEIGHT: f32 = 18.0;

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_height(f32::INFINITY)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .max_scroll_height(f32::INFINITY)
                    .column(Column::remainder())
                    .body(|body| {
                        body.rows(ROW_HEIGHT, self.packet_log.len(), |row_index, mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    self.packet_log[row_index].show(ui);
                                });
                            });
                        });
                    });
            });
    }

    // TODO: add a right click tooltip to re-sent the packet if it wasn't acked
    fn commands_view(&mut self, ui: &mut Ui) {
        const ROW_HEIGHT: f32 = 20.0;

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_height(f32::INFINITY)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .max_scroll_height(f32::INFINITY)
                    .column(Column::initial(65.0).resizable(true))
                    .column(Column::remainder())
                    .header(ROW_HEIGHT + 5.0, |mut row| {
                        row.col(|ui| {
                            ui.heading("Status");
                        });
                        row.col(|ui| {
                            ui.heading("Command");
                        });
                    })
                    .body(|body| {
                        body.rows(
                            ROW_HEIGHT,
                            self.command_history.len(),
                            |row_index, mut row| {
                                let (_, (cmd, status)) = self
                                    .command_history
                                    .iter()
                                    .nth(row_index)
                                    .expect("Tried to access a command that didn't exist.");

                                // show the status in the first column and the command in the second
                                row.col(|ui| {
                                    let color = match status {
                                        CommandStatus::Unsent => Color32::RED,
                                        CommandStatus::Sent { .. } => Color32::YELLOW,
                                        CommandStatus::Acked => Color32::GREEN,
                                    };

                                    let r = 10.0;
                                    let size = Vec2::splat(2.0 * r + 5.0);
                                    let (rect, _response) =
                                        ui.allocate_at_least(size, Sense::hover());

                                    ui.painter().circle_filled(rect.center(), r, color);
                                });

                                row.col(|ui| {
                                    ui.label(cmd);
                                });
                            },
                        );
                    });
            });
    }

    fn radio_window(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Serial port: ");
            ui.vertical_centered(|ui| {
                let Ok(ports) = serialport::available_ports() else {
                    ui.label("Failed to get availble ports.");
                    return;
                };

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_source("radio_port_combobox")
                        .selected_text(&self.radio_port)
                        .show_ui(ui, |ui| {
                            for port in ports {
                                if matches!(port.port_type, SerialPortType::UsbPort(_)) {
                                    let value = ui.selectable_value(
                                        &mut self.radio_port,
                                        port.port_name.clone(),
                                        &port.port_name,
                                    );

                                    if value.changed() {
                                        tracing::info!("Set radio port to {:?}", port.port_name);
                                        self.close_radio();
                                    }
                                }
                            }
                        });
                });
            });
        });

        ui.horizontal(|ui| {
            ui.label("Baud rate: ");
            ui.vertical_centered(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_source("radio_baud_combobox")
                        .selected_text(self.radio_baud.to_string())
                        .show_ui(ui, |ui| {
                            for baud in BAUD_RATES {
                                let value = ui.selectable_value(
                                    &mut self.radio_baud,
                                    baud,
                                    baud.to_string(),
                                );

                                if value.changed() {
                                    tracing::info!("Set radio baud to {baud}");
                                    if let Some(radio) = &self.radio {
                                        let mut guard = radio.lock();
                                        if let Err(e) = guard.set_baud_rate(self.radio_baud) {
                                            tracing::warn!("Encountered error setting baud rate on radio - {e:?}")
                                        }
                                    }
                                }
                            }
                        });
                });
            });
        });

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            // if we don't have a radio show an open button
            if self.radio.is_none() {
                if ui.button("Open port").clicked() {
                    self.open_radio_connection();
                }
            } else if ui.button("Disconnect").clicked() {
                self.close_radio();
            }
        });

        ui.separator();

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            if self.radio.is_some() {
                ui.colored_label(Color32::GREEN, "Connected");
            } else {
                ui.colored_label(Color32::RED, "Disconnected");
            }
        });
    }

    fn sim_window(&mut self, ui: &mut Ui) {
        ui.set_min_width(300.0);

        ui.horizontal(|ui| {
            ui.label("Choose file: ");
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                if ui.button("Open file").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Err(e) = self.load_sim_file(path) {
                            tracing::warn!("Failed to load sim file - {e:?}");
                        }
                    }
                }
            });
        });

        // if we have pressure values display a little graph of them
        if let Some(simps) = &self.simp_graph_values {
            Plot::new("simp_plot").view_aspect(1.5).show(ui, |ui| {
                let sent = SENT_SIMPS.load(ORDER) as usize;
                let sent_simps = simps[..sent].to_vec();
                let unsent_simps = simps[sent..].to_vec();
                let sent_line = Line::new(PlotPoints::Owned(sent_simps)).color(Color32::GREEN);
                let unsent_line = Line::new(PlotPoints::Owned(unsent_simps)).color(Color32::RED);
                ui.line(sent_line);
                ui.line(unsent_line);
            });

            ui.separator();

            // if the thread has been started add these buttons instead
            if SEND_THREAD_STARTED.load(ORDER) {
                ui.horizontal(|ui| {
                    // pause / play button
                    ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                        if SEND_THREAD_PAUSED.load(ORDER) {
                            if ui.button("play").clicked() {
                                tracing::info!("Playing simulation mode playback");
                                SEND_THREAD_PAUSED.store(false, ORDER);
                            }
                        } else if ui.button("pause").clicked() {
                            tracing::info!("Pausing simulation mode playback");
                            SEND_THREAD_PAUSED.store(true, ORDER);
                        }
                    });

                    // cancel button
                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        if ui.button("cancel").clicked() {
                            tracing::info!("Cancelling simulation mode sender thread");
                            SEND_THREAD_CANCEL.store(true, ORDER);
                        }
                    });
                });
            } else {
                let button = ui
                    .with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.button("Start sending")
                    })
                    .inner;

                if button.clicked() {
                    // set the thread as started
                    SEND_THREAD_STARTED.store(true, ORDER);
                    tracing::info!("Starting simulation mode sender thread");

                    // make a copy of the simp data to send to the thread
                    let Some(simp_data) = self.simp_values.clone() else {
                        tracing::error!("Encountered invalid state - simp_graph_values is Some, but simp_values is None, resetting both to None.");
                        self.simp_graph_values = None;
                        return;
                    };

                    let cmd_sender = self.cmd_sender.clone();
                    let thread_res = thread::Builder::new()
                        .name(String::from("simp"))
                        .spawn(move || Self::simp_thread(cmd_sender, simp_data));

                    if let Err(e) = thread_res {
                        tracing::error!("Failed to start SIMP command sender thread - {e:?}");
                    }
                }
            }
        }
    }

    fn simp_thread(cmd_sender: Sender<String>, simp_data: Vec<u32>) {
        tracing::info!("simp thread started");

        // send SIM,ENABLE then SIM,ACTIVATE
        cmd_sender
            .send(String::from("CMD,1047,SIM,ENABLE"))
            .map_err(|e| {
                tracing::error!("Failed to send SIM,ENABLE.");
                e
            })
            .expect("Failed to send SIM,ENABLE.");
        cmd_sender
            .send(String::from("CMD,1047,SIM,ACTIVATE"))
            .map_err(|e| {
                tracing::error!("Failed to send SIM,ACTIVATE.");
                e
            })
            .expect("Failed to send SIM,ACTIVATE.");

        // iterate through the commands, sleeping for one second before sending the next
        let mut simp_iter = simp_data.into_iter();
        loop {
            // if SEND_THREAD_CANCEL is true, replace with false and cancel this thread
            if let Ok(true) = SEND_THREAD_CANCEL.compare_exchange(true, false, ORDER, ORDER) {
                tracing::info!("Cancelling simulation mode thread");
                SEND_THREAD_PAUSED.store(false, ORDER);
                SEND_THREAD_STARTED.store(false, ORDER);
                SENT_SIMPS.store(0, ORDER);
                return;
            }

            // wait until we are unpaused to send the command
            if SEND_THREAD_PAUSED.load(ORDER) {
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            if let Some(simp) = simp_iter.next() {
                // send it!
                let cmd = format!("CMD,{TEAM_ID},SIMP,{simp}");
                if let Err(e) = cmd_sender.send(cmd) {
                    tracing::error!("Failed to send command over cmd_sender - {e:?}");
                } else {
                    SENT_SIMPS.fetch_add(1, ORDER);
                    tracing::debug!("SENT_SIMPS={SENT_SIMPS:?}");
                }
            } else {
                // we have reached the end of the iterator, cancel the thread
                return;
            }

            // sleep for a second
            thread::sleep(Duration::from_secs(1));
        }
    }

    fn missed_packets_widget(&self, ui: &mut Ui) {
        let color = match self.missed_packets {
            0 => Color32::GREEN,
            1..=10 => Color32::YELLOW,
            11.. => Color32::RED,
        };

        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            ui.colored_label(color, self.missed_packets.to_string());
            ui.label("Missed Packets: ");
        });
    }
}

// TODO: add radio / similar status indicators to the top bar
// TODO: make the simulation mode disable itself upon pausing etc
impl eframe::App for GroundStationGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // attempt to receive any telemetry thats availble from the radio
        self.recv_telem();

        // handle any command we have left to send
        self.handle_commands();

        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Manchester CanSat Project 🚀");
                ui.separator();

                egui::global_dark_light_mode_switch(ui);
                ui.separator();

                // main view buttons
                for view in all::<MainPanelView>() {
                    let label = ui.selectable_label(self.main_view == view, view.as_str());
                    if label.clicked() {
                        self.main_view = view;
                    }
                }

                // optional windows
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        // rightmost
                        ui.checkbox(&mut self.show_sim_window, "🔁 Simulation");
                        ui.checkbox(&mut self.show_command_window, "🖧 Commands");
                        ui.checkbox(&mut self.show_radio_window, "📻 Radio");
                        ui.checkbox(&mut self.show_settings_window, "⚙ Settings");
                        // leftmost
                    });
                });
            });
        });

        // scuffed but yeah
        let mut open;
        if self.show_settings_window {
            open = true;
            egui::Window::new("settings")
                .open(&mut open)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                });
            self.show_settings_window = open;
        }

        if self.show_command_window {
            open = true;

            // show the window and capture the response
            let resp = egui::Window::new("commands")
                .open(&mut open)
                .show(ctx, |ui| self.command_center.show(ui));

            // get the inner response and flatten the nested Options
            let maybe_cmd = resp.and_then(|inner| inner.inner.flatten());

            // send the command down the channel if there was one
            if let Some(cmd) = maybe_cmd {
                tracing::debug!("Sending cmd={cmd:?} over channel");
                // log any errors that occur
                if let Err(e) = self.cmd_sender.send(cmd) {
                    tracing::warn!("Failed to send command down channel - {e:?}");
                }
            }

            self.show_command_window = open;
        }

        if self.show_radio_window {
            open = true;
            egui::Window::new("radio")
                .open(&mut open)
                .show(ctx, |ui| self.radio_window(ui));
            self.show_radio_window = open;
        }

        if self.show_sim_window {
            open = true;
            egui::Window::new("simulation mode")
                .open(&mut open)
                .show(ctx, |ui| self.sim_window(ui));
            self.show_sim_window = open;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // match on the current view to decide what to draw
            match self.main_view {
                MainPanelView::OneGraph => self.one_graph_view(ui),
                MainPanelView::AllGraphs => self.all_graphs_view(ui),
                MainPanelView::Table => self.data_table_view(ui),
                MainPanelView::Packets => self.packets_view(ui),
                MainPanelView::Commands => self.commands_view(ui),
            }
        });

        // we must request a repaint otherwise we do not receive any data
        ctx.request_repaint();
    }
}

// the different states a command can have
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CommandStatus {
    // used if the radio isn't connected
    Unsent,
    // sent but not acked
    Sent { frame_id: u8 },
    // sent and acked
    Acked,
}

// the packets used to store in the packet log
pub enum Packet {
    Sent(TxRequest),
    Received(ReceivedPacket),
}

impl Packet {
    // show the packet in the given UI
    fn show(&self, ui: &mut Ui) {
        const SENT_COLOR: Color32 = Color32::from_rgb(20, 182, 51);
        const RECV_COLOR: Color32 = Color32::from_rgb(173, 0, 252);

        match self {
            Packet::Sent(req) => {
                ui.colored_label(SENT_COLOR, format!("{req}"));
            }
            Packet::Received(packet) => {
                ui.colored_label(RECV_COLOR, format!("{packet}"));
            }
        }
    }
}
