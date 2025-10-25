use interstellar::*;
use interstellar::math::{hex_rgb_u8, vec3};
use minifb::{Window, WindowOptions, Key, KeyRepeat};

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

    let mut center_x: f32 = 0.0; // pan horizontal (-1..1)
    let mut center_y: f32 = 0.0; // pan vertical (-1..1)
    let mut zoom: f32 = 1.0;     // 1.0 = sin zoom (mayor = más lejos)

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Permite cambiar entre planetas
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            active_shader = Body::Rocky;
        } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            active_shader = Body::GasGiant;
        } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            active_shader = Body::Ice;
        } else if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            active_shader = Body::AccretionDisk;
        } else if window.is_key_pressed(Key::Key5, KeyRepeat::No) {
            active_shader = Body::BlackHole;
        }

        // --- PAN ---
        let pan_step = 0.05 / zoom; // pan estable ajustado por zoom
        if window.is_key_down(Key::Left)  { center_x -= pan_step; }
        if window.is_key_down(Key::Right) { center_x += pan_step; }
        if window.is_key_down(Key::Up)    { center_y -= pan_step; }
        if window.is_key_down(Key::Down)  { center_y += pan_step; }

        // --- ZOOM ---
        if window.is_key_pressed(Key::Equal, KeyRepeat::Yes) { // '+' suele ser Shift+'='
            zoom /= 1.1; // acercar
        }
        if window.is_key_pressed(Key::Minus, KeyRepeat::Yes) {
            zoom *= 1.1; // alejar
        }
        if zoom < 0.3 { zoom = 0.3; }
        if zoom > 5.0 { zoom = 5.0; }

        // --- RESET ---
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            center_x = 0.0; center_y = 0.0; zoom = 1.0;
        }

        // Render
        for y in 0..height {
            for x in 0..width {
                // Coordenadas normalizadas (-1..1)
                let nx = (((x as f32 / width as f32) * 2.0 - 1.0) - center_x) / zoom;
                let ny = (((y as f32 / height as f32) * 2.0 - 1.0) - center_y) / zoom;

                // Mapeo simple a una esfera (planetita)
                let r2 = nx*nx + ny*ny;
                if r2 > 1.0 {
                    buffer[y * width + x] = 0; // fondo negro
                    continue;
                }

                let z = (1.0 - r2).sqrt(); // esfera
                let n = vec3(nx, ny, z).normalized();
                let v = vec3(0.0, 0.0, -1.0);
                let l0 = vec3(0.0, 0.15, 1.0).normalized();
                let l1 = vec3(0.0, 0.15, -1.0).normalized();

                let ctx = ShadingCtx {
                    p: n,
                    n,
                    v,
                    l0,
                    l1,
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