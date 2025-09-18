mod networking;
mod level;
use level::Level;
use std::{collections::HashMap, net::SocketAddr, thread::sleep, time::{Duration, Instant}};
use axum::extract::ws::Message;
use glam::IVec2;
use networking::{Server, Event};
use uuid::Uuid;
use rapier2d::{na::Vector2, prelude::*};

const SERVER_UUID: Uuid = Uuid::nil();

#[repr(u8)]
enum ServerToClient {
  PlayerUpdate,
  LevelUpdate,
}

struct GameState {
  player_list: HashMap<Uuid, ColliderHandle>,
  current_level: usize,
  levels: Vec<Level>,
}
impl GameState {
  
  pub fn new() -> Self {
    Self {
      player_list: HashMap::new(),
      current_level: 0,
      levels: vec![Level::new(vec![])],
    }
  }

  fn update_player(&mut self, id: Uuid, new_pos: IVec2) {
    let cur_level = self.levels[self.current_level];
    cur_level.
    *self.player_list.get_mut(&id).unwrap() = new_pos;
  }

  pub fn handle_events(&mut self, server: &mut Server) {
    while let Ok(event) = server.mailbox.try_recv() {
      match event {
        Event::Connect(socket) => { self.player_list.insert(server.connect_socket(socket), IVec2::ZERO); }
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
  
  let update_interval = Duration::from_millis(50); // 20 updates per second
  let mut last_update = Instant::now();
  loop {
    let now = Instant::now();
    if now.duration_since(last_update) < update_interval {
      sleep(Duration::from_millis(1));
      continue
    }
    last_update = now;

    game_state.handle_events(&mut server);
    
    update_clients(&game_state, &mut server);
  }

}

// Fix this to also update level if a flag is set
fn update_clients(state: &GameState, server: &mut Server) {
  // Start with flag
  let mut pos_data = vec!(ServerToClient::PlayerUpdate as u8 as i32);
  for (_, player) in state.player_list.iter() {
    pos_data.push(player.x);
    pos_data.push(player.y);
  }
  let bin_pos: Vec<u8> = bytemuck::cast_slice(&pos_data).to_vec();

  // Start with flag
  let mut geo_data = vec!(ServerToClient::LevelUpdate as u8 as i32);
  geo_data.extend(state.levels[state.current_level].geometry().into_iter().flatten());
  let bin_geo: Vec<u8> = bytemuck::cast_slice(&geo_data).to_vec();
  
  for (_, connection) in server.list.iter() {
    let _ = connection.send(Event::Binary(SERVER_UUID, Message::Binary(bin_pos.clone().into())));
    let _ = connection.send(Event::Binary(SERVER_UUID, Message::Binary(bin_geo.clone().into())));
  }
}

