//! Baker et SDF-atlas av Inter til `assets/font/`, så teksten på skiltene
//! og knappene i byen er samme skrift som i Fristil-dokumentasjonen, og er
//! skarp uansett avstand.
//!
//!   cargo run --release --bin bake_font -- /usr/share/fonts/opentype/inter/Inter-Medium.otf
//!
//! Skriver `assets/font/inter.png` (8-bit, 0.5 er kanten) og `inter.json`
//! med mål per tegn i piksler ved `em` piksler per em.

use std::collections::BTreeMap;

const EM: f32 = 48.0;
const SPREAD: f32 = 8.0;
/// Hvor mange ganger større glyfene tegnes før avstanden regnes ut.
const OVERSAMPLE: usize = 8;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/usr/share/fonts/opentype/inter/Inter-Medium.otf".into());
    let data = std::fs::read(&path).expect("fant ikke skriftfila");
    let font = fontdue::Font::from_bytes(
        data,
        fontdue::FontSettings {
            scale: EM * OVERSAMPLE as f32,
            ..Default::default()
        },
    )
    .expect("ugyldig skrift");

    let mut chars: Vec<char> = (32u8..127).map(|c| c as char).collect();
    chars.extend("ÆØÅæøåÉéÜüÖöÄä«»–—…·×€©".chars());

    struct Glyph {
        ch: char,
        w: usize,
        h: usize,
        xoff: f32,
        yoff: f32,
        adv: f32,
        sdf: Vec<f32>,
    }
    let big = EM * OVERSAMPLE as f32;
    let pad = (SPREAD * OVERSAMPLE as f32).ceil() as usize + OVERSAMPLE;
    let mut glyphs = vec![];
    for &ch in &chars {
        let (m, bitmap) = font.rasterize(ch, big);
        if m.width == 0 || m.height == 0 {
            glyphs.push(Glyph {
                ch,
                w: 0,
                h: 0,
                xoff: 0.0,
                yoff: 0.0,
                adv: m.advance_width / OVERSAMPLE as f32,
                sdf: vec![],
            });
            continue;
        }
        // Dekning med luft rundt, så avstanden kan vokse ut forbi kanten.
        let bw = m.width + 2 * pad;
        let bh = m.height + 2 * pad;
        let mut inside = vec![false; bw * bh];
        for y in 0..m.height {
            for x in 0..m.width {
                inside[(y + pad) * bw + x + pad] = bitmap[y * m.width + x] >= 128;
            }
        }
        let d_out = edt(&inside, bw, bh, false);
        let d_in = edt(&inside, bw, bh, true);
        // Signert avstand i store piksler, positiv inne i glyfen.
        // Inne: avstand til nærmeste piksel utenfor. Ute: til nærmeste inne.
        let signed: Vec<f32> = (0..bw * bh)
            .map(|i| {
                if inside[i] {
                    d_out[i].sqrt()
                } else {
                    -d_in[i].sqrt()
                }
            })
            .collect();
        // Ned til målestokken i atlaset ved å ta snittet av hver blokk.
        let w = bw / OVERSAMPLE;
        let h = bh / OVERSAMPLE;
        let mut sdf = vec![0.0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                let mut s = 0.0;
                for yy in 0..OVERSAMPLE {
                    for xx in 0..OVERSAMPLE {
                        s += signed[(y * OVERSAMPLE + yy) * bw + x * OVERSAMPLE + xx];
                    }
                }
                sdf[y * w + x] = s / (OVERSAMPLE * OVERSAMPLE) as f32 / OVERSAMPLE as f32;
            }
        }
        let o = OVERSAMPLE as f32;
        glyphs.push(Glyph {
            ch,
            w,
            h,
            // Fra atlasrutas nedre venstre hjørne til grunnlinja og pennposisjonen.
            xoff: (m.xmin as f32 - pad as f32) / o,
            yoff: (m.ymin as f32 - pad as f32) / o,
            adv: m.advance_width / o,
            sdf,
        });
    }

    // Pakk i rader.
    let atlas_w = 1024usize;
    let (mut x, mut y, mut row_h) = (0usize, 0usize, 0usize);
    let mut placed = vec![];
    for g in &glyphs {
        if x + g.w > atlas_w {
            x = 0;
            y += row_h + 1;
            row_h = 0;
        }
        placed.push((x, y));
        x += g.w + 1;
        row_h = row_h.max(g.h);
    }
    let atlas_h = (y + row_h + 1).next_power_of_two();
    let mut pixels = vec![0u8; atlas_w * atlas_h];
    let mut meta = BTreeMap::new();
    for (g, (px, py)) in glyphs.iter().zip(&placed) {
        for yy in 0..g.h {
            for xx in 0..g.w {
                // Rad 0 er toppen av glyfen, både i fontdue og i atlaset.
                let v = g.sdf[yy * g.w + xx];
                let t = (0.5 + v / (2.0 * SPREAD)).clamp(0.0, 1.0);
                pixels[(py + yy) * atlas_w + px + xx] = (t * 255.0 + 0.5) as u8;
            }
        }
        meta.insert(
            g.ch.to_string(),
            serde_json::json!({
                "x": px, "y": py, "w": g.w, "h": g.h,
                "xoff": round(g.xoff), "yoff": round(g.yoff), "adv": round(g.adv),
            }),
        );
    }

    // Kerning for parene som faktisk forekommer i norsk tekst.
    let mut kern = BTreeMap::new();
    for &a in &chars {
        for &b in &chars {
            if let Some(k) = font.horizontal_kern(a, b, EM) {
                if k.abs() > 0.05 {
                    kern.insert(format!("{a}{b}"), round(k));
                }
            }
        }
    }
    let line = font.horizontal_line_metrics(EM).expect("linjemål");

    std::fs::create_dir_all("assets/font").unwrap();
    let file = std::fs::File::create("assets/font/inter.png").unwrap();
    let mut enc = png::Encoder::new(
        std::io::BufWriter::new(file),
        atlas_w as u32,
        atlas_h as u32,
    );
    enc.set_color(png::ColorType::Grayscale);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(&pixels)
        .unwrap();

    let json = serde_json::json!({
        "font": std::path::Path::new(&path).file_name().unwrap().to_string_lossy(),
        "em": EM,
        "spread": SPREAD,
        "atlas": [atlas_w, atlas_h],
        "ascent": round(line.ascent),
        "descent": round(line.descent),
        "glyphs": meta,
        "kern": kern,
    });
    std::fs::write(
        "assets/font/inter.json",
        serde_json::to_string(&json).unwrap(),
    )
    .unwrap();
    println!(
        "{} tegn i {}×{} atlas, {} kerningpar",
        glyphs.len(),
        atlas_w,
        atlas_h,
        json["kern"].as_object().unwrap().len()
    );
}

fn round(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

/// Kvadrert euklidsk avstand fra hver piksel til nærmeste piksel som er
/// `target` (Felzenszwalb og Huttenlocher), først langs radene, så kolonnene.
fn edt(inside: &[bool], w: usize, h: usize, target: bool) -> Vec<f32> {
    const INF: f32 = 1e12;
    let mut f = vec![INF; w * h];
    for i in 0..w * h {
        if inside[i] == target {
            f[i] = 0.0;
        }
    }
    let mut tmp = vec![0.0f32; w.max(h)];
    let mut v = vec![0usize; w.max(h)];
    let mut z = vec![0.0f32; w.max(h) + 1];
    let mut row = vec![0.0f32; w.max(h)];
    for y in 0..h {
        row[..w].copy_from_slice(&f[y * w..y * w + w]);
        dt1d(&row[..w], &mut tmp[..w], &mut v, &mut z);
        f[y * w..y * w + w].copy_from_slice(&tmp[..w]);
    }
    for x in 0..w {
        for y in 0..h {
            row[y] = f[y * w + x];
        }
        dt1d(&row[..h], &mut tmp[..h], &mut v, &mut z);
        for y in 0..h {
            f[y * w + x] = tmp[y];
        }
    }
    f
}

fn dt1d(f: &[f32], d: &mut [f32], v: &mut [usize], z: &mut [f32]) {
    let n = f.len();
    let mut k = 0usize;
    v[0] = 0;
    z[0] = -1e20;
    z[1] = 1e20;
    for q in 1..n {
        loop {
            let vk = v[k] as f32;
            let s = ((f[q] + (q * q) as f32) - (f[v[k]] + vk * vk)) / (2.0 * q as f32 - 2.0 * vk);
            if s <= z[k] {
                if k == 0 {
                    // Kan ikke gå lenger tilbake; erstatt.
                    v[0] = q;
                    z[0] = -1e20;
                    z[1] = 1e20;
                    break;
                }
                k -= 1;
            } else {
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = 1e20;
                break;
            }
        }
    }
    k = 0;
    for q in 0..n {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let dq = q as f32 - v[k] as f32;
        d[q] = dq * dq + f[v[k]];
    }
}
