use crate::math::*;
use crate::noise::*;

#[derive(Copy, Clone)]
pub enum Body { BlackHole, AccretionDisk, Rocky, GasGiant, Ice }

#[derive(Copy, Clone)]
pub struct ShadingCtx {
    pub p: Vec3, // position in world
    pub n: Vec3, // normal (unit)
    pub v: Vec3, // view dir (unit, from point to camera)
    pub l0: Vec3, // primary light dir (from point to light) for "disk"
    pub l1: Vec3, // secondary "curved" light
    pub t: f32,  // time
    pub seed: f32,
}

#[derive(Copy, Clone)]
pub struct CommonParams {
    pub warm: Color,
    pub cool: Color,
}

#[derive(Copy, Clone)]
pub struct DiskParams {
    pub rin: f32, pub rout: f32,
    pub bands_w: f32, pub bands_phi: f32,
    pub noise_freq: f32, pub noise_amp: f32,
    pub beaming: f32,
    pub c1: Color, pub c2: Color, pub c3: Color,
}

#[derive(Copy, Clone)]
pub struct RockyParams {
    pub bioma_freq: f32,
    pub height_freq: f32,
    pub grad_amp: f32,
    pub k_atm: f32,
    pub c_land1: Color, pub c_land2: Color, pub c_ocean: Color,
}

#[derive(Copy, Clone)]
pub struct GasParams {
    pub k_bands: f32,
    pub dist_amp: f32,
    pub noise_freq: f32,
    pub storm_speed: f32,
    pub c_a: Color, pub c_b: Color, pub c_c: Color,
}

#[derive(Copy, Clone)]
pub struct IceParams {
    pub freq: f32,
    pub marbling: f32,
    pub c_ice: Color, pub c_snow: Color, pub c_crack: Color,
}

pub struct Params {
    pub common: CommonParams,
    pub disk: DiskParams,
    pub rocky: RockyParams,
    pub gas: GasParams,
    pub ice: IceParams,
}

#[inline]
fn palette3(u: f32, a: Color, b: Color, c: Color) -> Color {
    if u < 0.5 { a.mix(b, u*2.0) } else { b.mix(c, (u-0.5)*2.0) }
}

#[inline]
fn nl_mix(n: Vec3, l0: Vec3, l1: Vec3) -> f32 {
    let a = saturate(n.dot(l0));
    let b = saturate(n.dot(l1));
    0.7*a + 0.3*b
}

pub fn shade(ctx: &ShadingCtx, body: Body, params: &Params) -> Color {
    match body {
        Body::BlackHole    => shade_black_hole(ctx),
        Body::AccretionDisk=> shade_accretion(ctx, &params.disk),
        Body::Rocky        => shade_rocky(ctx, &params.common, &params.rocky),
        Body::GasGiant     => shade_gas_giant(ctx, &params.common, &params.gas),
        Body::Ice          => shade_ice(ctx, &params.common, &params.ice),
    }.clamp01()
}

fn shade_black_hole(ctx: &ShadingCtx) -> Color {
    // negro puro + halo suave en el borde (no físico, solo estético)
    let _r = ctx.p.length(); // si usas pantalla, puedes usar coord en viewspace
    // halo según rim con la vista
    let glow = rim_term(ctx.n, ctx.v, 3.5);
    rgb(0.06, 0.03, 0.10) * (glow*0.25) // leve morado
}

fn shade_accretion(ctx: &ShadingCtx, p: &DiskParams) -> Color {
    // asumimos disco en plano XZ: usa la posición (p) proyectada
    let r = (ctx.p.x*ctx.p.x + ctx.p.z*ctx.p.z).sqrt();
    let mut col = rgb(0.0,0.0,0.0);

    // 1) emisión radial (más caliente cerca del borde interno)
    let heat = ((-(r - p.rin)*3.0).exp()).clamp(0.0, 1.0);

    // 2) bandas radiales
    let bands = (p.bands_w*r + p.bands_phi).sin()*0.5 + 0.5;

    // 3) granulado + animación
    let rp = vec3(ctx.p.x, 0.0, ctx.p.z) * p.noise_freq + vec3(ctx.t*0.05, 0.0, ctx.t*0.05);
    let g = fbm3(rp, 4, 2.0, 0.5);
    let distort = (bands + p.noise_amp*(g-0.5)).clamp(0.0,1.0);

    // paleta cálida 3 tonos
    let warm = palette3(distort, p.c1, p.c2, p.c3);

    // 4) beaming falso (lado que viene hacia la cámara más brillante)
    let ndv = saturate(ctx.n.dot(-ctx.v));
    let beam = 0.6 + p.beaming * ndv.powf(3.0);

    // 5) apagar fuera del disco y recortar interior duro
    let inside = ((r - p.rin) / (p.rout - p.rin)).clamp(0.0, 1.0);
    let ring_mask = (1.0 - (1.0 - inside).powf(16.0)) * (1.0 - ((r - p.rout).max(0.0)).min(1.0));

    col = warm * (0.35 + 0.65*heat) * beam * ring_mask;
    col
}

fn shade_rocky(ctx: &ShadingCtx, common: &CommonParams, p: &RockyParams) -> Color {
    let (lat, lon) = lat_lon_from_normal(ctx.n);
    // 1) biomas base
    let k = fbm3(vec3(lat*p.bioma_freq, lon*p.bioma_freq, ctx.seed), 5, 2.0, 0.5);
    let mut base = palette3(k, p.c_land1, p.c_land2, p.c_ocean);

    // 2) altura sintética + sombreado falso
    let h = fbm3(vec3(lat*p.height_freq, lon*p.height_freq, ctx.seed+17.0), 4, 2.1, 0.5);
    let nl = nl_mix(ctx.n, ctx.l0, ctx.l1);
    let shade = 0.6 + 0.4 * (nl + 0.15*(h-0.5)).clamp(0.0,1.0);
    base = base * shade;

    // 3) montañas/nieves
    let peaks = ((h - 0.62)/0.08).clamp(0.0,1.0);
    let snow = hex_rgb_u8("#e6edf3");
    base = base.mix(snow, peaks);

    // 4) polos (latitud 0..1; polos cerca de 0 y 1)
    let pole_mask = ((lat-0.5).abs()-0.35);
    let pole = (1.0 - (pole_mask/0.15).clamp(0.0,1.0)).powf(2.0);
    base = base.mix(snow, 0.35*pole);

    // 5) atmósfera fina (rim)
    let rim = rim_term(ctx.n, ctx.v, 2.5);
    let atm = common.cool * (0.12*rim);
    (base + atm).clamp01()
}

fn shade_gas_giant(ctx: &ShadingCtx, common: &CommonParams, p: &GasParams) -> Color {
    let (mut lat, mut lon) = lat_lon_from_normal(ctx.n);

    // 1) bandas latitudinales
    let mut bands = (p.k_bands*lat*2.0*PI).sin()*0.5 + 0.5;

    // 2) distorsión por ruido (ondula límites)
    let d = fbm3(vec3(ctx.p.x*p.noise_freq, ctx.p.y*p.noise_freq, ctx.p.z*p.noise_freq), 4, 2.0, 0.5);
    lat = (lat + p.dist_amp*(d-0.5)).clamp(0.0,1.0);
    bands = (p.k_bands*lat*2.0*PI).sin()*0.5 + 0.5;

    // 3) tormentas / gran mancha (elipse en lat/lon)
    let storm_lon = (lon + ctx.t*p.storm_speed).fract();
    let center = (0.35, 0.15); // (lon, lat)
    let el = (((storm_lon-center.0+1.0).fract()-0.5)/0.12).powi(2) + ((lat-center.1)/0.08).powi(2);
    let storm = (1.0 - el).clamp(0.0,1.0).powf(3.0);

    // paleta
    let base = palette3(bands, p.c_a, p.c_b, p.c_c);
    let spot = hex_rgb_u8("#b24d2a"); // mancha rojiza
    let mut col = base.mix(spot, 0.6*storm);

    // 4) terminador tipo atmósfera espesa (rim + difuso del disco)
    let nl = nl_mix(ctx.n, ctx.l0, ctx.l1);
    col = col * (0.45 + 0.55*nl);
    let rim = rim_term(ctx.n, ctx.v, 2.8);
    col = col + common.warm * (0.10*rim);

    col.clamp01()
}

fn shade_ice(ctx: &ShadingCtx, common: &CommonParams, p: &IceParams) -> Color {
    let (lat, lon) = lat_lon_from_normal(ctx.n);
    let m = (lon*2.0*PI*p.freq + p.marbling*fbm3(vec3(lat*p.freq, lon*p.freq, ctx.seed), 4, 2.0, 0.5)).sin()*0.5 + 0.5;
    let cracks = ((m-0.65)/0.03).clamp(0.0,1.0);
    let mut col = p.c_ice.mix(p.c_snow, m);
    col = col.mix(p.c_crack, cracks);

    // luz del disco
    let nl = nl_mix(ctx.n, ctx.l0, ctx.l1);
    col = col * (0.5 + 0.5*nl);

    // aire frío en rim
    let rim = rim_term(ctx.n, ctx.v, 2.2);
    col = col + common.cool*(0.12*rim);
    col.clamp01()
}