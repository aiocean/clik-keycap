//! `keycap`: a keyboard key with a lighter top face, a darker bottom edge and a centered label.

use clik_plugin::v1;

#[cfg(target_family = "wasm")]
mod guest {
    use clik_plugin::{Output, Plugin, v1};

    struct Keycap;

    impl Plugin for Keycap {
        fn run(step: &str, _args: &[String], _input: v1::Input) -> Result<Output, String> {
            Err(format!("unknown step: {step}"))
        }

        fn render(kind: &str, input: v1::RenderInput) -> Result<v1::RenderOutput, String> {
            match kind {
                "keycap" => Ok(super::keycap(&input)),
                other => Err(format!("unknown shape kind: {other}")),
            }
        }
    }

    clik_plugin::export!(Keycap);
}

/// clik lays plugin text out in a box `size * LINE_HEIGHT` tall from `y`
/// (`canvas-core::bounds::LINE_HEIGHT`).
const LINE_HEIGHT: f32 = 1.35;

struct Theme {
    /// The key body; its bottom band shows below the face as the key's edge.
    base: u32,
    face: u32,
    outline: u32,
    label: u32,
}

const LIGHT: Theme = Theme {
    base: 0xC4C7CEFF,
    face: 0xFBFBFCFF,
    outline: 0x9EA2AAFF,
    label: 0x2C2C2EFF,
};

const DARK: Theme = Theme {
    base: 0x161618FF,
    face: 0x3A3A3CFF,
    outline: 0x000000FF,
    label: 0xF2F2F7FF,
};

fn keycap(input: &v1::RenderInput) -> v1::RenderOutput {
    let (w, h) = (input.width.max(1.0), input.height.max(1.0));
    let theme = match input.props.get("theme") {
        Some(v1::PropValue::Enum(t)) if t == "dark" => &DARK,
        _ => &LIGHT,
    };
    let label = match input.props.get("label") {
        Some(v1::PropValue::Text(t)) => t.as_str(),
        _ => "⌘K",
    };
    let size = match input.props.get("size") {
        Some(v1::PropValue::Number(n)) => (*n as f32).clamp(10.0, 48.0),
        _ => 16.0,
    };

    // Edge depth and side inset scale with the key, within limits that read as a key.
    let depth = (h * 0.12).clamp(2.0, 8.0);
    let inset = (depth * 0.4).max(1.5);
    let radius = (w.min(h) * 0.18).clamp(2.0, 10.0);
    let face = Rect {
        x: inset,
        y: inset * 0.5,
        w: (w - 2.0 * inset).max(0.0),
        h: (h - inset * 0.5 - depth).max(0.0),
    };
    let half = 0.5; // keeps the 1 px outline inside the box
    let body = Rect {
        x: half,
        y: half,
        w: w - 2.0 * half,
        h: h - 2.0 * half,
    };

    let mut primitives = vec![
        v1::Primitive::Path {
            segs: rounded_rect(&body, radius),
            fill: Some(theme.base),
            stroke: Some(v1::StrokeSpec {
                width: 1.0,
                color: theme.outline,
                dash: None,
            }),
        },
        v1::Primitive::Path {
            segs: rounded_rect(&face, (radius - inset).max(1.5)),
            fill: Some(theme.face),
            stroke: None,
        },
    ];
    if !label.is_empty() {
        primitives.push(centered_text(label, &face, size, theme.label));
    }
    v1::RenderOutput { primitives }
}

struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// The label centered on `face`, its size shrunk to fit the face. clik aligns plugin text to
/// the start of its box, so the line is centered from an estimated width; the box runs to the
/// face's right padding, so an underestimate never wraps the line.
fn centered_text(label: &str, face: &Rect, size: f32, color: u32) -> v1::Primitive {
    let pad = (face.h * 0.15).clamp(2.0, 8.0);
    let room = (face.w - 2.0 * pad).max(1.0);
    let em = estimate_em(label).max(0.1);
    let size = size.min(room / em).min(face.h / LINE_HEIGHT).max(1.0);
    let text_w = em * size;
    let x = (face.x + (face.w - text_w) / 2.0).max(face.x + pad);
    v1::Primitive::Text {
        x,
        y: face.y + (face.h - size * LINE_HEIGHT) / 2.0,
        width: (face.x + face.w - pad - x).max(1.0),
        text: label.to_string(),
        font: "sans".into(),
        size,
        color,
    }
}

/// Width of `text` in em for the system sans font, close enough to center a short label.
fn estimate_em(text: &str) -> f32 {
    text.chars()
        .map(|c| match c {
            ' ' => 0.28,
            'i' | 'j' | 'l' | 'I' | '.' | ',' | ':' | ';' | '!' | '|' | '\'' => 0.28,
            'm' | 'w' | 'M' | 'W' => 0.85,
            'A'..='Z' => 0.66,
            'a'..='z' | '0'..='9' => 0.56,
            c if c.is_ascii() => 0.6,
            // Modifier glyphs (⌘ ⌥ ⇧ ⌃ ↩ ⌫) and other symbols are about a square.
            _ => 0.92,
        })
        .sum()
}

/// A closed rounded rectangle; the radius is clamped to half the shorter side.
fn rounded_rect(r: &Rect, radius: f32) -> Vec<v1::Seg> {
    let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    let k = rad * 0.552_284_8;
    let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
    vec![
        v1::Seg::Move(x0 + rad, y0),
        v1::Seg::Line(x1 - rad, y0),
        v1::Seg::Cubic(x1 - rad + k, y0, x1, y0 + rad - k, x1, y0 + rad),
        v1::Seg::Line(x1, y1 - rad),
        v1::Seg::Cubic(x1, y1 - rad + k, x1 - rad + k, y1, x1 - rad, y1),
        v1::Seg::Line(x0 + rad, y1),
        v1::Seg::Cubic(x0 + rad - k, y1, x0, y1 - rad + k, x0, y1 - rad),
        v1::Seg::Line(x0, y0 + rad),
        v1::Seg::Cubic(x0, y0 + rad - k, x0 + rad - k, y0, x0 + rad, y0),
        v1::Seg::Close,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use clik_plugin::testing;
    use std::collections::BTreeMap;

    fn render(w: f32, h: f32, props: &[(&str, v1::PropValue)]) -> v1::RenderOutput {
        let props: BTreeMap<_, _> = props
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        keycap(&testing::render_input(w, h, props))
    }

    /// (x, y, width, size) of the label.
    fn text(out: &v1::RenderOutput) -> Option<(f32, f32, f32, f32)> {
        out.primitives.iter().find_map(|p| match p {
            v1::Primitive::Text {
                x, y, width, size, ..
            } => Some((*x, *y, *width, *size)),
            _ => None,
        })
    }

    fn fills(out: &v1::RenderOutput) -> Vec<u32> {
        out.primitives
            .iter()
            .filter_map(|p| match p {
                v1::Primitive::Path { fill, .. } => *fill,
                _ => None,
            })
            .collect()
    }

    #[test]
    fn label_stays_inside_the_face_across_sizes() {
        for (w, h) in [
            (40.0, 40.0),
            (60.0, 40.0),
            (120.0, 40.0),
            (300.0, 80.0),
            (40.0, 80.0),
        ] {
            for size in [10.0, 16.0, 48.0] {
                let out = render(w, h, &[("size", v1::PropValue::Number(size))]);
                let (x, y, width, s) = text(&out).unwrap_or_default();
                assert!(
                    x >= 0.0 && x + width <= w,
                    "x out of box at {w}x{h} size {size}"
                );
                assert!(
                    y >= 0.0 && y + s * LINE_HEIGHT <= h,
                    "y out of box at {w}x{h} size {size}"
                );
                assert!(s <= size as f32 && s >= 1.0);
                // The estimated line fits in the text box, so it never wraps.
                assert!(
                    estimate_em("⌘K") * s <= width + 0.01,
                    "wraps at {w}x{h} size {size}"
                );
            }
        }
    }

    #[test]
    fn label_is_centered_on_a_wide_key() {
        let out = render(300.0, 80.0, &[]);
        let (x, _, _, s) = text(&out).unwrap_or_default();
        let center = x + estimate_em("⌘K") * s / 2.0;
        assert!((center - 150.0).abs() < 0.5, "center {center}");
        assert_eq!(s, 16.0);
    }

    #[test]
    fn face_is_lighter_than_base_in_both_themes() {
        for theme in ["light", "dark"] {
            let out = render(120.0, 40.0, &[("theme", v1::PropValue::Enum(theme.into()))]);
            let f = fills(&out);
            assert_eq!(f.len(), 2);
            let luma = |c: u32| (c >> 24) + ((c >> 16) & 0xFF) + ((c >> 8) & 0xFF);
            assert!(luma(f[1]) > luma(f[0]), "{theme}: face not lighter");
        }
    }

    #[test]
    fn empty_label_draws_only_the_key() {
        let out = render(40.0, 40.0, &[("label", v1::PropValue::Text(String::new()))]);
        assert_eq!(out.primitives.len(), 2);
        assert!(text(&out).is_none());
    }

    #[test]
    fn degenerate_box_stays_finite() {
        let out = render(0.0, 0.0, &[]);
        for p in &out.primitives {
            if let v1::Primitive::Path { segs, .. } = p {
                for s in segs {
                    if let v1::Seg::Cubic(a, b, c, d, e, f) = s {
                        assert!([a, b, c, d, e, f].iter().all(|v| v.is_finite()));
                    }
                }
            }
        }
    }
}
