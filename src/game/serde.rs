use crate::game::Object;
use super::Material;
use super::object::Step;
use glam::IVec2;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct InitialLevel {
  pub spawnpoints: [IVec2; 3],
  pub objects: Vec<MinimalRect>
}

#[derive(Deserialize)]
pub struct MinimalRect {
  position: IVec2,
  length: IVec2,
  material: Material,
  animation: Option<Vec<Step>>,
}
impl MinimalRect {
  pub fn full_rect(&self) -> Object {
    Object::new_rect(
      self.position,
      self.length,
      self.material,
      self.animation.clone(),
    )
  }
}
