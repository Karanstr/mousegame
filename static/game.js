// Connect to WebSocket
const socket = new WebSocket("ws://localhost:8080/ws");
const log = document.getElementById("log");

socket.onopen = () => {
  logMessage("Connected to server");
};

socket.onmessage = (event) => {
  logMessage(event.data);
};

socket.onclose = () => {
  logMessage("Disconnected");
};

function sendMessage() {
  const input = document.getElementById("msg");
  const text = input.value;
  socket.send(text);
  logMessage("You: " + text);
  input.value = "";
}

function logMessage(msg) {
  const li = document.createElement("li");
  li.textContent = msg;
  log.appendChild(li);
}
