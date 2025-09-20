import Vec2 from "./math.js";

// This should link with the enum in src/level
const COLORS = new Map();
COLORS.set(0, "red");
COLORS.set(1, "blue");

export default class Level {
  // Deserializer
  // Binary Format:
  // [key, color, point_count, position, points (x, y, x, y)]
  constructor(data) {
    this.entities = new Map();
    let cur_idx = 0;
    let point_count = 0;
    let cur_key;
    let cur_entity = new Entity();
    while (cur_idx < data.length) {
      // Load next shape
      if (point_count == 0) { 
        cur_key = data[cur_idx];
        cur_idx += 1;
        cur_entity.color = COLORS.get(data[cur_idx]);
        cur_idx += 1;
        point_count = data[cur_idx];
        cur_idx += 1;
        cur_entity.pos = new Vec2(data[cur_idx], data[cur_idx + 1]);
        cur_idx += 2;
      }
      cur_entity.points.push(new Vec2(data[cur_idx], data[cur_idx + 1]));
      cur_idx += 2; point_count -= 1;
      if (point_count == 0) {
        this.entities.set(cur_key, cur_entity);
        cur_entity = new Entity();
      }
    }
  }
  
  // Deserializer
  // Binary Format:
  // [key, pos(x, y)]
  update(data) {
    for (let i = 0; i < data.length; i += 3) {
      let entity = this.entities.get(data[i]);
      entity.pos = new Vec2(data[i + 1], data[i + 2]);
    }
  }
  
  render(ctx) { this.entities.forEach(entity => entity.render(ctx)); }
}

class Entity {
  constructor(pos = new Vec2(), color = "Black", points = []) {
    this.pos = pos;
    this.color = color
    this.points = points; // Points = [Vec2]
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
  }

}
