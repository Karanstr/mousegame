use rapier2d::{na::Vector2, prelude::*};
use glam::IVec2;
use std::collections::{HashMap, HashSet};
use lilypads::Pond;
use parking_lot::Mutex;
use super::Object;
use super::Physics;

pub struct Level {
  objects: Pond<Object>,
  pub list: HashMap<usize, RigidBodyHandle>,
  pub movable: HashSet<usize>,
  events: Mutex<Vec<CollisionEvent>>,
}
impl Level {
  pub fn tick(&mut self, physics: &mut Physics) {
    let (rigids, _) = physics.body_sets();
    // Update object list
    for id in &self.movable {
      self.objects.get_mut(*id).unwrap().origin = self.get_rapier_pos(*id, rigids);
    }
    let events = self.events.get_mut();
    for event in events.into_iter() {
      dbg!(event);
    }
    events.clear();
  }

  pub fn apply_vel(&mut self, rigids: &mut RigidBodySet, id: usize, velocity: IVec2) {
    if self.movable.contains(&id) {
      let handle = self.list.get(&id).unwrap();
      let rb = rigids.get_mut(*handle).unwrap();
      let mass = rb.mass();
      let impulse = velocity.as_vec2() * mass;
      rb.apply_impulse(Vector2::new(impulse.x as f32, impulse.y as f32), true);
    } else { dbg!("Failed to add velocity to collider"); }
  }

  pub fn get_obj(&self, id: usize) -> Option<&Object> { self.objects.get(id) }

  pub fn get_pos(&self, id: usize) -> IVec2 { self.objects.get(id).unwrap().origin }

  fn get_rapier_pos(&self, id: usize, rigids: &mut RigidBodySet) -> IVec2 {
    let handle = self.list.get(&id).unwrap();
    let bad_pos = rigids.get(*handle).unwrap().position().translation;
    IVec2::new(bad_pos.x as i32, bad_pos.y as i32)
  }

  pub fn add_object(&mut self, object: Object, physics: &mut Physics) -> usize {
    let id = self.objects.alloc(object);
    let object = self.objects.get_mut(id).unwrap();
    object.rigidbody.user_data = id as u128;
    let (rigids, colliders) = physics.body_sets();
    let rb_handle = rigids.insert(object.rigidbody.clone());
    colliders.insert_with_parent(
      object.collider.clone(),
      rb_handle,
      rigids
    );
    self.list.insert(id, rb_handle);
    if object.movable { self.movable.insert(id); }
    id
  }

  pub fn new(objects: Vec<Object>, physics: &mut Physics) -> Self { 
    let mut new = Self { 
      objects: Pond::new(),
      list: HashMap::new(),
      movable: HashSet::new(),
      events: Mutex::new(Vec::new()),
    };
    for object in objects { new.add_object(object, physics); }
    new
  }

}

impl EventHandler for Level {
  
  fn handle_collision_event(
    &self,
    bodies: &RigidBodySet,
    colliders: &ColliderSet,
    event: CollisionEvent,
    contact_pair: Option<&ContactPair>,
  ) { self.events.lock().push(event); }
  
  fn handle_contact_force_event( &self, _: f32, _: &RigidBodySet, _: &ColliderSet, _: &ContactPair, _: f32,) { }
}
