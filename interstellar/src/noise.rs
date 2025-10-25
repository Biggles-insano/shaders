use crate::math::{Vec3, vec3, mix};

// simple hash/murmur-ish
pub fn hash31(p: [f32;3]) -> f32 {
    let mut x = p[0]*127.1 + p[1]*311.7 + p[2]*74.7;
    x = (x.sin()*43758.5453).fract();
    x
}

fn floor3(p: Vec3) -> Vec3 { vec3(p.x.floor(), p.y.floor(), p.z.floor()) }
fn fract3(p: Vec3) -> Vec3 { vec3(p.x.fract(), p.y.fract(), p.z.fract()) }

pub fn value_noise3(p: Vec3) -> f32 {
    let i = floor3(p);
    let f = fract3(p);
    let u = vec3( f.x*f.x*(3.0-2.0*f.x), f.y*f.y*(3.0-2.0*f.y), f.z*f.z*(3.0-2.0*f.z) );

    let h000 = hash31([i.x,   i.y,   i.z  ]);
    let h100 = hash31([i.x+1.0,i.y,  i.z  ]);
    let h010 = hash31([i.x,   i.y+1.0,i.z  ]);
    let h110 = hash31([i.x+1.0,i.y+1.0,i.z ]);
    let h001 = hash31([i.x,   i.y,   i.z+1.0]);
    let h101 = hash31([i.x+1.0,i.y,  i.z+1.0]);
    let h011 = hash31([i.x,   i.y+1.0,i.z+1.0]);
    let h111 = hash31([i.x+1.0,i.y+1.0,i.z+1.0]);

    let x00 = mix(h000, h100, u.x);
    let x10 = mix(h010, h110, u.x);
    let x01 = mix(h001, h101, u.x);
    let x11 = mix(h011, h111, u.x);

    let y0 = mix(x00, x10, u.y);
    let y1 = mix(x01, x11, u.y);

    mix(y0, y1, u.z)
}

pub fn fbm3(mut p: Vec3, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    let mut amp = 0.5;
    let mut sum = 0.0;
    for _ in 0..octaves {
        sum += amp * value_noise3(p);
        p = Vec3::new(p.x*lacunarity, p.y*lacunarity, p.z*lacunarity);
        amp *= gain;
    }
    sum
}