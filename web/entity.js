import Vec2 from "./math.js";

export default class Entity {
  constructor(pos = new Vec2(), color = "Black") {
    this.pos = pos;
    this.priority = 0;
    this.color = color;
    this.outline = false;
    this.points = []; // Points = [Vec2]
    this.hidden = false;
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
      case 3: // WinOn
        this.color = "lime";
        this.priority = 1;
        break;
      case 4: // WinOff
        this.color = "green";
        this.priority = 1;
        break;
    }
  }
  
  render(ctx) {
    let center = new Vec2(window.innerWidth / 2, window.innerHeight / 2);
    let real_pos = this.pos.add(center);
    if (this.hidden) { return }
    ctx.fillStyle = this.color;
    ctx.beginPath();
    ctx.moveTo(this.points[0].x + real_pos.x, this.points[0].y + real_pos.y);
    for (let i = 1; i < this.points.length; i++) {
      let translated = this.points[i].add(real_pos);
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
