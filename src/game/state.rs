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
  Delete   = 0b00000001, // Size: 0
  Position = 0b00000010, // Size: 8 -- 2
  Shape    = 0b00000100, // Size: 4 + 8*length -- 1 + 2 * length
  Material = 0b00001000, // Size: 4 -- 1
  Hide     = 0b00010000, // Size: 0
  Show     = 0b00100000, // Size: 0
}
#[derive(Clone)]
pub struct ObjectUpdate {
  position: Option<IVec2>,
  shape: Option<Vec<IVec2>>,
  material: Option<Material>,
  hidden: bool,
  delete: bool,
}
impl ObjectUpdate {
  pub fn new() -> Self {
    Self { position: None, shape: None, material: None, hidden: false, delete: false }
  }
  pub fn delete(&mut self) -> &mut Self {
    self.delete = true;
    self
  }
  pub fn position(&mut self, position: IVec2) -> &mut Self {
    self.position = Some(position);
    self
  }
  pub fn shape(&mut self, shape: Vec<IVec2>) -> &mut Self {
    self.shape = Some(shape);
    self
  }
  pub fn material(&mut self, material: Material) -> &mut Self {
    self.material = Some(material);
    self
  }
  pub fn hidden(&mut self, hidden: bool) -> &mut Self {
    self.hidden = hidden;
    self
  }
  pub fn to_binary(&self) -> Vec<i32> {
    let flag = 
      if self.position.is_some() { StateFlags::Position as i32 } else { 0 }      |
      if self.shape.is_some() { StateFlags::Shape as i32 } else { 0 }            | 
      if self.material.is_some() { StateFlags::Material as i32 } else { 0 }      |
      if self.hidden { StateFlags::Hide as i32 } else { StateFlags::Show as i32} |
      if self.delete { StateFlags::Delete as i32 } else { 0 };
    let mut data = Vec::new();
    data.push(flag);
    if let Some(position) = self.position {
      data.push(position.x);
      data.push(position.y);
    }
    if let Some(shape) = self.shape.clone() {
      data.push(shape.len() as i32);
      for point in shape {
        data.push(point.x);
        data.push(point.y);
      }
    }
    if let Some(material) = self.material {
      data.push(material.color())
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
  pub send_full: bool,
  pub send_new: bool,
}
impl GameState {
  
  pub fn new(initial_level: String) -> Self {
    let mut physics = Physics::new();
    let level = Level::new(initial_level, &mut physics);
    Self {
      player_list: HashMap::new(),
      level,
      physics,
      state_changes: HashMap::new(),
      send_full: false,
      send_new: true,
    }
  }

  pub fn load(&mut self, level: String) {
    self.level = Level::new(level, &mut self.physics);
    for (uuid, _) in &self.player_list.clone() {
      let obj_id = self.level.add_object(Object::new_mouse(), Vec::new(), &mut self.physics, true);
      self.player_list.insert(*uuid, obj_id);
    }
    self.send_new = true;
  }

  pub fn tick(&mut self) { 
    self.level.step_animations(&mut self.physics);
    self.physics.step(&mut self.level);
    if let Some(next_level) = self.level.tick(&mut self.physics, &mut self.state_changes) {
      self.load(next_level);
    }
  }

  pub fn handle_events(&mut self, server: &mut Server) {
    while let Ok(event) = server.mailbox.try_recv() {
      match event {
        Event::Connect(socket) => {
          self.add_player(server.connect_socket(socket));
          self.send_full = true;
        },
        Event::Disconnect(id) => { 
          server.list.remove(&id);
          let obj_id = self.player_list.remove(&id).unwrap();
          self.level.delete(obj_id, &mut self.physics);
          self.state_changes.entry(obj_id)
            .or_insert(ObjectUpdate::new()).delete();
        }
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

// Add Player, UpdatePlayer
impl GameState {
  fn update_player(&mut self, id: Uuid, delta: IVec2) {
    let handle = self.player_list.get(&id).unwrap();
    self.level.apply_vel(self.physics.body_sets().0, *handle, delta);
  }

  fn add_player(&mut self, connection_id: Uuid) {
    let object_id = self.level.add_object(
      Object::new_mouse(),
      Vec::new(), &mut self.physics, true
    );
    self.player_list.insert(connection_id, object_id);
    let object = self.level.get_obj(object_id).unwrap();
    let update = ObjectUpdate::new()
      .position(object.position)
      .shape(object.points.clone())
      .material(object.material)
      .clone();
    self.state_changes.insert(object_id, update);
  }
}

