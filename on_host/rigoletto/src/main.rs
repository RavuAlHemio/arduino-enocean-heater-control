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
use image::ImageFormat;
use image::io::Reader as ImageReader;
use once_cell::sync::OnceCell;


static DISPLAY_STATE: OnceCell<RwLock<DisplayState>> = OnceCell::new();

type CommMessage = Vec<u8>;


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
        }
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

            ui.image(self.display.id(), self.display.size_vec2());
       });
   }
}
