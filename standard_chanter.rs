//! Cut files for a Great Highland bagpipe practice chanter with a real
//! chanter's holes: where they are and about how big, on a square bore of
//! four long strips of ply, glued, all the way up to the collar the top
//! rests on.
//!
//!     target/release/standard_chanter --no-write                  # the checks
//!     target/release/standard_chanter --out=DIR                   # the sheet
//!     target/release/standard_chanter --joint=butt --out=DIR      # butt-jointed strips
//!     target/release/standard_chanter --plate=2.9 ...             # for ply as measured
//!     target/release/standard_chanter --kerf=0.18 ...             # for that kerf
//!
//! THE REFERENCE is a practice chanter measured from aero1016's model of the
//! one they own (Thingiverse thing 6658761): a 4.87mm round bore 238.5mm
//! from the sole to the reed's seat, front holes 25mm apart, the seat 5.57mm
//! across, and a collar 10mm under the seat that the top rests on. Its holes
//! are kept where they are. The bore here is square and bigger, SIDE across,
//! and the holes go through a 3mm strip instead of the reference's 5 to 7mm
//! of wall, so each hole's size is solved for its note to sound where the
//! reference's does, in chanter_model's terms; the strips are lengthened at
//! the sole for low G. The model cannot reproduce the reference's own scale
//! exactly (it is about 40 cents out at best, with REF_REED of reed), so it
//! is used only for the difference between the two chanters, not the pitch.
//!
//! From the sole up:
//!
//!   * the STRIPS, front, back and two sides, from the sole into the collar,
//!     finger-jointed (--joint=finger, the default) or butt-jointed, glued;
//!   * the FOOT, N_FOOT layers of strips lying flat round the strips'
//!     bottom, four to a layer, each layer turned the other way to the one
//!     below so no seam runs through; each strip's outer edge an arc of a
//!     FOOT_D circle, so it is widest mid-face and shorter to the corners;
//!   * the BAND, three 10mm strips glued round the strips' front and sides,
//!     under the collar, which thicken them where they meet it; none on the
//!     back, where the thumb reaches up past its hole, and the side pieces
//!     cut away there along a quarter circle up to the collar;
//!   * the COLLAR: a COLLAR_D plate with a square hole the strips' top
//!     stands in, glued to the strips and the band; then N_COLLAR plates
//!     with the reference's round bore, as wide as the cap, which rests on
//!     them;
//!   * the TENON: N_SEAT round plates with the reed's seat through them,
//!     which the cap slides over. The collar's round plates and the tenon
//!     press together on two pins;
//!   * the CAP, the reference's: a plain round tube, REF_CAP_D outside and
//!     REF_CAP_BORE inside, REF_CAP_LEN long, its top plate closed but for
//!     the blowing hole; glued, no pins. Then the mouthpiece, tapered as the
//!     reference's is (practice_chanters::mouthpiece).
//!
//! And a JIG, not part of the chanter: a base plate and N_FOOT rings with
//! the foot's circle cut through them, pinned into a cup. The chanter stands
//! in it on its sole while the foot's layers are glued round it, one at a
//! time; the circle centres each and the base keeps the foot flush with the
//! sole.
use crate::chanter_model::{self as m, plate, HOLES};
use crate::practice_chanters::{self as pc, cc, circle, fp, page_part, plate_m, polar, CPart, Checks, Parts, Sheet, C, CAPS, P2};
use crate::strip_chanter as sc;
use bore_tmm::models::{scan, Reed};
use lasermade_tools::die;
use lasermade_tools::py::{self, g};
use lasermade_tools::turn3d::{Mat, Num, Pt, F, I, J};
use square_flute::{self as sf, Cut};
use std::f64::consts::PI;
use std::sync::OnceLock;

// --- the reference chanter, measured (mm) ---------------------------------------
pub const REF_BORE: f64 = 4.87; // round, the sole to the seat
pub const REF_TOP: f64 = 238.5; // the seat's bottom, up from the sole
pub const REF_SEAT_D: f64 = 5.57;
pub const REF_SEAT_DEEP: f64 = 25.5;
pub const REF_COLLAR: f64 = 228.0; // the collar's bottom, up from the sole
/// each hole: up from the sole, across, and the wall it goes through
pub const REF_HOLES: [(&str, f64, f64, f64); 8] = [
    ("A", 43.02, 4.15, 7.4),
    ("B", 75.02, 3.95, 7.0),
    ("C", 100.02, 3.95, 6.6),
    ("D", 125.02, 3.95, 6.3),
    ("E", 150.02, 3.95, 5.9),
    ("F", 175.03, 3.35, 5.5),
    ("G", 195.43, 3.05, 5.2),
    ("hA", 207.43, 2.95, 5.1),
];
pub const REF_REED: f64 = 130.0; // the reed allowance that fits the reference's scale best
// --- this chanter -----------------------------------------------------------------
pub const SIDE: f64 = 5.5; // the square bore: the holes the reference's size
pub const N_COLLAR: usize = 2; // round-bore plates over the socket plate
pub const N_SEAT: usize = 9; // the tenon: 27mm of seat, the reference's 25.5
pub const BAND: f64 = 10.0; // the band's length, and the side pieces' quarter circle
pub const REF_FOOT_D: f64 = 25.4;
pub const REF_FOOT_H: f64 = 19.0;
pub const FOOT_D: f64 = 50.0; // the foot, round, as asked: wider and lower than the
pub const N_FOOT: usize = 2; // reference's, 6mm tall
pub const JIG: f64 = FOOT_D + 10.0; // the foot jig's square
pub const JIG_CLEAR: f64 = 0.3;
pub const REF_CAP_D: f64 = 22.2;
pub const REF_CAP_BORE: f64 = 17.6;
pub const REF_CAP_LEN: f64 = 122.0;
pub const TENON_D: f64 = REF_CAP_BORE - 2.0 * pc::CAP_CLEAR; // 17.0: the cap slides over it
pub const COLLAR_D: f64 = 25.4; // the collar's socket plate, over the band's corners

/// A bore for the reed kernel: round d across, or square.
#[derive(Clone, Debug)]
pub struct SBore {
    pub area: f64,
    pub a: f64,
    pub rh: f64,
}
impl SBore {
    pub fn new(d: f64, square: bool) -> SBore {
        let area = if square { py::powf(d / 1000.0, 2.0) } else { PI * py::powf(d / 2000.0, 2.0) };
        SBore { area, a: (area / PI).sqrt(), rh: d / 2000.0 }
    }
}

/// A hole: its name, mm down from the seat, across, and the wall.
#[derive(Clone, Debug)]
pub struct Hole {
    pub name: &'static str,
    pub z: f64,
    pub d: f64,
    pub wall: f64,
}

/// A frequency for each of NOTES.
#[derive(Clone, Copy, Debug)]
pub struct Notes(pub [f64; 9]);
impl Notes {
    pub fn get(&self, nt: &str) -> f64 {
        self.0[m::NOTES.iter().position(|n| *n == nt).unwrap()]
    }
}

/// The fingering's resonance nearest `near`.
pub fn sounding(b: &SBore, l: f64, holes: &[Hole], note: &str, near: f64) -> f64 {
    let fing = m::fingering(note);
    let mut order: Vec<&Hole> = holes.iter().collect();
    order.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
    let (mut x, mut seg, mut r, mut t, mut op) = (0.0, vec![], vec![], vec![], vec![]);
    for h in order {
        seg.push((h.z - x) / 1000.0);
        r.push(h.d / 2000.0);
        t.push(h.wall);
        op.push(fing.contains(&h.name) as u8);
        x = h.z;
    }
    let reed = Reed {
        c: m::c(),
        rho: m::RHO,
        rh0: b.rh,
        area0: b.area,
        l0: REF_REED / 1000.0,
        rh: b.rh,
        area: b.area,
        a: b.a,
        seg: &seg,
        r: &r,
        t: &t,
        open: &op,
        last: (l - x) / 1000.0,
        q: 0.25,
        j: 0.6133,
    };
    let root = scan(near, 1.3, 120, 40, |f| reed.y(f));
    if root.is_nan() {
        die(format!("{note}: no resonance near {near:.0}Hz"));
    }
    root
}

/// The reference's notes, in the model.
pub fn reference() -> Notes {
    static R: OnceLock<Notes> = OnceLock::new();
    *R.get_or_init(|| {
        let b = SBore::new(REF_BORE, false);
        let holes: Vec<Hole> = REF_HOLES.iter().map(|&(h, z, d, w)| Hole { name: h, z: REF_TOP - z, d, wall: w }).collect();
        let low_a = sounding(&b, REF_TOP, &holes, "low A", 255.0);
        Notes(std::array::from_fn(|i| sounding(&b, REF_TOP, &holes, m::NOTES[i], low_a * m::RATIOS[i])))
    })
}

pub fn bisect(f: &mut dyn FnMut(f64) -> f64, goal: f64, mut lo: f64, mut hi: f64, rising: bool) -> f64 {
    for _ in 0..36 {
        let mid = (lo + hi) / 2.0;
        if (f(mid) < goal) == rising {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// Each hole's size, solved through a bore b: hole i bisected between D_MIN
/// and d_max for its note, with what to say if even d_max is flat.
pub fn size_holes(b: &SBore, l: f64, holes: &mut [Hole], r: &Notes, d_max: f64, flat: &dyn Fn(&str) -> String) {
    for i in 0..8 {
        let nt = m::SETS[i];
        holes[i].d = d_max;
        if sounding(b, l, holes, nt, r.get(nt)) < r.get(nt) {
            die(flat(HOLES[i]));
        }
        let mut hc = holes.to_vec();
        holes[i].d = bisect(
            &mut |d| {
                hc[i].d = d;
                sounding(b, l, &hc, nt, r.get(nt))
            },
            r.get(nt),
            m::D_MIN,
            d_max,
            true,
        );
    }
}

/// The bore's length and each hole's size that sound the reference's notes,
/// the holes where the reference's are, through a strip wall.
pub fn solve() -> (Notes, Notes, f64, Vec<Hole>) {
    let r = reference();
    let t = plate();
    let b = SBore::new(SIDE, true);
    let mut holes: Vec<Hole> = REF_HOLES.iter().map(|&(h, z, d, _)| Hole { name: h, z: REF_TOP - z, d, wall: t }).collect();
    let mut l = REF_TOP;
    let d_max = SIDE - 0.35;
    for _ in 0..3 {
        l = bisect(&mut |x| sounding(&b, x, &holes, "low G", r.get("low G")), r.get("low G"), REF_TOP - 20.0, REF_TOP + 40.0, false);
        size_holes(&b, l, &mut holes, &r, d_max, &|h| format!("error: {h} is flat even {d_max:.2}mm across."));
    }
    let got = Notes(m::NOTES.map(|nt| sounding(&b, l, &holes, nt, r.get(nt))));
    (r, got, l, holes)
}

/// The tenon, the collar's round plates, its socket plate (sock_d round,
/// sock_cuts through it, at x0), the cap and the mouthpiece. Returns the
/// tenon's pins and the cap's plate count.
pub fn head(parts: &mut Parts, x_collar: f64, x0: f64, sock_cuts: Vec<C>, sock_d: f64) -> (Vec<P2>, usize) {
    let t = plate();
    let tenon = circle(0.0, 0.0, TENON_D / 2.0, 72);
    let r_pin = (REF_SEAT_D / 2.0 + TENON_D / 2.0) / 2.0;
    let pins = vec![polar(45.0, r_pin), polar(225.0, r_pin)];
    let pin_cut = || pins.iter().map(|&(x, y)| cc(x, y, (pc::PIN - pc::PIN_INTERF) / 2.0)).collect::<Vec<_>>();
    for k in 0..N_SEAT {
        let mut cuts = vec![cc(0.0, 0.0, REF_SEAT_D / 2.0)];
        cuts.extend(pin_cut());
        parts.add("tenon", &tenon, cuts, k as f64 * t, "seat", "", Some(TENON_D / 2.0));
    }
    let collar = circle(0.0, 0.0, REF_CAP_D / 2.0, 72);
    for k in 0..N_COLLAR {
        let mut cuts = vec![cc(0.0, 0.0, REF_BORE / 2.0)];
        cuts.extend(pin_cut());
        parts.add("collar", &collar, cuts, x_collar + k as f64 * t, "collar", "", Some(REF_CAP_D / 2.0));
    }
    parts.add("collar socket", &circle(0.0, 0.0, sock_d / 2.0, 72), sock_cuts, x0, "sock", "", Some(sock_d / 2.0));
    let n_cap = py::round(REF_CAP_LEN / t) as usize;
    let ring = circle(0.0, 0.0, REF_CAP_D / 2.0, 72);
    for k in 0..n_cap - 1 {
        parts.add(
            "cap",
            &ring,
            vec![cc(0.0, 0.0, REF_CAP_BORE / 2.0)],
            x_collar - (k + 1) as f64 * t,
            if k < N_SEAT { "socket" } else { "chamber" },
            "",
            Some(REF_CAP_D / 2.0),
        );
    }
    let x_top = x_collar - n_cap as f64 * t;
    parts.add("cap top", &ring, vec![cc(0.0, 0.0, pc::MOUTH_D / 2.0)], x_top, "top", "", Some(REF_CAP_D / 2.0));
    pc::mouthpiece(parts, x_top);
    (pins, n_cap)
}

/// (ok, what, detail): the seat takes the largest staple.
pub fn seat_check(t: f64) -> (bool, &'static str, String) {
    (
        REF_SEAT_D >= pc::STAPLE_MAX,
        "the seat takes any staple, held by its hemp as the reference's is",
        format!(
            "seat {}mm, {}mm deep (the reference's {}), over the largest staple, {}; the collar's bore {}mm, the reference's, is wider too, so nothing stops a staple but its hemp",
            g(REF_SEAT_D),
            g(N_SEAT as f64 * t),
            g(REF_SEAT_DEEP),
            g(pc::STAPLE_MAX),
            g(REF_BORE)
        ),
    )
}

/// The foot jig: a base and N_FOOT rings, a cup the foot is glued up in.
pub fn jig(parts: &mut Parts, r: f64, n_foot: usize, jig_side: f64, jig_clear: f64) {
    let jo = pc::ngon(4, jig_side / 2.0, 0.0);
    let pin_d = pc::PIN - pc::PIN_INTERF;
    let jr = (r + jig_clear / 2.0 + pin_d / 2.0 + pc::MIN_WEB + (jig_side / 2.0 - pin_d / 2.0 - pc::MIN_WEB) * 2f64.sqrt()) / 2.0;
    let jig_pins = || [polar(45.0, jr), polar(225.0, jr)].iter().map(|&(x, y)| cc(x, y, pin_d / 2.0)).collect::<Vec<_>>();
    parts.add("jig base", &jo, jig_pins(), 0.0, "jig", "the foot jig's base", None);
    for k in 0..n_foot {
        let mut cuts = vec![cc(0.0, 0.0, r + jig_clear / 2.0)];
        cuts.extend(jig_pins());
        parts.add("jig ring", &jo, cuts, 0.0, "jig", &format!("the foot jig, ring {}", k + 1), None);
    }
}

pub struct Built {
    pub r: Notes,
    pub got: Notes,
    pub l: f64,
    pub holes: Vec<Hole>,
    pub s: f64,
    pub t: f64,
    pub ls: f64,
    pub x0: f64,
    pub own: sc::Owners<&'static str>,
    pub strip_holes: Vec<(&'static str, f64, f64)>,
    pub seat_bottom: f64,
    pub a: f64,
    pub rr: f64,
    pub n_cap: usize,
}

pub fn build(joint: &str) -> (Vec<CPart>, Built) {
    let (r, got, l, holes) = solve();
    let t = plate();
    let s = SIDE;
    let mut parts = Parts::default();
    let x_collar = N_SEAT as f64 * t;
    let x0 = x_collar + N_COLLAR as f64 * t; // the strips' top, in the socket plate
    let seat_bottom = N_SEAT as f64 * t;
    let ls = l - (x0 - seat_bottom);
    let strip_holes: Vec<(&'static str, f64, f64)> =
        holes.iter().map(|h| (if h.name == "hA" { "back" } else { "front" }, h.z - (x0 - seat_bottom), h.d)).collect();
    let own = sc::owners(ls, &strip_holes, joint);
    for &(name, (lo, hi)) in sc::STRIPS.iter() {
        let runs = |e: &str| -> Vec<(f64, f64, bool)> { own.get(e).iter().map(|&(a, c, w, _)| (a, c, w == name)).collect() };
        let o = sc::strip_outline(&runs(lo), &runs(hi), s, t, None);
        let cuts: Vec<C> = strip_holes.iter().filter(|h| h.0 == name).map(|&(_, x, d)| cc(x, t + s / 2.0, d / 2.0)).collect();
        let q = parts.add(&format!("{name} strip"), &o, cuts, x0, "strip", &format!("the {name} strip"), None);
        q.strip = name;
    }
    // the band, front and sides
    let a = s / 2.0 + t;
    let band_x = x0 + t;
    let w = 2.0 * a;
    let arc: Vec<P2> = (0..31)
        .map(|i| {
            let q = PI / 2.0 * i as f64 / 30.0;
            (BAND * py::msin(q), w - BAND + BAND * py::mcos(q))
        })
        .collect();
    let mut side: Vec<Pt> = vec![[I(0), I(0)], [I(0), F(w)]];
    side.extend(fp(&arc[1..]));
    side.push([F(BAND), I(0)]);
    let right: Vec<Pt> = side.iter().map(|q| [q[0], F(w - q[1].f())]).rev().collect();
    let front: Vec<Pt> = vec![[I(0), I(0)], [F(BAND), I(0)], [F(BAND), F(2.0 * (a + t))], [I(0), F(2.0 * (a + t))]];
    for (name, o) in [("front", front), ("right", right), ("left", side)] {
        let q = parts.add_pts(&format!("{name} band"), o, vec![], band_x, "band", &format!("the {name} band piece"), None);
        q.strip = name;
    }
    // the foot: a pinwheel of flat strips each layer, each cut to the round foot
    let rr = FOOT_D / 2.0;
    let e = (rr * rr - a * a).sqrt();
    let (t0, t1) = (py::matan2(-a, -e), py::matan2(-e, a));
    let arc: Vec<P2> = (0..61)
        .map(|i| {
            let q = t0 + (t1 - t0) * i as f64 / 60.0;
            (rr * py::mcos(q), rr * py::msin(q))
        })
        .collect();
    for k in 0..N_FOOT {
        let sg = if k % 2 == 0 { 1.0 } else { -1.0 };
        for i in 0..4 {
            let mut pts: Vec<P2> = vec![(a * sg, -a), (-e * sg, -a)];
            pts.extend(arc[1..60].iter().map(|&(x, y)| (x * sg, y)));
            pts.push((a * sg, -e));
            let (c, sn) = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)][i];
            let mut pts: Vec<P2> = pts.iter().map(|&(x, y)| (c * x - sn * y, sn * x + c * y)).collect();
            if sg < 0.0 {
                pts.reverse();
            }
            parts.add("foot", &pts, vec![], x0 + ls - (k + 1) as f64 * t, "foot", &format!("foot layer {}", k + 1), None);
        }
    }
    jig(&mut parts, rr, N_FOOT, JIG, JIG_CLEAR);
    let tube = pc::ngon(4, s / 2.0 + t, 0.0);
    let (_, n_cap) = head(&mut parts, x_collar, x0, vec![C::Poly(tube)], COLLAR_D);
    (parts.v, Built { r, got, l, holes, s, t, ls, x0, own, strip_holes, seat_bottom, a, rr, n_cap })
}

/// How many notes are off the reference's, at the most, in cents.
pub fn worst_cents(got: &Notes, r: &Notes) -> f64 {
    sf::fmax(&(0..9).map(|i| (1200.0 * m::mlog2(got.0[i] / r.0[i])).abs()).collect::<Vec<_>>())
}

/// The jig's check, shared with ghb-chanter.
pub fn jig_check(note: &mut dyn FnMut(bool, &str, String), parts: &[CPart], rr: f64, n_foot: usize, t: f64, jig_clear: f64) {
    let jig: Vec<&CPart> = parts.iter().filter(|p| p.kind == "jig").collect();
    let rings: Vec<f64> = jig.iter().flat_map(|p| p.p.cuts.iter()).filter_map(|c| if let Cut::Circle(_, _, r) = c { Some(*r) } else { None }).filter(|r| *r > 5.0).collect();
    note(
        jig.len() == n_foot + 1 && rings.len() == n_foot && rings.iter().all(|r| (r - (rr + jig_clear / 2.0)).abs() < 1e-9),
        "the foot jig holds every layer on the foot's circle",
        format!(
            "a base and {n_foot} rings, {}mm deep, the circle {:.1}mm for the foot's {}mm",
            g(n_foot as f64 * t),
            2.0 * rr + jig_clear,
            g(2.0 * rr)
        ),
    );
}

/// Each corner one strip's all the way: covered end to end, without gaps.
pub fn covered<W>(own: &[Vec<sc::Run<W>>], ls: f64) -> bool {
    own.iter().all(|r| {
        r[0].0.abs() < 1e-9 && (r[r.len() - 1].1 - ls).abs() < 1e-9 && r.windows(2).all(|w| (w[0].1 - w[1].0).abs() < 1e-9)
    })
}

fn checks(parts: &[CPart], info: &Built, sheets: &[Sheet], joint: &str) -> Checks {
    let mut out: Checks = vec![];
    let mut note = |ok: bool, what: &str, detail: String| out.push((ok, what.to_string(), detail));
    let (s, t, ls) = (info.s, info.t, info.ls);
    let worst = worst_cents(&info.got, &info.r);
    note(
        worst <= 1.0,
        "every note sounds where the reference's does, in the model",
        format!("within {worst:.2} cents; the reference's low A {:.0}Hz in the model", info.r.get("low A")),
    );
    let moved = sf::fmax(&(0..8).map(|i| ((REF_TOP - REF_HOLES[i].1) - info.holes[i].z).abs()).collect::<Vec<_>>());
    note(
        moved < 1e-9,
        "every hole is where the reference's is",
        (0..8).map(|i| format!("{} {:.1}", HOLES[i], REF_TOP - REF_HOLES[i].1)).collect::<Vec<_>>().join(", ") + "mm under the seat",
    );
    let ds: Vec<f64> = info.holes.iter().map(|h| h.d).collect();
    let refd: Vec<f64> = REF_HOLES.iter().map(|h| h.2).collect();
    note(
        ds.iter().all(|&d| m::D_MIN <= d && d <= s - 0.35 + 1e-9),
        "every hole cuts round, and fits the bore's face",
        (0..8).map(|i| format!("{} {:.2}", HOLES[i], ds[i])).collect::<Vec<_>>().join(", ")
            + &format!("mm, against the reference's {:.2} to {:.2}; the face {}mm", sf::fmin(&refd), sf::fmax(&refd), g(s)),
    );
    let &(_, hz, _) = info.strip_holes.iter().find(|h| h.0 == "back").unwrap();
    let room = hz - t;
    let ref_room = REF_COLLAR - REF_HOLES[7].1;
    let backs = parts.iter().filter(|p| p.kind == "band" && p.strip == "back").count();
    note(
        backs == 0 && room >= ref_room - 1e-9,
        "the thumb has the reference's room above its hole",
        format!("no band on the back; {room:.1}mm from the hole's centre to the collar, the reference {ref_room:.1}mm"),
    );
    let (a, rr) = (info.a, info.rr);
    let feet: Vec<&CPart> = parts.iter().filter(|p| p.kind == "foot").collect();
    let mut bad = 0;
    for k in 0..N_FOOT {
        let layer: Vec<Vec<P2>> = feet.iter().filter(|p| (p.x - (info.x0 + ls - (k + 1) as f64 * t)).abs() < 1e-9).map(|p| p.of()).collect();
        for i in 0..50 {
            for j in 0..50 {
                let q = (-rr + (i as f64 + 0.5) * 2.0 * rr / 50.0, -rr + (j as f64 + 0.5) * 2.0 * rr / 50.0);
                let r = py::hypot(q.0, q.1);
                if (r - rr).abs() < 0.05 {
                    continue;
                }
                let want = if py::max(q.0.abs(), q.1.abs()) < a || r > rr { 0 } else { 1 };
                bad += (layer.iter().filter(|o| pc::inside(q, o)).count() != want) as usize;
            }
        }
    }
    let widest = sf::fmax(&feet.iter().flat_map(|p| p.of()).map(|v| py::hypot(v.0, v.1)).collect::<Vec<_>>());
    let mid = rr - a;
    let lowest = sf::fmax(&info.strip_holes.iter().map(|&(_, x, d)| x + d / 2.0).collect::<Vec<_>>());
    let clear = (ls - N_FOOT as f64 * t) - lowest;
    note(
        bad == 0 && feet.len() == 4 * N_FOOT && clear >= pc::MIN_WEB,
        "the foot is round, and clear of the low A hole",
        format!(
            "{:.1}mm round and {}mm tall, against the reference's {}mm and {}mm; {N_FOOT} layers of 4 strips, {mid:.2}mm wide mid-face; {clear:.1}mm under the low A hole",
            2.0 * widest,
            g(N_FOOT as f64 * t),
            g(REF_FOOT_D),
            g(REF_FOOT_H)
        ),
    );
    jig_check(&mut note, parts, rr, N_FOOT, t, JIG_CLEAR);
    let mut ok = true;
    for &(st, x, d) in &info.strip_holes {
        for c in sc::corners_of(st) {
            for &(a, e, w, _) in info.own.get(c) {
                if a < x + d / 2.0 + pc::MIN_WEB && e > x - d / 2.0 - pc::MIN_WEB && w != st {
                    ok = false;
                }
            }
        }
    }
    note(
        ok,
        "every hole keeps ply all round it",
        if joint == "finger" { format!("its strip fingered both sides, {}mm past it along", g(pc::MIN_WEB)) } else { "the front and back are the full width".into() },
    );
    let own = &info.own;
    let all: Vec<Vec<sc::Run<&str>>> = sc::CORNERS.iter().map(|c| own.get(c).clone()).collect();
    let runs: Vec<f64> = all.iter().flatten().filter(|r| !r.3).map(|r| r.1 - r.0).collect();
    note(
        covered(&all, ls) && (runs.is_empty() || sf::fmin(&runs) >= sc::MIN_FINGER - 1e-9),
        "every corner is one strip's, all the way",
        format!("{joint} joints") + &if joint == "finger" { format!("; the shortest finger {:.1}mm", sf::fmin(&runs)) } else { String::new() },
    );
    let socket: Vec<&CPart> = parts.iter().filter(|p| p.kind == "sock").collect();
    let into = socket.len() == 1 && (socket[0].x - info.x0).abs() < 1e-9;
    note(
        into && (ls + info.x0 - info.seat_bottom - info.l).abs() < 1e-9,
        "the strips run from the sole into the collar",
        format!(
            "{ls:.1}mm, their top {}mm into its socket plate; the bore {:.1}mm, {:+.1}mm on the reference's, at the sole, for low G",
            g(t),
            info.l,
            info.l - REF_TOP
        ),
    );
    let web = pc::pin_web(parts, Some(&["seat", "collar", "sock", "socket", "chamber", "top", "stem", "jig"]), &[(pc::PIN - pc::PIN_INTERF) / 2.0], false);
    note(web >= pc::MIN_WEB, "every pin keeps ply between it and every cut", format!("{web:.2}mm at the least, against {}", g(pc::MIN_WEB)));
    let (ok, what, detail) = seat_check(t);
    note(ok, what, detail);
    let caps: Vec<&CPart> = parts.iter().filter(|p| ["socket", "chamber", "top"].contains(&p.kind)).collect();
    let rings: Vec<&&CPart> = caps.iter().filter(|p| p.kind != "top").collect();
    let band_r = py::hypot(info.a + t, info.a + t);
    let ok = caps.len() == info.n_cap
        && caps.iter().all(|p| (p.round.unwrap() - REF_CAP_D / 2.0).abs() < 1e-9)
        && rings.iter().all(|p| p.p.cuts.iter().all(|c| matches!(c, Cut::Circle(_, _, r) if *r == REF_CAP_BORE / 2.0)))
        && !caps.iter().any(|p| p.p.cuts.len() > 1)
        && (sf::fmax(&rings.iter().map(|p| p.x).collect::<Vec<_>>()) - (info.x0 - (N_COLLAR + 1) as f64 * t)).abs() < 1e-9
        && COLLAR_D / 2.0 > band_r;
    note(
        ok,
        "the cap is the reference's, and slides over the tenon onto the collar",
        format!(
            "{} plates, {}mm long (the reference's {}), {}mm round, {}mm bore, no pins; over a {}mm tenon, {}mm clear all round; rests on the collar, as wide as it; the collar's socket plate {}mm, over the band's corners at {:.1}mm",
            caps.len(),
            g(caps.len() as f64 * t),
            g(REF_CAP_LEN),
            g(REF_CAP_D),
            g(REF_CAP_BORE),
            g(TENON_D),
            g(pc::CAP_CLEAR),
            g(COLLAR_D),
            2.0 * band_r
        ),
    );
    let (ok, d) = pc::mouthpiece_check(parts);
    note(ok, "the mouthpiece tapers as a real practice chanter's does", d);
    pc::sheet_checks(&mut note, sheets, parts.len(), "");
    out
}

pub fn label(joint: &str) -> String {
    format!("standard-strips-{joint}-joint")
}

pub fn svg_sheet(k: usize, nsh: usize, sh: &Sheet, joint: &str) -> String {
    let txt = pc::svg_sheet(k, nsh, sh, 4);
    let what = format!("Practice chanter, a measured chanter's holes on a {}mm square bore of four {joint}-jointed strips", g(SIDE));
    txt.replace("Practice chanter, 4-sided bore", &what)
        .replace("cuts the bores, seats and pin holes first; then black", "cuts the bores, seats, finger and pin holes first; then black")
}

/// Where a part sits: strips as strip_chanter places them; the band's
/// pieces round the front and sides.
pub fn placed(p: &CPart, info: &Built) -> Mat {
    let (s, t, a) = (info.s, info.t, info.a);
    if p.kind == "strip" {
        return sc::placed(p, s, t, info.x0);
    }
    if p.kind != "band" {
        return plate_m(p.x);
    }
    let (b, x) = (a + t, p.x);
    let r = |v: [Num; 4]| v.to_vec();
    match p.strip {
        "front" => vec![r([I(1), I(0), I(0), F(x)]), r([I(0), I(0), I(1), F(-b)]), r([I(0), I(-1), I(0), F(b)]), r([I(0), I(0), I(0), I(1)])],
        "right" => vec![r([I(1), I(0), I(0), F(x)]), r([I(0), I(-1), I(0), F(a)]), r([I(0), I(0), I(-1), F(-a)]), r([I(0), I(0), I(0), I(1)])],
        _ => vec![r([I(1), I(0), I(0), F(x)]), r([I(0), I(1), I(0), F(-a)]), r([I(0), I(0), I(1), F(a)]), r([I(0), I(0), I(0), I(1)])],
    }
}

/// A strip's explode direction, out from the bore.
pub fn strip_ex(name: &str) -> [Num; 3] {
    let d = match name {
        "front" => (0.0, -1.0, 0.0),
        "back" => (0.0, 1.0, 0.0),
        "left" => (0.0, 0.0, 1.0),
        _ => (0.0, 0.0, -1.0),
    };
    [I(0), F(d.1 * 0.12), F(d.2 * 0.12)]
}

/// The chanter for practice_chanters' 3D page, finger-jointed.
pub fn design3d() -> J {
    let (parts, info) = build("finger");
    let mut xs: Vec<f64> = parts.iter().map(|p| p.x).collect();
    xs.push(info.x0 + info.ls);
    let mid = (sf::fmin(&xs) + sf::fmax(&xs)) / 2.0;
    let out: Vec<J> = parts
        .iter()
        .filter(|p| p.kind != "jig")
        .map(|p| {
            let cap = CAPS.contains(&p.kind);
            let mut ex = if p.kind == "strip" || p.kind == "band" {
                strip_ex(p.strip)
            } else {
                [F((p.x - mid) / 60.0 - if cap { 1.0 } else { 0.0 }), I(0), I(0)]
            };
            let wood = if p.kind == "strip" { "dark" } else if cap { "pale" } else { "ply" };
            if p.kind == "foot" {
                ex = [F((p.x - mid) / 60.0), I(0), I(0)];
            }
            page_part(p, &placed(p, &info), ex, wood)
        })
        .collect();
    pc::design_j(out)
}

pub fn main() {
    let (mut out, mut write, mut joint) = (None::<String>, true, "finger".to_string());
    for x in std::env::args().skip(1) {
        let val = || x.split_once('=').unwrap().1.to_string();
        if x == "--no-write" {
            write = false;
        } else if x.starts_with("--out=") {
            out = Some(val());
        } else if x.starts_with("--joint=") {
            joint = val();
            if joint != "finger" && joint != "butt" {
                die(format!("error: --joint={joint}: finger or butt."));
            }
        } else if x.starts_with("--plate=") {
            m::PLATE.set(pc::num_arg(&x));
        } else if x.starts_with("--kerf=") {
            sf::set_kerf(&x);
        } else {
            die(format!("error: {x} is not a flag this generator reads. It takes --joint=, --out=, --plate=, --kerf= and --no-write."));
        }
    }
    if write && out.is_none() {
        die("error: --out=DIR, or --no-write to run the checks only.");
    }
    let (parts, info) = build(&joint);
    let sheets = pc::pack(&parts);
    let res = checks(&parts, &info, &sheets, &joint);
    let t = info.t;
    println!(
        "practice chanter, a measured chanter's holes on a {}mm square bore of four {joint}-jointed strips: strips {:.1}mm long, {}mm outside; body {:.1}mm, sole to tenon top; {} cap plates",
        g(SIDE),
        info.ls,
        g(SIDE + 2.0 * t),
        info.ls + info.x0,
        parts.iter().filter(|p| CAPS.contains(&p.kind)).count()
    );
    for i in 0..8 {
        let h = HOLES[i];
        let d_ref = REF_HOLES[i].2;
        let (z, d) = (info.holes[i].z, info.holes[i].d);
        let up = info.l - z;
        println!(
            "  {h:<3} {} strip, {up:6.1}mm up from the sole ({z:5.1}mm under the seat, as the reference's), {d:.2}mm round (the reference {d_ref:.2})",
            if h == "hA" { "back " } else { "front" }
        );
    }
    println!(
        "  top down: {N_SEAT} tenon, {N_COLLAR} collar, 1 collar socket over the strips; the band, {}mm, front and sides, under it; at the sole the round foot, {N_FOOT} layers of 4 strips",
        g(BAND)
    );
    if pc::print_checks(&res) {
        std::process::exit(1);
    }
    if write {
        let lb = label(&joint);
        pc::write_sheets(&out.unwrap(), &sheets, &|k| format!("practice-chanter-{lb}-sheet{k}-cut-files.svg"), &|k, n, sh| svg_sheet(k, n, sh, &joint));
    }
}

