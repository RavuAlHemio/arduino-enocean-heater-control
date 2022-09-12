use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use png;


#[derive(Parser)]
struct Args {
    png_path: PathBuf,
    out_path: Option<PathBuf>,
    #[clap(short, long)] mode: Option<ImageMode>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ImageMode {
    RGB565,
    RGB666,
}
impl Default for ImageMode {
    fn default() -> Self { Self::RGB565 }
}
impl FromStr for ImageMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rgb565" => Ok(Self::RGB565),
            "rgb666" => Ok(Self::RGB666),
            _ => Err("unknown image mode"),
        }
    }
}


fn main() {
    let args = Args::parse();
    let png_file = File::open(&args.png_path)
        .expect("failed to open PNG file");
    let decoder = png::Decoder::new(png_file);
    let mut reader = decoder.read_info()
        .expect("failed to read file info");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];

    let mut out_file = args.out_path.map(|op|
        File::create(&op)
            .expect("failed to create output file")
    );

    if bytes.len() % 3 != 0 {
        panic!("not RGB?");
    }

    let mut counter = 0;
    for rgb in bytes.chunks(3) {
        match args.mode.unwrap_or(ImageMode::RGB565) {
            ImageMode::RGB565 => {
                let new_r = rgb[0] >> 3;
                let new_g = rgb[1] >> 2;
                let new_b = rgb[2] >> 3;

                let rgb_565 =
                    u16::from(new_r) << 11
                    | u16::from(new_g) << 5
                    | u16::from(new_b) << 0
                ;

                let bytes_565 = rgb_565.to_be_bytes();

                if let Some(of) = &mut out_file {
                    of.write(&bytes_565)
                        .expect("failed to write to output file");
                } else {
                    print!("0x{:02X}, 0x{:02X},", bytes_565[0], bytes_565[1]);
                    counter += 2;
                    if counter % 16 == 0 {
                        println!();
                    } else {
                        print!(" ");
                    }
                }
            },
            ImageMode::RGB666 => {
                let new_r = rgb[0] >> 2;
                let new_g = rgb[1] >> 2;
                let new_b = rgb[2] >> 2;

                if let Some(of) = &mut out_file {
                    of.write(&[new_r, new_g, new_b])
                        .expect("failed to write to output file");
                } else {
                    print!("0x{:02X}, 0x{:02X}, 0x{:02X}", new_r, new_g, new_b);
                    counter += 3;
                    if counter % 12 == 0 {
                        println!();
                    } else {
                        print!(" ");
                    }
                }
            },
        }
    }
}
