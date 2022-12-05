use std::collections::HashSet;
use std::fmt;
use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::RwLock;
use std::sync::mpsc;
use std::thread::{sleep, spawn};
use std::time::Duration;

use clap::Parser;
use eframe::egui;
use eframe::epaint::{Color32, ColorImage, TextureHandle};
use egui::TextureFilter;
use egui::menu;
use from_to_repr::FromToRepr;
use image::ImageFormat;
use image::io::Reader as ImageReader;
use once_cell::sync::OnceCell;


static DISPLAY_STATE: OnceCell<RwLock<DisplayState>> = OnceCell::new();

type CommMessage = Vec<u8>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Channel {
    Off,
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D10,
    D11,
    D12,
    D13,
    D14,
    D15,
}
impl Channel {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Ch1 => "CHAN1",
            Self::Ch2 => "CHAN2",
            Self::Ch3 => "CHAN3",
            Self::Ch4 => "CHAN4",
            Self::D0 => "D0",
            Self::D1 => "D1",
            Self::D2 => "D2",
            Self::D3 => "D3",
            Self::D4 => "D4",
            Self::D5 => "D5",
            Self::D6 => "D6",
            Self::D7 => "D7",
            Self::D8 => "D8",
            Self::D9 => "D9",
            Self::D10 => "D10",
            Self::D11 => "D11",
            Self::D12 => "D12",
            Self::D13 => "D13",
            Self::D14 => "D14",
            Self::D15 => "D15",
        }
    }
}
impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Ch1 => write!(f, "Ch1"),
            Self::Ch2 => write!(f, "Ch2"),
            Self::Ch3 => write!(f, "Ch3"),
            Self::Ch4 => write!(f, "Ch4"),
            Self::D0 => write!(f, "D0"),
            Self::D1 => write!(f, "D1"),
            Self::D2 => write!(f, "D2"),
            Self::D3 => write!(f, "D3"),
            Self::D4 => write!(f, "D4"),
            Self::D5 => write!(f, "D5"),
            Self::D6 => write!(f, "D6"),
            Self::D7 => write!(f, "D7"),
            Self::D8 => write!(f, "D8"),
            Self::D9 => write!(f, "D9"),
            Self::D10 => write!(f, "D10"),
            Self::D11 => write!(f, "D11"),
            Self::D12 => write!(f, "D12"),
            Self::D13 => write!(f, "D13"),
            Self::D14 => write!(f, "D14"),
            Self::D15 => write!(f, "D15"),
        }
    }
}


const ALL_CHANNELS_AND_OFF: [Channel; 21] = [
    Channel::Off,
    Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4,
    Channel::D0, Channel::D1, Channel::D2, Channel::D3,
    Channel::D4, Channel::D5, Channel::D6, Channel::D7,
    Channel::D8, Channel::D9, Channel::D10, Channel::D11,
    Channel::D12, Channel::D13, Channel::D14, Channel::D15,
];


#[derive(Parser)]
struct Opts {
    #[arg(short, long, default_value = "5555")]
    pub port: u16,

    pub ip_address: IpAddr,
}


#[derive(Clone)]
struct DisplayState {
    pub ordinal: usize,
    pub image: egui::ImageData,
}


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum WindowId {
    Channels,
    Decoder,

    DecoderParallelClockComboBox(u64),
}
impl WindowId {
    pub fn to_id(&self) -> egui::Id {
        egui::Id::new(self)
    }
}


fn main() {
    let opts = Opts::parse();

    let sock_addr = SocketAddr::from((opts.ip_address, opts.port));
    let stream = TcpStream::connect(sock_addr)
        .expect("failed to connect to scope");

    let display_state = DisplayState {
        ordinal: 0,
        image: egui::ImageData::Color(ColorImage::new([1, 1], Color32::from_gray(0))),
    };
    let display_state_lock = RwLock::new(display_state);
    if DISPLAY_STATE.set(display_state_lock).is_err() {
        panic!("DISPLAY_STATE already set?!");
    }

    let (command_sender, command_receiver) = mpsc::channel();
    spawn(move || {
        comm_thread(stream, command_receiver)
    });

    // wait for initial image data
    loop {
        {
            let state_guard = DISPLAY_STATE
                .get().expect("DISPLAY_STATE not set?!")
                .read().expect("DISPLAY_STATE poisoned?!");
            if state_guard.ordinal != 0 {
                break;
            }
        }

        sleep(Duration::from_millis(50));
    }

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rigoletto",
        native_options,
        Box::new(|cc| Box::new(RigolettoApp::new(cc, command_sender))),
    );
}

fn comm_thread(mut stream: TcpStream, command_receiver: mpsc::Receiver<CommMessage>) {
    loop {
        // handle outgoing commands
        match command_receiver.try_recv() {
            Ok(msg) => {
                stream.write_all(&msg)
                    .expect("failed to write into stream");
                continue;
            },
            Err(mpsc::TryRecvError::Disconnected) => break,
            Err(mpsc::TryRecvError::Empty) => {},
        }

        // update display
        stream.write_all(b":DISP:DATA?\n")
            .expect("failed to write display update request");
        let mut display_start_buf = [0u8; 11];
        stream.read_exact(&mut display_start_buf[0..2])
            .expect("failed to read start of display data");
        if display_start_buf[0] != b'#' {
            panic!("display data does not start with '#'");
        }
        if display_start_buf[1] < b'0' || display_start_buf[1] > b'9' {
            panic!("display data length digit count not an ASCII digit");
        }
        let display_data_length_digits = usize::from(display_start_buf[1] - b'0');
        let mut display_data_length: usize = 0;
        stream.read_exact(&mut display_start_buf[2..(2+display_data_length_digits)])
            .expect("failed to read display data length digits");
        for digit_ascii in &display_start_buf[2..(2+display_data_length_digits)] {
            if *digit_ascii < b'0' || *digit_ascii > b'9' {
                panic!("unexpected length digit 0x{:02X}", digit_ascii);
            }
            display_data_length *= 10;
            display_data_length += usize::from(*digit_ascii - b'0');
        }

        // read the image data including the newline following it
        let mut image_buf = vec![0u8; display_data_length+1];
        stream.read_exact(&mut image_buf)
            .expect("failed to read image");
        let mut reader = ImageReader::new(Cursor::new(&image_buf[0..display_data_length]));
        reader.set_format(ImageFormat::Bmp);
        let image = reader.decode()
            .expect("failed to decode image");

        let color_image = ColorImage::from_rgba_unmultiplied(
            [
                image.width().try_into().unwrap(),
                image.height().try_into().unwrap(),
            ],
            image.to_rgba8().as_flat_samples().as_slice(),
        );
        let image_data = egui::ImageData::Color(color_image);

        {
            let mut state_guard = DISPLAY_STATE
                .get().expect("DISPLAY_STATE not set?!")
                .write().expect("DISPLAY_STATE poisoned?!");
            if state_guard.image != image_data {
                state_guard.image = image_data;
                state_guard.ordinal = state_guard.ordinal.wrapping_add(1);
            }
        }
    }
}


#[derive(Clone)]
struct RigolettoApp {
    last_display_ordinal: usize,
    display: TextureHandle,
    command_sender: mpsc::Sender<CommMessage>,
    open_windows: HashSet<WindowId>,
    //decoder_to_parallel_clk: HashMap<u64, usize>,
}

impl RigolettoApp {
    fn new(cc: &eframe::CreationContext<'_>, command_sender: mpsc::Sender<CommMessage>) -> Self {
        let (ordinal, image) = {
            let display_guard = DISPLAY_STATE
                .get().expect("DISPLAY_STATE not set?!")
                .read().expect("DISPLAY_STATE is poisoned?!");
            (display_guard.ordinal, display_guard.image.clone())
        };

        Self {
            last_display_ordinal: ordinal,
            display: cc.egui_ctx.load_texture(
                "scope",
                image,
                TextureFilter::Linear,
            ),
            command_sender,
            open_windows: HashSet::new(),
        }
    }

    fn send_command<C: AsRef<str>>(&self, command: C) {
        self.command_sender
            .send(command.as_ref().as_bytes().to_vec())
            .expect("failed to send command")
    }
}

impl eframe::App for RigolettoApp {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ctx, |ui| {
            {
                // check if image has updated
                let display_guard = DISPLAY_STATE
                    .get().expect("DISPLAY_STATE not set?!")
                    .read().expect("DISPLAY_STATE is poisoned?!");
                if display_guard.ordinal != self.last_display_ordinal {
                    self.display.set(
                        display_guard.image.clone(),
                        TextureFilter::Linear,
                    );
                }
            }

            menu::bar(ui, |ui| {
                if ui.button("Channels").clicked() {
                    self.open_windows.insert(WindowId::Channels);
                }
                if ui.button("Decoder").clicked() {
                    self.open_windows.insert(WindowId::Decoder);
                }
                if ui.button("Single").clicked() {
                    self.send_command(":SING\n");
                }
            });

            if self.open_windows.contains(&WindowId::Channels) {
                egui::Window::new("Channels")
                    .id(WindowId::Channels.to_id())
                    .show(ctx, |ui| {
                        for channel in 1..=4 {
                            ui.horizontal_top(|ui| {
                                ui.label(format!("Channel {}", channel));
                                if ui.button("On").clicked() {
                                    self.send_command(format!(":CHAN{}:DISP 1\n", channel));
                                }
                                if ui.button("Off").clicked() {
                                    self.send_command(format!(":CHAN{}:DISP 0\n", channel));
                                }
                                ui.end_row();
                            });
                        }
                        if ui.button("Close").clicked() {
                            self.open_windows.remove(&WindowId::Channels);
                        }
                    });
            }
            if self.open_windows.contains(&WindowId::Decoder) {
                egui::Window::new("Decoder")
                    .id(WindowId::Decoder.to_id())
                    .show(ctx, |ui| {
                        for decoder in 1..=2 {
                            ui.collapsing(format!("Decoder {}", decoder), |ui| {
                                ui.horizontal(|ui| {
                                    // On/Off
                                    if ui.button("On").clicked() {
                                        self.send_command(format!(":DEC{}:DISP 1\n", decoder));
                                    }
                                    if ui.button("Off").clicked() {
                                        self.send_command(format!(":DEC{}:DISP 0\n", decoder));
                                    }
                                    ui.end_row();

                                    ui.label("Format");
                                    let formats = [
                                        ("Hex", "HEX"),
                                        ("ASCII", "ASC"),
                                        ("Decimal", "DEC"),
                                        ("Binary", "BIN"),
                                        ("Line", "LINE"),
                                    ];
                                    for (label, arg) in formats {
                                        if ui.button(label).clicked() {
                                            self.send_command(format!(":DEC{}:FORM {}\n", decoder, arg));
                                        }
                                    }
                                    ui.end_row();

                                    /*
                                    ui.collapsing(format!("Parallel {}", decoder), |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.button("Activate").clicked() {
                                                self.send_command(format!(":DEC{}:MODE PAR\n", decoder, arg));
                                            }
                                            ui.end_row();

                                            egui::ComboBox::from_id_source(Window::IdDecoderParallelClockComboBox(decoder))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(current_value, selected_value, text)
                                                });
                                        });
                                    });
                                    */

                                    let protocols = [
                                        ("Parallel", "PAR"),
                                        ("UART", "UART"),
                                        ("SPI", "SPI"),
                                        ("I\u{B2}C", "IIC"),
                                        ("Line", "LINE"),
                                    ];
                                    for (label, arg) in protocols {
                                        if ui.button(label).clicked() {
                                            self.send_command(format!(":DEC{}:MODE {}\n", decoder, arg));
                                        }
                                    }
                                    ui.end_row();
                                });
                            });
                        }
                        if ui.button("Close").clicked() {
                            self.open_windows.remove(&WindowId::Decoder);
                        }
                    });
            }

            ui.image(self.display.id(), self.display.size_vec2());
       });
   }
}
