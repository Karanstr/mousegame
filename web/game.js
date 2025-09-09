const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d"); // 2D context
ctx.fillStyle = 'black';
ctx.fillRect(0,0, canvas.width, canvas.height);

// Connect to WebSocket
const socket = new WebSocket("ws://localhost:8080/ws");

socket.onopen = () => { };
socket.onclose = () => { };

function sendMousePos(x, y) { 
  console.log(new Int8Array( new Int32Array([x, y]).buffer));
  socket.send(new Int32Array([x, y]));
}

class Player {
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }

  move_to(x, y) {
    this.x = x;
    this.y = y;
  }
}

let serverState = { players: [] };   // last snapshot from server
let clientState = { players: [] };   // smoothed/predicted positions

canvas.style.cursor = "none";
let current = new Player(0, 0)

canvas.addEventListener("mousemove", (e) => {
  const rect = canvas.getBoundingClientRect();
  current.move_to(e.clientX - rect.left, e.clientY - rect.top);
  sendMousePos(current.x, current.y);
});

// --- Receive Updates ---
socket.onmessage = (msg) => {
  msg.data.bytes().then(bytes => { handleUpdate(new Int32Array(bytes.buffer)) })
};

// Data is an i32Array
function handleUpdate(data) {
  serverState = { players: [] }
  for (let i = 0; i < data.length / 2; i++) {
    serverState.players.push(new Player(
      data[i * 2],
      data[i * 2 + 1]
    ));
  }
  clientState = {
    players: serverState.players.map(p => new Player(p.x, p.y))
  };
}

// --- Rendering ---
function render() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Draw players
  ctx.fillStyle = "blue";
  for (const p of clientState.players) {
    ctx.fillRect(p.x - 10, p.y - 10, 20, 20);
  }

  ctx.fillStyle = "red";
  for (const p of serverState.players) {
    ctx.fillRect(p.x - 10, p.y - 10, 20, 20);
  }

}

// --- Game Loop ---
function gameLoop(timestamp) {
  render();
  requestAnimationFrame(gameLoop);
}

requestAnimationFrame(gameLoop);

