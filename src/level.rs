use rapier2d::{na::Vector2, prelude::*};
use glam::IVec2;
use std::collections::{HashMap, HashSet};
use lilypads::Pond;

// We need to write and expose a custom event hook I think
pub struct Physics {
  pub colliders: ColliderSet,
  pub rigids: RigidBodySet,
  pipeline: PhysicsPipeline,
  islands: IslandManager,
  broad_phase: BroadPhaseBvh,
  narrow_phase: NarrowPhase,
  impulse_joints: ImpulseJointSet,
  multibody_joints: MultibodyJointSet,
  ccd_solver: CCDSolver,
  integration_params: IntegrationParameters,
}
impl Physics {
  pub fn new() -> Self {
    let mut integration_params = IntegrationParameters::default();
    integration_params.length_unit = 5.0;
    integration_params.normalized_max_corrective_velocity = 200.0;
    integration_params.max_ccd_substeps = 10;
    Self {
      colliders: ColliderSet::new(),
      rigids: RigidBodySet::default(),
      pipeline: PhysicsPipeline::new(),
      islands: IslandManager::new(),
      broad_phase: BroadPhaseBvh::new(),
      narrow_phase: NarrowPhase::new(),
      impulse_joints: ImpulseJointSet::new(),
      multibody_joints: MultibodyJointSet::new(),
      ccd_solver: CCDSolver::default(),
      integration_params,
    }
  }

  pub fn step(&mut self) {
    self.pipeline.step(
      &Vector2::zeros(),
      &self.integration_params,
      &mut self.islands,
      &mut self.broad_phase,
      &mut self.narrow_phase,
      &mut self.rigids,
      &mut self.colliders,
      &mut self.impulse_joints,
      &mut self.multibody_joints,
      &mut self.ccd_solver,
      &(),
      &(),
    );
  }
}

#[derive(Clone, Copy)]
pub enum Owner {
  RigidBody(RigidBodyHandle),
  Collider(ColliderHandle),
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum RenderData {
  Player,
  Wall,
  Death,
  WinZone,
  BlueButton,
  OrangeButton,
  PurpleButton,
  PinkButton,
}

pub struct Object {
  points: Vec<IVec2>,
  origin: IVec2,
  body_type: Option<RigidBodyType>,
  collider: Collider,
  render: RenderData,
  moving: bool,
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
    Self {
      points,
      origin: position.into(),
      body_type: Some(RigidBodyType::Dynamic),
      collider,
      render: RenderData::Player,
      moving: true,
    }
  }
  
  pub fn new_rect(top_left: IVec2, length: IVec2, data: RenderData) -> Self {
    let halves = length / 2;
    let collider = ColliderBuilder::cuboid(halves.x as f32, halves.y as f32)
      .position(Point::new(halves.x as f32, halves.y as f32).into())
      .build();
    let body_type = if let RenderData::WinZone = data { None } else { Some(RigidBodyType::Fixed) };
    Self {
      points: vec![IVec2::ZERO, length.with_x(0), length, length.with_y(0)],
      origin: top_left,
      body_type,
      collider,
      render: data,
      moving: false,
    }
  }

  // Serializer
  // Binary Format:
  // [key, color, position, point_count, points (x, y, x, y)]
  // Passing the key in is stupid, but I want this to work soon and I have stuff to do
  pub fn serialize(&self, key: usize) -> Vec<i32> {
    let mut data = Vec::new();
    data.push(key as i32);
    data.push(self.render as i32);
    data.push(self.origin.x);
    data.push(self.origin.y);
    data.push(self.points.len() as i32);
    for point in &self.points {
      data.push(point.x);
      data.push(point.y);
    }
    data
  }
  
}

pub struct Level {
  objects: Pond<Object>,
  pub list: HashMap<usize, Owner>,
  pub moving: HashSet<usize>,
  physics: Physics,
}
impl Level {
  pub fn tick(&mut self) {
    self.physics.step();
    // Update object list
    for id in self.list.keys() {
      if let Some(pos) = self.get_rapier_pos(*id) {
        self.objects.get_mut(*id).unwrap().origin = pos;
      }
    }
  }

  pub fn apply_vel(&mut self, id: usize, velocity: IVec2) {
    if let Owner::RigidBody(handle) = self.list.get(&id).unwrap() {
      let rb = self.physics.rigids.get_mut(*handle).unwrap();
      let mass = rb.mass();
      let impulse = velocity.as_vec2() * mass;
      rb.apply_impulse(Vector2::new(impulse.x as f32, impulse.y as f32), true);
    } else { dbg!("Failed to add velocity to collider"); }
  }

  pub fn get_obj(&self, id: usize) -> Option<&Object> { self.objects.get(id) }

  pub fn get_pos(&self, id: usize) -> IVec2 { self.objects.get(id).unwrap().origin }

  fn get_rapier_pos(&self, id: usize) -> Option<IVec2> {
    if let Owner::RigidBody(handle) = self.list.get(&id).unwrap() {
      let bad_pos = self.physics.rigids.get(*handle).unwrap().position().translation;
      Some(IVec2::new(bad_pos.x as i32, bad_pos.y as i32))
    } else { None }
  }

  pub fn add_object(&mut self, object: Object) -> usize {
    let owner = if let Some(rb_type) = object.body_type {
      let rigid_body = RigidBodyBuilder::new(rb_type)
        .translation(Vector2::new(object.origin.x as f32, object.origin.y as f32))
        .locked_axes(LockedAxes::ROTATION_LOCKED)
        .ccd_enabled(true)
        .linear_damping(50.0)
        .build();
      let rb_handle = self.physics.rigids.insert(rigid_body);
      self.physics.colliders.insert_with_parent(
        object.collider.clone(),
        rb_handle,
        &mut self.physics.rigids
      );
      Owner::RigidBody(rb_handle)
    } else {
      Owner::Collider(self.physics.colliders.insert(object.collider.clone()))
    };
    let moving = object.moving;
    let id = self.objects.alloc(object);
    self.list.insert(id, owner);
    if moving { self.moving.insert(id); }
    id
  }

  pub fn new(objects: Vec<Object>) -> Self { 
    let mut new = Self { 
      objects: Pond::new(),
      list: HashMap::new(),
      moving: HashSet::new(),
      physics: Physics::new(),
    };
    for object in objects { new.add_object(object); }
    new
  }

}

