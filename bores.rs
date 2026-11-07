//! The practice chanters' bores as bore-tmm reads them, on stdout as one
//! JSON object: "cut", each chanter as its generator draws it (bore-tmm's
//! tools/read_cut_files.py writes them to bores/cut/), and "models",
//! chanter_model.rs's own design (tools/export_models.py writes it to
//! bores/models/).
//!
//!     target/release/chanter_bores
//!
//! A chanter's cut file cannot give its bore: the plate chanter's plates are
//! packed on the sheet by size, not in stack order, and a strip's holes do not
//! say which end is the reed's. So the chanters are read from the PARTS the
//! generators draw their sheets from -- each part's place down the body, each
//! hole's cut -- which the repository's .repro gate holds equal to the
//! committed SVGs. Sizes are the parts' own, before the kerf.
use crate::chanter_model::{self as m, plate, HOLES};
use crate::practice_chanters::{self as pc, pf, CPart};
use crate::standard_chanter::{self as s, Notes};
use crate::standard_octagon as so;
use crate::strip_chanter as sc;
use lasermade_tools::turn3d::{F, J};
use square_flute::Cut;
use std::f64::consts::PI;

pub fn n(x: f64) -> J {
    J::N(F(x))
}
pub fn st(x: &str) -> J {
    J::S(x.to_string())
}
pub fn strs(v: &[&str]) -> J {
    J::A(v.iter().map(|x| st(x)).collect())
}

/// svgparts' area of a closed polygon, as it sums it.
pub fn area(p: &[(f64, f64)]) -> f64 {
    let k = p.len();
    let mut sum = 0.0;
    for i in 0..k {
        let j = (i + k - 1) % k;
        sum += p[i].0 * p[j].1 - p[j].0 * p[i].1;
    }
    sum.abs() / 2.0
}

/// The round hole of that area's diameter.
pub fn eq_d(area: f64) -> f64 {
    (4.0 * area / PI).sqrt()
}

/// A hole cut: where it is along its part, and its round equivalent.
pub fn cut_d(c: &Cut) -> (f64, f64) {
    match c {
        Cut::Circle(x, _, r) => (*x, 2.0 * r),
        Cut::Poly(v) => {
            let pts = pf(v);
            let xs: Vec<f64> = pts.iter().map(|q| q.0).collect();
            let (lo, hi) = (xs.iter().copied().fold(f64::NAN, |a, b| if a.is_nan() || b < a { b } else { a }), xs.iter().copied().fold(f64::NAN, |a, b| if a.is_nan() || b > a { b } else { a }));
            ((lo + hi) / 2.0, eq_d(area(&pts)))
        }
    }
}

/// (mm down the bore, round equivalent, part name) of every hole cut in a
/// strip; top is the bore's top in the parts' coordinates.
pub fn strip_holes(parts: &[CPart], top: f64) -> Vec<(f64, f64, String)> {
    let mut found = vec![];
    for p in parts.iter().filter(|p| p.kind == "strip") {
        for c in &p.p.cuts {
            let (x, d) = cut_d(c);
            found.push((p.x + x - top, d, p.p.name.clone()));
        }
    }
    found
}

/// Name strip holes by the model's order down the bore.
pub fn by_place(mut found: Vec<(f64, f64, String)>, order: &[&'static str], what: &str) -> Vec<(&'static str, f64, f64)> {
    if found.len() != order.len() {
        lasermade_tools::die(format!("UNREADABLE {what}: {} holes cut in the strips, not {}", found.len(), order.len()));
    }
    found.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.partial_cmp(&b.1).unwrap()).then(a.2.cmp(&b.2)));
    order.iter().zip(found).map(|(h, (z, d, _))| (*h, z, d)).collect()
}

fn hole_j(h: &str, at: f64, d: f64, chimney: f64) -> J {
    J::obj(vec![("name", st(h)), ("at_mm", n(at)), ("d_mm", n(d)), ("chimney_mm", n(chimney))])
}

/// A chanter solved in standard_chanter.rs's model: REF_REED of closed bore
/// of the chanter's own section, an unflanged sole.
#[allow(clippy::too_many_arguments)]
fn standard_family(title: &str, src: &str, b: &s::SBore, l: f64, holes: &[(&str, f64, f64)], r: &Notes, chimney: f64, top_note: &str, whole_plates: bool) -> J {
    J::obj(vec![
        ("name", st(title)),
        ("source", st(&format!("{src}; section, wall, reed allowance and the reference's notes from standard_chanter.rs's model"))),
        ("temp_c", n(m::T_C)),
        ("drive", J::obj(vec![("kind", st("reed"))])),
        ("bore", J::A(vec![J::obj(vec![("len_mm", n(s::REF_REED + l)), ("d_mm", n(2000.0 * b.a)), ("loss_ratio", n(b.rh / b.a))])])),
        ("end", st("unflanged")),
        ("holes", J::A(holes.iter().map(|&(h, z, d)| hole_j(h, s::REF_REED + z, d, chimney)).collect())),
        ("targets", st(top_note)),
        (
            "notes",
            J::A(
                m::NOTES
                    .iter()
                    .map(|&nt| {
                        let mut kv = vec![("name", st(nt)), ("hz", n(r.get(nt))), ("open", strs(m::fingering(nt))), ("span", n(1.3))];
                        if whole_plates && nt == "low G" {
                            // low G has no hole of its own: on a plate stack the bore is a whole
                            // number of plates, the nearest, which leaves low G up to half a
                            // plate out, about 7 cents at 3mm
                            kv.push(("tol_cents", n(10.0)));
                            kv.push(("tol_why", st("the bore is whole plates, the nearest; low G has no hole to tune it")));
                        }
                        J::obj(kv)
                    })
                    .collect(),
            ),
        ),
    ])
}

fn cut() -> Vec<(String, J)> {
    let mut out = vec![];
    let down: Vec<&'static str> = HOLES.iter().rev().copied().collect();
    let r = s::reference();
    let targets = "the measured reference chanter's notes in the same model";
    let t = plate();
    // the standard strip chanter: square, round holes in the strips
    let (parts, info) = s::build("finger");
    let holes = by_place(strip_holes(&parts, info.seat_bottom), &down, "standard_chanter.rs");
    let sole = parts.iter().filter(|p| p.kind == "strip").map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + info.ls - info.seat_bottom;
    out.push((
        "practice-chanter-standard-strips".into(),
        standard_family(
            "practice chanter, standard strips (square bore), as drawn",
            "practice-chanters/standard_chanter.rs build(): strip holes and the sole",
            &s::SBore::new(s::SIDE, true),
            sole,
            &holes,
            &r,
            t,
            targets,
            false,
        ),
    ));
    // the standard octagon: slots, cut as stadiums in the front and back strips
    let (parts, info) = so::build();
    let holes = by_place(strip_holes(&parts, info.seat_bottom), &down, "standard_octagon.rs");
    let sole = parts.iter().filter(|p| p.kind == "strip").map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + info.ls - info.seat_bottom;
    out.push((
        "practice-chanter-standard-octagon".into(),
        standard_family(
            "practice chanter, standard octagon, as drawn",
            "practice-chanters/standard_octagon.rs build(): strip slots and the sole",
            &so::bore().0,
            sole,
            &holes,
            &r,
            t,
            targets,
            false,
        ),
    ));
    // the plate chanter: every plate's place in the stack, the hole plates'
    // slots or drills
    for nn in pc::BUILT {
        let (parts, info) = pc::build(nn);
        let seat = pc::N_SEAT as f64 * t;
        let body: Vec<&CPart> = parts.iter().filter(|p| ["bore", "slot", "drill"].contains(&p.kind)).collect();
        let sole = body.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + t - seat;
        let mut hp: Vec<&&CPart> = body.iter().filter(|p| p.kind != "bore").collect();
        hp.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        let holes: Vec<(&str, f64, f64)> = hp
            .iter()
            .map(|p| (p.finger.unwrap(), p.x + t / 2.0 - seat, if p.kind == "drill" { p.drill.unwrap() } else { eq_d(p.width * t) }))
            .collect();
        if holes.iter().map(|h| h.0).collect::<Vec<_>>() != down {
            lasermade_tools::die(format!("UNREADABLE practice_chanters.rs {nn} sides: hole plates out of order"));
        }
        let wall = pc::OUTER_FLATS / 2.0 - m::inradius(pc::bore_area(), nn);
        out.push((
            format!("practice-chanter-{nn}-sided"),
            standard_family(
                &format!("practice chanter, {nn}-sided plate stack, as drawn"),
                &format!("practice-chanters/practice_chanters.rs build({nn}): every plate's place, the hole plates' slots and drills"),
                &info.b,
                sole,
                &holes,
                &r,
                wall,
                targets,
                true,
            ),
        ));
    }
    // the strip chanter: chanter_model.rs's own design, a round top over the strips
    let (parts, info) = sc::build("finger");
    let top = info.x0 - info.n_round as f64 * t;
    let holes = by_place(strip_holes(&parts, top), &down, "strip_chanter.rs");
    let b = &info.b;
    let sole = info.x0 + info.ls - top;
    let reed = m::REED.get();
    out.push((
        "practice-chanter-strips".into(),
        J::obj(vec![
            ("name", st("practice chanter, square strips under a round top, as drawn")),
            ("source", st("practice-chanters/strip_chanter.rs build(): round top, strip holes and the sole; section and reed allowance from chanter_model.rs")),
            ("temp_c", n(m::T_C)),
            ("drive", J::obj(vec![("kind", st("reed"))])),
            (
                "bore",
                J::A(vec![
                    J::obj(vec![("len_mm", n(reed + info.n_round as f64 * t)), ("d_mm", n(info.round_d)), ("loss_ratio", n(1.0))]),
                    J::obj(vec![("len_mm", n(sole - info.n_round as f64 * t)), ("d_mm", n(2000.0 * b.a)), ("loss_ratio", n(b.rh / b.a))]),
                ]),
            ),
            ("end", st("flanged")),
            ("holes", J::A(holes.iter().map(|&(h, z, d)| hole_j(h, reed + z, d, b.wall)).collect())),
            (
                "notes",
                J::A(m::NOTES.iter().map(|&nt| J::obj(vec![("name", st(nt)), ("hz", n(m::target(nt))), ("open", strs(m::fingering(nt))), ("span", n(1.3))])).collect()),
            ),
        ]),
    ));
    out
}

fn models() -> Vec<(String, J)> {
    let mut out = vec![];
    for nn in pc::BUILT {
        let (b, l, pos, widths) = m::design(nn, 0.0, None).unwrap_or_else(|e| lasermade_tools::die(e));
        let a = 1000.0 * b.a;
        let reed = m::REED.get();
        // NOT the chanter practice_chanters builds: since 2026-10-23 that is
        // solved in standard_chanter.rs's model against a measured reference
        // (bores/cut/ has it as drawn). This is chanter_model.rs's own design,
        // kept because it is the model's, and strip_chanter.rs still uses it.
        out.push((
            format!("chanter-model-{nn}-sided"),
            J::obj(vec![
                ("name", st(&format!("chanter_model.rs's own design at {nn} sides (not the chanter built)"))),
                ("source", st(&format!("practice-chanters/chanter_model.rs design({nn})"))),
                ("temp_c", n(m::T_C)),
                ("drive", J::obj(vec![("kind", st("reed"))])),
                (
                    "bore",
                    J::A(vec![
                        J::obj(vec![("len_mm", n(reed)), ("d_mm", n(2.0 * a)), ("loss_ratio", n(1.0))]),
                        J::obj(vec![("len_mm", n(l)), ("d_mm", n(2.0 * a)), ("loss_ratio", n(1000.0 * b.rh / a))]),
                    ]),
                ),
                ("end", st("flanged")),
                ("holes", J::A((0..8).map(|i| hole_j(HOLES[i], reed + pos[i], 2000.0 * b.hole_r(widths[i]), b.wall)).collect())),
                (
                    "notes",
                    J::A(
                        m::NOTES
                            .iter()
                            .map(|&nt| {
                                let hz = m::sounding(&b, l, &pos, &widths, nt, None, 1.3).unwrap_or_else(|e| lasermade_tools::die(e));
                                J::obj(vec![("name", st(nt)), ("hz", n(m::target(nt))), ("open", strs(m::fingering(nt))), ("span", n(1.3)), ("model_hz", n(hz))])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ));
    }
    out
}

pub fn main() {
    let obj = |v: Vec<(String, J)>| J::O(v);
    let mut s = String::new();
    J::O(vec![("cut".into(), obj(cut())), ("models".into(), obj(models()))]).dump(&mut s);
    println!("{s}");
}
