use std::{ops::Add, path::Path};

pub struct Image {
    dims: Dimensions,
    container: Vec<Color>,
}

impl Image {
    pub fn from_buffer(buf: &[u8], width: usize, height: usize) -> Self {
        let colors = buf
            .chunks(3)
            .map(|s| Color::new(s[0], s[1], s[2]))
            .collect();
        Self {
            dims: (width, height).into(),
            container: colors,
        }
    }

    pub fn resize_canvas<D: Into<Dimensions>>(&mut self, dimensions: D) -> Dimensions {
        let old_dimensions = self.dims;
        let dims = dimensions.into();
        let mut new = vec![Color::default(); dims.width * dims.height];

        for x in 0..self.dims.width.min(dims.width) {
            for y in 0..self.dims.height.min(dims.height) {
                new[dims.xytoi(x, y)] = self.container[self.dims.xytoi(x, y)];
            }
        }
        self.dims = dims;
        self.container = new;
        old_dimensions
    }

    pub fn draw_image<D: Into<Dimensions>>(&mut self, other: &Image, pos: D) {
        let pos = pos.into();
        for x in pos.width..pos.width + other.dims.width {
            for y in pos.height..pos.height + other.dims.height {
                let ours = self.dims.xytoi(x, y);
                let theirs = other.dims.xytoi(x - pos.width, y - pos.height);

                if ours >= self.dims.maxi() {
                    continue;
                }

                self.container[ours] = other.container[theirs];
            }
        }
    }

    // Thank you for the help, waffle <3
    pub fn as_bytes(&self) -> &[u8] {
        let colors: &[Color] = &self.container;
        unsafe {
            // SAFETY: Color is repr(C) and pod (__doesn't have holes or bools__)
            match colors.align_to::<u8>() {
                ([], bytes, []) => bytes,
                (..) => unreachable!("align of u8 <= align of Color"),
            }
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dims
    }

    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<(), ::image::ImageError> {
        ::image::save_buffer_with_format(
            path,
            self.as_bytes(),
            self.dims.width as u32,
            self.dims.height as u32,
            ::image::ColorType::Rgb8,
            ::image::ImageFormat::Png,
        )
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ::image::ImageError> {
        let path = path.as_ref();
        let format = ::image::ImageFormat::from_path(path)?;

        ::image::save_buffer_with_format(
            path,
            self.as_bytes(),
            self.dims.width as u32,
            self.dims.height as u32,
            ::image::ColorType::Rgb8,
            format,
        )
    }
}

impl From<::image::ImageBuffer<::image::Rgb<u8>, Vec<u8>>> for Image {
    fn from(img: ::image::ImageBuffer<::image::Rgb<u8>, Vec<u8>>) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        let img = img.into_flat_samples().samples;

        Self::from_buffer(&img, width, height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    pub fn new(width: usize, height: usize) -> Self {
        Dimensions { width, height }
    }

    pub fn xytoi(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn maxi(&self) -> usize {
        self.width * self.height
    }
}

impl From<(usize, usize)> for Dimensions {
    fn from(t: (usize, usize)) -> Self {
        Dimensions {
            width: t.0,
            height: t.1,
        }
    }
}

impl Add<Dimensions> for Dimensions {
    type Output = Dimensions;

    fn add(self, rhs: Dimensions) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

impl From<&[u8]> for Color {
    fn from(s: &[u8]) -> Self {
        Color {
            r: s[0],
            g: s[1],
            b: s[2],
        }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(t: (u8, u8, u8)) -> Self {
        Color {
            r: t.0,
            g: t.1,
            b: t.2,
        }
    }
}

impl IntoIterator for Color {
    type Item = u8;

    type IntoIter = <[u8; 3] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <[u8; 3] as IntoIterator>::into_iter([self.r, self.g, self.b])
    }
}
