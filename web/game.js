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

// Connect to WebSocket
const socket = new WebSocket("ws://localhost:8080/ws");
// socket.onopen = () => { };
// socket.onclose = () => { };

canvas.addEventListener("click", async () => { await canvas.requestPointerLock(); });
canvas.addEventListener("mousemove", (e) => {
  socket.send(new Int32Array( [e.movementX, e.movementY] ))
});

const ServerToClient = Object.freeze({
  LEVEL_INIT: 0,
  LEVEL_UPDATE: 1
});
socket.onmessage = (msg) => {
  msg.data.bytes().then(bytes => {
    const data = new Int32Array(bytes.buffer);
    let flagless_data = data.subarray(1)
    switch (data[0]) {
      case ServerToClient.LEVEL_INIT: {
        level = new Level(flagless_data);
        break;
      }
      case ServerToClient.LEVEL_UPDATE: {
        level.update(flagless_data);
        break;
      }
      default: { console.log("Unknown Flag: " + data[0]); }
    }
  })
};



// Add a camera scaling to go from real game size to display size
function render() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  level.render(ctx);
}


