mod networking;
mod game;
use game::Level;
use std::{net::SocketAddr, thread::sleep, time::{Duration, Instant}};
use std::collections::HashMap;
use axum::extract::ws::Message;
use glam::IVec2;
use networking::{Server, Event};
use uuid::Uuid;
use crate::game::{Object, RenderData};
use crate::game::Physics;

const SERVER_UUID: Uuid = Uuid::nil();

struct GameState {
  player_list: HashMap<Uuid, usize>,
  current_level: usize,
  levels: Vec<Level>,
  physics: Physics,
  full_update: bool,
}
impl GameState {
  
  pub fn new() -> Self {
    let mut physics = Physics::new();
    Self {
      player_list: HashMap::new(),
      current_level: 0,
      levels: vec![Level::new(vec![
        Object::new_rect(IVec2::new(225, 150), IVec2::new(50, 200), RenderData::Wall),
        Object::new_rect(IVec2::new(150, 225), IVec2::new(200, 50), RenderData::Wall),
        Object::new_rect(IVec2::new(300, 300), IVec2::new(200, 50), RenderData::WinZone),
      ], &mut physics)],
      physics,
      full_update: true
    }
  }

  fn active_objects(&self) -> u32 { self.levels[self.current_level].list.len() as u32 }

  fn update_player(&mut self, id: Uuid, delta: IVec2) {
    let cur_level = &mut self.levels[self.current_level];
    let handle = self.player_list.get(&id).unwrap();
    cur_level.apply_vel(self.physics.body_sets().0, *handle, delta);
  }

  fn add_player(&mut self, connection_id: Uuid) {
    let cur_level = &mut self.levels[self.current_level];
    let object_id = cur_level.add_object(
      Object::new_mouse(IVec2::new(100, 100)),
      &mut self.physics
     );
    self.player_list.insert(connection_id, object_id);
    self.full_update = true;
  }

  fn tick(&mut self) { 
    self.physics.step(&mut self.levels[self.current_level]);
    self.levels[self.current_level].tick(&mut self.physics);
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

    broadcast_state(&game_state, &mut server);
  }

}

fn broadcast_state(state: &GameState, server: &mut Server) {
  let mut message_data = vec!(if state.full_update { state.active_objects() as i32 } else { 0 });
  // If the state is dirty, an object was added, quick and easy update by refreshing all objects
  // This isn't optimal, but I don't wanna implement per connection state tracking right now
  if state.full_update {
    for obj_id in state.levels[state.current_level].list.keys() {
      let object = state.levels[state.current_level].get_obj(*obj_id).unwrap();
      message_data.extend(object.serialize(*obj_id));
    }
  } else {
    for obj_id in &state.levels[state.current_level].movable {
      let pos = state.levels[state.current_level].get_pos(*obj_id);
      message_data.push(*obj_id as i32);
      message_data.push(pos.x);
      message_data.push(pos.y);
    }
  }
  let message = Message::Binary(bytemuck::cast_slice(&message_data).to_vec().into());
  for (_, connection) in &server.list {
    let _ = connection.send(Event::Binary(SERVER_UUID, message.clone()));
  }
}

