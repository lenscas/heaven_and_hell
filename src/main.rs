//use crate::screens::screen::Screen;
use mergui::Context;
use quicksilver::lifecycle::Event::{self, PointerMoved};
use quicksilver::{
    geom::Vector,
    graphics::{Graphics, Image as QSImage},
    lifecycle::{run, EventStream, Settings, Window},
    load_file,
    mint::Vector2,
    Result,
};
mod directions;
mod screens;
use async_trait::async_trait;
use quicksilver::golem::ColorFormat;
use std::collections::HashMap;
use rand::seq::SliceRandom;

#[async_trait(?Send)]
pub(crate) trait Screen {
    async fn draw(&mut self, wrapper: &mut Wrapper<'_>) -> Result<()>;
    async fn update(&mut self, wrapper: &mut Wrapper<'_>) -> Result<Option<Box<dyn Screen>>>;
    async fn event(
        &mut self,
        _wrapper: &mut Wrapper<'_>,
        _event: &Event,
    ) -> Result<Option<Box<dyn Screen>>> {
        Ok(None)
    }
}

fn main() {
    run(
        Settings {
            size: Vector::new(640, 640).into(),
            title: "Heaven and Hello",
            resizable: false,

            ..Settings::default()
        },
        app,
    );
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Block {
    Dirt,
    Air,
    PlayerStart,
}

impl Block {
    pub fn can_render(self) -> bool {
        match self {
            Block::Air | Block::PlayerStart => false,
            _ => true,
        }
    }
    pub fn is_colideable(self) -> bool {
        match self {
            Block::Air | Block::PlayerStart => false,
            x => true,
        }
    }
}

impl From<char> for Block {
    fn from(c: char) -> Self {
        let c = c
            .to_lowercase()
            .next()
            .expect(&format!("{} was not able to be lowercased", c));
        match c {
            'b' => Block::Dirt,
            'a' => Block::Air,
            'p' => Block::PlayerStart,
            x => unreachable!("Got invalid char {}", x),
        }
    }
}

impl From<Block> for &'static str {
    fn from(from: Block) -> Self {
        match from {
            Block::Dirt => "blocks/dirt.png",
            Block::Air | Block::PlayerStart => panic!("has no valid image"),
        }
    }
}
impl From<Block> for String {
    fn from(from: Block) -> Self {
        let s: &'static str = from.into();
        String::from(s)
    }
}

pub(crate) struct Wrapper<'a> {
    pub window: Window,
    pub gfx: Graphics,
    pub events: EventStream,
    pub context: Context<'a>,
    pub cursor_at: Vector2<f32>,
    pub levels: HashMap<u32, Vec<Vec<Block>>>,
    pub images: HashMap<(u32, u32), QSImage>,
}

impl<'a> Wrapper<'a> {
    pub(crate) fn get_cursor_loc(&self) -> Vector2<f32> {
        self.cursor_at
    }
    pub(crate) fn get_pos_vector(&self, x: f32, y: f32) -> Vector {
        let res = self.window.size();
        Vector::new(x * res.x, y * res.y)
    }
    pub(crate) async fn get_block(&mut self, block: Block, x: f64, y: f64) -> QSImage {
        let bx = x.floor() as u32 / 32;
        let by = y.floor() as u32 / 32;
        if !self.images.contains_key(&(bx, by)) {
            let raw = image::load_from_memory(&load_file(String::from(block)).await.unwrap())
                .unwrap()
                .into_rgb();
            let mut dithered = image::ImageBuffer::new(16, 16);
            for (rx, ry, pixel) in raw.enumerate_pixels() {
                let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
                let count = [
                    (r as f32 / (255. / 4.)).round() as u8,
                    (g as f32 / (255. / 4.)).round() as u8,
                    (b as f32 / (255. / 4.)).round() as u8,
                ];
                let mut channels = [
                    [0u8; 4],
                    [0u8; 4],
                    [0u8; 4],
                ];
                let mut pixels = [[image::Rgb([0, 0, 0]); 2]; 2];
                for c in 0..3 {
                    for i in 0..count[c] {
                        channels[c][i as usize] = 255u8;
                    }
                    channels[c].shuffle(&mut rand::thread_rng());
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
            let g = QSImage::from_raw(
                &self.gfx,
                Some(&dithered.into_raw()),
                16,
                16,
                ColorFormat::RGB,
            )
            .unwrap();
            self.images.insert((bx, by), g);
        }
        self.images
            .get(&(bx, by))
            .expect("shouldn't happen")
            .clone()

        // if self.images.get(&String::from("blocks/dirt.png")).is_none() {
        //     self.images.insert(
        //         String::from("blocks/dirt.png"),
        //         QSImage::load(&self.gfx, "blocks/dirt.png").await.unwrap(),
        //     );
        // }
        // self.images
        //     .get(&String::from("blocks/dirt.png"))
        //     .unwrap()
        //     .clone()
    }
    pub(crate) async fn get_level(&mut self, level_id: u32) -> Result<Vec<Vec<Block>>> {
        self.images = HashMap::new();
        if let Some(block) = self.levels.get(&level_id) {
            Ok(block.clone())
        } else {
            // println!("got here?");
            let loaded = load_file(&format!("levels/{}.txt", level_id)).await?;
            // println!("but not here?");
            let mut blocks = vec![];
            let mut last = Vec::new();
            for c in loaded.into_iter().map(|v| char::from(v)) {
                if c == '\n' {
                    let mut new = Vec::new();
                    std::mem::swap(&mut last, &mut new);
                    blocks.push(new);
                } else {
                    last.push(Block::from(c))
                }
            }
            self.levels.insert(level_id, blocks);
            Ok(self.levels.get(&level_id).expect("HOW!?").clone())
        }
    }
}

async fn app(window: Window, gfx: Graphics, events: EventStream) -> Result<()> {
    let context = Context::new([0.0, 0.0].into());
    let mut wrapper = Wrapper {
        window,
        gfx,
        events,
        context,
        cursor_at: Vector2::from_slice(&[0f32, 0f32]),
        levels: HashMap::new(),
        images: HashMap::new(),
    };
    let mut v: Box<dyn Screen> = Box::new(screens::menu::Menu::new(&mut wrapper).await?);
    v.draw(&mut wrapper).await?;
    loop {
        while let Some(e) = wrapper.events.next_event().await {
            if let PointerMoved(e) = &e {
                wrapper.cursor_at = e.location();
            }
            wrapper.context.event(&e, &wrapper.window);
            if let Some(x) = v.event(&mut wrapper, &e).await? {
                v = x;
            }
        }
        if let Some(x) = v.update(&mut wrapper).await? {
            v = x;
        }
        v.draw(&mut wrapper).await?;
        wrapper.context.render(&mut wrapper.gfx, &wrapper.window)?;
        wrapper.gfx.present(&wrapper.window)?;
    }
}
