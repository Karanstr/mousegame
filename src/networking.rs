use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, SinkExt};
use std::{collections::hash_map::HashMap, net::SocketAddr};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use axum::{ extract::{ws::WebSocketUpgrade, State}, Router};
use tower_http::services::ServeDir;
use tokio::net::TcpListener;

pub enum Event {
  Connect(WebSocket),
  Message(uuid::Uuid, Message),
  Disconnect(uuid::Uuid),
}

pub struct Server {
  pub mailbox: UnboundedReceiver<Event>,
  pub tx: UnboundedSender<Event>,
  pub list: HashMap<uuid::Uuid, UnboundedSender<Event>>
}
impl Server {
  pub fn new(address: SocketAddr) -> Self {
    let (tx, mailbox) = unbounded_channel();
    
    let app = Router::new().route("/ws", axum::routing::get(
      |ws: WebSocketUpgrade, State(svr_tx): State<UnboundedSender<Event>>| async move {
        ws.on_upgrade(move |socket| async move { let _ = svr_tx.send(Event::Connect(socket)); })
    },),).with_state(tx.clone()).fallback_service(ServeDir::new("static"));
    
    tokio::spawn(async move {
      axum::serve(TcpListener::bind(address).await.unwrap(), app).await.unwrap();
    });
    println!("Running at http://{}", address);
    Self { mailbox, tx, list: HashMap::new() }
  }
  
  pub fn connect_socket(&mut self, socket: WebSocket) {
    let id = uuid::Uuid::new_v4();
    let (client_tx, client_mailbox) = unbounded_channel::<Event>();
    let server_tx = self.tx.clone();
    tokio::spawn(async move { handle_socket(socket, id, server_tx, client_mailbox).await });
    self.list.insert(id, client_tx);
  }

}

async fn handle_socket(socket: WebSocket, id: uuid::Uuid, server_tx: UnboundedSender<Event>, mut mailbox: UnboundedReceiver<Event>) {
  let (mut sender, mut receiver) = socket.split();
  
  // In the future we could update this to, say, close the websocket remotely
  // but for now we only care about it forwarding messages
  let ws_output = tokio::spawn(async move {
    while let Some(Event::Message(_, msg)) = mailbox.recv().await {
      if sender.send(msg).await.is_err() { break; }
    }
  });

  let ws_input = {
    let tx = server_tx.clone();
    tokio::spawn(async move {
      while let Some(Ok(msg)) = receiver.next().await {
        tx.send(Event::Message(id, msg)).unwrap();
      }
    })
  };

  tokio::select! {
    _ = ws_output => (),
    _ = ws_input => (),
  }
  dbg!("Socket shut down");
  server_tx.send(Event::Disconnect(id)).unwrap();
}

