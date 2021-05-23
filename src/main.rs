use image::{io::Reader as ImageReader, ImageBuffer, Rgb};
use std::path::Path;

type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();

    if files.len() == 0 {
        eprintln!("Usage: colorogram <png1> <png2> ...");
        return;
    }

    for file in files {
        //make_histogram(file);
        let img = read_png(file).unwrap();
        let lumetri = make_lumetri_scope(&img);
        write_png("test_lumetri.png", lumetri);
    }
}

fn make_histogram<P: AsRef<Path>>(fname: P) {
    let file = fname.as_ref();
    let mut ofile = file.to_path_buf();
    ofile.set_file_name(format!(
        "{}_histogram.png",
        file.file_stem().unwrap().to_string_lossy()
    ));

    let img_data = if let Some(data) = read_png(file) {
        data
    } else {
        return;
    };

    let (max, reds, greens, blues) = count_whole_image(&img_data.into_flat_samples().samples);
    let histogram = generate_image(max, &reds, &greens, &blues);
    write_png(ofile, histogram);
}

fn read_png<P: AsRef<Path>>(fname: P) -> Option<RgbImage> {
    let img = match ImageReader::open(fname.as_ref()) {
        Ok(read_img) => match read_img.decode() {
            Ok(decoded) => decoded,
            Err(_e) => {
                eprintln!(
                    "colorogram: Failed to decode image '{}'",
                    fname.as_ref().to_string_lossy()
                );
                return None;
            }
        },
        Err(_e) => {
            eprintln!(
                "towebp: Failed to read image '{}'",
                fname.as_ref().to_string_lossy()
            );
            return None;
        }
    }
    .to_rgb8();

    Some(img)
}

fn write_png<P: AsRef<Path>>(fname: P, imgbuf: RgbImage) {
    imgbuf
        .save_with_format(fname, image::ImageFormat::Png)
        .unwrap()
}

fn count_whole_image(data: &Vec<u8>) -> (f64, [f64; 256], [f64; 256], [f64; 256]) {
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

fn make_lumetri_scope(img: &RgbImage) -> RgbImage {
    let mut reds = [0.0; 256];
    let mut greens = [0.0; 256];
    let mut blues = [0.0; 256];

    let width = img.width() as usize;
    let height = 2048;
    let mut buffer = vec![0; width * height * 3];

    for x in 0..img.width() {
        reds = [0.0; 256];
        greens = [0.0; 256];
        blues = [0.0; 256];

        for y in 0..img.height() {
            let color = img.get_pixel(x, y);
            let (r, g, b) = (color.0[0], color.0[1], color.0[2]);
            reds[r as usize] += 1.0;
            greens[g as usize] += 1.0;
            blues[b as usize] += 1.0;
        }

        let mut max = 0.0f64;
        for x in 0..256 {
            max = max.max(reds[x]).max(greens[x]).max(blues[x]);
        }

        let normalize = |val: f64| -> u8 { (val.log(max) * 255.0) as u8 };
        //let normalize = |val: f64| -> u8 { ((val / max) * 256.0) as u8 };

        for y_inverse in 0..height {
            let index = ((y_inverse as f64 / height as f64) * 255.0).round() as usize;
            //let index = y_inverse;

            buffer[((height - 1 - y_inverse) * width + x as usize) * 3] = normalize(reds[(index)]);
            buffer[((height - 1 - y_inverse) * width + x as usize) * 3 + 1] =
                normalize(greens[index]);
            buffer[((height - 1 - y_inverse) * width + x as usize) * 3 + 2] =
                normalize(blues[index]);
        }
    }

    ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap()
}

fn generate_image(
    max: f64,
    reds: &[f64; 256],
    greens: &[f64; 256],
    blues: &[f64; 256],
) -> RgbImage {
    let width = 256;
    let height = 192;
    let mut image = vec![0; width * height * 3];

    let mut draw = |channel: usize, x: usize, value: f64| {
        //let col_height = (value.log(max) * height as f64) as usize;
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

    ImageBuffer::from_raw(width as u32, height as u32, image).unwrap()
}
