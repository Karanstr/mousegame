mod networking;
mod game;
use std::{net::SocketAddr, thread::sleep, time::{Duration, Instant}};
use axum::extract::ws::Message;
use networking::{Server, Event};
use uuid::Uuid;
use crate::game::GameState;
const SERVER_UUID: Uuid = Uuid::nil();

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
  // If the state is dirty an object was added or removed, quick and easy update by refreshing all objects
  // This isn't optimal, but I don't wanna implement per connection state tracking right now
  if state.full_update {
    for obj_id in state.level.list.keys() {
      let object = state.level.get_obj(*obj_id).unwrap();
      message_data.extend(object.to_binary(*obj_id));
    }
  } else {
    for obj_id in &state.level.movable {
      let pos = state.level.get_pos(*obj_id);
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

