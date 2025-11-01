use interstellar::*;
use interstellar::math::{vec3, hex_rgb_u8};
use minifb::{Window, WindowOptions, Key, KeyRepeat};

fn rotate_y(v: (f32, f32, f32), yaw: f32) -> (f32, f32, f32) {
    let (x,y,z) = v;
    let cy = yaw.cos();
    let sy = yaw.sin();
    ( x*cy + z*sy, y, -x*sy + z*cy )
}

fn rotate_x(v: (f32, f32, f32), pitch: f32) -> (f32, f32, f32) {
    let (x,y,z) = v;
    let cp = pitch.cos();
    let sp = pitch.sin();
    ( x, y*cp - z*sp, y*sp + z*cp )
}

fn main() {
    // --- Window setup ---
    let width: usize = 800;
    let height: usize = 600;
    let mut __window__ = Window::new("Interstellar Renderer 🚀", width, height, WindowOptions::default())
        .expect("No se pudo crear la ventana");
    __window__.set_target_fps(60);

    // --- Framebuffer ---
    let mut __buffer__: Vec<u32> = vec![0; width * height];

    // --- Global params for shaders ---
    let params = Params {
        common: CommonParams {
            warm: hex_rgb_u8("#ffb347"),
            cool: hex_rgb_u8("#8bb6ff"),
        },
        disk: DiskParams {
            rin: 1.2,
            rout: 5.0,
            bands_w: 22.0,
            bands_phi: 0.3,
            noise_freq: 2.8,
            noise_amp: 0.08,
            beaming: 0.4,
            c1: hex_rgb_u8("#ff9a00"),
            c2: hex_rgb_u8("#ffd65c"),
            c3: hex_rgb_u8("#fff3e0"),
        },
        rocky: RockyParams {
            bioma_freq: 7.0,
            height_freq: 8.0,
            grad_amp: 0.35,
            k_atm: 0.15,
            c_land1: hex_rgb_u8("#6b4f2a"),
            c_land2: hex_rgb_u8("#9db36b"),
            c_ocean: hex_rgb_u8("#1c3b6b"),
        },
        gas: GasParams {
            k_bands: 16.0,
            dist_amp: 0.06,
            noise_freq: 3.0,
            storm_speed: 0.12,
            c_a: hex_rgb_u8("#f0e1c2"),
            c_b: hex_rgb_u8("#d9a066"),
            c_c: hex_rgb_u8("#9b6b43"),
        },
        ice: IceParams {
            freq: 10.0,
            marbling: 1.6,
            c_ice: hex_rgb_u8("#9fd0ff"),
            c_snow: hex_rgb_u8("#e6f4ff"),
            c_crack: hex_rgb_u8("#284a73"),
        },
    };

    // --- Time & state ---
    let mut __t__: f32 = 0.0;
    let mut __active_shader__ = Body::Rocky;
    let mut __mode_ringed__: bool = false;

    // --- Camera orbit state - MEJOR POSICIÓN INICIAL PARA VER ANILLOS ---
    let mut __yaw__: f32 = 0.0;
    let mut __pitch__: f32 = 0.6;    // Más inclinado para ver anillos
    let mut __radius__: f32 = 4.0;   // Más lejos para ver todo
    let mut __zoom__: f32 = 1.4;     // Zoom para dejar espacio visible al anillo

    while __window__.is_open() && !__window__.is_key_down(Key::Escape) {
        // --- Switch shaders ---
        if __window__.is_key_pressed(Key::Key1, KeyRepeat::No) {
            __active_shader__ = Body::Rocky; __mode_ringed__ = false;
        } else if __window__.is_key_pressed(Key::Key2, KeyRepeat::No) {
            __active_shader__ = Body::GasGiant; __mode_ringed__ = false;
        } else if __window__.is_key_pressed(Key::Key3, KeyRepeat::No) {
            __active_shader__ = Body::Ice; __mode_ringed__ = false;
        } else if __window__.is_key_pressed(Key::Key4, KeyRepeat::No) {
            __active_shader__ = Body::GasGiant; __mode_ringed__ = true;
            if __zoom__ < 1.2 { __zoom__ = 1.2; } // asegurar espacio para ver el anillo
        } else if __window__.is_key_pressed(Key::Key5, KeyRepeat::No) {
            __active_shader__ = Body::BlackHole; __mode_ringed__ = false;
        }

        // --- Orbit controls ---
        let orbit_speed = 1.2 * 0.016;
        if __window__.is_key_down(Key::Left)  { __yaw__   -= orbit_speed; }
        if __window__.is_key_down(Key::Right) { __yaw__   += orbit_speed; }
        if __window__.is_key_down(Key::Up)    { __pitch__ -= orbit_speed; }
        if __window__.is_key_down(Key::Down)  { __pitch__ += orbit_speed; }

        let max_pitch = 1.3;
        if __pitch__ >  max_pitch { __pitch__ =  max_pitch; }
        if __pitch__ < -max_pitch { __pitch__ = -max_pitch; }

        // --- Zoom controls (Z/X) ---
        if __window__.is_key_pressed(Key::Z, KeyRepeat::Yes) { __zoom__ /= 1.1; }
        if __window__.is_key_pressed(Key::X, KeyRepeat::Yes) { __zoom__ *= 1.1; }
        if __zoom__ < 0.3 { __zoom__ = 0.3; }
        if __zoom__ > 5.0 { __zoom__ = 5.0; }

        // --- Reset ---
        if __window__.is_key_pressed(Key::R, KeyRepeat::No) {
            __yaw__ = 0.0; __pitch__ = 0.6; __zoom__ = 0.8; __radius__ = 4.0;
        }

        // --- Camera position in world (once per frame) ---
        let cam_dir_frame = {
            let d = (0.0, 0.0, __radius__);
            let d = rotate_x(d, __pitch__);
            rotate_y(d, __yaw__)
        };
        let cam_pos_frame = vec3(cam_dir_frame.0, cam_dir_frame.1, cam_dir_frame.2);

        // --- Render ---
        let aspect = width as f32 / height as f32;
        for y in 0..height {
            for x in 0..width {
                let nx = (((x as f32 / width as f32) * 2.0 - 1.0)) / __zoom__;
                let ny = ((((y as f32 / height as f32) * 2.0 - 1.0)) / __zoom__) / aspect;

                // --- Black Hole (Body::BlackHole) ---
                if matches!(__active_shader__, Body::BlackHole) {
                    // Screen-space radius
                    let r = (nx*nx + ny*ny).sqrt();
                    let rh = 0.42;      // event horizon radius
                    let ring_w = 0.06;  // photon ring width

                    // Photon ring intensity around rh
                    let dr = r - (rh + ring_w * 0.5);
                    let glow = (-(dr * dr) / (0.12 * 0.12)).exp();

                    // Accretion glow (procedural), brighter near the equatorial plane (y≈0)
                    let theta = ny.atan2(nx);
                    let swirl = (10.0 * theta + 3.0 * __t__).sin() * 0.5 + 0.5;
                    let equator = (1.0 - (ny * ny * 2.0).min(1.0)).max(0.0);
                    let acc = (equator * 0.8) * (0.5 + 0.5 * swirl);

                    let c1 = hex_rgb_u8("#ff9a00");
                    let c2 = hex_rgb_u8("#ffd65c");
                    let c3 = hex_rgb_u8("#fff3e0");

                    // Base accretion color
                    let mut col = c1.mix(c2, acc);
                    // Add photon ring highlight
                    col = col.mix(c3, (glow * 0.6).min(1.0));

                    // Apply horizon mask: inside rh → black
                    if r < rh { col = vec3(0.0, 0.0, 0.0); }

                    // Vignette to fade to black at far edges
                    let vign = (1.0 - ((r - 0.9) / 0.9).max(0.0)).max(0.0);
                    col = col * vign.max(0.0);

                    let rr = (col.x.max(0.0).min(1.0) * 255.0) as u32;
                    let gg = (col.y.max(0.0).min(1.0) * 255.0) as u32;
                    let bb = (col.z.max(0.0).min(1.0) * 255.0) as u32;
                    __buffer__[y * width + x] = (rr << 16) | (gg << 8) | bb;
                    continue;
                }

                // Rings — anillo estrecho que rodea el planeta (y=0)
                if __mode_ringed__ && !matches!(__active_shader__, Body::BlackHole) {
                    // Dirección del rayo desde la cámara hacia este píxel (en vista)
                    let d_view = (nx, ny, 1.0);
                    // Rotar a mundo con la orientación de la cámara
                    let d_world = {
                        let tmp = rotate_x(d_view, __pitch__);
                        let tmp = rotate_y(tmp, __yaw__);
                        let v = vec3(tmp.0, tmp.1, tmp.2).normalized();
                        (v.x, v.y, v.z)
                    };

                    // Intersección con plano horizontal y=0 (plano del anillo)
                    let eps = 1e-5;
                    if d_world.1.abs() > eps {
                        let t_hit = -cam_pos_frame.y / d_world.1; // cam + t*d → y=0
                        if t_hit > 0.0 {
                            let hit_x = cam_pos_frame.x + t_hit * d_world.0;
                            let hit_z = cam_pos_frame.z + t_hit * d_world.2;
                            let r_ring = (hit_x*hit_x + hit_z*hit_z).sqrt();

                            // Un solo anillo más ancho y visible
                            let rin  = 1.1;   // más cerca del planeta
                            let rout = 1.6;   // anillo más grande y notorio

                            if r_ring >= rin && r_ring <= rout {
                                // Color uniforme para un solo anillo
                                let bands = 0.5;

                                // Paleta sobria para anillos
                                let c1 = hex_rgb_u8("#e8dcc8"); // crema
                                let c2 = hex_rgb_u8("#b9a994"); // beige/gris
                                let c = c1.mix(c2, bands);

                                // (opcional) leve atenuación radial hacia el borde
                                let radial = ((r_ring - rin) / (rout - rin)).min(1.0).max(0.0);
                                let c = c * (1.0 - 0.15 * radial);

                                let rr = (c.x.max(0.0).min(1.0) * 255.0) as u32;
                                let gg = (c.y.max(0.0).min(1.0) * 255.0) as u32;
                                let bb = (c.z.max(0.0).min(1.0) * 255.0) as u32;
                                __buffer__[y * width + x] = (rr << 16) | (gg << 8) | bb;
                            } else {
                                __buffer__[y * width + x] = 0;
                            }
                        } else {
                            __buffer__[y * width + x] = 0;
                        }
                    } else {
                        __buffer__[y * width + x] = 0;
                    }
                }

                // Sphere mask
                let r2 = nx*nx + ny*ny;
                if r2 > 1.0 {
                    if !__mode_ringed__ { __buffer__[y * width + x] = 0; }
                    continue;
                }

                let z_view = (1.0 - r2).sqrt();
                let n_view = (nx, ny, z_view);

                let n_world = {
                    let tmp = rotate_y(n_view, -__yaw__);
                    let tmp = rotate_x(tmp, -__pitch__);
                    vec3(tmp.0, tmp.1, tmp.2).normalized()
                };

                let p_world = n_world;

                let v_world = (cam_pos_frame - p_world).normalized();
                let l0_world = vec3(0.0, 0.15,  1.0).normalized();
                let l1_world = vec3(0.0, 0.15, -1.0).normalized();

                let ctx = ShadingCtx { p: p_world, n: n_world, v: v_world, l0: l0_world, l1: l1_world, t: __t__, seed: 0.5 };

                let color = shade(&ctx, __active_shader__, &params);

                // Aplicar transparencia si es GasGiant sin anillos
                let (final_r, final_g, final_b) = if matches!(__active_shader__, Body::GasGiant) && !__mode_ringed__ {
                    let alpha = 0.7;
                    let bg_color = 0.0;
                    let r_blend = color.x * alpha + bg_color * (1.0 - alpha);
                    let g_blend = color.y * alpha + bg_color * (1.0 - alpha);
                    let b_blend = color.z * alpha + bg_color * (1.0 - alpha);
                    (
                        (r_blend.max(0.0).min(1.0) * 255.0) as u32,
                        (g_blend.max(0.0).min(1.0) * 255.0) as u32,
                        (b_blend.max(0.0).min(1.0) * 255.0) as u32,
                    )
                } else {
                    (
                        (color.x.max(0.0).min(1.0) * 255.0) as u32,
                        (color.y.max(0.0).min(1.0) * 255.0) as u32,
                        (color.z.max(0.0).min(1.0) * 255.0) as u32,
                    )
                };

                __buffer__[y * width + x] = (final_r << 16) | (final_g << 8) | final_b;
            }
        }

        __window__.update_with_buffer(&__buffer__, width, height).unwrap();
        __t__ += 0.01;
    }
}