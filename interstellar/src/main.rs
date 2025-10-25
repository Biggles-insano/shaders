use interstellar::*;
use interstellar::math::{hex_rgb_u8, vec3};
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
    // Dimensiones
    let width = 800;
    let height = 600;

    // Crear ventana
    let mut window = Window::new("Interstellar Renderer 🚀", width, height, WindowOptions::default())
        .expect("No se pudo crear la ventana");
    window.set_target_fps(60);

    // Framebuffer
    let mut buffer: Vec<u32> = vec![0; width * height];

    // Configurar parámetros globales
    let params = Params {
        common: CommonParams {
            warm: hex_rgb_u8("#ffb347"),
            cool: hex_rgb_u8("#8bb6ff"),
        },
        disk: DiskParams {
            rin: 1.2, rout: 5.0,
            bands_w: 22.0, bands_phi: 0.3,
            noise_freq: 2.8, noise_amp: 0.08,
            beaming: 0.4,
            c1: hex_rgb_u8("#ff9a00"),
            c2: hex_rgb_u8("#ffd65c"),
            c3: hex_rgb_u8("#fff3e0"),
        },
        rocky: RockyParams {
            bioma_freq: 7.0, height_freq: 8.0, grad_amp: 0.35, k_atm: 0.15,
            c_land1: hex_rgb_u8("#6b4f2a"),
            c_land2: hex_rgb_u8("#9db36b"),
            c_ocean: hex_rgb_u8("#1c3b6b"),
        },
        gas: GasParams {
            k_bands: 16.0, dist_amp: 0.06, noise_freq: 3.0, storm_speed: 0.12,
            c_a: hex_rgb_u8("#f0e1c2"),
            c_b: hex_rgb_u8("#d9a066"),
            c_c: hex_rgb_u8("#9b6b43"),
        },
        ice: IceParams {
            freq: 10.0, marbling: 1.6,
            c_ice: hex_rgb_u8("#9fd0ff"),
            c_snow: hex_rgb_u8("#e6f4ff"),
            c_crack: hex_rgb_u8("#284a73"),
        },
    };

    let mut t: f32 = 0.0;
    let mut active_shader = Body::Rocky;
    // Modo especial: planeta con anillos
    let mut mode_ringed: bool = false;

    // Cámara en órbita alrededor del planeta
    let mut yaw: f32 = 0.0;      // giro horizontal (rad)
    let mut pitch: f32 = 0.0;    // giro vertical (rad)
    let mut radius: f32 = 3.0;   // distancia de la cámara al centro (para v_world)
    let mut zoom: f32 = 1.0;     // escala de pantalla (1 = sin zoom)

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Permite cambiar entre planetas
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            active_shader = Body::Rocky;
            mode_ringed = false;
        } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            active_shader = Body::GasGiant; // ← planeta gaseoso
            mode_ringed = false;
        } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            active_shader = Body::Ice;
            mode_ringed = false;
        } else if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            // Planeta con anillos: base gaseoso + anillos renderizados en pantalla
            active_shader = Body::GasGiant;
            mode_ringed = true;
        } else if window.is_key_pressed(Key::Key5, KeyRepeat::No) {
            active_shader = Body::BlackHole;
            mode_ringed = false;
        }

        // --- ÓRBITA (cámara gira alrededor del planeta) ---
        let orbit_speed = 1.2 * 0.016; // ~rad/s (aprox 60 FPS)
        if window.is_key_down(Key::Left)  { yaw   -= orbit_speed; }
        if window.is_key_down(Key::Right) { yaw   += orbit_speed; }
        if window.is_key_down(Key::Up)    { pitch -= orbit_speed; }
        if window.is_key_down(Key::Down)  { pitch += orbit_speed; }
        // limita pitch para no volcar la cámara
        let max_pitch = 1.3; // ~75°
        if pitch >  max_pitch { pitch =  max_pitch; }
        if pitch < -max_pitch { pitch = -max_pitch; }

        // --- ZOOM (nuevas teclas: Z = in, X = out) ---
        if window.is_key_pressed(Key::Z, KeyRepeat::Yes) {
            zoom /= 1.1; // acercar
        }
        if window.is_key_pressed(Key::X, KeyRepeat::Yes) {
            zoom *= 1.1; // alejar
        }
        if zoom < 0.3 { zoom = 0.3; }
        if zoom > 5.0 { zoom = 5.0; }

        // --- RESET ---
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            yaw = 0.0; pitch = 0.0; zoom = 1.0; radius = 3.0;
        }

        // Render
        // Posición de cámara en mundo (según yaw/pitch/radius), una vez por frame
let cam_dir_frame = {
    let d = (0.0, 0.0, radius);
    let d = rotate_x(d, pitch);
    rotate_y(d, yaw)
};
let cam_pos_frame = vec3(cam_dir_frame.0, cam_dir_frame.1, cam_dir_frame.2);
        for y in 0..height {
            for x in 0..width {
                // Coordenadas normalizadas (-1..1) con zoom
                let nx = (((x as f32 / width as f32) * 2.0 - 1.0)) / zoom;
                let ny = (((y as f32 / height as f32) * 2.0 - 1.0)) / zoom;

                let r2 = nx*nx + ny*ny;

                // Dibujo de anillos (en pantalla) cuando está activo el modo con anillos
               // Dibujo de anillos (ray-plane) cuando está activo el modo con anillos
if mode_ringed {
    // Dirección del rayo desde la cámara hacia este píxel (en vista)
    let d_view = (nx, ny, 1.0);
    // Rotar a mundo con la orientación de la cámara
    let d_world = {
        let tmp = rotate_x(d_view, pitch);
        let tmp = rotate_y(tmp, yaw);
        let v = vec3(tmp.0, tmp.1, tmp.2).normalized();
        (v.x, v.y, v.z)
    };

    // Intersección con el plano horizontal y = 0 (plano de anillos)
    let eps = 1e-5;
    if d_world.1.abs() > eps {
        let t_hit = -cam_pos_frame.y / d_world.1; // cam + t*d → y=0
        if t_hit > 0.0 {
            let hit_x = cam_pos_frame.x + t_hit * d_world.0;
            let hit_z = cam_pos_frame.z + t_hit * d_world.2;
            let r_ring = (hit_x*hit_x + hit_z*hit_z).sqrt();

            // Anillos más separados del planeta
            let rin  = 1.25;
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

                // Si el punto está fuera de la esfera, pintar fondo (a menos que ya pintamos anillos)
                if r2 > 1.0 {
                    if !mode_ringed { buffer[y * width + x] = 0; }
                    continue;
                }
                let z_view = (1.0 - r2).sqrt();
                // normal en vista
                let n_view = (nx, ny, z_view);

                // Transformar normal de VISTA a MUNDO usando la inversa de la rotación de cámara
                // Inversa de R_y(yaw)*R_x(pitch) es R_x(-pitch)*R_y(-yaw)
                let n_world = {
                    let tmp = rotate_y(n_view, -yaw);
                    let tmp = rotate_x(tmp, -pitch);
                    vec3(tmp.0, tmp.1, tmp.2).normalized()
                };

                // Punto sobre la esfera en mundo (centro en origen)
                let p_world = n_world; // para esfera unitaria

                // Posición de cámara en mundo a partir de yaw/pitch/radius
                let cam_dir = {
                    // dirección desde origen hacia cámara
                    let d = (0.0, 0.0, radius);
                    let d = rotate_x(d, pitch);
                    rotate_y(d, yaw)
                };
                let cam_pos = vec3(cam_dir.0, cam_dir.1, cam_dir.2);

                // Vector hacia la cámara desde el punto
                let v_world = (cam_pos_frame - p_world).normalized();

                // Luces en mundo (fijas respecto al disco de acreción)
                let l0_world = vec3(0.0, 0.15, 1.0).normalized();
                let l1_world = vec3(0.0, 0.15, -1.0).normalized();

                let ctx = ShadingCtx {
                    p: p_world,
                    n: n_world,
                    v: v_world,
                    l0: l0_world,
                    l1: l1_world,
                    t,
                    seed: 0.5,
                };

                let color = shade(&ctx, active_shader, &params);
                let r = (color.x * 255.0) as u32;
                let g = (color.y * 255.0) as u32;
                let b = (color.z * 255.0) as u32;
                buffer[y * width + x] = (r << 16) | (g << 8) | b;
            }
        }

        // Mostrar en pantalla
        window.update_with_buffer(&buffer, width, height).unwrap();
        t += 0.01;
    }
}