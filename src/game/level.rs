use rapier2d::{na::Vector2, prelude::*};
use glam::IVec2;
use std::{collections::{HashMap, HashSet}};
use lilypads::Pond;
use parking_lot::Mutex;
use crate::game::Material;

use super::{Object, Physics, serde::InitialLevel};
use super::state::ObjectUpdate;


pub struct Level {
  objects: Pond<Object>,
  pub list: HashMap<usize, RigidBodyHandle>,
  pub movable: HashSet<usize>,
  events: Mutex<Vec<CollisionEvent>>,
}
impl Level {
  pub fn tick(&mut self, physics: &mut Physics, state_changes: &mut HashMap<usize, ObjectUpdate>) {
    let (rigids, colliders) = physics.body_sets();
    for event in self.events.get_mut().into_iter() {
      let (started, handle_1, handle_2) = match event {
        CollisionEvent::Started(handle_1, handle_2, _) => {
          (true, handle_1, handle_2)
        }
        CollisionEvent::Stopped(handle1, handle2, _) => {
          (false, handle1, handle2)
        }
      };
      let id_1 = id_from_collider(*handle_1, &rigids, &colliders);
      let id_2= id_from_collider(*handle_2, &rigids, &colliders);
      let (player_id, sensor_id) = if matches!(
        self.objects.get(id_1).unwrap().material,
        Material::Player(_)
      ) { (id_1, id_2) } else { (id_2, id_1) };
      let sensor_type = self.objects.get(sensor_id).unwrap().material;
      match sensor_type {
        Material::Death => {

        }
        Material::Button(channel, active) => {
          let button= self.objects.get_mut(sensor_id).unwrap();
          button.material.toggle();
          state_changes.entry(sensor_id)
            .or_insert(ObjectUpdate::new()).material(button.material);
        }
        _ => unimplemented!()
      }
    }
    self.events.get_mut().clear();
    self.register_movement(rigids, state_changes);
  }


  pub fn register_movement(&mut self, rigids: &mut RigidBodySet, state_changes: &mut HashMap<usize, ObjectUpdate>) {
    for id in &self.movable {
      let new_pos = self.get_rapier_pos(*id, rigids);
      let old_pos = &mut self.objects.get_mut(*id).unwrap().position;
      if *old_pos != new_pos {
        state_changes.entry(*id)
          .or_insert(ObjectUpdate::new()).position(new_pos);
        *old_pos = new_pos;
      }
    }
  }

}

fn id_from_collider(handle: ColliderHandle, rigids: &RigidBodySet, colliders: &ColliderSet) -> usize {
  let rbh = colliders.get(handle).unwrap().parent().unwrap();
  rigids.get(rbh).unwrap().user_data as usize
}

impl Level {
  pub fn new(level: String, physics: &mut Physics) -> Self {
    physics.reset_bodies();
    let mut new = Self { 
      objects: Pond::new(),
      list: HashMap::new(),
      movable: HashSet::new(),
      events: Mutex::new(Vec::new()),
    };
    let json = std::fs::read_to_string(
      format!("{}/levels/{}", env!("CARGO_MANIFEST_DIR"), level)
    ).unwrap();
    let deser_level: InitialLevel = serde_json::from_str(&json).unwrap();
    for min_obj in deser_level.objects {
      new.add_object(min_obj.full_rect(), physics, false);
    }
    new
  }

  pub fn add_object(&mut self, object: Object, physics: &mut Physics, can_move: bool) -> usize {
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
    if can_move { self.movable.insert(id); }
    id
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

  fn get_rapier_pos(&self, id: usize, rigids: &mut RigidBodySet) -> IVec2 {
    let handle = self.list.get(&id).unwrap();
    let bad_pos = rigids.get(*handle).unwrap().position().translation;
    IVec2::new(bad_pos.x as i32, bad_pos.y as i32)
  }
}

impl EventHandler for Level {
  fn handle_collision_event(
    &self,
    _: &RigidBodySet,
    _: &ColliderSet,
    event: CollisionEvent,
    _: Option<&ContactPair>,
  ) { self.events.lock().push(event); }
  
  fn handle_contact_force_event( &self, _: f32, _: &RigidBodySet, _: &ColliderSet, _: &ContactPair, _: f32,) { }
}

