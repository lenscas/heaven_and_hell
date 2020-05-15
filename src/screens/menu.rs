use crate::{directions::Directions, Block, Screen, Wrapper};
use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::Color,
    lifecycle::{Event, Key},
    mint::Vector2,
    Result,
};

use async_trait::async_trait;
use std::any::Any;

extern crate nalgebra as na;

use na::Isometry2;
use na::Vector2 as V2;
use ncollide2d::shape::ShapeHandle;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::material::{BasicMaterial, MaterialHandle};
use nphysics2d::object::ColliderDesc;
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, DefaultBodyHandle, DefaultBodySet, DefaultColliderHandle,
    DefaultColliderSet, RigidBodyDesc,
};
use nphysics2d::{
    algebra::ForceType,
    math::Force,
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};

use std::{collections::HashSet, convert::TryFrom};

const AUTOMATIC_HORIZONTAL_SLOWDOWN: f32 = 3.;
const MAX_HORIZONTAL_SPEED: f32 = 15.;

const JUMP_FORCE: f32 = 25.;
const JUMP_FORCE_SLOWDOWN: f32 = 2.;
const JUMP_FORCE_MAX_FALL: f32 = -20.;

const PLAYER_WIDTH: i32 = 16;
const PLAYER_HEIGHT: i32 = 32;

pub struct Menu {
    level: Vec<Vec<Block>>,
    player_pos: Vector,
    pressed: HashSet<Directions>,
    momentum: Vector,
    jump_force: Option<f32>,
    mechanical_world: DefaultMechanicalWorld<f64>,
    geometrical_world: DefaultGeometricalWorld<f64>,
    bodies: DefaultBodySet<f64>,
    colliders: DefaultColliderSet<f64>,
    joint_constraints: DefaultJointConstraintSet<f64>,
    force_generators: DefaultForceGeneratorSet<f64>,
    player_body: DefaultColliderHandle,
    level_as_colliders: Vec<DefaultColliderHandle>,
}

impl Menu {
    pub(crate) async fn new(wrapper: &mut Wrapper<'_>) -> Result<Self> {
        let mut mechanical_world = DefaultMechanicalWorld::new(V2::new(0.0, 600.)); //9.81
        let mut geometrical_world = DefaultGeometricalWorld::new();

        let mut bodies = DefaultBodySet::new();
        let mut colliders = DefaultColliderSet::new();
        let mut joint_constraints = DefaultJointConstraintSet::new();
        let mut force_generators = DefaultForceGeneratorSet::new();

        let level = wrapper.get_level(1).await?;

        let mut level_as_colliders = Vec::new();
        //*
        for (y, line) in level.iter().enumerate() {
            for (x, block) in line.iter().enumerate() {
                if block.is_colideable() {
                    //dbg!((x, y, block));

                    let body = RigidBodyDesc::new()
                        .translation(V2::new((x * BLOCK_SIZE) as f64, (y * BLOCK_SIZE) as f64))
                        .status(BodyStatus::Static)
                        .gravity_enabled(false)
                        .build();
                    let reference = bodies.insert(body);
                    let block_handler =
                        ColliderDesc::new(ShapeHandle::new(ncollide2d::shape::Cuboid::new(
                            V2::new(BLOCK_SIZE_I32 as f64 / 2., BLOCK_SIZE_I32 as f64 / 2.),
                        )))
                        .user_data(*block)
                        .build(BodyPartHandle(reference, 0));
                    let collider_handle = colliders.insert(block_handler);
                    level_as_colliders.push(collider_handle);
                } else {
                    dbg!((block, x, y));
                }
            }
        }

        let player_pos = level
            .iter()
            .enumerate()
            .flat_map(|(y, v)| v.iter().enumerate().map(move |(x, v)| (y, x, v)))
            .find(|(_, _, v)| **v == Block::PlayerStart)
            .map(|(y, x, _)| Vector::new((x * BLOCK_SIZE) as i32, (y * BLOCK_SIZE) as i32))
            .expect(&format!("Level : {} has no player position", 1));

        let mut player_body = RigidBodyDesc::new()
            .translation(V2::new(player_pos.x as f64, player_pos.y as f64))
            .gravity_enabled(true)
            .status(BodyStatus::Dynamic)
            //.linear_damping(5.0)
            //.max_linear_velocity(10.0)
            //.max_angular_velocity(0.)
            .mass(2000000.)
            .build();
        player_body.disable_all_rotations();
        let reference = bodies.insert(player_body);
        let player_shape = ColliderDesc::new(ShapeHandle::new(ncollide2d::shape::Cuboid::new(
            V2::new(PLAYER_WIDTH as f64 / 2., PLAYER_HEIGHT as f64 / 2.),
        )))
        .build(BodyPartHandle(reference, 0));
        let collider_handle = colliders.insert(player_shape);

        mechanical_world.step(
            &mut geometrical_world,
            &mut bodies,
            &mut colliders,
            &mut joint_constraints,
            &mut force_generators,
        );

        Ok(Self {
            level,
            player_pos,
            pressed: HashSet::new(),
            momentum: Vector::new(0, 0),
            jump_force: None,
            mechanical_world,
            geometrical_world,
            bodies,
            colliders,
            joint_constraints,
            force_generators,
            player_body: collider_handle,
            level_as_colliders,
        })
    }
    pub fn check_collision(&self, (x, y): (usize, usize)) -> bool {
        self.level
            .get(x)
            .and_then(|v| v.get(y))
            .map(|v| v.is_colideable())
            .unwrap_or(true)
    }
}

const BLOCK_SIZE: usize = 32;
const BLOCK_SIZE_I32: i32 = 32;

#[async_trait(?Send)]
impl Screen for Menu {
    async fn draw(&mut self, wrapper: &mut crate::Wrapper<'_>) -> quicksilver::Result<()> {
        wrapper.gfx.clear(Color::WHITE);
        for collider in self.level_as_colliders.iter().cloned() {
            if let Some(collider) = self.colliders.get(collider) {
                let pos = collider.position().translation;
                let rec = Rectangle::new(
                    (pos.x as f32, pos.y as f32),
                    (BLOCK_SIZE_I32, BLOCK_SIZE_I32),
                );
                let block = wrapper
                    .get_block(
                        collider
                            .user_data()
                            .and_then(|v| v.downcast_ref::<Block>().map(|v| *v))
                            .unwrap_or(Block::Dirt),
                    )
                    .await?;
                wrapper.gfx.draw_image(&block, rec)
            }
        }
        wrapper.gfx.fill_rect(
            &Rectangle::new(
                (self.player_pos.x, self.player_pos.y),
                (BLOCK_SIZE_I32, BLOCK_SIZE_I32),
            ),
            Color::CYAN,
        );
        if let Some(player) = self.colliders.get(self.player_body) {
            let pos = player.position().translation;
            dbg!(pos);
            /*
            let pos = Vector::new(
                (pos.x - (PLAYER_WIDTH as f64 / 2.)) as f32,
                (pos.y - (PLAYER_HEIGHT as f64 / 2.)) as f32,
            );*/
            dbg!(pos);
            dbg!(self.player_pos);
            let rect = Rectangle::new((pos.x as f32, pos.y as f32), (PLAYER_WIDTH, PLAYER_HEIGHT));

            wrapper.gfx.fill_rect(&rect, Color::RED);
        }
        Ok(())
    }
    async fn update(
        &mut self,
        wrapper: &mut crate::Wrapper<'_>,
    ) -> quicksilver::Result<Option<Box<dyn Screen>>> {
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );

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
        } else if momentum.x < self.momentum.x && momentum.x > 0. {
            momentum.x -= AUTOMATIC_HORIZONTAL_SLOWDOWN;
        } else if momentum.x > self.momentum.x && momentum.x < 0. {
            momentum.x += AUTOMATIC_HORIZONTAL_SLOWDOWN;
        }

        if momentum.x > MAX_HORIZONTAL_SPEED {
            momentum.x = MAX_HORIZONTAL_SPEED;
        } else if momentum.x < -MAX_HORIZONTAL_SPEED {
            momentum.x = -MAX_HORIZONTAL_SPEED;
        }
        if let Some(jump_force) = &mut self.jump_force {
            momentum.y = -*jump_force;
            *jump_force -= JUMP_FORCE_SLOWDOWN;
            if *jump_force < JUMP_FORCE_MAX_FALL {
                *jump_force = JUMP_FORCE_MAX_FALL;
            }
        }
        self.momentum = momentum;
        let pos = self.player_pos + momentum;
        let top_right = Vector::new(pos.x + PLAYER_WIDTH as f32, pos.y);
        let bottom_right = Vector::new(top_right.x, pos.y + PLAYER_HEIGHT as f32);
        let bottom_left = Vector::new(pos.x, bottom_right.y);
        let decode = |point: Vector| {
            let x = (point.x as f32 / BLOCK_SIZE as f32).ceil() as usize;
            let y = (point.y as f32 / BLOCK_SIZE as f32).ceil() as usize;
            (x, y)
        };

        let left_top_corner = self.check_collision(decode(pos));
        let right_top_corner = self.check_collision(decode(top_right));
        let right_bottom_corner = self.check_collision(decode(bottom_right));
        let left_bottom_corner = self.check_collision(decode(bottom_left));
        /*
        match (
            left_top_corner,
            right_top_corner,
            left_bottom_corner,
            right_bottom_corner,
        ) {
            (true, true, false, false) => {
                self.momentum.y = 0.;
                self.player_pos.y = top_right.y
            }
            (true, false, false, false) => {
                match (
                    momentum.x > 0,
                    momentum.x < 0,
                    momentum.y > 0,
                    momentum.y < 0,
                ) {
                    //going left and maybe down
                    (false, true, false, _) => {
                        self.momentum.x = 0.;
                        self.player_pos.x = pos.x;
                    }
                    //going left and up
                    ()
                    (true, false, false, _) => {
                        panic!("collision top left, while going right and down")
                    }
                    (true, true, _, _) | (_, _, true, true) => unreachable!("can't happen"),
                }
            }
        }*/
        //let player = Rectangle::new(pos, (PLAYER_WIDTH * 2, PLAYER_HEIGHT * 2));

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
                    if let Some(player) = self.colliders.get_mut(self.player_body) {
                        dbg!(player.position());
                        if let Some(body) = self.bodies.get_mut(player.body()) {
                            body.apply_force(
                                0,
                                &Force::new(V2::new(200000., 20000000.), 0.),
                                ForceType::VelocityChange,
                                true,
                            );
                            //body.set_velocity(V2::new(20., 20.));
                            dbg!(body.is_dynamic());
                            dbg!(body.status());
                        }
                    }
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
