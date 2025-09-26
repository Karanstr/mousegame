import Vec2 from "./math.js";
const sensitivity = document.getElementById("sensitivity");

export default class Level {
  constructor() { this.entities = new Map(); }


  handle_update(data) {
    let key = data[0];
    let flags = data[1];
    let idx = 2;
    if (flags == 0) { this.entities.delete(key); return; }
    let entity = this.entities.get(key) ?? new Entity();
    if ((flags & 0b1) != 0) {
      entity.pos = new Vec2(data[idx], data[idx + 1]);
      idx += 2;
    }
    if ((flags & 0b10) != 0) {
      entity.points = [];
      let point_count = data[idx];
      idx += 1;
      for (let point = 0; point < point_count; point += 1) {
        entity.points.push(new Vec2(data[idx], data[idx + 1]));
        idx += 2;
      }
    }
    if ((flags & 0b100) != 0) {
      entity.update_material(data[idx]);
      idx += 1;
    }
    this.entities.set(key, entity);
  }




  render(ctx) { 
    [...this.entities.entries()]
      .sort((a, b) => a[1].priority - b[1].priority)
      .forEach(entity => {entity[1].render(ctx)} );
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

  update_material(material) {
    switch (material) {
      case 0: // Player
        this.color = "white";
        this.outline = true;
        this.priority = 3;
        break;
      case 1: // Wall
        this.color = "black";
        this.priority = 1;
        break;
      case 2: // Death
        this.color = "red";
        this.priority = 2;
        break;
      case 3: // WinZone
        this.color = "green";
        this.priority = 1;
        break;
    }
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
