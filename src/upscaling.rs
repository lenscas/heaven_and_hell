#![allow(dead_code)]
use {
    image::{DynamicImage, ImageBuffer},
    quicksilver::{golem::ColorFormat, graphics::Graphics, graphics::Image as QSImage, Result},
    std::collections::HashMap,
};
pub struct Loader {
    loaded: HashMap<String, QSImage>,
}
impl Loader {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
        }
    }
    pub fn strip_image_parts(bytes: Vec<u8>) -> Vec<u8> {
        image::load_from_memory(&bytes)
            .expect("not a good image")
            .to_rgba()
            .into_raw()
    }
    pub fn scale(
        &mut self,
        bytes: Vec<u8>,
        path: String,
        gfx: &Graphics,
        (x, y): (u32, u32),
        is_raw: bool,
    ) -> Result<QSImage> {
        let bytes = if is_raw {
            bytes
        } else {
            Self::strip_image_parts(bytes)
        };
        if !self.loaded.contains_key(&path) {
            let raw: ImageBuffer<image::Rgba<u8>, _> = ImageBuffer::from_raw(x, y, bytes.clone())
                .unwrap_or_else(|| {
                    let x: ImageBuffer<image::Rgb<u8>, _> =
                        ImageBuffer::from_raw(x, y, bytes).unwrap();
                    DynamicImage::ImageRgb8(x).into_rgba()
                });
            let mut scaled = image::ImageBuffer::<image::Rgba<u8>, _>::new(x * 2, y * 2);
            for (rx, ry, &pixel) in raw.enumerate_pixels() {
                for x in 0..2 {
                    for y in 0..2 {
                        scaled.put_pixel(rx * 2 + x, ry * 2 + y, pixel);
                    }
                }
            }
            let image = QSImage::from_raw(
                gfx,
                Some(&scaled.into_raw()),
                x * 2,
                y * 2,
                ColorFormat::RGBA,
            )?;
            self.loaded.insert(path.clone(), image);
        }
        Ok(self.loaded.get(&path).unwrap().clone())
    }

    pub async fn load_and_scale(
        &mut self,
        path: String,
        (x, y): (u32, u32),
        gfx: &Graphics,
    ) -> Result<QSImage> {
        self.scale(
            quicksilver::load_file(path.clone()).await?,
            path,
            gfx,
            (x, y),
            false,
        )
    }
}
