use glam::IVec2;
use rapier2d::prelude::*;
use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

// This could be adapted to work with bezier curves + rotations, etc
#[derive(Deserialize, Clone, Copy)]
pub struct Step {
  destination: IVec2,
  duration: u32,
  sleep: u32,
}
pub struct Path {
  last_checkpoint: IVec2,
  steps: Vec<Step>,
  current_step: u32,
  current_tick: u32,
  ticks_sleeping: u32,
}
impl Path {
  pub fn step(&mut self) -> IVec2 {
    if self.ticks_sleeping != 0 { self.ticks_sleeping -= 1; }
    else { self.current_tick += 1; }
    let step = &self.steps[self.current_step as usize];
    let interpolate = self.current_tick as f32 / step.duration as f32;
    let current = self.last_checkpoint.as_vec2().lerp(step.destination.as_vec2(), interpolate);
    if self.current_tick > step.duration {
      self.current_tick = 0;
      self.last_checkpoint = step.destination;
      self.ticks_sleeping = step.sleep;
      self.current_step += 1;
      self.current_step %= self.steps.len() as u32;
    }
    current.as_ivec2()
  }
}

pub struct Object {
  pub position: IVec2,
  pub points: Vec<IVec2>,
  pub collider: Collider,
  pub rigidbody: RigidBody,
  pub material: Material,
  pub animation: Option<Path>,
  pub hidden: bool,
  pub frozen: bool,

}
impl Object {
  pub fn new_mouse(position: IVec2, player: u32) -> Self {
    let points = vec![
      IVec2::new(0, 0),
      IVec2::new(0, 16),
      IVec2::new(4, 13),
      IVec2::new(6, 18),
      IVec2::new(9, 17),
      IVec2::new(6, 12),
      IVec2::new(11, 12),
    ];
    let vertices: Vec<Point<f32>> = points.iter()
      .map(|point| Point::new(point.x as f32, point.y as f32)).collect();
    let indices: Vec<[u32; 2]> = (0..vertices.len() as u32)
      .map(|i| [i, (i + 1) % vertices.len() as u32]).collect();
    let collider = ColliderBuilder::convex_decomposition(&vertices, &indices).build();
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Dynamic)
      .translation(Vector2::new(position.x as f32, position.y as f32))
      .locked_axes(LockedAxes::ROTATION_LOCKED)
      .ccd_enabled(true)
      .linear_damping(50.0)
      .build();
    Self {
      points,
      position: position.into(),
      collider,
      rigidbody,
      material: Material::Player(player),
      animation: None,
      hidden: false,
      frozen: false,
    }
  }
  
  pub fn new_rect(top_left: IVec2, length: IVec2, material: Material, animation: Option<Vec<Step>>) -> Self {
    let halves = length / 2;
    let mut collider = ColliderBuilder::cuboid(halves.x as f32, halves.y as f32)
      .position(Point::new(halves.x as f32, halves.y as f32).into());
    if material.is_sensor() {
      collider = collider.sensor(true);
    }
    if material.has_event() {
      collider = collider.active_events(ActiveEvents::COLLISION_EVENTS);
    }
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Fixed)
      .translation(Vector2::new(top_left.x as f32, top_left.y as f32))
      .build();
    let animation = if let Some(steps) = animation { Some(Path {
      last_checkpoint: top_left,
      steps,
      current_step: 0,
      current_tick: 0,
      ticks_sleeping: 0,
    }) } else { None };
    Self {
      points: vec![IVec2::ZERO, length.with_x(0), length, length.with_y(0)],
      position: top_left,
      collider: collider.build(),
      rigidbody,
      material,
      animation,
      hidden: false,
      frozen: false,
    }
  }

}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Material {
  Player(u32),
  Wall,
  Death,
  None,
  Button(u32, u8),
}
impl Material {
  pub fn is_sensor(&self) -> bool {
    match self {
      Self::Button(_, _) => true,
      _ => false,
    }
  }
  pub fn has_event(&self) -> bool {
    match self {
      Self::Death => true,
      Self::Button(_, _) => true,
      _ => false,
    }
  }
  pub fn color(&self) -> i32 {
    match self {
      Self::Player(_) => 0, // White + Black Outline
      Self::Wall => 1,      // Black
      Self::Death => 2,     // Red
                            
      Self::Button(x, active) 
        if *x == 0 && *active != 0 => 3, // Lime

      Self::Button(x, active)
        if *x == 0 && *active == 0 => 4, // Green

      _ => unimplemented!(),
    }
  }
  pub fn set_active(&mut self, activity: bool) {
    if let Self::Button(_, active) = self { 
      *active = if activity { *active + 1 } else { *active - 1 };
    }
  }
}

