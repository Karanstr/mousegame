const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d"); // 2D context
ctx.fillStyle = 'black';
ctx.fillRect(0,0, canvas.width, canvas.height);

// Connect to WebSocket
const socket = new WebSocket("ws://localhost:8080/ws");

socket.onopen = () => { };
socket.onclose = () => { };

const ServerToClient = Object.freeze({
  STATE_UPDATE: 0,
  LEVEL_UPDATE: 1
});

function sendMousePos(x, y) { 
  socket.send(new Int32Array( [x, y] ));
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
let geometry = new Int32Array;
let current = new Player(0, 0);

canvas.addEventListener("mousemove", (e) => {
  current.move_to(e.offsetX, e.offsetY);
  sendMousePos(current.x, current.y);
});

socket.onmessage = (msg) => {
  msg.data.bytes().then(bytes => {
    const data = new Int32Array(bytes.buffer);
    let flagless_data = data.subarray(1)
    switch (data[0]) {
      case ServerToClient.STATE_UPDATE: {
        update_players(flagless_data);
        break;
      }
      case ServerToClient.LEVEL_UPDATE: {
        geometry = Int32Array.from(flagless_data);
        break;
      }
      default: { console.log("Unknown Flag: " + data[0]); }
    }
  })
};

// Data is an i32Array
function update_players(data) {
  serverState.players = [];
  for (let i = 0; i < data.length / 2; i++) {
    serverState.players.push(new Player(
      data[i * 2],
      data[i * 2 + 1]
    ));
  }
  clientState.players = serverState.players.map(p => new Player(p.x, p.y));
}

// This should link with the enum in src/level
const COLORS = new Map();
COLORS.set(0, "red");

function draw_static(point_array) {
  let cur_idx = 0;
  let rem_points = 0;
  while (cur_idx < point_array.length) {
    if (rem_points == 0) { // Load next shape
      rem_points = point_array[cur_idx];
      ctx.fillStyle = COLORS.get(point_array[cur_idx + 1]);
      ctx.beginPath();
      ctx.moveTo(point_array[cur_idx + 2], point_array[cur_idx + 3]);
      cur_idx += 4; rem_points -= 1;
    } 
    ctx.lineTo(point_array[cur_idx], point_array[cur_idx + 1]);
    cur_idx += 2; rem_points -= 1;
    if (rem_points == 0) { ctx.closePath(); ctx.fill(); }
  }
}

function render() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  draw_static(geometry);
  
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

function gameLoop(_timestamp) {
  render();
  requestAnimationFrame(gameLoop);
}

requestAnimationFrame(gameLoop);

