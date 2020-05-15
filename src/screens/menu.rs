use crate::{directions::Directions, Block, Screen, Wrapper};
use quicksilver::{
    geom::{Rectangle, Transform, Vector},
    graphics::Color,
    lifecycle::{Event, Key},
    Result,
};

use async_trait::async_trait;

extern crate nalgebra as na;

use na::Vector2 as V2;
use ncollide2d::shape::ShapeHandle;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::ColliderDesc;
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, DefaultBodySet, DefaultColliderHandle, DefaultColliderSet,
    RigidBodyDesc,
};
use nphysics2d::{
    algebra::ForceType,
    math::Force,
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};

use std::{collections::HashSet, convert::TryFrom};

const JUMP_VELOCITY: f64 = -100.;
const WALK_VELOCITY: f32 = 10.;

const PLAYER_WIDTH: i32 = 16;
const PLAYER_HEIGHT: i32 = 32;

pub struct Menu {
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
    jump_count: u32,
    max_jumps: u32,
    translate_x: f32,
}

impl Menu {
    pub(crate) async fn new(wrapper: &mut Wrapper<'_>) -> Result<Self> {
        let mut mechanical_world =
            DefaultMechanicalWorld::new(V2::new(0.0, 9.81 * BLOCK_SIZE_I32 as f64)); //9.81
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
            .mass(1.)
            .build();
        player_body.disable_all_rotations();
        let reference = bodies.insert(player_body);
        let player_shape = ColliderDesc::new(ShapeHandle::new(ncollide2d::shape::Cuboid::new(
            V2::new(PLAYER_WIDTH as f64 / 2., PLAYER_HEIGHT as f64 / 2.),
        )))
        .density(2.)
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
            jump_count: 0,
            max_jumps: 1,
            translate_x: 0.,
        })
    }
}

const BLOCK_SIZE: usize = 32;
const BLOCK_SIZE_I32: i32 = 32;

#[async_trait(?Send)]
impl Screen for Menu {
    async fn draw(&mut self, wrapper: &mut crate::Wrapper<'_>) -> quicksilver::Result<()> {
        if self.player_pos.x > 320. {
            wrapper.gfx.set_transform(
                Transform::translate(Vector::new((self.player_pos.x - 320.).floor(), 0)).inverse(),
            );
        } else {
            wrapper.gfx.set_transform(Transform::IDENTITY);
        }
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
                        pos.x,
                        pos.y,
                    )
                    .await;
                wrapper.gfx.draw_image(&block, rec)
            }
        }
        if let Some(player) = self.colliders.get(self.player_body) {
            let pos = player.position().translation;
            let pos = Vector::new(pos.x as f32, pos.y as f32);
            let rect = Rectangle::new(pos, (PLAYER_WIDTH, PLAYER_HEIGHT));

            wrapper.gfx.fill_rect(&rect, Color::RED);
            self.player_pos = pos;
        }
        Ok(())
    }
    async fn update(
        &mut self,
        _: &mut crate::Wrapper<'_>,
    ) -> quicksilver::Result<Option<Box<dyn Screen>>> {
        if let Some(player) = self.colliders.get_mut(self.player_body) {
            if let Some(body) = self.bodies.get_mut(player.body()) {
                let mut momentum = Vector::new(0, 0);
                for v in &self.pressed {
                    momentum += Vector::from(*v);
                }
                body.apply_force(
                    0,
                    &Force::new(V2::new((momentum.x * WALK_VELOCITY) as f64, 0.), 0.),
                    ForceType::VelocityChange,
                    true,
                );
            }
        }
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );

        for contact in self.geometrical_world.contact_events() {
            match contact {
                ncollide2d::pipeline::ContactEvent::Started(x, _) => {
                    if x == &self.player_body {
                        self.jump_count = 0;
                    }
                }
                ncollide2d::pipeline::ContactEvent::Stopped(_, _) => {}
            }
        }
        self.player_pos += self.momentum;

        Ok(None)
    }
    async fn event(
        &mut self,
        _wrapper: &mut Wrapper<'_>,
        event: &Event,
    ) -> Result<Option<Box<dyn Screen>>> {
        match event {
            Event::KeyboardInput(x) => {
                if x.key() == Key::W {
                    if let Some(player) = self.colliders.get_mut(self.player_body) {
                        if let Some(body) = self.bodies.get_mut(player.body()) {
                            if self.max_jumps >= self.jump_count {
                                self.jump_count += 1;
                                body.apply_force(
                                    0,
                                    &Force::new(V2::new(0., JUMP_VELOCITY), 0.),
                                    ForceType::VelocityChange,
                                    true,
                                );
                            }
                        }
                    }
                    if self.jump_force.is_none() {
                        self.jump_force = Some(JUMP_VELOCITY as f32);
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
