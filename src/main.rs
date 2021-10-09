mod image;

use crate::image::Image;
use ::image::{io::Reader as ImageReader, ImageBuffer, Rgb};
use std::{path::Path, time::Instant};

type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() < 2 || args.contains(&"-h".into()) || args.contains(&"--help".into()) {
        eprintln!("Usage: colorogram <input path> <output path>");
        return;
    }

    let input = &args[0];
    let output = &args[1];

    let data = if let Some(data) = read_image(&input) {
        data
    } else {
        eprintln!("colorogram: File '{}' not found", input);
        return;
    };

    let histogram = make_histogram(&data, data.width() as usize, (data.height() / 4) as usize);

    let histo_image: Image = histogram.into();
    let mut image: Image = data.into();
    let old_dims = image.resize_canvas((
        image.dimensions().width,
        image.dimensions().height + histo_image.dimensions().height,
    ));
    image.draw_image(&histo_image, (0, old_dims.height));
    image.save(output).unwrap();
}

fn make_histogram(img: &RgbImage, width: usize, height: usize) -> RgbImage {
    let (max, reds, greens, blues) = count_whole_image(img.as_flat_samples().samples);
    generate_histogram_image(max, &reds, &greens, &blues, width, height)
}

fn read_image<P: AsRef<Path>>(fname: P) -> Option<RgbImage> {
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
                "colorogram: Failed to read image '{}'",
                fname.as_ref().to_string_lossy()
            );
            return None;
        }
    }
    .to_rgb8();

    Some(img)
}

fn count_whole_image(data: &[u8]) -> (f64, [f64; 256], [f64; 256], [f64; 256]) {
    let mut reds = [0.0; 256];
    let mut greens = [0.0; 256];
    let mut blues = [0.0; 256];

    let start = Instant::now();
    for (index, value) in data.iter().enumerate() {
        if index % 3 == 0 {
            reds[*value as usize] += 1.0;
        } else if index % 3 == 1 {
            greens[*value as usize] += 1.0;
        } else {
            blues[*value as usize] += 1.0;
        }
    }
    //println!("Took {}s to count image", start.elapsed().as_secs_f32());

    // find the max in the channels
    let mut max = 0.0f64;
    for x in 0..256 {
        max = max.max(reds[x]).max(greens[x]).max(blues[x]);
    }

    (max, reds, greens, blues)
}

// Something about lumetri
// https://wildflourmedia.com/blog/lumetri-scopes-curves
fn make_waveform_graph(img: &RgbImage, width: usize, height: usize) -> RgbImage {
    let mut reds;
    let mut greens;
    let mut blues;

    //let width_scale = 10;
    let width = img.width() as usize;
    let height = img.height() as usize;
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

        //if x % width_scale == 9 {
        let mut max = 0.0f64;
        for x in 0..256 {
            max = max.max(reds[x]).max(greens[x]).max(blues[x]);
        }

        let normalize = |val: f64| -> u8 { (val.log(max) * 255.0) as u8 };
        //let normalize = |val: f64| -> u8 { ((val / max) * 256.0) as u8 };

        for y_inverse in 0..height {
            let index = ((y_inverse as f64 / height as f64) * 255.0).round() as usize;

            //let x = x / width_scale;
            buffer[((height - 1 - y_inverse) * width + x as usize) * 3] = normalize(reds[(index)]);
            buffer[((height - 1 - y_inverse) * width + x as usize) * 3 + 1] =
                normalize(greens[index]);
            buffer[((height - 1 - y_inverse) * width + x as usize) * 3 + 2] =
                normalize(blues[index]);
        }
        // }
    }

    ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap()
}

fn generate_histogram_image(
    max: f64,
    reds: &[f64; 256],
    greens: &[f64; 256],
    blues: &[f64; 256],
    width: usize,
    height: usize,
) -> RgbImage {
    let mut image = vec![0; width * height * 3];

    let lerp = |from: f64, to: f64, howfar: f32| from + (to - from) * howfar as f64;

    let mut channel = |channel: usize, x: usize, lereped_value: f64| {
        let col_height = ((lereped_value / max) * height as f64) as usize;

        for y in height - col_height..height {
            image[(y * width + x) * 3 + channel] = 255;
        }
    };

    for col in 0..255 {
        let low = (((width as f32) / 255.0) * col as f32) as u32;
        let high = ((width as f32 / 255.0) * (col + 1) as f32) as u32;
        let pixel_count = high - low;

        for x_off in 0..pixel_count {
            let x = (x_off + low) as usize;
            let percent = x_off as f32 / pixel_count as f32;

            channel(0, x, lerp(reds[col], reds[col + 1], percent));
            channel(1, x, lerp(greens[col], greens[col + 1], percent));
            channel(2, x, lerp(blues[col], blues[col + 1], percent));
        }
    }

    ImageBuffer::from_raw(width as u32, height as u32, image).unwrap()
}
