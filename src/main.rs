use std::path::Path;

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();

    if files.len() == 0 {
        eprintln!("Usage: colorogram <png1> <png2> ...");
        return;
    }

    for file in files {
        make_histogram(file);
    }
}

fn make_histogram<P: AsRef<Path>>(fname: P) {
    let file = fname.as_ref();
    let mut ofile = file.to_path_buf();
    ofile.set_file_name(format!(
        "{}_histogram.png",
        file.file_stem().unwrap().to_string_lossy()
    ));

    let img_data = read_png(file);
    let (max, reds, greens, blues) = count(&img_data);
    let histogram = generate_image(max, &reds, &greens, &blues);
    write_png(ofile, &histogram, 256, 192);
}

fn read_png<P: AsRef<Path>>(fname: P) -> Vec<u8> {
    use png::Decoder;
    use std::fs::File;

    let decoder = Decoder::new(File::open(fname).unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buffer = vec![0; info.buffer_size()];
    reader.next_frame(&mut buffer).unwrap();

    buffer
}

fn write_png<P: AsRef<Path>>(fname: P, data: &Vec<u8>, width: u32, height: u32) {
    use png::Encoder;
    use std::fs::File;
    use std::io::BufWriter;

    let mut encoder = Encoder::new(BufWriter::new(File::create(fname).unwrap()), width, height);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);

    encoder
        .write_header()
        .unwrap()
        .write_image_data(data)
        .unwrap()
}

fn count(data: &Vec<u8>) -> (f64, [f64; 256], [f64; 256], [f64; 256]) {
    let mut reds = [0.0; 256];
    let mut greens = [0.0; 256];
    let mut blues = [0.0; 256];

    for (index, value) in data.iter().enumerate() {
        if index % 3 == 0 {
            reds[*value as usize] += 1.0;
        } else if index % 3 == 1 {
            greens[*value as usize] += 1.0;
        } else {
            blues[*value as usize] += 1.0;
        }
    }

    // find the max in the channels
    let mut max = 0.0f64;
    for x in 0..256 {
        max = max.max(reds[x]).max(greens[x]).max(blues[x]);
    }

    (max, reds, greens, blues)
}

fn generate_image(max: f64, reds: &[f64; 256], greens: &[f64; 256], blues: &[f64; 256]) -> Vec<u8> {
    let width = 256;
    let height = 192;
    let mut image = vec![0; width * height * 3];

    let mut draw = |channel: usize, x: usize, value: f64| {
        let col_height = ((value / max) * height as f64) as usize;

        for y_inverse in 0..col_height {
            image[((height - 1 - y_inverse) * width + x) * 3 + channel] = 255;
        }
    };

    for x in 0..256 {
        draw(0, x, reds[x]);
        draw(1, x, greens[x]);
        draw(2, x, blues[x]);
    }

    image
}
