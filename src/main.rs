mod networking;
use std::{collections::HashMap, net::SocketAddr};
use axum::extract::ws::Message;
use glam::IVec2;
use networking::{Server, Event};
use uuid::Uuid;

const SERVER_UUID: Uuid = Uuid::nil();

struct Player(IVec2);

struct Level;

struct GameState {
  player_list: HashMap<Uuid, IVec2>,
  current_level: Level,
  levels: Vec<Level>,
}
impl GameState {
  
  pub fn new() -> Self {
    Self {
      player_list: HashMap::new(),
      current_level: Level,
      levels: Vec::new(),
    }
  }

  pub fn handle_events(&mut self, server: &mut Server) {
    while let Ok(event) = server.mailbox.try_recv() {
      match event {
        Event::Connect(socket) => { self.player_list.insert(server.connect_socket(socket), IVec2::ZERO); },
        Event::Disconnect(id) => { server.list.remove(&id); }
        Event::Message(sender, message) => {
          if let Message::Binary(data) = message {
            let a = &data.to_vec();
            let data: &[i32] = bytemuck::cast_slice(&a);
            let player = self.player_list.get_mut(&sender).unwrap();
            *player = IVec2::from_slice(data);

            let mut positions = Vec::new();
            for (_, player) in self.player_list.iter() {
              positions.push(player.x);
              positions.push(player.y);
            }
            let binary: Vec<u8> = bytemuck::cast_slice(&positions).to_vec();

            let _ = server.list.get(&sender).unwrap().send(
              Event::Message(SERVER_UUID, Message::Binary(binary.into()))
            );
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

  loop {
    game_state.handle_events(&mut server);
  }

}

