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
// socket.onopen = () => { console.log("Connected"); };
// socket.onclose = () => { console.log("Disconnected"); };


document.addEventListener('mousemove', function(event) {
    console.log('X: ' + event.clientX + ', Y: ' + event.clientY);
});

canvas.addEventListener("click", async () => { await canvas.requestPointerLock({ unadjustedMovement: true }); });
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
    let new_obj_count = data[0];
    let cur_obj_start = 1; // 0 is the flag
    const point_count_offset = 4; // 4 from the first index of the object
    for (let obj = 0; obj < new_obj_count; obj += 1) {
      let point_count = data[cur_obj_start + point_count_offset];
      let data_length = 1 + point_count_offset + 2 * point_count;
      level.load_obj(data.subarray(cur_obj_start, cur_obj_start + data_length));
      cur_obj_start += data_length;
    }
    // Push all updated positions
    level.update_pos(data.subarray(cur_obj_start, data.length));
  })
};

// Add a camera scaling to go from real game size to display size
function render() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  level.render(ctx);
}

