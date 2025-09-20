use rapier2d::{na::Vector2, prelude::*};
use glam::IVec2;
use std::collections::HashMap;

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
      integration_params: IntegrationParameters::default(),
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

pub struct Object {
  points: Vec<IVec2>,
  body_type: RigidBodyType,
  origin: IVec2,
  color: u32,
}
impl Object {
  pub fn new_player(position: IVec2) -> Self {
    Self {
      points: vec![
        IVec2::new(-10, -10),
        IVec2::new(10, -10),
        IVec2::new(10, 10),
        IVec2::new(-10, 10),
      ],
      origin: position.into(),
      body_type: RigidBodyType::Dynamic,
      color: 0,
    }
  }

  pub fn new_wall(position: IVec2, points: Vec<IVec2>) -> Self {
    Self {
      points,
      origin: position.into(),
      body_type: RigidBodyType::Fixed,
      color: 1,
    }
  }

  // Serializer
  // Binary Format:
  // [key, color, point_count, position, points (x, y, x, y)]
  // Passing the key in is stupid, but I want this to work soon and I have stuff to do
  pub fn serialize(&self, key: RigidBodyHandle) -> Vec<i32> {
    let mut data = Vec::new();
    data.push(key.into_raw_parts().0 as i32);
    data.push(self.color as i32);
    data.push(self.points.len() as i32);
    data.push(self.origin.x);
    data.push(self.origin.y);
    for point in &self.points {
      data.push(point.x);
      data.push(point.y);
    }
    data
  }
  
}

pub struct Level {
  pub objects: HashMap<RigidBodyHandle, Object>,
  physics: Physics,
}

impl Level {

  pub fn tick(&mut self) {
    self.physics.step();
    // Update object list and remove velocity
    for handle in self.objects.keys().copied().collect::<Vec<_>>() {
      let origin = self.get_pos(handle);
      let obj = self.objects.get_mut(&handle).unwrap();
      obj.origin = origin;
    }
  }

  pub fn add_vel(&mut self, handle: RigidBodyHandle, velocity: IVec2) {
    let rb = self.physics.rigids.get_mut(handle).unwrap();
    rb.add_force(Vector2::new(velocity.x as f32, velocity.y as f32), true);
  }

  pub fn get_pos(&self, handle: RigidBodyHandle) -> IVec2 {
    let bad_pos = self.physics.rigids.get(handle).unwrap().position().translation;
    IVec2::new(bad_pos.x as i32, bad_pos.y as i32)
  }

  pub fn add_object(&mut self, object: Object) -> RigidBodyHandle {
    let rigid_body = RigidBodyBuilder::new(object.body_type)
      .translation(Vector2::new(object.origin.x as f32, object.origin.y as f32))
      .locked_axes(LockedAxes::ROTATION_LOCKED)
      .ccd_enabled(true)
      .build();
    let rb_handle = self.physics.rigids.insert(rigid_body);

    let vertices: Vec<Point<f32>> = object.points.iter()
      .map(|point| Point::new(point.x as f32, point.y as f32)).collect();
    let indices: Vec<[u32; 2]> = (0..vertices.len() as u32)
      .map(|i| [i, (i + 1) % vertices.len() as u32]).collect();
    let collider = ColliderBuilder::convex_decomposition(&vertices, &indices)
      .build();
    self.physics.colliders.insert_with_parent(
      collider,
      rb_handle,
      &mut self.physics.rigids
    );
    self.objects.insert(rb_handle, object);
    rb_handle
  }

  pub fn new(things: Vec<Object>) -> Self { 
    let mut new = Self { 
      objects: HashMap::new(),
      physics: Physics::new(),
    };
    for object in things { new.add_object(object); }
    new
  }

}
