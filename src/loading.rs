use {
    quicksilver::{
        graphics::Graphics,
        graphics::Image as QSImage,
        golem::ColorFormat,
    },
    image::ImageBuffer,
    rand::seq::SliceRandom,
};

pub fn loading_screen(gfx: Graphics) -> QSImage {
    let mut raw = ImageBuffer::new(160, 160);
    for (x, y, pix) in raw.enumerate_pixels_mut() {
        let delta = (x as f32 - 79.5, y as f32 - 79.5);
        let angle = (delta.1).atan2(delta.0);
        let distance = (delta.0.abs().powf(2.) + delta.1.abs().powf(2.)).sqrt() / 115.;
        let saturation = 1. - ((distance - 0.5).abs() * 2.);
        let hsl = palette::Hsl::new(palette::RgbHue::from_radians(angle), saturation, distance);
        let rgb = palette::Srgb::from(hsl);
        *pix = image::Rgb([(rgb.red * 255.) as u8, (rgb.green * 255.) as u8, (rgb.blue * 255.) as u8]);
    }
    let mut dithered = image::ImageBuffer::new(320, 320);
    let mut rng = rand::thread_rng();
    for (rx, ry, pixel) in raw.enumerate_pixels() {
        let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
        let count = [
            (r as f32 / (255. / 4.)).round() as u8,
            (g as f32 / (255. / 4.)).round() as u8,
            (b as f32 / (255. / 4.)).round() as u8,
        ];
        let mut channels = [[0u8; 4], [0u8; 4], [0u8; 4]];
        let mut pixels = [[image::Rgb([0, 0, 0]); 2]; 2];
        for c in 0..3 {
            for i in 0..count[c] {
                channels[c][i as usize] = 255u8;
            }
            channels[c].shuffle(&mut rng);
            for x in 0..2 {
                for y in 0..2 {
                    pixels[x][y][c] = channels[c][x * 2 + y];
                }
            }
        }
        for x in 0..2 {
            for y in 0..2 {
                dithered.put_pixel(rx * 2 + x, ry * 2 + y, pixels[x as usize][y as usize]);
            }
        }
    }
    QSImage::from_raw(
        &gfx,
        Some(&dithered.into_raw()),
        320,
        320,
        ColorFormat::RGB,
    ).unwrap()
}