mod networking;
mod level;
use level::Level;
use std::{collections::HashMap, net::SocketAddr, thread::sleep, time::{Duration, Instant}};
use axum::extract::ws::Message;
use glam::IVec2;
use networking::{Server, Event};
use uuid::Uuid;
use rapier2d::prelude::*;

use crate::level::Object;

const SERVER_UUID: Uuid = Uuid::nil();

struct GameState {
  player_list: HashMap<Uuid, RigidBodyHandle>,
  current_level: usize,
  levels: Vec<Level>,
}
impl GameState {
  
  pub fn new() -> Self {
    Self {
      player_list: HashMap::new(),
      current_level: 0,
      levels: vec![Level::new(vec![
        Object::new_wall(IVec2::splat(250), vec![
          IVec2::new(-25, -100),
          IVec2::new(-25, 100),
          IVec2::new(25, 100),
          IVec2::new(25, -100),
        ])
      ])],
    }
  }

  fn update_player(&mut self, id: Uuid, delta: IVec2) {
    let cur_level = &mut self.levels[self.current_level];
    let handle = self.player_list.get(&id).unwrap();
    cur_level.add_vel(*handle, delta * 10);
  }

  fn add_player(&mut self, id: Uuid) {
    let cur_level = &mut self.levels[self.current_level];
    let level_id = cur_level.add_object(Object::new_player(IVec2::ZERO));
    self.player_list.insert(id, level_id);
  }

  fn tick(&mut self) { self.levels[self.current_level].tick() }

  pub fn handle_events(&mut self, server: &mut Server) {
    while let Ok(event) = server.mailbox.try_recv() {
      match event {
        Event::Connect(socket) => {
          let id = server.connect_socket(socket);
          self.add_player(id);
          let _ = server.list.get(&id).unwrap().send(Event::Binary(SERVER_UUID, level_init(&self)));
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

#[tokio::main]
async fn main() {
  let mut server = Server::new(SocketAddr::from(([127, 0, 0, 1], 8080)));
  let mut game_state = GameState::new();
  
  let update_interval = Duration::from_millis(20); // 20 updates per second
  let mut last_update = Instant::now();
  loop {
    let now = Instant::now();
    let elapsed = now.duration_since(last_update);
    if elapsed < update_interval { sleep(update_interval - elapsed); }
    last_update = now;

    game_state.handle_events(&mut server);
    game_state.tick();

    update_clients(&game_state, &mut server);
  }

}



#[repr(u8)]
enum ServerToClient {
  LevelInit = 0,
  LevelUpdate = 1,
}
fn update_clients(state: &GameState, server: &mut Server) {
  let mut pos_data = vec!(ServerToClient::LevelUpdate as i32);
  for (_, rb_handle) in &state.player_list {
    let pos = state.levels[state.current_level].get_pos(*rb_handle);
    pos_data.push(rb_handle.into_raw_parts().0 as i32);
    pos_data.push(pos.x);
    pos_data.push(pos.y);
  }
  let message = Message::Binary(bytemuck::cast_slice(&pos_data).to_vec().into());
  for (_, connection) in &server.list {
    let _ = connection.send(Event::Binary(SERVER_UUID, message.clone()));
  }
}

fn level_init(state: &GameState) -> Message {
  // Start with flag
  let mut geo_data = vec!(ServerToClient::LevelInit as i32);
  for (key, object) in &state.levels[state.current_level].objects {
    geo_data.extend(object.serialize(*key))
  }
  // Start with flag
  Message::Binary(bytemuck::cast_slice(&geo_data).to_vec().into())
}

