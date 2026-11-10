//! Cut files for the standard chanter on eight sides: a Great Highland
//! practice chanter with the measured chanter's holes, where they are, on an
//! octagonal bore of eight long strips of ply, finger-jointed and glued, the
//! holes cut as slots.
//!
//!     target/release/standard_octagon --no-write              # the checks
//!     target/release/standard_octagon --out=DIR               # the sheet
//!     target/release/standard_octagon --plate=2.9 ...         # for ply as measured
//!     target/release/standard_octagon --kerf=0.18 ...         # for that kerf
//!
//! Round each slot its strip owns both edges, MIN_WEB past it, and every
//! strip round from it gives up its near edge to own its far one. With eight
//! strips and eight edges that leaves the strip opposite owning neither
//! there, narrowed to FACE_OUT - 2 CUT for the slot's length: the back strip
//! opposite the finger holes, the front strip opposite the thumb's.
//!
//! The model is standard_chanter.rs's, the reed's allowance taken as more of
//! this chanter's own bore, as the square's is, so the two are alike.
//!
//! From the sole up: the FOOT, N_FOOT round plates with the octagon through
//! them, slid onto the strips' bottom and glued flush with the sole; the
//! STRIPS into the collar's socket plate; then the collar, tenon, cap and
//! mouthpiece, standard_chanter.rs's.
use crate::chanter_model::{self as m, plate, HOLES};
use crate::practice_chanters::{self as pc, page_part, plate_m, CPart, Checks, Parts, Sheet, C, CAPS, P2};
use crate::standard_chanter as s;
use crate::strip_chanter::{self as sc, Run};
use lasermade_tools::die;
use lasermade_tools::py::{self, g};
use lasermade_tools::turn3d::{Mat, F, I, J};
use square_flute as sf;
use std::f64::consts::PI;

pub const N: usize = 8;
pub const FLATS: f64 = 8.0; // the octagon across its flats, inside
pub fn tan() -> f64 {
    py::mtan(PI / N as f64)
}
pub const SLOT_MAX: f64 = 12.0; // the longest slot, along: under a finger's pad
pub const FOOT_D: f64 = s::FOOT_D;
pub const N_FOOT: usize = s::N_FOOT;
pub const SOCK_D: f64 = s::REF_CAP_D; // the collar's socket plate, flush with the cap
pub const WALLS: [&str; 8] = ["front", "front left", "left", "back left", "back", "back right", "right", "front right"];

/// face_in, face_out, cut, groove, slot_w
pub fn dims() -> (f64, f64, f64, f64, f64) {
    let t = plate();
    let face_in = FLATS * tan();
    let face_out = (FLATS + 2.0 * t) * tan();
    let cut = t * py::msin(py::radians(135.0));
    let groove = cut - t * tan();
    let slot_w = face_out - 2.0 * pc::MIN_WEB;
    (face_in, face_out, cut, groove, slot_w)
}

/// The octagon, with its eight corner grooves: (the bore, its area in mm^2).
pub fn bore() -> (s::SBore, f64) {
    let mut b = s::SBore::new(FLATS, true);
    let gr = dims().3;
    let area = N as f64 * tan() * py::powf(FLATS / 2.0, 2.0) + N as f64 * gr * gr / 2.0;
    b.area = area / 1e6;
    b.a = (b.area / PI).sqrt();
    (b, area)
}

/// The slot SLOT_W across with a d round hole's area; d itself if a round
/// hole that size fits.
pub fn slot_len(d: f64) -> f64 {
    let w = dims().4;
    if d <= w {
        return d;
    }
    let area = PI * d * d / 4.0;
    (area - PI * w * w / 4.0) / w + w
}

pub fn solve() -> (s::Notes, s::Notes, f64, Vec<s::Hole>, s::SBore, f64) {
    let r = s::reference();
    let t = plate();
    let (b, area) = bore();
    let w = dims().4;
    let mut holes: Vec<s::Hole> = s::REF_HOLES.iter().map(|&(h, z, d, _)| s::Hole { name: h, z: s::REF_TOP - z, d, wall: t }).collect();
    let d_max = 2.0 * ((w * (SLOT_MAX - w) + PI * w * w / 4.0) / PI).sqrt();
    let mut l = s::REF_TOP;
    for _ in 0..3 {
        l = s::bisect(&mut |x| s::sounding(&b, x, &holes, "low G", r.get("low G")), r.get("low G"), s::REF_TOP - 20.0, s::REF_TOP + 40.0, false);
        s::size_holes(&b, l, &mut holes, &r, d_max, &|h| format!("error: {h} is flat even as a {}mm slot.", g(SLOT_MAX)));
    }
    let got = s::Notes(m::NOTES.map(|nt| s::sounding(&b, l, &holes, nt, r.get(nt))));
    (r, got, l, holes, b, area)
}

/// Each edge's owner along the strips. Edge k is between strip k and k + 1.
pub fn owners(ls: f64, slots: &[(usize, f64, f64)]) -> Vec<Vec<Run<usize>>> {
    let n = py::round(ls / sc::FINGER).max(1) as usize;
    let mut cuts: Vec<f64> = (0..=n).map(|i| ls * i as f64 / n as f64).collect();
    let guard: Vec<(f64, f64, usize)> = slots.iter().map(|&(k, x, ln)| (x - ln / 2.0 - pc::MIN_WEB, x + ln / 2.0 + pc::MIN_WEB, k)).collect();
    for &(a, b, _) in &guard {
        cuts.push(a);
        cuts.push(b);
    }
    cuts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cuts.dedup();
    (0..N)
        .map(|e| {
            let iv = cuts
                .windows(2)
                .map(|w| {
                    let mid = (w[0] + w[1]) / 2.0;
                    let i = ((mid / ls * n as f64) as usize).min(n - 1);
                    let mut who = if i % 2 == 0 { e } else { (e + 1) % N };
                    let h = guard.iter().find(|&&(a, b, _)| a <= mid && mid <= b).map(|x| x.2);
                    if let Some(h) = h {
                        let j = (e + N - h) % N;
                        who = if j < N / 2 { (h + j) % N } else { (e + 1) % N };
                    }
                    (w[0], w[1], who, h.is_some())
                })
                .collect();
            sc::tidy(iv)
        })
        .collect()
}

/// A slot centred (cx, cy), `along` long in x and `across` wide in y.
pub fn stadium(cx: f64, cy: f64, along: f64, across: f64, steps: usize) -> Vec<P2> {
    let (r, s) = (across / 2.0, (along - across) / 2.0);
    let mut pts: Vec<P2> = (0..=steps)
        .map(|k| {
            let a = -PI / 2.0 + PI * k as f64 / steps as f64;
            (cx + s + r * py::mcos(a), cy + r * py::msin(a))
        })
        .collect();
    pts.extend((0..=steps).map(|k| {
        let a = PI / 2.0 + PI * k as f64 / steps as f64;
        (cx - s + r * py::mcos(a), cy + r * py::msin(a))
    }));
    let mut out: Vec<P2> = vec![];
    for p in pts {
        if out.last().is_none_or(|l| py::hypot(p.0 - l.0, p.1 - l.1) > 1e-9) {
            out.push(p);
        }
    }
    if py::hypot(out[0].0 - out[out.len() - 1].0, out[0].1 - out[out.len() - 1].1) < 1e-9 {
        out.pop();
    }
    out
}

pub struct Built {
    pub r: s::Notes,
    pub got: s::Notes,
    pub l: f64,
    pub holes: Vec<s::Hole>,
    pub area: f64,
    pub t: f64,
    pub ls: f64,
    pub x0: f64,
    pub own: Vec<Vec<Run<usize>>>,
    pub slots: Vec<(usize, f64, f64, f64)>,
    pub seat_bottom: f64,
    pub pins: Vec<P2>,
    pub n_cap: usize,
    pub sock: Vec<P2>,
    pub face_in: f64,
    pub face_out: f64,
    pub cut: f64,
    pub groove: f64,
    pub slot_w: f64,
}

pub fn build() -> (Vec<CPart>, Built) {
    let (r, got, l, holes, _b, area) = solve();
    let t = plate();
    let (face_in, face_out, cut, groove, slot_w) = dims();
    let mut parts = Parts::default();
    let x_collar = s::N_SEAT as f64 * t;
    let x0 = x_collar + s::N_COLLAR as f64 * t;
    let seat_bottom = s::N_SEAT as f64 * t;
    let ls = l - (x0 - seat_bottom);
    let slots: Vec<(usize, f64, f64, f64)> =
        holes.iter().map(|h| (if h.name == "hA" { 4 } else { 0 }, h.z - (x0 - seat_bottom), h.d, slot_len(h.d))).collect();
    let own = owners(ls, &slots.iter().map(|&(k, x, _, ln)| (k, x, ln)).collect::<Vec<_>>());
    for k in 0..N {
        let lo: Vec<(f64, f64, bool)> = own[k].iter().map(|&(a, c, w, _)| (a, c, w == k)).collect();
        let hi: Vec<(f64, f64, bool)> = own[(k + N - 1) % N].iter().map(|&(a, c, w, _)| (a, c, w == k)).collect();
        let o = sc::strip_outline(&lo, &hi, face_out - 2.0 * t, t, Some(cut));
        let cuts: Vec<C> =
            slots.iter().filter(|q| q.0 == k).map(|&(_, x, d, ln)| C::Poly(stadium(x, face_out / 2.0, ln, py::min(slot_w, d), 16))).collect();
        let q = parts.add(&format!("{} strip", WALLS[k]), &o, cuts, x0, "strip", &format!("the {} strip", WALLS[k]), None);
        q.strip_k = k;
    }
    let sock = pc::ngon(N, FLATS / 2.0 + t, 0.0);
    for k in 0..N_FOOT {
        parts.add(
            "foot",
            &pc::circle(0.0, 0.0, FOOT_D / 2.0, 72),
            vec![C::Poly(sock.clone())],
            x0 + ls - (k + 1) as f64 * t,
            "foot",
            &format!("foot plate {}", k + 1),
            Some(FOOT_D / 2.0),
        );
    }
    let (pins, n_cap) = s::head(&mut parts, x_collar, x0, vec![C::Poly(sock.clone())], SOCK_D);
    (
        parts.v,
        Built { r, got, l, holes, area, t, ls, x0, own, slots, seat_bottom, pins, n_cap, sock, face_in, face_out, cut, groove, slot_w },
    )
}

/// Each strip's outside width along it: [(x0, x1, width, guarded)].
pub fn widths(info: &Built) -> Vec<Vec<(f64, f64, f64, bool)>> {
    let (own, ls, w, c) = (&info.own, info.ls, info.face_out, info.cut);
    (0..N)
        .map(|k| {
            let (lo, hi) = (&own[k], &own[(k + N - 1) % N]);
            let mut xs: Vec<f64> = lo.iter().chain(hi.iter()).map(|r| r.0).collect();
            xs.push(ls);
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            xs.dedup();
            xs.windows(2)
                .map(|ab| {
                    let (a, e) = (ab[0], ab[1]);
                    let mm = (a + e) / 2.0;
                    let rl = lo.iter().find(|r| r.0 <= mm && mm <= r.1).unwrap();
                    let rh = hi.iter().find(|r| r.0 <= mm && mm <= r.1).unwrap();
                    let wd = w - c * ((rl.2 != k) as u8 as f64) - c * ((rh.2 != k) as u8 as f64);
                    (a, e, wd, rl.3 || rh.3)
                })
                .collect()
        })
        .collect()
}

fn checks(parts: &[CPart], info: &Built, sheets: &[Sheet]) -> Checks {
    let mut out: Checks = vec![];
    let mut note = |ok: bool, what: &str, detail: String| out.push((ok, what.to_string(), detail));
    let (t, ls) = (info.t, info.ls);
    let worst = s::worst_cents(&info.got, &info.r);
    note(
        worst <= 1.0,
        "every note sounds where the reference's does, in the model",
        format!("within {worst:.2} cents; the reference's low A {:.0}Hz in the model", info.r.get("low A")),
    );
    let moved = sf::fmax(&(0..8).map(|i| ((s::REF_TOP - s::REF_HOLES[i].1) - info.holes[i].z).abs()).collect::<Vec<_>>());
    note(
        moved < 1e-9,
        "every hole is where the reference's is",
        (0..8).map(|i| format!("{} {:.1}", HOLES[i], s::REF_TOP - s::REF_HOLES[i].1)).collect::<Vec<_>>().join(", ") + "mm under the seat",
    );
    let ref_area = PI * py::powf(s::REF_BORE, 2.0) / 4.0;
    let oct_area = N as f64 * tan() * py::powf(FLATS / 2.0, 2.0);
    note(
        (info.area - oct_area - N as f64 * py::powf(info.groove, 2.0) / 2.0).abs() < 1e-9,
        "the bore is the octagon and its corner grooves",
        format!(
            "{}mm across the flats, {oct_area:.1}mm^2 and {N} grooves of {:.2}mm, {:.1}mm^2: {:.1} times the reference's {}mm round",
            g(FLATS),
            info.groove,
            info.area,
            info.area / ref_area,
            g(s::REF_BORE)
        ),
    );
    let sl = &info.slots;
    let (w, ww, fin) = (info.slot_w, info.face_out, info.face_in);
    note(
        sl.iter().all(|q| q.3 <= SLOT_MAX + 1e-9) && w <= fin - 0.35 + 1e-9,
        "every slot opens into its face, and fits under a finger",
        (0..8).map(|i| format!("{} {:.1}", HOLES[i], sl[i].3)).collect::<Vec<_>>().join(", ")
            + &format!("mm long, {w:.2}mm across, the face {fin:.2}mm inside; {}mm at most", g(SLOT_MAX)),
    );
    let mut ok = true;
    for &(k, x, _, ln) in sl {
        for e in [k, (k + N - 1) % N] {
            for &(a, b, who, _) in &info.own[e] {
                if a < x + ln / 2.0 + pc::MIN_WEB && b > x - ln / 2.0 - pc::MIN_WEB && who != k {
                    ok = false;
                }
            }
        }
    }
    let side = (ww - w) / 2.0;
    note(
        ok && side >= pc::MIN_WEB - 1e-9,
        "every slot keeps ply all round it",
        format!("its strip owns both edges, {}mm past it along; {side:.2}mm each side", g(pc::MIN_WEB)),
    );
    let own = &info.own;
    let runs: Vec<f64> = own.iter().flatten().filter(|r| !r.3).map(|r| r.1 - r.0).collect();
    note(
        s::covered(own, ls) && sf::fmin(&runs) >= sc::MIN_FINGER - 1e-9,
        "every corner is one strip's, all the way",
        format!(
            "{} fingers on {N} corners; the shortest unguarded {:.1}mm, against {}",
            own.iter().map(|r| r.len()).sum::<usize>(),
            sf::fmin(&runs),
            g(sc::MIN_FINGER)
        ),
    );
    let ws = widths(info);
    let narrow: Vec<(usize, f64, f64, f64, bool)> = (0..N)
        .flat_map(|k| ws[k].iter().filter(|r| r.2 < ww - info.cut - 1e-9).map(move |&(a, b, wd, gd)| (k, a, b, wd, gd)))
        .collect();
    let least = sf::fmin(&ws.iter().flatten().map(|r| r.2).collect::<Vec<_>>());
    let mut wh: Vec<&str> = narrow.iter().map(|r| WALLS[r.0]).collect();
    wh.sort();
    wh.dedup();
    note(
        least >= pc::MIN_WEB && narrow.iter().all(|r| r.4),
        "every strip keeps ply between its edges",
        format!(
            "{:.2}mm wide outside but where a slot's strip owns both edges; opposite it the {} strip {least:.2}mm wide for {:.0}mm in all, against {}",
            ww - info.cut,
            wh.join(" and "),
            narrow.iter().map(|r| r.2 - r.1).sum::<f64>(),
            g(pc::MIN_WEB)
        ),
    );
    let &(_, hz, _, _) = sl.iter().find(|q| q.0 == 4).unwrap();
    let room = hz - t;
    let ref_room = s::REF_COLLAR - s::REF_HOLES[7].1;
    note(
        room >= ref_room - 1e-9,
        "the thumb has the reference's room above its hole",
        format!("{room:.1}mm from the slot's centre to the collar, the reference {ref_room:.1}mm"),
    );
    let socket = t;
    note(
        sl.iter().all(|&(_, x, _, ln)| x - ln / 2.0 - pc::MIN_WEB >= socket),
        "every slot is below the collar",
        format!("the highest {:.1}mm under it", sf::fmin(&sl.iter().map(|&(_, x, _, ln)| x - ln / 2.0).collect::<Vec<_>>()) - socket),
    );
    let lowest = sf::fmax(&sl.iter().map(|&(_, x, _, ln)| x + ln / 2.0).collect::<Vec<_>>());
    let clear = (ls - N_FOOT as f64 * t) - lowest;
    note(
        clear >= pc::MIN_WEB,
        "the foot is clear of the low A slot",
        format!("{}mm round, {}mm tall; {clear:.1}mm under the slot", g(FOOT_D), g(N_FOOT as f64 * t)),
    );
    note(
        (ls + info.x0 - info.seat_bottom - info.l).abs() < 1e-9,
        "the strips run from the sole into the collar",
        format!(
            "{ls:.1}mm, their top {}mm into its socket plate; the bore {:.1}mm, {:+.1}mm on the reference's, at the sole, for low G",
            g(t),
            info.l,
            info.l - s::REF_TOP
        ),
    );
    let r = py::hypot(info.pins[0].0, info.pins[0].1);
    let pin_r = (pc::PIN - pc::PIN_INTERF) / 2.0;
    let inr = FLATS / 2.0;
    note(
        inr + 0.2 <= r - pin_r && r + pin_r <= inr + t - 0.2,
        "the collar's pins land on the strips' top",
        format!("{:.2} to {:.2}mm out, the wall {} to {}mm", r - pin_r, r + pin_r, g(inr), g(inr + t)),
    );
    let web = pc::pin_web(parts, Some(&["seat", "collar", "sock", "socket", "chamber", "top", "stem"]), &[pin_r], false);
    note(web >= pc::MIN_WEB, "every pin keeps ply between it and every cut", format!("{web:.2}mm at the least, against {}", g(pc::MIN_WEB)));
    let (ok, what, detail) = s::seat_check(t);
    note(ok, what, detail);
    let caps = parts.iter().filter(|p| ["socket", "chamber", "top"].contains(&p.kind)).count();
    let sock_r = sf::fmax(&info.sock.iter().map(|v| py::hypot(v.0, v.1)).collect::<Vec<_>>());
    note(
        caps == info.n_cap && SOCK_D / 2.0 >= sock_r + pc::MIN_WEB,
        "the cap is the reference's, and slides over the tenon onto the collar",
        format!(
            "{caps} plates, {}mm long, {}mm round; the socket plate {}mm round over the octagon's {:.1}mm corners",
            g(caps as f64 * t),
            g(s::REF_CAP_D),
            g(SOCK_D),
            2.0 * sock_r
        ),
    );
    let (ok, d) = pc::mouthpiece_check(parts);
    note(ok, "the mouthpiece tapers as a real practice chanter's does", d);
    pc::sheet_checks(&mut note, sheets, parts.len(), "");
    out
}

pub fn label() -> &'static str {
    "standard-octagon"
}

fn svg_sheet(k: usize, nsh: usize, sh: &Sheet) -> String {
    let txt = pc::svg_sheet(k, nsh, sh, N);
    let what = format!("Practice chanter, a measured chanter's holes as slots on a {}mm octagonal bore of eight finger-jointed strips", g(FLATS));
    txt.replace(&format!("Practice chanter, {N}-sided bore"), &what)
        .replace("cuts the bores, seats and pin holes first; then black", "cuts the bores, seats, slots and pin holes first; then black")
}

/// Where a part sits: a plate as practice_chanters lays them; strip k with
/// its outside facing out at k x 45 degrees from the front.
pub fn placed(p: &CPart, info: &Built) -> Mat {
    if p.kind != "strip" {
        return plate_m(p.x);
    }
    let th = py::radians(45.0 * p.strip_k as f64);
    let n = [0.0, -py::mcos(th), py::msin(th)];
    let u = [0.0, -py::msin(th), -py::mcos(th)];
    let (r, w) = (FLATS / 2.0 + info.t, info.face_out);
    let o: Vec<f64> = (0..3).map(|i| if i == 0 { info.x0 } else { r * n[i] - w / 2.0 * u[i] }).collect();
    vec![
        vec![I(1), F(u[0]), F(-n[0]), F(o[0])],
        vec![I(0), F(u[1]), F(-n[1]), F(o[1])],
        vec![I(0), F(u[2]), F(-n[2]), F(o[2])],
        vec![I(0), I(0), I(0), I(1)],
    ]
}

/// The chanter for practice_chanters' 3D page.
pub fn design3d() -> J {
    let (parts, info) = build();
    let mut xs: Vec<f64> = parts.iter().map(|p| p.x).collect();
    xs.push(info.x0 + info.ls);
    let mid = (sf::fmin(&xs) + sf::fmax(&xs)) / 2.0;
    let out: Vec<J> = parts
        .iter()
        .map(|p| {
            let cap = CAPS.contains(&p.kind);
            let ex = if p.kind == "strip" {
                let th = py::radians(45.0 * p.strip_k as f64);
                [I(0), F(-py::mcos(th) * 0.12), F(py::msin(th) * 0.12)]
            } else {
                [F((p.x - mid) / 60.0 - if cap { 1.0 } else { 0.0 }), I(0), I(0)]
            };
            let wood = if p.kind == "strip" { "dark" } else if cap { "pale" } else { "ply" };
            page_part(p, &placed(p, &info), ex, wood)
        })
        .collect();
    pc::design_j(out)
}

pub fn main() {
    let (mut out, mut write) = (None::<String>, true);
    for x in std::env::args().skip(1) {
        if x == "--no-write" {
            write = false;
        } else if let Some(v) = x.strip_prefix("--out=") {
            out = Some(v.to_string());
        } else if x.starts_with("--plate=") {
            m::PLATE.set(pc::num_arg(&x));
        } else if x.starts_with("--kerf=") {
            sf::set_kerf(&x);
        } else if let Some(dir) = x.strip_prefix("--solidcraft=") {
            // this chanter as its parametric SolidCraft design (practice_chanters
            // --solidcraft= writes it too, with the others): the program that
            // writes a design is the one that draws its sheets, so sc-replay
            // finds this one
            std::fs::create_dir_all(dir).unwrap_or_else(|e| die(format!("{dir}: {e}")));
            let f = format!("{}/practice-chanters-standard-octagon.solidcraft.json", dir.trim_end_matches('/'));
            std::fs::write(&f, crate::chanters_solidcraft::standard_octagon()).unwrap_or_else(|e| die(format!("{f}: {e}")));
            println!("  wrote {}", f.rsplit('/').next().unwrap());
            return;
        } else {
            die(format!("error: {x} is not a flag this generator reads. It takes --out=, --plate=, --kerf=, --solidcraft= and --no-write."));
        }
    }
    if write && out.is_none() {
        die("error: --out=DIR, or --no-write to run the checks only.");
    }
    let (parts, info) = build();
    let sheets = pc::pack(&parts);
    let res = checks(&parts, &info, &sheets);
    let t = info.t;
    println!(
        "practice chanter, a measured chanter's holes as slots on a {}mm octagonal bore of eight finger-jointed strips: strips {:.1}mm long, {}mm across the flats outside; body {:.1}mm, sole to tenon top",
        g(FLATS),
        info.ls,
        g(FLATS + 2.0 * t),
        info.ls + info.x0
    );
    for i in 0..8 {
        let (k, _, d, ln) = info.slots[i];
        let d_ref = s::REF_HOLES[i].2;
        let z = info.holes[i].z;
        let shape = if ln > d - 1e-9 && d > info.slot_w { format!("slot {ln:.2} x {:.2}mm", info.slot_w) } else { format!("round {d:.2}mm") };
        println!(
            "  {:<3} {:<5} strip, {:6.1}mm up from the sole ({z:5.1}mm under the seat), {shape}, as a {d:.2}mm hole (the reference {d_ref:.2})",
            HOLES[i],
            WALLS[k],
            info.l - z
        );
    }
    println!("  top down: {} tenon, {} collar, 1 collar socket over the strips; at the sole {N_FOOT} foot plates", s::N_SEAT, s::N_COLLAR);
    if pc::print_checks(&res) {
        std::process::exit(1);
    }
    if write {
        pc::write_sheets(&out.unwrap(), &sheets, &|k| format!("practice-chanter-{}-sheet{k}-cut-files.svg", label()), &|k, n, sh| svg_sheet(k, n, sh));
    }
}

