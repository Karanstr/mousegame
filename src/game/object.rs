use glam::IVec2;
use rapier2d::prelude::*;
use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

// This could be adapted to work with bezier curves + rotations, etc
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Step {
  vector: IVec2, 
  ticks_duration: u32,
  ticks_pause: i32,
}

pub struct Object {
  pub position: IVec2,
  pub points: Vec<IVec2>,
  pub collider: Collider,
  pub rigidbody: RigidBody,
  pub material: Material,
  pub animation: Option<Vec<Step>>,
}
impl Object {
  pub fn new_mouse(position: IVec2) -> Self {
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
      material: Material::Player,
      animation: None,
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
    Self {
      points: vec![IVec2::ZERO, length.with_x(0), length, length.with_y(0)],
      position: top_left,
      collider: collider.build(),
      rigidbody,
      material,
      animation
    }
  }

  // Serializer
  // Binary Format:
  // [key, color, position, point_count, points (x, y, x, y)]
  // Passing the key in is stupid, but I want this to work soon and I have stuff to do
  pub fn to_binary(&self, key: usize) -> Vec<i32> {
    let mut data = Vec::new();
    data.push(key as i32);
    data.push(self.material as i32);
    data.push(self.position.x);
    data.push(self.position.y);
    data.push(self.points.len() as i32);
    for point in &self.points {
      data.push(point.x);
      data.push(point.y);
    }
    data
  }
  
}


#[repr(u8)]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Material {
  Player,
  Wall,
  Death,
  WinZone,
  // BlueButton,
  // OrangeButton,
  // PurpleButton,
  // PinkButton,
}
impl Material {
  pub fn can_move(&self) -> bool {
    match self {
      Material::Player => true,
      Material::Wall => false,
      Material::Death => false,
      Material::WinZone => false,
    }
  }
  pub fn is_sensor(&self) -> bool {
    match self {
      Material::Player => false,
      Material::Wall => false,
      Material::Death => false,
      Material::WinZone => true,
    }
  }
  pub fn has_event(&self) -> bool {
    match self {
      Material::Player => false,
      Material::Wall => false,
      Material::Death => true,
      Material::WinZone => true,
    }
  }
}
