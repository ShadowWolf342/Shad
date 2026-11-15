//! Timeline primitives based on BBS Film replay keyframe sections.

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe<T> { pub tick:f32, pub value:T }
impl<T> Keyframe<T> { pub fn new(tick:f32,value:T)->Self { Self{tick,value} } }

/// A repeated source interval in a replay channel. Source-tick bounds are inclusive;
/// the repeated output interval is half-open.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeSection { pub start:f32, pub duration:f32, pub repeats:u16, pub accumulate:bool, pub channels:Vec<String> }
impl Default for KeyframeSection { fn default()->Self { Self{start:0.0,duration:1.0,repeats:2,accumulate:true,channels:Vec::new()} } }
impl KeyframeSection {
 pub const MAX_REPEATS:u16=256;
 pub fn new(start:f32,duration:f32,repeats:u16)->Self { Self{start,duration,repeats:repeats.clamp(1,Self::MAX_REPEATS),..Self::default()} }
 pub fn source_end(&self)->f32 { self.start+self.duration }
 pub fn output_end(&self)->f32 { self.output_end_for(self.repeats) }
 pub fn output_end_for(&self,repeats:u16)->f32 { self.start+self.duration*f32::from(repeats) }
 pub fn is_source_tick(&self,tick:f32)->bool { tick>=self.start && tick<=self.source_end() }
 pub fn contains_tick(&self,tick:f32)->bool { self.contains_tick_for(tick,self.repeats) }
 pub fn contains_tick_for(&self,tick:f32,repeats:u16)->bool { self.duration>0.0 && repeats>=1 && tick>=self.start && tick<self.output_end_for(repeats) }
 pub fn is_ghost_tick(&self,tick:f32)->bool { self.is_ghost_tick_for(tick,self.repeats) }
 pub fn is_ghost_tick_for(&self,tick:f32,repeats:u16)->bool { tick>=self.source_end() && tick<self.output_end_for(repeats) }
 pub fn has_channel(&self,channel:&str)->bool { self.channels.iter().any(|v|v==channel) }
 pub fn overlaps(&self,other:&Self)->bool { self.start<other.output_end() && other.start<self.output_end() }
 pub fn source_tick(&self,tick:f32)->Option<(f32,u16)> { if !self.contains_tick(tick)||self.duration<=0.0{return None;} let elapsed=tick-self.start;let repeat=(elapsed/self.duration).floor() as u16;Some((self.start+elapsed.rem_euclid(self.duration),repeat)) }
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum PlaybackState { #[default] Stopped, Playing, Paused }
#[derive(Clone,Debug,PartialEq)]
pub struct Playback { pub tick:f64,pub speed:f64,pub state:PlaybackState,pub looping:bool,pub duration:f64 }
impl Default for Playback { fn default()->Self { Self{tick:0.0,speed:1.0,state:PlaybackState::Stopped,looping:false,duration:0.0} } }
impl Playback {
 pub fn play(&mut self){self.state=PlaybackState::Playing;}
 pub fn pause(&mut self){if self.state==PlaybackState::Playing{self.state=PlaybackState::Paused;}}
 pub fn stop(&mut self){self.state=PlaybackState::Stopped;self.tick=0.0;}
 pub fn seek(&mut self,tick:f64){self.tick=tick.clamp(0.0,self.duration.max(0.0));}
 pub fn advance(&mut self,delta_seconds:f64,fps:f64){if self.state!=PlaybackState::Playing||!delta_seconds.is_finite()||!fps.is_finite()||fps<=0.0{return;} self.tick+=delta_seconds.max(0.0)*fps*self.speed;if self.duration>0.0&&self.tick>=self.duration{if self.looping{self.tick=self.tick.rem_euclid(self.duration);}else{self.tick=self.duration;self.state=PlaybackState::Stopped;}}else if self.tick<0.0{if self.looping&&self.duration>0.0{self.tick=self.tick.rem_euclid(self.duration);}else{self.tick=0.0;self.state=PlaybackState::Stopped;}}}
}

#[cfg(test)] mod tests { use super::*; #[test] fn bbs_section_boundaries(){let s=KeyframeSection::new(10.0,5.0,3);assert!(s.is_source_tick(15.0));assert!(s.contains_tick(10.0));assert!(s.contains_tick(24.999));assert!(!s.contains_tick(25.0));assert!(s.is_ghost_tick(15.0));assert!(!s.is_ghost_tick(10.0));} #[test] fn repeated_tick_maps_to_source(){let s=KeyframeSection::new(4.0,3.0,3);assert_eq!(s.source_tick(8.5),Some((5.5,1)));assert_eq!(s.source_tick(13.0),None);} #[test] fn playback_stops_or_wraps(){let mut p=Playback{duration:20.0,state:PlaybackState::Playing,..Playback::default()};p.advance(2.0,10.0);assert_eq!(p.tick,20.0);assert_eq!(p.state,PlaybackState::Stopped);p.looping=true;p.play();p.advance(2.5,10.0);assert_eq!(p.tick,5.0);} }
