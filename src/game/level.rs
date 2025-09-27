use rapier2d::{na::Vector2, prelude::*};
use glam::IVec2;
use std::collections::{HashMap, HashSet};
use lilypads::Pond;
use parking_lot::Mutex;
use crate::game::Material;
use serde::Deserialize;
use super::{Object, Physics, serde::InitialLevel};
use super::state::ObjectUpdate;

#[derive(Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Action {
  Pause,
  Hide,
}
#[derive(PartialEq, Eq, Hash)]
struct RemoteControl {
  id: usize,
  channel: u8,
  action: Action
}

pub struct Level {
  objects: Pond<Object>,
  pub list: HashMap<usize, RigidBodyHandle>,
  pub players: HashSet<usize>,
  pub animated: HashSet<usize>,
  next: Option<String>,
  
  events: Mutex<Vec<CollisionEvent>>,
  players_on_button: HashMap<usize, usize>, // id, channel
  button_requirements: [u8; 16], // Arbitrarily support 16 channels
  receivers: HashSet<RemoteControl>,
}
impl Level {
  pub fn tick(&mut self, physics: &mut Physics, state_changes: &mut HashMap<usize, ObjectUpdate>) -> Option<String> {
    for event in self.events.get_mut().clone() {
      self.handle_event(event, physics, state_changes);
    }
    let win = self.handle_remote(state_changes);
    if win { return self.next.clone();}
    self.events.get_mut().clear();
    self.register_movement(physics.body_sets().0, state_changes);
    None
  }
  
  fn handle_remote(&mut self, state_changes: &mut HashMap<usize, ObjectUpdate>) -> bool {
    let mut channels_held = [0; 16];
    for channel in self.players_on_button.values() { channels_held[*channel as usize] += 1; }
    if channels_held[0] >= self.button_requirements[0] { return true; }
    for receiver in self.receivers.iter() {
      let activate = channels_held[receiver.channel as usize] >= self.button_requirements[receiver.channel as usize];
      let obj = self.objects.get_mut(receiver.id).unwrap();
      match receiver.action {
        Action::Pause => {
          obj.frozen = activate;
        }
        Action::Hide => {
          if obj.hidden != activate {
            obj.hidden = activate;
            state_changes.entry(receiver.id)
              .or_insert(ObjectUpdate::new()).hidden(activate);
          }
        }
      }
    }
    false
  }

  fn handle_event(&mut self, event: CollisionEvent, physics: &mut Physics, state_changes: &mut HashMap<usize, ObjectUpdate>) {
    let (rigids, colliders) = physics.body_sets();
    let (started, handle_1, handle_2) = match event {
      CollisionEvent::Started(h1, h2, _) => (true, h1, h2),
      CollisionEvent::Stopped(h1, h2, _) => (false, h1, h2),
    };
    let id_1 = id_from_collider(handle_1, &rigids, &colliders);
    let id_2 = id_from_collider(handle_2, &rigids, &colliders);
    let (player_id_maybe, sensor_id) =
      if id_1.is_none() { (id_1, id_2.unwrap()) } else if id_2.is_none() { (id_2, id_1.unwrap()) } 
      else if matches!(
        self.objects.get(id_1.unwrap()).unwrap().material, Material::Player
      ) { (id_1, id_2.unwrap()) } else { (id_2, id_1.unwrap()) };
    let sensor_type = self.objects.get(sensor_id).unwrap().material;
    match sensor_type {
      Material::Death => {
        if let Some(player_id) = player_id_maybe {
          let Material::Player = self.objects.get(player_id).unwrap().material else { unreachable!() };
          self.set_rapier_pos(player_id, rigids, IVec2::ZERO);
        }
      }
      Material::Button(channel, _) => {
        let button= self.objects.get_mut(sensor_id).unwrap();
        button.material.set_active(started);
        state_changes.entry(sensor_id)
          .or_insert(ObjectUpdate::new()).material(button.material);
        if let Some(player_id) = player_id_maybe {
          if started {
            self.players_on_button.insert(player_id, channel as usize);
          } else {
            self.players_on_button.remove(&player_id);
          }
        }
      }
      _ => unimplemented!()
    }
  }

  pub fn step_animations(&mut self, physics: &mut Physics) {
    for id in &self.animated.clone() {
      let object = self.objects.get_mut(*id).unwrap();
      if object.frozen { continue }
      let new_pos = object.animation.as_mut().unwrap().step();
      self.set_rapier_pos(*id, physics.body_sets().0, new_pos);
    }
  }

  fn register_movement(&mut self, rigids: &mut RigidBodySet, state_changes: &mut HashMap<usize, ObjectUpdate>) {
    for id in &self.animated {
      let new_pos = self.get_rapier_pos(*id, rigids);
      let old_pos = &mut self.objects.get_mut(*id).unwrap().position;
      if *old_pos != new_pos {
        state_changes.entry(*id)
          .or_insert(ObjectUpdate::new()).position(new_pos);
        *old_pos = new_pos;
      }
    }
    for id in &self.players {
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

impl Level {
  pub fn new(level: String, physics: &mut Physics) -> Self {
    physics.reset();
    let mut new = Self { 
      objects: Pond::new(),
      list: HashMap::new(),
      next: None,
      players: HashSet::new(),
      animated: HashSet::new(),
      events: Mutex::new(Vec::new()),
      players_on_button: HashMap::new(),
      button_requirements: [1; 16],
      receivers: HashSet::new(),
    };
    let json = std::fs::read_to_string(
      format!("{}/levels/{}", env!("CARGO_MANIFEST_DIR"), level)
    ).unwrap();
    let deser_level: InitialLevel = serde_json::from_str(&json).unwrap();
    new.button_requirements = deser_level.buttons;
    new.next = deser_level.next;
    for min_obj in deser_level.objects {
      let recievers = min_obj.receivers.clone();
      new.add_object(min_obj.full_rect(), recievers, physics, false);
    }
    new
  }

  pub fn delete(&mut self, id: usize, physics: &mut Physics) {
    self.objects.free(id);
    let handle = self.list.remove(&id).unwrap();
    self.players.remove(&id);
    self.animated.remove(&id);
    self.players_on_button.remove(&id);
    self.receivers.retain(|controller| controller.id != id);
    physics.remove(handle);
  }

  pub fn add_object(&mut self, object: Object, receivers: Vec<(Action, u8)>, physics: &mut Physics, player: bool) -> usize {
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
    if object.animation.is_some() { self.animated.insert(id); }
    if player { self.players.insert(id); }
    for (action, channel) in receivers {
      self.receivers.insert(RemoteControl {
        id,
        channel,
        action
      });
    }

    id
  }

  pub fn apply_vel(&mut self, rigids: &mut RigidBodySet, id: usize, velocity: IVec2) {
    if !self.players.contains(&id) { dbg!("Failed to add velocity to collider"); return; }
    let handle = self.list.get(&id).unwrap();
    let rb = rigids.get_mut(*handle).unwrap();
    let mass = rb.mass();
    let impulse = velocity.as_vec2() * mass;
    rb.apply_impulse(Vector2::new(impulse.x as f32, impulse.y as f32), true);
  }

  pub fn get_obj(&self, id: usize) -> Option<&Object> { self.objects.get(id) }

  fn get_rapier_pos(&self, id: usize, rigids: &RigidBodySet) -> IVec2 {
    let handle = self.list.get(&id).unwrap();
    let bad_pos = rigids.get(*handle).unwrap().position().translation;
    IVec2::new(bad_pos.x as i32, bad_pos.y as i32)
  }

  fn set_rapier_pos(&mut self, id: usize, rigids: &mut RigidBodySet, pos: IVec2) {
    let handle = self.list.get_mut(&id).unwrap();
    rigids.get_mut(*handle).unwrap().set_position(Translation::new(pos.x as f32, pos.y as f32).into(), true);
  }

}

fn id_from_collider(handle: ColliderHandle, rigids: &RigidBodySet, colliders: &ColliderSet) -> Option<usize> {
  let rbh = colliders.get(handle)?.parent().unwrap();
  Some(rigids.get(rbh).unwrap().user_data as usize)
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

