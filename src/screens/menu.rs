use crate::{directions::Directions, Block, Screen, Wrapper};
use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::Color,
    lifecycle::{Event, Key},
    mint::Vector2,
    Result,
};

use async_trait::async_trait;
use std::{collections::HashSet, convert::TryFrom};

const AUTOMATIC_HORIZONTAL_SLOWDOWN: f32 = 3.;
const MAX_HORIZONTAL_SPEED: f32 = 15.;

const JUMP_FORCE: f32 = 25.;
const JUMP_FORCE_SLOWDOWN: f32 = 2.;
const JUMP_FORCE_MAX_FALL: f32 = -20.;

pub struct Menu {
    level: Vec<Vec<Block>>,
    player_pos: Vector,
    pressed: HashSet<Directions>,
    momentum: Vector,
    jump_force: Option<f32>,
}

impl Menu {
    pub(crate) async fn new(wrapper: &mut Wrapper<'_>) -> Result<Self> {
        let level = wrapper.get_level(1).await?;
        let player_pos = level
            .iter()
            .enumerate()
            .flat_map(|(y, v)| v.iter().enumerate().map(move |(x, v)| (y, x, v)))
            .find(|(_, _, v)| **v == Block::PlayerStart)
            .map(|(y, x, _)| Vector::new((x * BLOCK_SIZE) as i32, (y * BLOCK_SIZE) as i32))
            .expect(&format!("Level : {} has no player position", 1));
        Ok(Self {
            level,
            player_pos,
            pressed: HashSet::new(),
            momentum: Vector::new(0, 0),
            jump_force: None,
        })
    }
}

const BLOCK_SIZE: usize = 32;
const BLOCK_SIZE_I32: i32 = 32;

#[async_trait(?Send)]
impl Screen for Menu {
    async fn draw(&mut self, wrapper: &mut crate::Wrapper<'_>) -> quicksilver::Result<()> {
        for (y, line) in self.level.iter().enumerate() {
            for (x, block) in line.iter().enumerate() {
                let location = Rectangle::new(
                    ((x * BLOCK_SIZE) as i32, (y * BLOCK_SIZE) as i32),
                    (BLOCK_SIZE_I32, BLOCK_SIZE_I32),
                );
                if block.can_render() {
                    let block = wrapper.get_block(*block, x as f64, y as f64).await;
                    wrapper.gfx.draw_image(&block, location);
                } else {
                    wrapper.gfx.fill_rect(&location, Color::WHITE)
                }
            }
        }
        wrapper.gfx.fill_rect(
            &Rectangle::new(
                (self.player_pos.x, self.player_pos.y),
                (BLOCK_SIZE_I32, BLOCK_SIZE_I32),
            ),
            Color::CYAN,
        );
        Ok(())
    }
    async fn update(
        &mut self,
        wrapper: &mut crate::Wrapper<'_>,
    ) -> quicksilver::Result<Option<Box<dyn Screen>>> {
        let mut momentum = self.momentum;
        for v in &self.pressed {
            //.iter()
            momentum += Vector::from(*v);
        }
        if momentum.x == self.momentum.x && momentum.x != 0. {
            if momentum.x < 0. {
                momentum.x += AUTOMATIC_HORIZONTAL_SLOWDOWN;
                if momentum.x > 0. {
                    momentum.x = 0.;
                }
            } else {
                momentum.x -= AUTOMATIC_HORIZONTAL_SLOWDOWN;
                if momentum.x < 0. {
                    momentum.x = 0.;
                }
            }
        }
        if momentum.x > MAX_HORIZONTAL_SPEED {
            momentum.x = MAX_HORIZONTAL_SPEED;
        } else if momentum.x < -MAX_HORIZONTAL_SPEED {
            momentum.x = -MAX_HORIZONTAL_SPEED;
        }
        if let Some(jump_force) = &mut self.jump_force {
            momentum.y = -*jump_force;
            *jump_force -= JUMP_FORCE_SLOWDOWN;
            // println!("{}", jump_force);
            if *jump_force < JUMP_FORCE_MAX_FALL {
                *jump_force = JUMP_FORCE_MAX_FALL;
            }
        }
        self.momentum = momentum;

        self.player_pos += self.momentum;
        Ok(None)
    }
    async fn event(
        &mut self,
        _wrapper: &mut Wrapper<'_>,
        event: &Event,
    ) -> Result<Option<Box<dyn Screen>>> {
        match event {
            //Event::Resized(_) => {}
            //Event::ScaleFactorChanged(_) => {}
            //Event::FocusChanged(_) => {}
            // Event::ReceivedCharacter(_) => {}
            Event::KeyboardInput(x) => {
                if x.key() == Key::W {
                    if self.jump_force.is_none() {
                        self.jump_force = Some(JUMP_FORCE);
                    }
                } else if let Ok(d) = Directions::try_from(x.key()) {
                    if x.is_down() {
                        self.pressed.insert(d);
                    } else {
                        self.pressed.remove(&d);
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }
}
