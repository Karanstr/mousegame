use rapier2d::{na::Vector2, prelude::*};

use super::Level;

pub struct Physics {
  colliders: ColliderSet,
  rigids: RigidBodySet,
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

  pub fn body_sets(&mut self) -> (&mut RigidBodySet, &mut ColliderSet) {
    (&mut self.rigids, &mut self.colliders)
  }

  pub fn step(&mut self, level: &mut Level) {
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
      level,
    );
  }
}

