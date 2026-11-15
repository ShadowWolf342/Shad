//! Small math types used by film and camera data.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    pub fn length(self) -> f64 { (self.x*self.x + self.y*self.y + self.z*self.z).sqrt() }
    pub fn lerp(self, rhs: Self, t: f64) -> Self { Self::new(self.x+(rhs.x-self.x)*t, self.y+(rhs.y-self.y)*t, self.z+(rhs.z-self.z)*t) }
}
impl std::ops::Add for Vec3 { type Output=Self; fn add(self,rhs:Self)->Self { Self::new(self.x+rhs.x,self.y+rhs.y,self.z+rhs.z) } }
impl std::ops::Sub for Vec3 { type Output=Self; fn sub(self,rhs:Self)->Self { Self::new(self.x-rhs.x,self.y-rhs.y,self.z-rhs.z) } }
impl std::ops::Mul<f64> for Vec3 { type Output=Self; fn mul(self,rhs:f64)->Self { Self::new(self.x*rhs,self.y*rhs,self.z*rhs) } }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform { pub position: Vec3, pub rotation: Vec3, pub scale: Vec3 }
impl Default for Transform { fn default()->Self { Self::IDENTITY } }
impl Transform { pub const IDENTITY:Self=Self { position:Vec3::ZERO, rotation:Vec3::ZERO, scale:Vec3{x:1.0,y:1.0,z:1.0} }; }

#[cfg(test)] mod tests { use super::*; #[test] fn vector_lerp_and_length(){let a=Vec3::new(0.0,2.0,4.0);let b=Vec3::new(10.0,4.0,8.0);assert_eq!(a.lerp(b,0.5),Vec3::new(5.0,3.0,6.0));assert_eq!(Vec3::new(3.0,4.0,0.0).length(),5.0);} }
