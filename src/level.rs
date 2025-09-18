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
}


pub struct Level {
  objects: HashMap<RigidBodyHandle, Object>,
  physics: Physics,
}

// Impl Serde
impl Level {
  
  pub fn add_object(&mut self, object: Object) -> RigidBodyHandle {
    let rigid_body = RigidBodyBuilder::new(object.body_type)
      .translation(Vector2::new(object.origin.x as f32, object.origin.y as f32))
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

    for object in things {
      new.add_object(object);
    }
    new
  }

  // The first pair of each shape is [num_points, color/texture_id], followed by each point of the polygon
  // For now all are hardcoded as red
  pub fn geometry(&self) -> Vec<[i32; 2]> {
    let mut geometry = Vec::new();
    for (_, object) in &self.objects {
      geometry.push([object.points.len() as i32, 0]);
      for point in &object.points {
        geometry.push([point.x, point.y])
      }
    }
    return geometry;
  }

}
