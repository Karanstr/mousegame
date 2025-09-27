import Vec2 from "./math.js";
import Entity from "./entity.js";
const sensitivity = document.getElementById("sensitivity");

export default class Level {
  constructor() { this.entities = new Map(); }
  
  clear() {
    this.entities = new Map();
  }

  handle_update(data) {
    let key = data[0];
    let flags = data[1];
    let idx = 2;
    if ((flags & 0b1) != 0) { this.entities.delete(key); return; }
    let entity = this.entities.get(key) ?? new Entity();
    if ((flags & 0b10) != 0) {
      entity.pos = new Vec2(data[idx], data[idx + 1]);
      idx += 2;
    }
    if ((flags & 0b100) != 0) {
      entity.points = [];
      let point_count = data[idx];
      idx += 1;
      for (let point = 0; point < point_count; point += 1) {
        entity.points.push(new Vec2(data[idx], data[idx + 1]));
        idx += 2;
      }
    }
    if ((flags & 0b1000) != 0) {
      entity.update_material(data[idx]);
      idx += 1;
    }
    if ((flags & 0b10000) != 0) {
      entity.hidden = true;
    }
    if ((flags & 0b100000) != 0) {
      entity.hidden = false;
    }
    this.entities.set(key, entity);
  }

  render(ctx) { 
    [...this.entities.entries()]
      .sort((a, b) => a[1].priority - b[1].priority)
      .forEach(entity => {entity[1].render(ctx)} );
  }
}

