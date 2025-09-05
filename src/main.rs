use axum::{
  extract::ws::{WebSocketUpgrade, WebSocket, Message},
  extract::State,
  routing::any,
  Router,
};
use uuid::Uuid;
use tower_http::services::ServeDir;
use futures::{StreamExt, SinkExt};
use std::{
  net::SocketAddr,
  collections::hash_map::HashMap,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::net::TcpListener;

enum Event {
  Connect { id: Uuid, tx: UnboundedSender<Message> },
  Message { id: Uuid, msg: Message },
  Disconnect { id: Uuid },
}

async fn server(mut rx: UnboundedReceiver<Event>) {
  let mut clients: HashMap<Uuid, UnboundedSender<Message>> = HashMap::new();
  while let Some(event) = rx.recv().await {
    match event {
      Event::Connect { id, tx } => { clients.insert(id, tx); }
      Event::Disconnect { id } => { clients.remove(&id); }
      Event::Message { id, msg } => {
        match msg {
          Message::Text(text) => {
            let msg = text.as_str();
            println!("Recieved: {msg} from {id}");
            for (client, tx) in clients.iter() {
              let name = if client == &id { "You: ".to_owned() } else { format!("{client}: ") };
              let full_message = Message::Text((name + msg).into());
              let _ = tx.send(full_message); // The socket thread should clean this up, so we can ignore
            }
          }
          _ => {}
        }

      }
    }
  }

}

// Per-client message thread
async fn handle_socket(socket: WebSocket, session_uuid: Uuid, svr_tx: UnboundedSender<Event>) {
  let (client_tx, mut mailbox) = unbounded_channel::<Message>();
  let _ = svr_tx.send(Event::Connect { id: session_uuid, tx: client_tx } );
  let (mut sender, mut receiver) = socket.split();
  
  let ws_output = tokio::spawn(async move {
    while let Some(msg) = mailbox.recv().await {
      if sender.send(msg).await.is_err() { break; }
    }
  });

  let ws_input = {
    let svr_tx = svr_tx.clone();
    tokio::spawn(async move {
      while let Some(Ok(msg)) = receiver.next().await {
        let _ = svr_tx.send(Event::Message {id: session_uuid, msg} );
      }
    })
  };

  // When either thread fails, the client has disconnected
  tokio::select! {
    _ = ws_output => (),
    _ = ws_input => (),
  }
  let _ = svr_tx.send(Event::Disconnect { id: session_uuid });
}

async fn ws_handler(ws: WebSocketUpgrade, State(svr_tx): State<UnboundedSender<Event>>) -> impl axum::response::IntoResponse {
  ws.on_upgrade(move |socket| handle_socket(socket, Uuid::new_v4(), svr_tx))
}

#[tokio::main]
async fn main() {
  let (svr_tx, svr_rx) = unbounded_channel::<Event>();
  tokio::spawn( async move { server(svr_rx).await; });

  let app = Router::new()
    .route("/ws", any(ws_handler))
    .with_state(svr_tx)
  .fallback_service(ServeDir::new("static"));

  let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
  println!("Running at http://{}", addr);

  axum::serve(TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}

