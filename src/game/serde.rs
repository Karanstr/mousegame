use crate::game::{Level, Object};

use super::Material;
use super::object::Step;
use glam::IVec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InitialLevel {
  spawnpoints: Vec<IVec2>,
  pub objects: Vec<MinimalRect>
}
impl InitialLevel {
  pub fn new(level: &Level) -> Self {
    let mut objects= Vec::new();
    for (id, _) in &level.list {
      let full_obj = level.get_obj(*id).unwrap();
      objects.push(MinimalRect {
        position: full_obj.position,
        length: full_obj.points[2],
        material: full_obj.material,
        animation: full_obj.animation.clone(),
      });
    }
    Self {
      spawnpoints: Vec::new(),
      objects,
    }
  }
}

#[derive(Serialize, Deserialize)]
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
