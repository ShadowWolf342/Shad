//! Film and replay containers, following the high-level BBS object model.
use crate::{math::Transform,timeline::{Keyframe,KeyframeSection}};

#[derive(Clone,Debug,PartialEq)]
pub struct Marker { pub name:String,pub tick:f32,pub color:u32 }
impl Marker { pub fn new(name:impl Into<String>,tick:f32,color:u32)->Self{Self{name:name.into(),tick,color}} }
#[derive(Clone,Debug,PartialEq)]
pub struct Replay { pub name:String,pub duration:f32,pub sections:Vec<KeyframeSection>,pub position:Vec<Keyframe<Vec<Transform>>> }
impl Replay { pub fn new(name:impl Into<String>)->Self{Self{name:name.into(),duration:0.0,sections:Vec::new(),position:Vec::new()}} pub fn sort_keyframes(&mut self){self.position.sort_by(|a,b|a.tick.total_cmp(&b.tick));} }
#[derive(Clone,Debug,PartialEq)]
pub struct Film { pub format:u32,pub name:String,pub description:String,pub fps:f32,pub camera:Vec<Keyframe<Transform>>,pub replays:Vec<Replay>,pub markers:Vec<Marker> }
impl Default for Film { fn default()->Self{Self{format:1,name:"Untitled".into(),description:String::new(),fps:20.0,camera:Vec::new(),replays:Vec::new(),markers:Vec::new()}} }
impl Film { pub fn duration(&self)->f32{let camera=self.camera.last().map_or(0.0,|k|k.tick);let replay=self.replays.iter().map(|r|r.duration).fold(0.0,f32::max);let marker=self.markers.iter().map(|m|m.tick).fold(0.0,f32::max);camera.max(replay).max(marker)} pub fn add_marker(&mut self,marker:Marker){self.markers.push(marker);self.markers.sort_by(|a,b|a.tick.total_cmp(&b.tick));} }
#[cfg(test)] mod tests{use super::*;#[test]fn duration_tracks_all_sources(){let mut f=Film::default();f.camera.push(Keyframe::new(45.0,Transform::IDENTITY));f.replays.push(Replay{duration:80.0,..Replay::new("actor")});f.add_marker(Marker::new("cut",60.0,u32::MAX));assert_eq!(f.duration(),80.0);}}
