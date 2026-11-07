//! Cut files for a Great Highland bagpipe practice chanter whose bore is a
//! regular octagon, built as a stack of cross-section plates clamped on two
//! threaded rods, with a stacked cap and mouthpiece.
//!
//!     target/release/practice_chanters --no-write                # the checks
//!     target/release/practice_chanters --out=DIR                 # the sheet
//!     target/release/practice_chanters --plate=2.9 ...           # for ply as measured
//!     target/release/practice_chanters --kerf=0.18 ...           # for that kerf
//!     target/release/practice_chanters --page-to=turn.html       # the 3D view, every design
//!
//! Everything below takes any polygon, as chanter_model.rs does, but only
//! the octagon is built: the triangle to the heptagon were dropped on
//! 2026-10-23 (BUILT). ghb-chanter uses these functions for its own 4, 6
//! and 8 sides, and uilleann-chanter for its plates.
//!
//! THE REFERENCE is standard_chanter.rs's measured practice chanter, as the
//! two standard chanters' is. The bore is an octagon of the reference's
//! bore's area, 4.74mm across its flats; the outside OUTER_FLATS across, a
//! 5.6mm wall where the reference tapers from 7.4 to 5.1. Each hole sits in
//! the plate where the reference's centre falls, and its slot is solved, in
//! standard_chanter.rs's model, to sound the reference's note there; the
//! bore is the whole number of plates nearest the reference's low G. The
//! model is used for the difference between the two chanters, not the pitch.
//!
//! OUTER_FLATS is the narrowest the rods allow, and the widest the holes do:
//! a thicker wall wants bigger holes, and at 17mm the low A hole would need
//! a slot wider than W_MAX, the 5mm a finger pad closes. At 15.5mm the rods
//! would keep under MIN_WEB of ply round them.
//!
//! From the top down:
//!
//!   * the CAP: a mouthpiece tapered as a real practice chanter's is, two
//!     top plates, a chamber round the reed's blades, and a socket that
//!     slides over the body's top plates;
//!   * the SEAT: N_SEAT plates with the reference's seat through them. Its
//!     bore is wider than the smallest staple, so a staple is held by its
//!     hemp, as on the reference, not stopped by a step;
//!   * the BORE: one plate per PLATE of bore, the octagon through each, down
//!     to the open sole. A finger hole is a slot cut in its plate from the
//!     bore out through the face, which leaves the plate C-shaped; one too
//!     small to cut is drilled after assembly instead, at a V cut in its
//!     plate's edge.
//!
//! Every body plate carries the same two rod holes, placed so that no turn
//! or flip of the polygon brings them back into line: a plate goes on only
//! one way, so no slot can face the wrong way. The rods are M2 threaded rod
//! with a nut and washer at each end, which clamp the stack: a press-fit pin
//! cannot be driven through 88 plates, and nothing is glued. Shellac seals
//! the seams. The cap comes off to set the reed, so it is not on the rods:
//! it slides over the body's top plates and rests on the two top nuts. The
//! cap, 19 plates, is short enough to press together on its own pins. The
//! mouthpiece's plates are too narrow for pins: they are glued, lined up on
//! a rod through the blowing hole.
//!
//! A PLATE IS NOT ALWAYS 3MM. A stack multiplies the stock's error by every
//! plate in it: a thinner ply shortens the bore and moves every hole up it,
//! and every note goes sharp. Measure the ply and pass --plate=; the model
//! then places every hole on that grid, and the plate count follows.
use crate::chanter_model::{self as m, plate, HOLES};
use crate::standard_chanter as s;
use four_hole_ocarinas::offset;
use lasermade_tools::die;
use lasermade_tools::py::{self, g};
use lasermade_tools::turn3d::{self, placed_raw, Mat, Num, Pt, F, I, J};
use ribbon_bore::path;
use square_flute::{self as sf, kerf, Cut, Part};
use std::f64::consts::PI;

pub const MIN_WEB: f64 = sf::MIN_WEB;
pub const BED: (f64, f64) = sf::BED;
pub const MARGIN: f64 = sf::MARGIN;

// --- the reed and its seat (mm) -------------------------------------------------
// A practice-chanter reed's staple is 4 to 4.8mm across, by the reed makers'
// figures; pipers wrap it in hemp to fit the seat, so the seat is cut over
// the largest, and the hemp holds it there.
pub const STAPLE_MIN: f64 = 4.0;
pub const STAPLE_MAX: f64 = 4.8;
pub const SEAT_D: f64 = 5.57; // the reference's
pub const N_SEAT: usize = 9; // 27mm of seat, the reference's 25.5
// --- the body -----------------------------------------------------------------
pub const REF_BORE: f64 = 4.87; // standard_chanter's REF_BORE: the bore is its area
pub const OUTER_FLATS: f64 = 16.0; // the outside across its flats
// --- the rods and pins ----------------------------------------------------------
pub const ROD: f64 = 2.0; // M2 threaded rod, through the body
pub const ROD_HOLE: f64 = 2.2; // a sliding fit: the nuts clamp, not the holes
pub const WASHER_D: f64 = 5.0; // an M2 washer
pub const NUT_AF: f64 = 4.0; // an M2 nut across its flats
pub const NUT_T: f64 = 1.6; // and thick
pub const ROD_SPARE: f64 = 3.0; // rod left past the sole's nut
pub const PIN: f64 = 2.0; // brass rod, the cap's and mouthpiece's pins
pub const PIN_INTERF: f64 = 0.1; // the hole this much under the rod, a press-fit
// --- the cap ------------------------------------------------------------------
pub const CAP_CLEAR: f64 = 0.3; // each side of the body, for hemp or tape
pub const CAP_WEB: f64 = 2.0; // ply each side of the cap's pins
pub const N_SOCKET: usize = 5; // plates of socket over the body's top
pub const CHAMBER_D: f64 = 13.0; // round the reed's blades: under the top nuts
pub const N_CHAMBER: usize = 12; // 36mm: the reed stands about 30mm proud
pub const MOUTH_D: f64 = 4.5; // the blowing hole, the reference chanter's
pub const N_TOP: usize = 2;
/// The mouthpiece's outside, up from the cap's top: the measured practice
/// chanter's top piece, its diameter at each height, straight between.
pub const MOUTHPIECE: [(f64, f64); 8] =
    [(0.0, 22.2), (3.0, 16.5), (8.5, 16.5), (10.25, 14.3), (17.5, 14.3), (19.75, 12.43), (70.0, 11.04), (71.5, 9.6)];
// --- marks --------------------------------------------------------------------
pub const V_W: f64 = 1.2; // a drill mark: a V this wide and deep in the edge
pub const V_D: f64 = 0.6;
pub const STEP: f64 = 0.5; // degrees, the boundary's sampling

pub type P2 = (f64, f64);

/// Python's float %: the sign of the divisor.
pub fn pymod(a: f64, b: f64) -> f64 {
    let r = a % b;
    if r != 0.0 {
        if (b < 0.0) != (r < 0.0) {
            r + b
        } else {
            r
        }
    } else {
        0.0f64.copysign(b)
    }
}

/// The point r from the centre at t degrees round from the front: the front
/// is -v, the thumb's side of an odd polygon -u.
pub fn polar(t: f64, r: f64) -> P2 {
    let a = py::radians(t);
    (r * py::msin(a), -r * py::mcos(a))
}

pub fn ngon(n: usize, inr: f64, rot: f64) -> Vec<P2> {
    let r = inr / py::mcos(PI / n as f64);
    (0..n).map(|k| polar(rot + (k as f64 + 0.5) * 360.0 / n as f64, r)).collect()
}

fn ring<T: Copy>(v: &[T]) -> impl Iterator<Item = (T, T)> + '_ {
    (0..v.len()).map(move |i| (v[i], v[(i + 1) % v.len()]))
}

/// How far the ray from the centre at t meets the convex polygon.
pub fn ray(poly: &[P2], t: f64) -> f64 {
    let (dx, dy) = polar(t, 1.0);
    let mut best = f64::INFINITY;
    for ((x0, y0), (x1, y1)) in ring(poly) {
        let (ex, ey) = (x1 - x0, y1 - y0);
        let den = dx * ey - dy * ex;
        if den.abs() < 1e-12 {
            continue;
        }
        let s = (x0 * ey - y0 * ex) / den;
        let u = (x0 * dy - y0 * dx) / den;
        if s > 1e-9 && -1e-9 <= u && u <= 1.0 + 1e-9 {
            best = py::min(best, s);
        }
    }
    best
}

/// How far out the ray at t leaves a slot w wide running out along theta
/// from s0 off the centre, or 0 if it meets only the slot's back.
pub fn slot_r(t: f64, theta: f64, w: f64, s0: f64) -> f64 {
    let d = pymod(t - theta + 180.0, 360.0) - 180.0;
    if d.abs() >= 90.0 {
        return 0.0;
    }
    let s = py::msin(py::radians(d)).abs();
    if s < 1e-12 {
        return f64::INFINITY;
    }
    let r = (w / 2.0) / s;
    if r * py::mcos(py::radians(d)) >= s0 - 1e-9 {
        r
    } else {
        0.0
    }
}

/// Where each side of a slot w wide along theta meets the bore's edges:
/// (side, distance out along the slot, the point).
pub fn side_meets(bore: &[P2], theta: f64, w: f64) -> Vec<(i32, f64, P2)> {
    let (ux, uy) = polar(theta, 1.0);
    let mut out = vec![];
    for sg in [1, -1] {
        let sgf = sg as f64;
        let (ox, oy) = (-uy * sgf * w / 2.0, ux * sgf * w / 2.0);
        for ((x0, y0), (x1, y1)) in ring(bore) {
            let (ex, ey) = (x1 - x0, y1 - y0);
            let den = ux * ey - uy * ex;
            if den.abs() < 1e-12 {
                continue;
            }
            let k = ((x0 - ox) * ey - (y0 - oy) * ex) / den;
            let v = ((x0 - ox) * uy - (y0 - oy) * ux) / den;
            if k > 0.0 && (0.0..=1.0).contains(&v) {
                out.push((sg, k, (ox + k * ux, oy + k * uy)));
            }
        }
    }
    out
}

/// Drop repeated points, and points on a straight line between their
/// neighbours.
pub fn tidy(pts: &[P2]) -> Vec<P2> {
    let n0 = pts.len();
    let pts: Vec<P2> = (0..n0).filter(|&i| py::dist(pts[i], pts[(i + n0 - 1) % n0]) > 1e-6).map(|i| pts[i]).collect();
    let n = pts.len();
    let mut out = vec![];
    for i in 0..n {
        let (a, b, c) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
        if ((b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)).abs() > 1e-7 {
            out.push(b);
        }
    }
    out
}

fn deg_of(v: P2) -> f64 {
    py::degrees(py::matan2(v.0, -v.1))
}

/// A plate with a slot w wide from the bore out through the face at theta:
/// one outline, round the outside and back round the bore.
pub fn c_plate(outer: &[P2], bore: &[P2], theta: f64, w: f64) -> Vec<P2> {
    let meets = side_meets(bore, theta, w);
    let side_min = |side: i32| {
        let ks: Vec<f64> = meets.iter().filter(|m| m.0 == side).map(|m| m.1).collect();
        if ks.is_empty() {
            0.0
        } else {
            sf::fmin(&ks)
        }
    };
    let s0 = py::max(side_min(1), side_min(-1));
    let inner = |t: f64| py::max(ray(bore, t), slot_r(t, theta, w, s0));
    let cut = |t: f64| inner(t) >= ray(outer, t);
    let nts = (360.0 / STEP) as usize;
    let ts: Vec<f64> = (0..nts).map(|i| theta + 180.0 + STEP * i as f64).collect();
    let mut edges = vec![];
    for i in 0..nts {
        let (a, b) = (ts[i], if i + 1 < nts { ts[i + 1] } else { ts[0] + 360.0 });
        if cut(a) != cut(b) {
            let (mut lo, mut hi) = (a, b);
            for _ in 0..50 {
                let mid = (lo + hi) / 2.0;
                if cut(mid) == cut(lo) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            edges.push(if !cut(lo) { lo } else { hi });
        }
    }
    assert!(edges.len() == 2, "{edges:?}");
    edges.sort_by(|a, b| pymod(a - theta, 360.0).partial_cmp(&pymod(b - theta, 360.0)).unwrap());
    let (t1, t0) = (edges[0], edges[1]);
    let span = pymod(t0 - t1, 360.0);
    let m = (span / STEP) as i64;
    let mut angles: Vec<f64> = (0..=m).map(|i| t1 + span * i as f64 / m as f64).collect();
    for &v in outer.iter().chain(bore) {
        let a = deg_of(v);
        let r = pymod(a - t1, 360.0);
        if 0.0 < r && r < span {
            angles.push(t1 + r);
        }
    }
    for &(_, _, q) in &meets {
        let a = deg_of(q);
        let r = pymod(a - t1, 360.0);
        if 0.0 < r && r < span {
            angles.push(t1 + r);
        }
    }
    for d in [90.0f64, -90.0] {
        let a = t1 + pymod(theta + d - t1, 360.0);
        let e = 1e-7f64.copysign(d);
        if 0.0 < a - t1 && a - t1 < span && slot_r(a - e, theta, w, s0) > ray(bore, a) {
            angles.push(a - e);
            angles.push(a);
        }
    }
    angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
    angles.dedup_by(|a, b| a == b);
    let mut pts: Vec<P2> = angles.iter().map(|&t| polar(t, ray(outer, t))).collect();
    pts.extend(angles.iter().rev().map(|&t| polar(t, inner(t))));
    tidy(&pts)
}

/// The outside with a V cut into it at the middle of the face at theta.
pub fn v_mark(outer: &[P2], theta: f64) -> Vec<P2> {
    let n = outer.len();
    let key = |i: usize| {
        let (a, b) = (outer[(i + n - 1) % n], outer[i]);
        let ang = py::degrees(py::matan2((b.0 + a.0) / 2.0, -(b.1 + a.1) / 2.0));
        (pymod(ang - theta + 180.0, 360.0) - 180.0).abs()
    };
    let mut k = 0;
    for i in 1..n {
        if key(i) < key(k) {
            k = i;
        }
    }
    let (a, b) = (outer[(k + n - 1) % n], outer[k]);
    let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let l = py::hypot(b.0 - a.0, b.1 - a.1);
    let (ux, uy) = ((b.0 - a.0) / l, (b.1 - a.1) / l);
    let (nx, ny) = polar(theta, 1.0);
    let v = [(mx - ux * V_W / 2.0, my - uy * V_W / 2.0), (mx - nx * V_D, my - ny * V_D), (mx + ux * V_W / 2.0, my + uy * V_W / 2.0)];
    let mut o = outer[..k].to_vec();
    o.extend(v);
    o.extend_from_slice(&outer[k..]);
    o
}

pub fn circle(cx: f64, cy: f64, r: f64, n: usize) -> Vec<P2> {
    (0..n)
        .map(|i| {
            let a = 2.0 * PI * i as f64 / n as f64;
            (cx + r * py::mcos(a), cy + r * py::msin(a))
        })
        .collect()
}

pub fn seg_dist(p: P2, a: P2, b: P2) -> f64 {
    let (ax, ay) = (b.0 - a.0, b.1 - a.1);
    let l2 = ax * ax + ay * ay;
    let t = if l2 == 0.0 { 0.0 } else { py::max(0.0, py::min(1.0, ((p.0 - a.0) * ax + (p.1 - a.1) * ay) / l2)) };
    py::hypot(p.0 - a.0 - t * ax, p.1 - a.1 - t * ay)
}

pub fn poly_dist(p: P2, poly: &[P2]) -> f64 {
    let mut best = f64::NAN;
    for (i, (a, b)) in ring(poly).enumerate() {
        let d = seg_dist(p, a, b);
        if i == 0 || d < best {
            best = d;
        }
    }
    best
}

pub fn inside(p: P2, poly: &[P2]) -> bool {
    let mut c = false;
    for ((x0, y0), (x1, y1)) in ring(poly) {
        if (y0 > p.1) != (y1 > p.1) && p.0 < x0 + (p.1 - y0) * (x1 - x0) / (y1 - y0) {
            c = !c;
        }
    }
    c
}

/// The front face's angle and the thumb's: straight back on an even polygon,
/// on an odd one the back face on the left hand's side.
pub fn faces(n: usize) -> (f64, f64) {
    (0.0, if n % 2 == 0 { 180.0 } else { 180.0 + 180.0 / n as f64 })
}

/// The polygon's turns and flips, as maps of a point.
pub fn symmetries(n: usize) -> Vec<Box<dyn Fn(P2) -> P2>> {
    let mut out: Vec<Box<dyn Fn(P2) -> P2>> = vec![];
    for k in 0..n {
        let a = 2.0 * PI * k as f64 / n as f64;
        let (c, s) = (py::mcos(a), py::msin(a));
        out.push(Box::new(move |p: P2| (c * p.0 - s * p.1, s * p.0 + c * p.1)));
        out.push(Box::new(move |p: P2| (c * -p.0 - s * p.1, s * -p.0 + c * p.1)));
    }
    out
}

/// {(round(x, 3), round(y, 3))} of two points, as Python compares the sets.
pub fn rset(ps: &[P2]) -> Vec<(u64, u64)> {
    let r = |x: f64| {
        let y = py::round_dp(x, 3);
        (if y == 0.0 { 0.0 } else { y }).to_bits()
    };
    let mut v: Vec<(u64, u64)> = ps.iter().map(|p| (r(p.0), r(p.1))).collect();
    v.sort();
    v.dedup();
    v
}

/// Two rod centres, as far from every cut as they can be, roughly across
/// the plate, and placed so that no turn or flip but none at all brings
/// both back into place.
pub fn pins_for(n: usize, outer: &[P2], bore: &[P2]) -> Vec<P2> {
    let (front, thumb) = faces(n);
    let clear = |p: P2| -> f64 {
        if !inside(p, outer) || inside(p, bore) || poly_dist(p, outer) < WASHER_D / 2.0 {
            return -1.0;
        }
        let h = py::hypot(p.0, p.1);
        let mut c = py::min(py::min(poly_dist(p, outer), poly_dist(p, bore)), h - SEAT_D / 2.0);
        let t = deg_of(p);
        for th in [front, thumb] {
            let d = pymod(t - th + 180.0, 360.0) - 180.0;
            let k = if d.abs() < 90.0 { py::msin(py::radians(d)).abs() } else { 1.0 };
            c = py::min(c, k * h - m::W_MAX / 2.0);
        }
        c - ROD_HOLE / 2.0
    };
    let r = sf::fmax(&outer.iter().map(|v| py::hypot(v.0, v.1)).collect::<Vec<_>>());
    let mut best: Vec<(i64, (f64, f64))> = vec![];
    for i in (0..360).step_by(2) {
        let mut b: Option<(f64, f64)> = None;
        for j in 1..60 {
            let rr = r * j as f64 / 60.0;
            let cand = (clear(polar(i as f64, rr)), rr);
            if b.is_none_or(|b| cand.0 > b.0 || (cand.0 == b.0 && cand.1 > b.1)) {
                b = Some(cand);
            }
        }
        best.push((i, b.unwrap()));
    }
    let syms = symmetries(n);
    let mut pairs: Vec<(f64, P2, P2)> = vec![];
    for &(a, ba) in &best {
        for &(b, bb) in &best {
            let sep = (b - a).rem_euclid(360);
            if !(130..=170).contains(&sep) {
                continue;
            }
            let (p, q) = (polar(a as f64, ba.1), polar(b as f64, bb.1));
            let fixed = syms.iter().filter(|f| rset(&[f(p), f(q)]) == rset(&[p, q])).count();
            if fixed == 1 {
                pairs.push((py::min(ba.0, bb.0), p, q));
            }
        }
    }
    let mut top = pairs[0];
    for &c in &pairs[1..] {
        if c.partial_cmp(&top) == Some(std::cmp::Ordering::Greater) {
            top = c;
        }
    }
    vec![top.1, top.2]
}

// --- a part of a chanter ----------------------------------------------------------

/// A greyscale engrave: (x, y, w, h, rows), white nothing.
#[derive(Clone, Debug)]
pub struct Dish {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub rows: Vec<Vec<u8>>,
}

/// A part with where it sits down the chanter, and what kind of part.
#[derive(Clone, Debug)]
pub struct CPart {
    pub p: Part,
    pub x: f64,
    pub kind: &'static str,
    pub round: Option<f64>,
    pub finger: Option<&'static str>,
    pub width: f64,
    pub theta: f64,
    pub drill: Option<f64>,
    /// a strip's name (standard, strip, uilleann) or index (octagon, ghb)
    pub strip: &'static str,
    pub strip_k: usize,
    pub layer: usize,
    /// ghb's foot: the strips' outside at its layer
    pub w_ring: Vec<P2>,
    pub dishes: Vec<Dish>,
    pub flip: bool,
}

impl CPart {
    pub fn of(&self) -> Vec<P2> {
        self.p.of()
    }
}

pub fn fp(v: &[P2]) -> Vec<Pt> {
    v.iter().map(|&(x, y)| [F(x), F(y)]).collect()
}
pub fn pf(v: &[Pt]) -> Vec<P2> {
    v.iter().map(|q| (q[0].f(), q[1].f())).collect()
}

pub enum C {
    Poly(Vec<P2>),
    Circle(f64, f64, f64),
}
pub fn cc(x: f64, y: f64, r: f64) -> C {
    C::Circle(x, y, r)
}

/// The parts as they are made, numbered from 1.
#[derive(Default)]
pub struct Parts {
    pub v: Vec<CPart>,
}

impl Parts {
    #[allow(clippy::too_many_arguments)]
    pub fn add_pts(&mut self, name: &str, outline: Vec<Pt>, cuts: Vec<C>, x: f64, kind: &'static str, note: &str, rnd: Option<f64>) -> &mut CPart {
        let mut p = Part::new(name, self.v.len() + 1, outline, note);
        p.cuts = cuts
            .into_iter()
            .map(|c| match c {
                C::Poly(v) => Cut::Poly(fp(&v)),
                C::Circle(x, y, r) => Cut::Circle(x, y, r),
            })
            .collect();
        self.v.push(CPart {
            p,
            x,
            kind,
            round: rnd,
            finger: None,
            width: 0.0,
            theta: 0.0,
            drill: None,
            strip: "",
            strip_k: 0,
            layer: 0,
            w_ring: vec![],
            dishes: vec![],
            flip: false,
        });
        self.v.last_mut().unwrap()
    }
    #[allow(clippy::too_many_arguments)]
    pub fn add(&mut self, name: &str, outline: &[P2], cuts: Vec<C>, x: f64, kind: &'static str, note: &str, rnd: Option<f64>) -> &mut CPart {
        self.add_pts(name, fp(outline), cuts, x, kind, note, rnd)
    }
}

/// A cut's polygon: its own, or the circle's.
pub fn cut_poly(c: &Cut, n: usize) -> Vec<P2> {
    match c {
        Cut::Poly(v) => pf(v),
        Cut::Circle(x, y, r) => circle(*x, *y, *r, n),
    }
}

/// The cap's and mouthpiece's plates for a body whose top plates are outer:
/// socket, chamber, top, stem; its own pins in its wall.
pub fn cap(parts: &mut Parts, outer: &[P2], n: usize, n_socket: Option<usize>, chamber: Option<f64>) -> (Vec<P2>, Vec<P2>, Vec<P2>) {
    let sock = offset(outer, CAP_CLEAR);
    let ring_ = offset(&sock, CAP_WEB + PIN / 2.0);
    let cap_pins = vec![ring_[0], ring_[n / 2]];
    let cap_o = offset(&sock, 2.0 * CAP_WEB + PIN);
    let pin_cut = || cap_pins.iter().map(|&(x, y)| cc(x, y, (PIN - PIN_INTERF) / 2.0)).collect::<Vec<_>>();
    let n_socket = n_socket.unwrap_or(N_SOCKET);
    let chamber = chamber.unwrap_or(CHAMBER_D);
    let p = plate();
    for k in 0..n_socket {
        let mut cuts = vec![C::Poly(sock.clone())];
        cuts.extend(pin_cut());
        parts.add("cap socket", &cap_o, cuts, (n_socket - 1 - k) as f64 * p, "socket", "", None);
    }
    for k in 0..N_CHAMBER {
        let mut cuts = vec![cc(0.0, 0.0, chamber / 2.0)];
        cuts.extend(pin_cut());
        parts.add("cap chamber", &cap_o, cuts, -((k + 1) as f64) * p, "chamber", "", None);
    }
    for k in 0..N_TOP {
        let mut cuts = vec![cc(0.0, 0.0, MOUTH_D / 2.0)];
        cuts.extend(pin_cut());
        parts.add("cap top", &cap_o, cuts, -((N_CHAMBER + k + 1) as f64) * p, "top", "", None);
    }
    mouthpiece(parts, -((N_CHAMBER + N_TOP) as f64) * p);
    (cap_o, cap_pins, sock)
}

/// The mouthpiece's plates, up from a cap whose topmost plate is at x_top.
pub fn mouthpiece(parts: &mut Parts, x_top: f64) {
    let p = plate();
    for k in 0..n_stem() {
        let d = stem_d((k as f64 + 0.5) * p);
        parts.add(
            "mouthpiece",
            &circle(0.0, 0.0, d / 2.0, 72),
            vec![cc(0.0, 0.0, MOUTH_D / 2.0)],
            x_top - (k + 1) as f64 * p,
            "stem",
            &format!("mouthpiece plate {}, {d:.2}mm", k + 1),
            Some(d / 2.0),
        );
    }
}

/// (ok, detail): the mouthpiece's plates step down the reference's profile
/// and keep ply round the blowing hole.
pub fn mouthpiece_check(parts: &[CPart]) -> (bool, String) {
    let mut st: Vec<&CPart> = parts.iter().filter(|p| p.kind == "stem").collect();
    st.sort_by(|a, b| (-a.x).partial_cmp(&-b.x).unwrap());
    let wall = sf::fmin(&st.iter().map(|p| p.round.unwrap()).collect::<Vec<_>>()) - MOUTH_D / 2.0;
    let p = plate();
    let ok = st.len() == n_stem()
        && wall >= MIN_WEB
        && st.iter().enumerate().all(|(k, q)| (2.0 * q.round.unwrap() - stem_d((k as f64 + 0.5) * p)).abs() < 1e-9);
    (
        ok,
        format!(
            "{} plates, {:.1} to {:.1}mm, {}mm long (the reference's {}), each plate its profile at mid-height; {wall:.1}mm of wall at the lip",
            st.len(),
            2.0 * st[0].round.unwrap(),
            2.0 * st[st.len() - 1].round.unwrap(),
            g(st.len() as f64 * p),
            g(MOUTHPIECE[7].0)
        ),
    )
}

/// The mouthpiece's plates: as many as make the reference's length.
pub fn n_stem() -> usize {
    py::round(MOUTHPIECE[7].0 / plate()) as usize
}

/// The mouthpiece's diameter z up from the cap's top, the reference's.
pub fn stem_d(z: f64) -> f64 {
    for w in MOUTHPIECE.windows(2) {
        let ((z0, d0), (z1, d1)) = (w[0], w[1]);
        if z <= z1 {
            return d0 + (d1 - d0) * (py::max(z, z0) - z0) / (z1 - z0);
        }
    }
    MOUTHPIECE[7].1
}

pub fn bore_area() -> f64 {
    PI * py::powf(REF_BORE / 2.0, 2.0)
}

/// What the stacked chanter's solve and build hand on.
pub struct Info {
    pub b: m::Bore,
    pub l: f64,
    pub pos: [f64; 8],
    pub widths: [f64; 8],
    pub r: s::Notes,
    pub got: s::Notes,
    pub holes: Vec<s::Hole>,
    pub bore: Vec<P2>,
    pub outer: Vec<P2>,
    pub pins: Vec<P2>,
    pub n_bore: usize,
}

/// The bore's length, whole plates, and each hole's slot that sound the
/// reference's notes, each hole in the plate the reference's centre falls
/// in.
pub fn solve(n: usize) -> (s::SBore, f64, [f64; 8], [f64; 8], s::Notes, s::Notes, Vec<s::Hole>) {
    let (r_b, t) = (m::inradius(bore_area(), n), plate());
    let mut b = s::SBore::new(2.0 * r_b, false); // its loss on the inradius
    b.area = bore_area() / 1e6;
    b.a = (b.area / PI).sqrt();
    let wall = OUTER_FLATS / 2.0 - r_b;
    let r = s::reference();
    let mut holes: Vec<s::Hole> =
        s::REF_HOLES.iter().map(|&(h, z, d, _)| s::Hole { name: h, z: m::snap(s::REF_TOP - z, None), d, wall }).collect();
    let d_max = m::drill(m::W_MAX);
    let mut l = 0.0;
    for _ in 0..4 {
        l = s::bisect(&mut |x| s::sounding(&b, x, &holes, "low G", r.get("low G")), r.get("low G"), s::REF_TOP - 30.0, s::REF_TOP + 60.0, false);
        l = t * py::round(l / t) as f64;
        for i in 0..8 {
            let nt = m::SETS[i];
            let f = |d: f64, holes: &mut Vec<s::Hole>| {
                holes[i].d = d;
                s::sounding(&b, l, holes, nt, r.get(nt))
            };
            if f(d_max, &mut holes) < r.get(nt) {
                die(format!(
                    "error: {} is flat even as a {}mm slot; a {}mm outside is too thick a wall.",
                    HOLES[i],
                    g(m::W_MAX),
                    g(OUTER_FLATS)
                ));
            }
            let mut hc = holes.clone();
            let d = s::bisect(&mut |d| f(d, &mut hc), r.get(nt), m::D_MIN, d_max, true);
            holes[i].d = d;
        }
    }
    let got = s::Notes(m::NOTES.map(|nt| s::sounding(&b, l, &holes, nt, r.get(nt))));
    let pos: [f64; 8] = std::array::from_fn(|i| holes[i].z);
    let widths: [f64; 8] = std::array::from_fn(|i| PI * py::powf(holes[i].d, 2.0) / 4.0 / t);
    (b, l, pos, widths, r, got, holes)
}

pub struct Built {
    pub b: s::SBore,
    pub l: f64,
    pub pos: [f64; 8],
    pub widths: [f64; 8],
    pub r: s::Notes,
    pub got: s::Notes,
    pub holes: Vec<s::Hole>,
    pub bore: Vec<P2>,
    pub outer: Vec<P2>,
    pub pins: Vec<P2>,
    pub n_bore: usize,
}

/// The parts, and what the checks need, for an n-sided chanter.
pub fn build(n: usize) -> (Vec<CPart>, Built) {
    let (b, l, pos, widths, r, got, holes) = solve(n);
    let bore = ngon(n, m::inradius(bore_area(), n), 0.0);
    let outer = ngon(n, OUTER_FLATS / 2.0, 0.0);
    let (front, thumb) = faces(n);
    let pins = pins_for(n, &outer, &bore);
    let pin_cut = || pins.iter().map(|&(x, y)| cc(x, y, ROD_HOLE / 2.0)).collect::<Vec<_>>();
    let p = plate();
    let n_bore = py::round(l / p) as usize;
    // at = {int(pos[h] // PLATE): h}: the last hole wins a shared plate
    let mut at: Vec<(i64, usize)> = vec![];
    for i in 0..8 {
        let k = (pos[i] / p).floor() as i64;
        if let Some(e) = at.iter_mut().find(|e| e.0 == k) {
            e.1 = i;
        } else {
            at.push((k, i));
        }
    }
    let mut parts = Parts::default();
    for k in 0..N_SEAT {
        let mut cuts = vec![cc(0.0, 0.0, SEAT_D / 2.0)];
        cuts.extend(pin_cut());
        parts.add("seat", &outer, cuts, k as f64 * p, "seat", "", None);
    }
    for k in 0..n_bore {
        let x = (N_SEAT + k) as f64 * p;
        let Some(&(_, i)) = at.iter().find(|e| e.0 == k as i64) else {
            let mut cuts = vec![C::Poly(bore.clone())];
            cuts.extend(pin_cut());
            parts.add("bore", &outer, cuts, x, "bore", "", None);
            continue;
        };
        let h = HOLES[i];
        let th = if h == "hA" { thumb } else { front };
        let w = widths[i];
        let q = if w >= m::W_SLOT {
            parts.add(&format!("{h} hole"), &c_plate(&outer, &bore, th, w), pin_cut(), x, "slot", &format!("the {h} hole: a slot {w:.2}mm wide"), None)
        } else {
            let d = m::drill(w);
            let mut cuts = vec![C::Poly(bore.clone())];
            cuts.extend(pin_cut());
            let q = parts.add(
                &format!("{h} hole"),
                &v_mark(&outer, th),
                cuts,
                x,
                "drill",
                &format!("the {h} hole: drill {d:.2}mm through at the V, after assembly"),
                None,
            );
            q.drill = Some(d);
            q
        };
        q.finger = Some(h);
        q.width = w;
        q.theta = th;
    }
    cap(&mut parts, &outer, n, None, None);
    (parts.v, Built { b, l, pos, widths, r, got, holes, bore, outer, pins, n_bore })
}

/// The part turned half round, for the packer.
pub fn turned(p: &CPart) -> CPart {
    let mut q = p.clone();
    let o: Vec<Pt> = p.p.outline.iter().map(|v| [-v[0], -v[1]]).collect();
    q.p = Part::new(&p.p.name, p.p.num, o, &p.p.note);
    q.p.cuts = p
        .p
        .cuts
        .iter()
        .map(|c| match c {
            Cut::Poly(v) => Cut::Poly(v.iter().map(|q| [-q[0], -q[1]]).collect()),
            Cut::Circle(x, y, r) => Cut::Circle(-x, -y, *r),
        })
        .collect();
    q
}

fn placed_pts(a: &CPart, ax: f64, ay: f64) -> Vec<P2> {
    a.p.outline.iter().map(|v| (v[0].f() - a.p.x0 + ax, v[1].f() - a.p.y0 + ay)).collect()
}

/// The least distance between two placed outlines, neither inside the other.
pub fn apart(a: &CPart, ax: f64, ay: f64, b: &CPart, bx: f64, by: f64) -> f64 {
    let (pa, pb) = (placed_pts(a, ax, ay), placed_pts(b, bx, by));
    if pa.iter().any(|&v| inside(v, &pb)) || pb.iter().any(|&v| inside(v, &pa)) {
        return -1.0;
    }
    let m1 = sf::fmin(&pa.iter().map(|&v| poly_dist(v, &pb)).collect::<Vec<_>>());
    let m2 = sf::fmin(&pb.iter().map(|&v| poly_dist(v, &pa)).collect::<Vec<_>>());
    py::min(m1, m2)
}

/// apart(...) >= clear, without computing the whole minimum.
pub fn clear_of(a: &CPart, ax: f64, ay: f64, b: &CPart, bx: f64, by: f64, clear: f64) -> bool {
    let (pa, pb) = (placed_pts(a, ax, ay), placed_pts(b, bx, by));
    let far = clear + 1e-6;
    for (pp, qq) in [(&pa, &pb), (&pb, &pa)] {
        let xs: Vec<f64> = qq.iter().map(|v| v.0).collect();
        let ys: Vec<f64> = qq.iter().map(|v| v.1).collect();
        let (qx0, qx1, qy0, qy1) = (sf::fmin(&xs), sf::fmax(&xs), sf::fmin(&ys), sf::fmax(&ys));
        let edges: Vec<(P2, P2, f64, f64, f64, f64)> =
            ring(qq).map(|(e, f)| (e, f, py::min(e.0, f.0), py::max(e.0, f.0), py::min(e.1, f.1), py::max(e.1, f.1))).collect();
        for &v in pp.iter() {
            let dx = py::max(py::max(qx0 - v.0, 0.0), v.0 - qx1);
            let dy = py::max(py::max(qy0 - v.1, 0.0), v.1 - qy1);
            if dx >= far || dy >= far {
                continue;
            }
            for &(e, f, x0, x1, y0, y1) in &edges {
                if py::max(x0 - v.0, v.0 - x1) >= far || py::max(y0 - v.1, v.1 - y1) >= far {
                    continue;
                }
                if seg_dist(v, e, f) < clear {
                    return false;
                }
            }
        }
    }
    !(pa.iter().any(|&v| inside(v, &pb)) || pb.iter().any(|&v| inside(v, &pa)))
}

pub struct Sheet {
    pub parts: Vec<(CPart, f64, f64)>,
    pub w: f64,
    pub h: f64,
}

/// Rows across the bed, tallest parts first. Each part goes in the row both
/// ways up and slides left until it comes GAP from the one before; whichever
/// way ends further left is kept. A row full, the next starts below; a sheet
/// full, the next sheet. `turn` makes a part's half-turned copy.
pub fn pack_with(parts: &[CPart], turn: &dyn Fn(&CPart) -> CPart) -> Vec<Sheet> {
    let k = kerf();
    let clear = sf::GAP + k;
    let mut order: Vec<&CPart> = parts.iter().collect();
    order.sort_by(|a, b| {
        (-a.p.h).partial_cmp(&-b.p.h).unwrap().then(a.kind.cmp(b.kind)).then(a.p.num.cmp(&b.p.num))
    });
    let mut sheets: Vec<Vec<(CPart, f64, f64)>> = vec![];
    let (mut sheet, mut row): (Vec<(CPart, f64, f64)>, Vec<(CPart, f64, f64)>) = (vec![], vec![]);
    let (mut y, mut row_h) = (MARGIN, 0.0);
    for p in order {
        let mut best: Option<(CPart, f64)> = None;
        for q in [p.clone(), turn(p)] {
            let x = if row.is_empty() {
                MARGIN
            } else {
                let (last, lx, _) = &row[row.len() - 1];
                let mut hi = lx + last.p.w + clear;
                let mut lo = *lx;
                for _ in 0..30 {
                    let mid = (lo + hi) / 2.0;
                    let from = row.len().saturating_sub(3);
                    if row[from..].iter().all(|(o, ox, oy)| clear_of(o, *ox, *oy, &q, mid, y, clear)) {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                hi
            };
            if best.as_ref().is_none_or(|b| x + q.p.w < b.1 + b.0.p.w - 1e-9) {
                best = Some((q, x));
            }
        }
        let (mut q, mut x) = best.unwrap();
        if x + q.p.w + k > BED.0 - MARGIN || (!row.is_empty() && q.p.h > row_h + 1e-9) {
            y += row_h + clear;
            row.clear();
            row_h = 0.0;
            x = MARGIN;
            q = p.clone();
            if y + q.p.h + k > BED.1 - MARGIN {
                sheets.push(std::mem::take(&mut sheet));
                y = MARGIN;
            }
        }
        row.push((q.clone(), x, y));
        sheet.push((q.clone(), x, y));
        row_h = py::max(row_h, q.p.h);
    }
    sheets.push(sheet);
    sheets
        .into_iter()
        .map(|items| {
            let w = sf::fmax(&items.iter().map(|(p, x, _)| x + p.p.w + k).collect::<Vec<_>>()) + MARGIN;
            let h = sf::fmax(&items.iter().map(|(p, _, y)| y + p.p.h + k).collect::<Vec<_>>()) + MARGIN;
            Sheet { parts: items, w, h }
        })
        .collect()
}

pub fn pack(parts: &[CPart]) -> Vec<Sheet> {
    pack_with(parts, &turned)
}

pub fn poly_area(pts: &[P2]) -> f64 {
    let mut s = 0.0;
    for ((x0, y0), (x1, y1)) in ring(pts) {
        s += x0 * y1 - x1 * y0;
    }
    s.abs() / 2.0
}

/// The body from the top down, as the builder threads it.
pub fn stack_order(parts: &[CPart]) -> String {
    let mut body: Vec<&CPart> = parts.iter().filter(|p| ["seat", "bore", "slot", "drill"].contains(&p.kind)).collect();
    body.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    let (mut out, mut run, mut kind): (Vec<String>, usize, Option<&str>) = (vec![], 0, None);
    for p in body {
        let k = if p.kind == "seat" || p.kind == "bore" { Some(p.kind) } else { None };
        if k.is_some() && k == kind {
            run += 1;
            continue;
        }
        if let Some(kd) = kind {
            out.push(format!("{run} {kd}"));
        }
        if k.is_some() {
            kind = k;
            run = 1;
        } else {
            kind = None;
            run = 0;
            out.push(p.finger.unwrap().to_string());
        }
    }
    if let Some(kd) = kind {
        out.push(format!("{run} {kd}"));
    }
    out.join(", ")
}

pub type Checks = Vec<(bool, String, String)>;

/// The least web between any pin hole of radius in `radii` and every other
/// cut on its plate, over the parts whose kind is in `kinds` (all if None).
pub fn pin_web(parts: &[CPart], kinds: Option<&[&str]>, radii: &[f64], plain_outline: bool) -> f64 {
    let mut web = f64::INFINITY;
    for p in parts {
        if kinds.is_some_and(|k| !k.contains(&p.kind)) {
            continue;
        }
        // edges[0] is the plate's edge: a circle when the part is round
        let round = if plain_outline { None } else { p.round.filter(|r| *r != 0.0) };
        for (ci, c) in p.p.cuts.iter().enumerate() {
            let Cut::Circle(cx, cy, cr) = *c else { continue };
            let near = radii.iter().map(|r| (cr - r).abs()).fold(f64::NAN, |a, b| if a.is_nan() || b < a { b } else { a });
            if near > 1e-9 {
                continue;
            }
            let mut check = |e: &Cut| {
                let d = match e {
                    Cut::Poly(v) => poly_dist((cx, cy), &pf(v)),
                    Cut::Circle(ex, ey, er) => (py::hypot(cx - ex, cy - ey) - er).abs(),
                };
                web = py::min(web, d - cr);
            };
            match round {
                Some(r) => check(&Cut::Circle(0.0, 0.0, r)),
                None => check(&Cut::Poly(p.p.outline.clone())),
            }
            for (ei, e) in p.p.cuts.iter().enumerate() {
                if ei != ci {
                    check(e);
                }
            }
        }
    }
    web
}

/// The least ply between cuts of any two parts on a sheet.
pub fn least_gap(sheets: &[Sheet]) -> f64 {
    let k = kerf();
    let gp = sf::GAP;
    let mut least = f64::INFINITY;
    for sh in sheets {
        let its = &sh.parts;
        for i in 0..its.len() {
            let (a, ax, ay) = &its[i];
            for (c, cx, cy) in &its[i + 1..] {
                if *cx > ax + a.p.w + gp + k || *ax > cx + c.p.w + gp + k || *cy > ay + a.p.h + gp + k || *ay > cy + c.p.h + gp + k {
                    continue;
                }
                least = py::min(least, apart(a, *ax, *ay, c, *cx, *cy));
            }
        }
    }
    least
}

/// The checks every chanter ends with: the bed, the gap, every part placed.
pub fn sheet_checks(note: &mut dyn FnMut(bool, &str, String), sheets: &[Sheet], n_parts: usize, bed_tail: &str) {
    let k = kerf();
    note(
        sheets.iter().all(|s| s.w <= BED.0 && s.h <= BED.1),
        "every sheet fits the bed",
        format!(
            "{} sheet(s), largest {:.0} x {:.0}mm{bed_tail}",
            sheets.len(),
            sf::fmax(&sheets.iter().map(|s| s.w).collect::<Vec<_>>()),
            sf::fmax(&sheets.iter().map(|s| s.h).collect::<Vec<_>>())
        ),
    );
    let least = least_gap(sheets);
    note(
        least >= sf::GAP + k - 1e-3,
        "no two parts on a sheet come closer than the gap",
        format!("{:.2}mm of ply between cuts at the least, against {}", least - k, g(sf::GAP)),
    );
    let placed: usize = sheets.iter().map(|s| s.parts.len()).sum();
    note(placed == n_parts, "every part is on a sheet", format!("{placed} of {n_parts}"));
}

fn checks(n: usize, parts: &[CPart], info: &Built, sheets: &[Sheet]) -> Checks {
    let mut out: Checks = vec![];
    let mut note = |ok: bool, what: &str, detail: String| out.push((ok, what.to_string(), detail));
    let (b, l, pos) = (&info.b, info.l, &info.pos);
    let (r, got) = (&info.r, &info.got);
    let p = plate();
    let worst = sf::fmax(&m::NOTES.iter().filter(|nt| **nt != "low G").map(|nt| m::cents(got.get(nt), r.get(nt)).abs()).collect::<Vec<_>>());
    let low_g = m::cents(got.get("low G"), r.get("low G"));
    let near: Vec<f64> = [-p, p].iter().map(|d| m::cents(s::sounding(b, l + d, &info.holes, "low G", r.get("low G")), r.get("low G"))).collect();
    note(
        worst <= 5.0 && near.iter().all(|c| low_g.abs() < c.abs()),
        "every note is the reference's, in the model",
        format!(
            "within {worst:.1} cents; low G {low_g:+.1}, the bore the nearest whole plates (a plate less {:+.1}, more {:+.1})",
            near[0], near[1]
        ),
    );
    let area = poly_area(&info.bore);
    let inr = m::inradius(bore_area(), n);
    note(
        (area - bore_area()).abs() < 1e-6,
        "the bore is the reference's area",
        format!(
            "{area:.2}mm^2, the {}mm round bore's; a {n}-gon {:.2}mm across its flats, {} outside, a {:.2}mm wall",
            g(REF_BORE),
            2.0 * inr,
            g(OUTER_FLATS),
            OUTER_FLATS / 2.0 - inr
        ),
    );
    let off = sf::fmax(&s::REF_HOLES.iter().enumerate().map(|(i, &(_, z, _, _))| (pos[i] - (s::REF_TOP - z)).abs()).collect::<Vec<_>>());
    note(off <= p / 2.0 + 1e-9, "every hole is where the reference's is, to the plate", format!("{off:.2}mm out at the most, against half a plate"));
    let bores = parts.iter().filter(|q| ["bore", "slot", "drill"].contains(&q.kind)).count();
    note(bores as f64 * p == l, "the bore is whole plates, down to the sole", format!("{bores} plates of {}mm, {}mm", g(p), g(l)));
    let mut holes: Vec<&CPart> = parts.iter().filter(|q| q.finger.is_some()).collect();
    let on = holes.iter().all(|q| (q.x - (N_SEAT as f64 * p + pos[m::hi(q.finger.unwrap())] - p / 2.0)).abs() < 1e-6);
    holes.sort_by(|a, b| (-a.x).partial_cmp(&-b.x).unwrap());
    note(
        holes.len() == 8 && on,
        "every finger hole is in its own plate, at its centre",
        holes.iter().map(|q| format!("{} {:.1}", q.finger.unwrap(), pos[m::hi(q.finger.unwrap())])).collect::<Vec<_>>().join(", "),
    );
    let slots: Vec<&CPart> = parts.iter().filter(|q| q.finger.is_some() && q.kind == "slot").collect();
    let drills: Vec<&CPart> = parts.iter().filter(|q| q.finger.is_some() && q.kind == "drill").collect();
    let ws: Vec<f64> = slots.iter().map(|q| q.width).collect();
    let dr = drills.iter().map(|q| format!("{} drilled {:.2}mm", q.finger.unwrap(), q.drill.unwrap())).collect::<Vec<_>>().join(", ");
    note(
        slots.iter().all(|q| m::W_SLOT <= q.width && q.width <= m::W_MAX + 1e-9) && drills.iter().all(|q| q.drill.unwrap() >= m::D_MIN - 1e-9),
        "every hole is cuttable or drillable",
        format!("{} slots {:.2} to {:.2}mm wide; ", slots.len(), sf::fmin(&ws), sf::fmax(&ws)) + if dr.is_empty() { "nothing to drill" } else { &dr },
    );
    let mut worst_w = 0.0;
    for q in &slots {
        let (ux, uy) = (py::mcos(py::radians(q.theta)), py::msin(py::radians(q.theta)));
        let rim: Vec<P2> = q
            .of()
            .into_iter()
            .filter(|&v| poly_dist(v, &info.outer) < 1e-6 && (pymod(deg_of(v) - q.theta + 180.0, 360.0) - 180.0).abs() < 90.0)
            .collect();
        let pa: Vec<f64> = rim.iter().map(|v| ux * v.0 + uy * v.1).filter(|d| *d > 0.0).collect();
        let pb: Vec<f64> = rim.iter().map(|v| ux * v.0 + uy * v.1).filter(|d| *d < 0.0).map(|d| -d).collect();
        let a = if pa.is_empty() { f64::INFINITY } else { sf::fmin(&pa) };
        let bb = if pb.is_empty() { f64::INFINITY } else { sf::fmin(&pb) };
        worst_w = py::max(worst_w, (a + bb - q.width).abs());
    }
    note(worst_w < 1e-6, "every slot runs the width it is tuned for", format!("{} slots", slots.len()));
    let web = pin_web(parts, None, &[ROD_HOLE / 2.0, (PIN - PIN_INTERF) / 2.0], false);
    note(web >= MIN_WEB, "every rod and pin keeps ply between it and every cut", format!("{web:.2}mm at the least, against {}", g(MIN_WEB)));
    let room = sf::fmin(&info.pins.iter().map(|&q| poly_dist(q, &info.outer)).collect::<Vec<_>>());
    note(
        room >= WASHER_D / 2.0,
        "the nuts sit on the end plates, inside the body",
        format!("{}mm washers, {:.1}mm of face round each rod", g(WASHER_D), 2.0 * room),
    );
    let fixed = symmetries(n).iter().filter(|f| rset(&info.pins.iter().map(|&q| f(q)).collect::<Vec<_>>()) == rset(&info.pins)).count();
    note(
        fixed == 1,
        "every body plate goes on only one way",
        format!("of the {} turns and flips of a {n}-gon, only leaving it be keeps both rods in place", 2 * n),
    );
    note(
        SEAT_D >= STAPLE_MAX,
        "the seat takes any staple, held by its hemp as the reference's is",
        format!(
            "seat {}mm, {}mm deep (the reference's {}), over the largest staple, {}; the bore under it {:.2}mm across its flats, as the reference's {} is, wider than the smallest, {}",
            g(SEAT_D),
            g(N_SEAT as f64 * p),
            g(s::REF_SEAT_DEEP),
            g(STAPLE_MAX),
            2.0 * inr,
            g(REF_BORE),
            g(STAPLE_MIN)
        ),
    );
    let r_in = OUTER_FLATS / 2.0;
    let under = sf::fmin(&info.pins.iter().map(|q| py::hypot(q.0, q.1)).collect::<Vec<_>>()) + NUT_AF / 2.0 - CHAMBER_D / 2.0;
    note(
        CHAMBER_D < 2.0 * r_in && under >= 1.0,
        "the cap slides over the body and stops on its nuts",
        format!(
            "socket {}mm clear all round; each top nut {under:.1}mm under the chamber's bottom plate at the least, the chamber {}mm",
            g(CAP_CLEAR),
            g(CHAMBER_D)
        ),
    );
    let longest = sf::fmax(&parts.iter().map(|q| py::max(q.p.w, q.p.h)).collect::<Vec<_>>());
    let (ok, d) = mouthpiece_check(parts);
    note(ok, "the mouthpiece tapers as a real practice chanter's does", d);
    sheet_checks(&mut note, sheets, parts.len(), &format!(", the largest plate {:.1}mm", longest + kerf()));
    out
}

/// The sheet: orange cuts first, then black frees the plates.
pub fn svg_sheet(k: usize, nsh: usize, sh: &Sheet, n: usize) -> String {
    let (mut orange, mut black) = (vec![], vec![]);
    let kf = kerf();
    for (p, sx, sy) in &sh.parts {
        let (dx, dy) = (sx - p.p.x0 + kf / 2.0, sy - p.p.y0 + kf / 2.0);
        let mv = |pts: &[P2]| -> Vec<P2> { pts.iter().map(|&(x, y)| (x + dx, y + dy)).collect() };
        match p.round.filter(|r| *r != 0.0) {
            Some(r) => black.push(format!("<circle cx=\"{dx:.3}\" cy=\"{dy:.3}\" r=\"{:.3}\"/>", r + kf / 2.0)),
            None => black.push(format!("<path d=\"{}\"/>", path(&mv(&offset(&p.of(), kf / 2.0)), true))),
        }
        for c in &p.p.cuts {
            match c {
                Cut::Poly(v) => orange.push(format!("<path d=\"{}\"/>", path(&mv(&offset(&pf(v), -kf / 2.0)), true))),
                Cut::Circle(cx, cy, r) => {
                    orange.push(format!("<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{:.3}\"/>", cx + dx, cy + dy, r - kf / 2.0))
                }
            }
        }
    }
    let desc = format!(
        "1 user unit = 1mm. Practice chanter, {n}-sided bore, sheet {k} of {nsh}. {}mm ply, drawn for a {}mm kerf. \
         Orange {} cuts the bores, seats and pin holes first; then black {} frees the plates.",
        g(plate()),
        g(kf),
        sf::ORANGE,
        sf::BLACK
    );
    let grp = |col: &str, items: &[String]| -> String {
        if items.is_empty() {
            return String::new();
        }
        let mut s = format!("  <g fill=\"none\" stroke=\"{col}\" stroke-width=\"0.2\">\n");
        for i in items {
            s += &format!("    {i}\n");
        }
        s + "  </g>\n"
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" \
         height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>Practice chanter, {n}-sided bore, sheet {k} of {nsh}</title>\n\
         <desc>{desc}</desc>\n{}{}</svg>\n",
        grp(sf::ORANGE, &orange),
        grp(sf::BLACK, &black),
        w = sh.w,
        h = sh.h
    )
}

pub fn names(n: usize) -> &'static str {
    ["", "", "", "triangle", "square", "pentagon", "hexagon", "heptagon", "octagon"][n]
}
pub const BUILT: [usize; 1] = [8]; // the plate chanters this repository ships: the octagon alone

/// The plate frame's x across, y down and z through, to the chanter's up,
/// front and length: a turn, not a mirror.
pub fn plate_m(x: f64) -> Mat {
    vec![vec![I(0), I(0), I(1), F(x)], vec![I(0), I(1), I(0), I(0)], vec![I(-1), I(0), I(0), I(0)], vec![I(0), I(0), I(0), I(1)]]
}

/// A part's holes for the 3D page: polygons as they are, circles in 20.
pub fn holes3d(p: &CPart) -> Vec<Vec<Pt>> {
    p.p.cuts.iter().map(|c| fp(&cut_poly(c, 20))).collect()
}

pub fn page_part(p: &CPart, m: &Mat, ex: [Num; 3], wood: &str) -> J {
    placed_raw(&p.p.outline, &holes3d(p), &[], m, &ex, wood)
}

pub fn design_j(parts: Vec<J>) -> J {
    J::obj(vec![("parts", J::A(parts)), ("ex", J::N(I(60))), ("view", turn3d::nums(&[F(-0.12), F(0.32), F(3.0)]))])
}

pub const CAPS: [&str; 4] = ["socket", "chamber", "top", "stem"];

pub const BY: &str = "practice_chanters.rs";

/// The 3D view: every design, lying along the page with its finger holes to
/// the front and the cap on the left.
fn page(path_out: &str) {
    let mut designs: Vec<(String, J)> =
        vec![("standard".into(), s::design3d()), ("standard octagon".into(), crate::standard_octagon::design3d())];
    for n in BUILT {
        let (parts, _) = build(n);
        let xs: Vec<f64> = parts.iter().map(|p| p.x).collect();
        let mid = (sf::fmin(&xs) + sf::fmax(&xs)) / 2.0;
        let out: Vec<J> = parts
            .iter()
            .map(|p| {
                let cap = CAPS.contains(&p.kind);
                let ex = [F((p.x - mid) / 60.0 - if cap { 1.0 } else { 0.0 }), I(0), I(0)];
                let wood = if p.kind == "slot" || p.kind == "drill" { "dark" } else if cap { "pale" } else { "ply" };
                page_part(p, &plate_m(p.x), ex, wood)
            })
            .collect();
        designs.push((format!("stacked {}", names(n)), design_j(out)));
    }
    turn3d::write(
        path_out,
        "Practice chanters in 3D",
        &mut designs,
        "Drag to turn; right-drag, shift-drag or move two fingers to slide the view; \
         scroll or pinch to zoom in on whatever is under the pointer; double-click a part \
         to turn round it, and Reset view to go back. Draw Explode to spread the plates along \
         the chanter. Every plate is the cut files\u{2019} own outline. The finger-hole plates \
         are darker: a slot through the face, or a V where a hole too small to cut is \
         drilled. The cap, paler, is on, its socket over the body\u{2019}s top plates. The \
         buttons switch between two strip chanters, bores of finger-jointed strips, darker, \
         under stacked plates: standard, square, with a measured chanter\u{2019}s holes; and \
         standard octagon, eight strips with those holes as slots; then the stacked \
         octagon, every plate a slice across the chanter.",
        BY,
        false,
    );
}

pub fn basename(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}
pub fn join(dir: &str, f: &str) -> String {
    if dir.is_empty() || dir.ends_with('/') {
        format!("{dir}{f}")
    } else {
        format!("{dir}/{f}")
    }
}

/// Print the checks; true if any failed (and say so).
pub fn print_checks(res: &Checks) -> bool {
    let mut bad = 0;
    for (ok, what, detail) in res {
        println!("  {}  {:<52} {}", if *ok { "pass" } else { "FAIL" }, what, detail);
        bad += !*ok as usize;
    }
    if bad > 0 {
        println!("\n  {bad} check(s) failed. Nothing written.");
    }
    bad > 0
}

/// Write each sheet as `name(k)`, saying so.
pub fn write_sheets(out: &str, sheets: &[Sheet], name: &dyn Fn(usize) -> String, svg: &dyn Fn(usize, usize, &Sheet) -> String) {
    std::fs::create_dir_all(out).unwrap_or_else(|e| die(format!("{out}: {e}")));
    for (k, sh) in sheets.iter().enumerate() {
        let f = join(out, &name(k + 1));
        std::fs::write(&f, svg(k + 1, sheets.len(), sh)).unwrap_or_else(|e| die(format!("{f}: {e}")));
        println!("  wrote {}  {:.0} x {:.0}mm, {} parts", basename(&f), sh.w, sh.h, sh.parts.len());
    }
}

pub fn num_arg(x: &str) -> f64 {
    py::float(x.split_once('=').unwrap().1).unwrap_or_else(|| die(format!("error: {x}: not a number.")))
}

pub fn main() {
    let (mut out, mut write, mut sides, mut page_to) = (None::<String>, true, BUILT[0], None::<String>);
    for x in std::env::args().skip(1) {
        let val = || x.split_once('=').unwrap().1.to_string();
        if x == "--no-write" {
            write = false;
        } else if x.starts_with("--out=") {
            out = Some(val());
        } else if x.starts_with("--sides=") {
            sides = val().trim().parse().unwrap_or_else(|_| die(format!("error: {x}: not a number.")));
            if !BUILT.contains(&sides) {
                die(format!(
                    "error: --sides={sides}: the stacked chanter is built with {} sides only. chanter_model --sides={sides} still models it.",
                    BUILT.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" or ")
                ));
            }
        } else if x.starts_with("--plate=") {
            m::PLATE.set(num_arg(&x));
        } else if x.starts_with("--kerf=") {
            sf::set_kerf(&x);
        } else if x.starts_with("--page-to=") {
            page_to = Some(val());
        } else {
            die(format!(
                "error: {x} is not a flag this generator reads. It takes --sides=, --out=, --plate=, --kerf=, --page-to= and --no-write."
            ));
        }
    }
    if let Some(pt) = page_to {
        page(&pt);
        println!("  wrote {}", basename(&pt));
        return;
    }
    if write && out.is_none() {
        die("error: --out=DIR, or --no-write to run the checks only.");
    }
    let (parts, info) = build(sides);
    let sheets = pack(&parts);
    let res = checks(sides, &parts, &info, &sheets);
    let p = plate();
    let body = N_SEAT + info.n_bore;
    let capn = N_SOCKET + N_CHAMBER + N_TOP;
    println!(
        "practice chanter, {sides} sides ({}), the reference's holes: {body} body plates of {}mm ({}mm), {capn} cap and {} mouthpiece plates; two M{} rods cut {:.0}mm long; {}mm pins {}mm through the cap",
        names(sides),
        g(p),
        g(body as f64 * p),
        n_stem(),
        g(ROD),
        body as f64 * p + 2.0 * (0.5 + NUT_T) + ROD_SPARE,
        g(PIN),
        g(capn as f64 * p)
    );
    for i in 0..8 {
        let h = HOLES[i];
        let w = info.widths[i];
        let k = (info.pos[i] / p).floor() as i64 + 1;
        let how = if w >= m::W_SLOT { format!("slot {w:.2}mm wide") } else { format!("drill {:.2}mm at the V", m::drill(w)) };
        println!(
            "  {h:<3} bore plate {k:3} ({:5.1}mm down the bore)  {how}  ({:.2}mm round's area; the reference's {})",
            info.pos[i],
            info.holes[i].d,
            g(s::REF_HOLES[i].2)
        );
    }
    println!("  stack, top down: {}", stack_order(&parts));
    if print_checks(&res) {
        std::process::exit(1);
    }
    if write {
        write_sheets(&out.unwrap(), &sheets, &|k| format!("practice-chanter-{sides}-sided-sheet{k}-cut-files.svg"), &|k, n, sh| {
            svg_sheet(k, n, sh, sides)
        });
    }
}
