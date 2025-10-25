use core::ops::{Add, AddAssign, Mul, Sub, Div, Neg};
pub const PI: f32 = core::f32::consts::PI;

#[derive(Copy, Clone, Debug, Default)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }
pub type Color = Vec3;

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn dot(self, o: Self) -> f32 { self.x*o.x + self.y*o.y + self.z*o.z }
    pub fn length(self) -> f32 { self.dot(self).sqrt() }
    pub fn normalized(self) -> Self { let l = self.length().max(1e-8); self / l }
    pub fn clamp01(self) -> Self { Self::new(self.x.clamp(0.0,1.0), self.y.clamp(0.0,1.0), self.z.clamp(0.0,1.0)) }
    pub fn mix(self, b: Self, k: f32) -> Self { self*(1.0-k) + b*k }
    pub fn mul_scalar(self, s: f32) -> Self { Self::new(self.x*s, self.y*s, self.z*s) }
}

impl Add for Vec3 { type Output = Self; fn add(self, o: Self) -> Self { Self::new(self.x+o.x, self.y+o.y, self.z+o.z) } }
impl AddAssign for Vec3 { fn add_assign(&mut self, o: Self) { *self = *self + o; } }
impl Sub for Vec3 { type Output = Self; fn sub(self, o: Self) -> Self { Self::new(self.x-o.x, self.y-o.y, self.z-o.z) } }
impl Mul for Vec3 { type Output = Self; fn mul(self, o: Self) -> Self { Self::new(self.x*o.x, self.y*o.y, self.z*o.z) } }
impl Mul<f32> for Vec3 { type Output = Self; fn mul(self, s: f32) -> Self { self.mul_scalar(s) } }
impl Div<f32> for Vec3 { type Output = Self; fn div(self, d: f32) -> Self { self.mul_scalar(1.0/d) } }

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 { v * self }
}

#[inline] pub fn saturate(x: f32) -> f32 { x.clamp(0.0, 1.0) }
#[inline] pub fn mix(a: f32, b: f32, k: f32) -> f32 { a*(1.0-k) + b*k }

#[inline] pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 { Vec3::new(x,y,z) }
#[inline] pub fn rgb(r: f32, g: f32, b: f32) -> Color { vec3(r,g,b) }

pub fn hex_rgb_u8(hex: &str) -> Color {
    // "#rrggbb"
    let h = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2],16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4],16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6],16).unwrap_or(255) as f32 / 255.0;
    rgb(r,g,b)
}

pub fn lat_lon_from_normal(n: Vec3) -> (f32, f32) {
    // lat in [0,1], lon in [0,1]
    let lat = 0.5 + n.y.asin()/PI;
    let lon = 0.5 + n.z.atan2(n.x)/(2.0*PI);
    (lat, lon)
}

pub fn rim_term(n: Vec3, v: Vec3, power: f32) -> f32 {
    (1.0 - n.dot(-v).clamp(-1.0, 1.0)).powf(power)
}