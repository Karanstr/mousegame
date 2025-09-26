use super::{Level, Physics, Object};
use std::collections::HashMap;
use glam::IVec2;
use uuid::Uuid;
use axum::extract::ws::Message;
use crate::networking::{Server, Event};

pub struct GameState {
  player_list: HashMap<Uuid, usize>,
  pub level: Level,
  physics: Physics,
  pub full_update: bool,
}
impl GameState {
  
  pub fn new() -> Self {
    let mut physics = Physics::new();
    Self {
      player_list: HashMap::new(),
      level: Level::new("level1".to_owned(), &mut physics),
      physics,
      full_update: true
    }
  }

  pub fn active_objects(&self) -> u32 { self.level.list.len() as u32 }

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
    self.full_update = true;
  }

  pub fn tick(&mut self) { 
    self.physics.step(&mut self.level);
    self.level.tick(&mut self.physics);
  }

  pub fn handle_events(&mut self, server: &mut Server) {
    while let Ok(event) = server.mailbox.try_recv() {
      match event {
        Event::Connect(socket) => {
          self.add_player(server.connect_socket(socket));
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
