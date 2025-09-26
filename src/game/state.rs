use super::{Level, Physics, Object};
use std::collections::HashMap;
use glam::IVec2;
use uuid::Uuid;
use axum::extract::ws::Message;
use crate::{game::Material, networking::{Event, Server}};

// If it really becomes a problem we can cut this down to bytes then reconstruct with ___views
// Sent in u32 blocks
// Size: bytes -- u32s
#[repr(u8)]
enum StateFlags {
  Deleted  = 0b00000000, // Size: 0
  Position = 0b00000001, // Size: 8 -- 2
  Shape    = 0b00000010, // Size: 4 + 8*length -- 1 + 2 * length
  Material = 0b00000100, // Size: 4 -- 1
}
pub struct ObjectUpdate {
  position: Option<IVec2>,
  shape: Option<Vec<IVec2>>,
  material: Option<Material>,
}
impl ObjectUpdate {
  pub fn new() -> Self {
    Self {position: None, shape: None, material: None, }
  }
  pub fn position(mut self, position: IVec2) -> Self {
    self.position = Some(position);
    self
  }
  pub fn shape(mut self, shape: Vec<IVec2>) -> Self {
    self.shape = Some(shape);
    self
  }
  pub fn material(mut self, material: Material) -> Self {
    self.material = Some(material);
    self
  }
  pub fn to_binary(self) -> Vec<i32> {
    let flag = 
      if self.position.is_some() { StateFlags::Position as i32 } else { 0 } |
      if self.shape.is_some() { StateFlags::Shape as i32 } else { 0 } | 
      if self.material.is_some() { StateFlags::Material as i32 } else { 0 };
    let mut data = Vec::new();
    data.push(flag);
    if let Some(position) = self.position {
      data.push(position.x);
      data.push(position.y);
    }
    if let Some(shape) = self.shape {
      data.push(shape.len() as i32);
      for point in shape {
        data.push(point.x);
        data.push(point.y);
      }
    }
    if let Some(material) = self.material {
      data.push(material as i32)
    }
    data
  }
}

pub struct GameState {
  // Pair connections to objects
  player_list: HashMap<Uuid, usize>,
  pub level: Level,
  physics: Physics,
  pub state_changes: HashMap<usize, ObjectUpdate>,
  pub send_full: bool
}
impl GameState {
  
  pub fn new() -> Self {
    let mut physics = Physics::new();
    let level = Level::new("level1".to_owned(), &mut physics);
    let mut state_changes = HashMap::new();
    for id in level.list.keys() {
      let object = level.get_obj(*id).unwrap();
      let update = ObjectUpdate::new()
        .position(object.position)
        .shape(object.points.clone())
        .material(object.material);
      state_changes.insert(*id, update);
    }
    Self {
      player_list: HashMap::new(),
      level,
      physics,
      state_changes,
      send_full: false,
    }
  }

  fn update_player(&mut self, id: Uuid, delta: IVec2) {
    let handle = self.player_list.get(&id).unwrap();
    self.level.apply_vel(self.physics.body_sets().0, *handle, delta);
  }

  fn add_player(&mut self, connection_id: Uuid) {
    let object_id = self.level.add_object(
      Object::new_mouse(IVec2::new(100, 100)),
      &mut self.physics
     );
    self.player_list.insert(connection_id, object_id);
    let object = self.level.get_obj(object_id).unwrap();
    let update = ObjectUpdate::new()
      .position(object.position)
      .shape(object.points.clone())
      .material(object.material);
    self.state_changes.insert(object_id, update);
  }

  pub fn tick(&mut self) { 
    self.physics.step(&mut self.level);
    self.level.tick(&mut self.physics, &mut self.state_changes);
  }

  pub fn handle_events(&mut self, server: &mut Server) {
    while let Ok(event) = server.mailbox.try_recv() {
      match event {
        Event::Connect(socket) => {
          self.add_player(server.connect_socket(socket));
          self.send_full = true;
        },
        Event::Disconnect(id) => { server.list.remove(&id); }
        Event::Binary(id, message) => {
          if let Message::Binary(bytes) = message {
            let real_bytes = bytes.to_vec();
            let data: &[i32] = bytemuck::cast_slice(&real_bytes);
            self.update_player(id, IVec2::new(data[0], data[1]));
          }
        }
      }
    }
  }

}
