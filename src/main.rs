use axum::{
  extract::ws::{WebSocketUpgrade, WebSocket, Message},
  extract::State,
  response::IntoResponse,
  routing::any,
  Router,
};
use uuid::Uuid;
use parking_lot::RwLock;
use tower_http::services::ServeDir;
use futures::{StreamExt, SinkExt};
use std::{
  net::SocketAddr,
  sync::Arc,
  collections::hash_map::HashMap,
};
use tokio::sync::mpsc;

type Clients = Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
  let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

  let app = Router::new()
    .route("/ws", any(ws_handler))
    .with_state(clients.clone())
  .fallback_service(ServeDir::new("static"));

  let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
  println!("Running at http://{}", addr);

  // Bind the listener and the router
  axum::serve(
    tokio::net::TcpListener::bind(addr).await.unwrap(),
    app,
  ).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(clients): State<Clients>) -> impl IntoResponse {
  ws.on_upgrade(move |socket| handle_socket(socket, clients))
}

// Per-client message thread
// Stream is the Websocket connection with the client
// Clients is an Arc containing a mutex list of broadcast ids (?)
async fn handle_socket(stream: WebSocket, clients: Clients) {
  let (mut sender, mut receiver) = stream.split();
  let (message_queue, mut mailbox) = mpsc::unbounded_channel();
  let session_uuid = Uuid::new_v4();
  // Register client
  clients.write().insert(session_uuid, message_queue);

  // Task: Push messages from the queue across the websocket
  let send_task = tokio::spawn(async move {
    // While the mailbox exists, wait for the next message.
    while let Some(msg) = mailbox.recv().await {
      if sender.send(msg).await.is_err() {
        // If the sender no longer exists, terminate the thread
        break;
      }
    }
  });

  // Task: Handle messages received from the websocket
  let recv_task = {
    // Give the thread a threadsafe list of all clients
    let clients = clients.clone();
    tokio::spawn(async move {
      while let Some(Ok(Message::Text(text))) = receiver.next().await {
        println!("Received: {}", text);
        let msg = Message::Text(text);
        for (id, client) in clients.read().iter() {
          if id == &session_uuid { continue }
          let text = Message::Text((format!("User {}: ", session_uuid) + msg.to_text().unwrap()).into());
          let _ = client.send(text); // Error if client is closed
        }
      }
    })
  };

  // When either thread fails, the client has disconnected
  tokio::select! {
    _ = send_task => {},
    _ = recv_task => {},
  }

  clients.write().remove(&session_uuid);
}
