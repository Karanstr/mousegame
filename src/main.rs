mod networking;
use std::net::SocketAddr;
use axum::extract::ws::Message;
use networking::{Server, Event};
use uuid::Uuid; // We could remove uuid but idc enough to write a custom id system

const SERVER_UUID: Uuid = Uuid::nil();

#[tokio::main]
async fn main() {
  let mut server = Server::new(SocketAddr::from(([127, 0, 0, 1], 8080)));
  
  loop {
    while let Ok(event) = server.mailbox.try_recv() {
      match event {
        Event::Connect(socket) =>  server.connect_socket(socket),
        Event::Disconnect(id) => { server.list.remove(&id); }
        Event::Message(sender, message) => {
          let text = message.into_text().unwrap();
          let string = text.as_str();
          println!("Recieved {string} from {sender}");
          for (client, tx) in server.list.iter() {
            let name = if client == &sender { "You: ".to_owned() } else { format!("{client}: ") };
            let full_message = Message::Text((name + string).into());
            let _ = tx.send(Event::Message(SERVER_UUID, full_message));
          }
        }
      }
    }
  }

}

