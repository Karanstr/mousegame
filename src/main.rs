mod networking;
mod game;
use std::{net::SocketAddr, thread::sleep, time::{Duration, Instant}};
use axum::extract::ws::Message;
use networking::{Server, Event};
use uuid::Uuid;
use crate::game::{GameState, ObjectUpdate};
const SERVER_UUID: Uuid = Uuid::nil();

#[tokio::main]
async fn main() {
  let mut server = Server::new(SocketAddr::from(([0, 0, 0, 0], 8080)));
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

    broadcast_state(&mut game_state, &mut server);
  }

}

// Update consists of [i32_count, id, data]
fn broadcast_state(state: &mut GameState, server: &mut Server) {
  let mut message_data = Vec::new();
  message_data.push(0);
  if state.send_full || state.send_new {
    for id in state.level.list.keys() {
      let obj = state.level.get_obj(*id).unwrap();
      let mut update_data = ObjectUpdate::new()
        .position(obj.position)
        .shape(obj.points.clone())
        .material(obj.material)
        .to_binary();
      message_data.push(update_data.len() as i32 + 1);
      message_data.push(*id as i32);
      message_data.append(&mut update_data);
      message_data[0] += 1;
    }
    state.send_full = false;
  } else if state.state_changes.len() == 0 { return } 
  if state.send_new { 
    message_data[0] *= -1;
    state.send_new = false;
    state.state_changes.clear();
  } else {
    for (id, update) in state.state_changes.drain() {
      let mut update_data = update.to_binary();
      message_data.push(update_data.len() as i32 + 1);
      message_data.push(id as i32);
      message_data.append(&mut update_data);
      message_data[0] += 1;
    }
  }
  let message = Message::Binary(bytemuck::cast_slice(&message_data).to_vec().into());
  for (_, connection) in &server.list {
    let _ = connection.send(Event::Binary(SERVER_UUID, message.clone()));
  }
}

