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
    let mut window = Window::new("Interstellar Renderer 🚀", width, height, WindowOptions::default())
        .expect("No se pudo crear la ventana");
    window.set_target_fps(60);

    // --- Framebuffer ---
    let mut buffer: Vec<u32> = vec![0; width * height];

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
    let mut t: f32 = 0.0;
    let mut active_shader = Body::Rocky;  // 1: Rocky, 2: Gas, 3: Ice, 4: Gas+Rings, 5: BlackHole
    let mut mode_ringed: bool = false;    // rings toggle for key 4

    // --- Camera orbit state ---
    let mut yaw: f32 = 0.0;      // horizontal
    let mut pitch: f32 = 0.35;   // vertical (slightly above to see horizontal rings)
    let mut radius: f32 = 3.0;   // distance from origin
    let mut zoom: f32 = 1.0;     // screen scale

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // --- Switch shaders ---
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            active_shader = Body::Rocky; mode_ringed = false;
        } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            active_shader = Body::GasGiant; mode_ringed = false;
        } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            active_shader = Body::Ice; mode_ringed = false;
        } else if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            active_shader = Body::GasGiant; mode_ringed = true; // gas giant + rings
        } else if window.is_key_pressed(Key::Key5, KeyRepeat::No) {
            active_shader = Body::BlackHole; mode_ringed = false;
        }

        // --- Orbit controls ---
        let orbit_speed = 1.2 * 0.016; // ~ per frame @60fps
        if window.is_key_down(Key::Left)  { yaw   -= orbit_speed; }
        if window.is_key_down(Key::Right) { yaw   += orbit_speed; }
        if window.is_key_down(Key::Up)    { pitch -= orbit_speed; }
        if window.is_key_down(Key::Down)  { pitch += orbit_speed; }
        let max_pitch = 1.3; // ~75 deg
        if pitch >  max_pitch { pitch =  max_pitch; }
        if pitch < -max_pitch { pitch = -max_pitch; }

        // --- Zoom controls (Z/X) ---
        if window.is_key_pressed(Key::Z, KeyRepeat::Yes) { zoom /= 1.1; }
        if window.is_key_pressed(Key::X, KeyRepeat::Yes) { zoom *= 1.1; }
        if zoom < 0.3 { zoom = 0.3; }
        if zoom > 5.0 { zoom = 5.0; }

        // --- Reset ---
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            yaw = 0.0; pitch = 0.35; zoom = 1.0; radius = 3.0;
        }

        // --- Camera position in world (once per frame) ---
        let cam_dir_frame = {
            let d = (0.0, 0.0, radius);
            let d = rotate_x(d, pitch);
            rotate_y(d, yaw)
        };
        let cam_pos_frame = vec3(cam_dir_frame.0, cam_dir_frame.1, cam_dir_frame.2);

        // --- Render ---
        let aspect = width as f32 / height as f32;
        for y in 0..height {
            for x in 0..width {
                // normalized device coords with aspect & zoom
                let nx = (((x as f32 / width as f32) * 2.0 - 1.0)) / zoom;
                let ny = ((((y as f32 / height as f32) * 2.0 - 1.0)) / zoom) / aspect;

                // Rings (ray-plane against y=0) BEFORE the sphere so planet can occlude them
                if mode_ringed {
                    // ray direction in view space
                    let d_view = (nx, ny, 1.0);
                    // rotate to world
                    let d_world = {
                        let tmp = rotate_x(d_view, pitch);
                        let tmp = rotate_y(tmp, yaw);
                        let v = vec3(tmp.0, tmp.1, tmp.2).normalized();
                        (v.x, v.y, v.z)
                    };
                    let eps = 1e-5;
                    if d_world.1.abs() > eps { // avoid parallel
                        let t_hit = -cam_pos_frame.y / d_world.1; // y=0 plane
                        if t_hit > 0.0 {
                            let hit_x = cam_pos_frame.x + t_hit * d_world.0;
                            let hit_z = cam_pos_frame.z + t_hit * d_world.2;
                            let r_ring = (hit_x*hit_x + hit_z*hit_z).sqrt();
                            let rin  = 1.25; // more separation from planet
                            let rout = 1.65;
                            if r_ring >= rin && r_ring <= rout {
                                let k = 32.0;
                                let bands = (k * r_ring + t * 0.12).sin() * 0.5 + 0.5;
                                let c1 = hex_rgb_u8("#e8dcc8");
                                let c2 = hex_rgb_u8("#b9a994");
                                let c = c1.mix(c2, bands);
                                let rr = (c.x * 255.0) as u32;
                                let gg = (c.y * 255.0) as u32;
                                let bb = (c.z * 255.0) as u32;
                                buffer[y * width + x] = (rr << 16) | (gg << 8) | bb;
                            } else {
                                buffer[y * width + x] = 0;
                            }
                        } else {
                            buffer[y * width + x] = 0;
                        }
                    } else {
                        buffer[y * width + x] = 0;
                    }
                }

                // Sphere mask (unit sphere in view)
                let r2 = nx*nx + ny*ny;
                if r2 > 1.0 {
                    if !mode_ringed { buffer[y * width + x] = 0; }
                    continue;
                }

                // view normal of the sphere
                let z_view = (1.0 - r2).sqrt();
                let n_view = (nx, ny, z_view);
                // transform normal from view->world using inverse camera rotation
                let n_world = {
                    let tmp = rotate_y(n_view, -yaw);
                    let tmp = rotate_x(tmp, -pitch);
                    vec3(tmp.0, tmp.1, tmp.2).normalized()
                };
                let p_world = n_world; // point on unit sphere

                // camera vector & lights
                let v_world = (cam_pos_frame - p_world).normalized();
                let l0_world = vec3(0.0, 0.15,  1.0).normalized();
                let l1_world = vec3(0.0, 0.15, -1.0).normalized();

                let ctx = ShadingCtx { p: p_world, n: n_world, v: v_world, l0: l0_world, l1: l1_world, t, seed: 0.5 };
                let color = shade(&ctx, active_shader, &params);

                let r = (color.x.max(0.0).min(1.0) * 255.0) as u32;
                let g = (color.y.max(0.0).min(1.0) * 255.0) as u32;
                let b = (color.z.max(0.0).min(1.0) * 255.0) as u32;
                buffer[y * width + x] = (r << 16) | (g << 8) | b;
            }
        }

        window.update_with_buffer(&buffer, width, height).unwrap();
        t += 0.01;
    }
}