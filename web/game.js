import Vec2 from './math.js';
import Level from './level.js';
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

let level = new Level(new Int32Array());
function gameLoop(_timestamp) {
  render();
  requestAnimationFrame(gameLoop);
}
requestAnimationFrame(gameLoop);

canvas.addEventListener("contextmenu", e => e.preventDefault());

// Connect to WebSocket
let connected = false;
const socket = new WebSocket("ws://localhost:8080/ws");
socket.onopen = () => { connected = true; };
socket.onclose = () => { connected = false; };

canvas.addEventListener("click", async () => { await canvas.requestPointerLock(); });
const sensitivity = document.getElementById("sensitivity");
canvas.addEventListener("mousemove", (e) => {
  // Only send mouse movement when locked
  if (document.pointerLockElement === canvas) {
    socket.send(new Int32Array( [
      e.movementX * sensitivity.value,
      e.movementY * sensitivity.value
    ] ));
  }
});

socket.onmessage = (msg) => {
  msg.data.arrayBuffer().then(bytes => {
    const data = new Int32Array(bytes);
    let update_count = Math.abs(data[0]);
    if (data[0] < 0) { level.clear(); }
    let cur_idx = 1;
    for (let update = 0; update < update_count; update += 1) {
      let size = data[cur_idx];
      cur_idx += 1;
      level.handle_update(data.subarray(cur_idx, cur_idx + size));
      cur_idx += size;
    }
  })
};

// Add a camera scaling to go from real game size to display size
function render() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  if (connected) { level.render(ctx); }
}

