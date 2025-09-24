import Vec2 from "./math.js";
const sensitivity = document.getElementById("sensitivity");

export default class Level {
  constructor() { this.entities = new Map(); }

  // Deserializer
  // Binary Format:
  // [key, color, position, point_count, points (x, y, x, y)]
  // You should pass in a slice of data which contains an object
  // Player,
  // Wall,
  // Death,
  // WinZone,
  load_obj(data) {
    let entity = new Entity();
    let key = data[0];
    switch (data[1]) {
      case 0: // Player
        entity.color = "white";
        entity.outline = true;
        entity.priority = 3;
        break;
      case 1: // Wall
        entity.color = "black";
        entity.priority = 1;
        break;
      case 2: // Death
        entity.color = "red";
        entity.priority = 2;
        break;
      case 3: // WinZone
        entity.color = "green";
        entity.priority = 1;
        break;
    }
    entity.pos = new Vec2(data[2], data[3]);
    let point_count = data[4];
    const first_point_idx = 5;
    for (let point = 0; point < point_count; point += 1) {
      entity.points.push(new Vec2(
        data[first_point_idx + point * 2],
        data[first_point_idx + point * 2 + 1],
      ));
    }
    this.entities.set(key, entity);
  }

  // Deserializer
  // Binary Format:
  // [key, pos(x, y)]
  // You should pass in a slice of data which contains all key location pairs that need updating
  update_pos(data) {
    for (let i = 0; i < data.length; i += 3) {
      let entity = this.entities.get(data[i]);
      entity.pos = new Vec2(data[i + 1], data[i + 2]);
    }
  }
  
  render(ctx) { 
    [...this.entities.entries()]
      .sort((a, b) => a[1].priority - b[1].priority)
      .forEach(entity => {entity[1].render(ctx)} );
    // this.entities.forEach(entity => entity.render(ctx));
  }
}

class Entity {
  constructor(pos = new Vec2(), color = "Black") {
    this.pos = pos;
    this.priority = 0;
    this.color = color;
    this.outline = false;
    this.points = []; // Points = [Vec2]
  }
  
  render(ctx) {
    ctx.fillStyle = this.color;
    ctx.beginPath();
    ctx.moveTo(this.points[0].x + this.pos.x, this.points[0].y + this.pos.y);
    for (let i = 1; i < this.points.length; i++) {
      let translated = this.points[i].add(this.pos);
      ctx.lineTo(translated.x, translated.y);
    }
    ctx.closePath();
    ctx.fill();
    if (this.outline) {
      ctx.strokeStyle = "black";
      ctx.lineWidth = 1;
      ctx.stroke();
    }
  }
}
