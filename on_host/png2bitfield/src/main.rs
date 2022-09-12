mod bit_field;


use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use png;

use crate::bit_field::DynamicBitField;


#[derive(Debug, Parser)]
struct Opts {
    pub png_file: PathBuf,
    pub output_file: PathBuf,
    #[clap(short = 'w', long)] pub cell_width: u32,
    #[clap(short = 'h', long)] pub cell_height: u32,
    #[clap(long)] pub cells_column_major: bool,
}


fn main() {
    let opts = Opts::parse();

    // read PNG file
    let png_file = File::open(&opts.png_file)
        .expect("failed to open PNG file");
    let png_decoder = png::Decoder::new(png_file);
    let mut png_reader = png_decoder.read_info()
        .expect("failed to read PNG info");

    if png_reader.info().width % opts.cell_width != 0 {
        panic!("PNG width {} not divisible by cell width {}", png_reader.info().width, opts.cell_width);
    }
    if png_reader.info().height % opts.cell_height != 0 {
        panic!("PNG height {} not divisible by cell height {}", png_reader.info().height, opts.cell_height);
    }

    if png_reader.info().color_type != png::ColorType::Indexed {
        panic!("PNG color type is {:?}; expected Indexed", png_reader.info().color_type);
    }
    if png_reader.info().bits_per_pixel() != 1 {
        panic!("PNG bits per pixel is {}, expected 1", png_reader.info().bits_per_pixel());
    }

    let palette = png_reader.info().palette.as_ref()
        .expect("PNG palette is missing");
    if palette.len() != 6 {
        panic!("PNG palette has {} bytes; expected 6", palette.len());
    }
    let one_is_white = if palette[0..3].iter().all(|b| *b == 0xFF) && palette[3..6].iter().all(|b| *b == 0x00) {
        true
    } else if palette[0..3].iter().all(|b| *b == 0x00) && palette[3..6].iter().all(|b| *b == 0xFF) {
        false
    } else {
        panic!("PNG palette does not consist of one instance of white (#FFFFFF) and one instance of black (#000000)");
    };

    if let Some(pt) = png_reader.info().trns.as_ref() {
        if pt.len() > 2 {
            panic!("PNG palette transparency has {} bytes; expected at most 2", pt.len());
        }
        if pt.iter().any(|b| *b != 0xFF) {
            panic!("PNG palette transparency {:?} has a non-opaque value (!= 255)", pt);
        }
    }

    // read the whole image into a bitfield
    let mut dbf = DynamicBitField::new();
    while let Some(row) = png_reader.next_row().expect("failed to read PNG row") {
        for b in row.data() {
            for i in 0..8 {
                let mut bit_is_set = *b & (1 << (7 - i)) == 0;
                if !one_is_white {
                    bit_is_set = !bit_is_set;
                }
                dbf.push(bit_is_set);
            }
        }
    }

    let cells_over_width = png_reader.info().width / opts.cell_width;
    let cells_over_height = png_reader.info().height / opts.cell_height;

    fn append_cell(
        row_index: u32, column_index: u32,
        cell_width: u32, cell_height: u32,
        image_width: u32,
        input_field: &DynamicBitField,
        output_field: &mut DynamicBitField,
        dest_index: &mut usize,
    ) {
        let cell_start =
            row_index * cell_height * image_width
            + column_index * cell_width
        ;

        // for each pixel within the cell
        for r2 in 0..cell_height {
            let row_start = cell_start + r2 * image_width;
            for c2 in 0..cell_width {
                let pixel_index = (row_start + c2).try_into().unwrap();
                if input_field.is_bit_set(pixel_index) {
                    output_field.set_bit(*dest_index);
                } else {
                    output_field.clear_bit(*dest_index);
                }
                *dest_index += 1;
            }
        }
    }

    // transfer the image into a correctly-oriented bitfield
    let mut dbf2 = DynamicBitField::with_capacity_bytes(dbf.size_bytes());
    if opts.cells_column_major {
        let mut dest_index = 0;
        for c in 0..cells_over_width {
            for r in 0..cells_over_height {
                append_cell(
                    r, c,
                    opts.cell_width, opts.cell_height,
                    png_reader.info().width,
                    &dbf, &mut dbf2,
                    &mut dest_index,
                );
            }
        }
    } else {
        // for each cell
        let mut dest_index = 0;
        for r in 0..cells_over_height {
            for c in 0..cells_over_width {
                append_cell(
                    r, c,
                    opts.cell_width, opts.cell_height,
                    png_reader.info().width,
                    &dbf, &mut dbf2,
                    &mut dest_index,
                );
            }
        }
    }

    {
        // write output file
        let mut output_file = File::create(&opts.output_file)
            .expect("failed to create output file");
        output_file.write(dbf2.as_bytes())
            .expect("failed to write output bytes");
    }
}
