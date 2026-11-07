//! Cut files for a Great Highland bagpipe practice chanter whose square bore
//! is four long strips of ply, glued, for the bottom four fifths of the body,
//! and a stack of plates (the cookies) above that, under the plate chanters'
//! cap and mouthpiece.
//!
//!     target/release/strip_chanter --no-write                  # the checks
//!     target/release/strip_chanter --out=DIR                   # the sheet
//!     target/release/strip_chanter --joint=butt --out=DIR      # butt-jointed strips
//!     target/release/strip_chanter --plate=2.9 ...             # for ply as measured
//!     target/release/strip_chanter --kerf=0.18 ...             # for that kerf
//!     target/release/strip_chanter --switch=0.75 ...           # cookies from 3/4 up instead
//!
//! From the sole up:
//!
//!   * the STRIPS: front, back and two sides, PLATE thick, round the square
//!     bore. The finger holes are round holes cut in the front strip, the
//!     thumb's in the back, anywhere along it, so no hole waits on a plate.
//!     --joint=finger (the default) joints every corner with fingers, which
//!     find their own places as the strips go together; each strip always has
//!     fingers along one edge or the other, and round a hole its strip has
//!     them along both, so the ply each side of the hole is never a notch.
//!     --joint=butt makes the front and back the full width and the sides as
//!     narrow as the bore, glued between them.
//!   * the COOKIES, from just above the high A hole (or --switch= of the way
//!     up the body): N_SOCK plates with a square hole the strips' outside,
//!     glued over the strips' top; then plates with a 4mm round hole, the
//!     square's area, so no note moves; then the reed's seat. Over 50 plates
//!     are too many to press onto pins, so they are clamped on two M2 rods,
//!     as the plate chanters' bodies are.
//!   * the CAP, the plate chanters' own, which slides over the cookies and
//!     rests on the two top nuts.
//!
//! The strips are glued, and into the socket: four strips cannot be pressed
//! together the way plates can. The cookies are clamped, the cap press-fit,
//! and shellac seals every seam and the bore.
use crate::chanter_model::{self as m, plate, HOLES};
use crate::practice_chanters::{self as pc, cc, CPart, Checks, Parts, Sheet, C, P2};
use lasermade_tools::die;
use lasermade_tools::py::{self, g};
use lasermade_tools::turn3d::{Mat, Num, F, I};
use square_flute::{self as sf, kerf};
use std::f64::consts::PI;
use std::sync::Mutex;

pub static SWITCH: Mutex<Option<f64>> = Mutex::new(None); // the cookies' start up the body; None: above high A
pub const N_SOCK: usize = 5; // cookie plates glued over the strips' top
pub const COOKIE: f64 = 20.0; // the cookies' square, mm: room for the rods' nuts
pub const FINGER: f64 = 10.0; // a finger's length, about
pub const MIN_FINGER: f64 = 4.0; // no finger or gap shorter than this
pub const THUMB_ROOM: f64 = 9.0; // high A's centre to the cookies
pub const G_GAP: f64 = 16.0; // F to G: a thin wall wants G nearer F
pub const SEAT_D: f64 = 5.5;
pub const N_SEAT: usize = 6;
pub const CHAMBER_D: f64 = 14.0;

/// the corners, each owned by one strip at a time: front-left, front-right,
/// back-left, back-right; and each strip's two edges, its u = 0 edge first
pub const CORNERS: [&str; 4] = ["FL", "FR", "KL", "KR"];
pub const STRIPS: [(&str, (&str, &str)); 4] =
    [("front", ("FL", "FR")), ("back", ("KR", "KL")), ("left", ("FL", "KL")), ("right", ("KR", "FR"))];

pub fn corners_of(st: &str) -> [&'static str; 2] {
    let (_, (a, b)) = STRIPS.iter().find(|s| s.0 == st).unwrap();
    [a, b]
}

/// A run of one owner along an edge: (x0, x1, owner, guarded).
pub type Run<W> = (f64, f64, W, bool);

/// Each corner's runs, by name.
#[derive(Clone, Debug)]
pub struct Owners<W>(pub Vec<(&'static str, Vec<Run<W>>)>);
impl<W> Owners<W> {
    pub fn get(&self, c: &str) -> &Vec<Run<W>> {
        &self.0.iter().find(|e| e.0 == c).unwrap().1
    }
}

/// The tune, with the cookies' round bore in place: its length depends on
/// the body's, which the tune sets, so twice round.
pub fn solve() -> (m::Bore, f64, [f64; 8], [f64; 8], usize) {
    let t = plate();
    let mut top = 0.0;
    let switch = *SWITCH.lock().unwrap();
    for _ in 0..4 {
        let gg = m::G_GAP.get();
        m::G_GAP.set(G_GAP);
        let r = m::design(4, top, Some(t));
        m::G_GAP.set(gg);
        let (b, l, pos, widths) = r.unwrap_or_else(|e| die(e));
        let body = N_SEAT as f64 * t + l;
        let n_round: i64 = match switch {
            None => ((pos[m::hi("hA")] - THUMB_ROOM) / t).floor() as i64 - N_SOCK as i64,
            Some(sw) => py::round((1.0 - sw) * body / t) - N_SEAT as i64 - N_SOCK as i64,
        };
        if n_round < 1 {
            die(format!("error: --switch={} leaves no room for round bore under the seat.", g(switch.unwrap_or(f64::NAN))));
        }
        if n_round as f64 * t == top {
            return (b, l, pos, widths, n_round as usize);
        }
        top = n_round as f64 * t;
    }
    die("error: the cookies' length did not settle.")
}

/// Joins runs with one owner, then gives every unguarded run shorter than
/// MIN_FINGER to its neighbours, shortest first.
pub fn tidy<W: Copy + PartialEq>(iv: Vec<Run<W>>) -> Vec<Run<W>> {
    fn join<W: Copy + PartialEq>(iv: Vec<Run<W>>) -> Vec<Run<W>> {
        let mut res: Vec<Run<W>> = vec![];
        for (x0, x1, w, gd) in iv {
            if let Some(l) = res.last_mut().filter(|l| l.2 == w) {
                l.1 = x1;
                l.3 = l.3 || gd;
            } else {
                res.push((x0, x1, w, gd));
            }
        }
        res
    }
    let mut iv = join(iv);
    loop {
        let short: Vec<usize> = (0..iv.len()).filter(|&k| iv[k].1 - iv[k].0 < MIN_FINGER - 1e-9 && !iv[k].3).collect();
        if short.is_empty() {
            return iv;
        }
        let mut k = short[0];
        for &j in &short[1..] {
            if iv[j].1 - iv[j].0 < iv[k].1 - iv[k].0 {
                k = j;
            }
        }
        let nb = if k > 0 { iv[k - 1] } else { iv[k + 1] };
        iv[k].2 = nb.2;
        iv = join(iv);
    }
}

/// Each corner's owner along the strips, from the strips' top. In turn each
/// strip has fingers along one edge and the other; round each hole its strip
/// owns both corners, out to MIN_WEB past the hole, and the strip opposite
/// gives way.
pub fn owners(ls: f64, holes: &[(&'static str, f64, f64)], joint: &str) -> Owners<&'static str> {
    if joint == "butt" {
        let own = |c: &str| if c.starts_with('F') { "front" } else { "back" };
        return Owners(CORNERS.iter().map(|&c| (c, vec![(0.0, ls, own(c), false)])).collect());
    }
    let n = py::round(ls / FINGER).max(1) as usize;
    let mut cuts: Vec<f64> = (0..=n).map(|i| ls * i as f64 / n as f64).collect();
    let guard: Vec<(f64, f64, &str)> = holes.iter().map(|&(st, x, d)| (x - d / 2.0 - pc::MIN_WEB, x + d / 2.0 + pc::MIN_WEB, st)).collect();
    for &(a, b, _) in &guard {
        cuts.push(a);
        cuts.push(b);
    }
    cuts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cuts.dedup();
    let base = |c: &str| match c {
        "FL" => ["front", "left"],
        "FR" => ["right", "front"],
        "KL" => ["left", "back"],
        _ => ["back", "right"],
    };
    let mut out = vec![];
    for c in CORNERS {
        let mut iv = vec![];
        for w in cuts.windows(2) {
            let (x0, x1) = (w[0], w[1]);
            let mid = (x0 + x1) / 2.0;
            let i = ((mid / ls * n as f64) as usize).min(n - 1);
            let mut who = base(c)[i % 2];
            let gd = guard.iter().find(|&&(a, b, _)| a <= mid && mid <= b).map(|e| e.2);
            if gd == Some("front") {
                who = if c == "FL" || c == "FR" { "front" } else if c == "KL" { "left" } else { "right" };
            } else if gd == Some("back") {
                who = if c == "KL" || c == "KR" { "back" } else if c == "FL" { "left" } else { "right" };
            }
            iv.push((x0, x1, who, gd.is_some()));
        }
        out.push((c, tidy(iv)));
    }
    Owners(out)
}

/// A strip from its top, x = 0, to the sole; u across it, the bore's face
/// from u = t to t + s, a finger out to 0 or s + 2t where the strip owns
/// that corner. lo, hi: its edges' runs, [(x0, x1, owns)]. cut: how far in
/// from 0 or s + 2t it stops where it does not, t if None.
pub fn strip_outline(lo: &[(f64, f64, bool)], hi: &[(f64, f64, bool)], s: f64, t: f64, cut: Option<f64>) -> Vec<P2> {
    let edge = |runs: &[(f64, f64, bool)], out_u: f64, in_u: f64| -> Vec<P2> {
        let mut pts = vec![];
        for &(x0, x1, owns) in runs {
            let u = if owns { out_u } else { in_u };
            pts.push((x0, u));
            pts.push((x1, u));
        }
        pts
    };
    let c = cut.unwrap_or(t);
    let mut pts = edge(lo, 0.0, c);
    let right = edge(hi, s + 2.0 * t, s + 2.0 * t - c);
    pts.extend(right.into_iter().rev());
    let mut out: Vec<P2> = vec![];
    for p in pts {
        if let Some(l) = out.last() {
            if (l.0 - p.0).abs() < 1e-9 && (l.1 - p.1).abs() < 1e-9 {
                continue;
            }
        }
        out.push(p);
    }
    let n = out.len();
    let mut clean = vec![];
    for i in 0..n {
        let (a, p, c) = (out[(i + n - 1) % n], out[i], out[(i + 1) % n]);
        if ((p.0 - a.0) * (c.1 - p.1) - (p.1 - a.1) * (c.0 - p.0)).abs() < 1e-9 {
            continue;
        }
        clean.push(p);
    }
    clean
}

/// The cookies' rods on their diagonal, midway between the nearest they can
/// come to the socket's corner and to the outline's.
pub fn rod_radius(tube: f64) -> f64 {
    let inner = tube / 2.0 * 2f64.sqrt() + pc::ROD_HOLE / 2.0 + pc::MIN_WEB;
    let outer = (COOKIE / 2.0 - pc::ROD_HOLE / 2.0 - pc::MIN_WEB) * 2f64.sqrt();
    (inner + outer) / 2.0
}

pub struct Built {
    pub b: m::Bore,
    pub l: f64,
    pub pos: [f64; 8],
    pub widths: [f64; 8],
    pub n_round: usize,
    pub s: f64,
    pub t: f64,
    pub ls: f64,
    pub x0: f64,
    pub holes: Vec<(&'static str, f64, f64)>,
    pub own: Owners<&'static str>,
    pub pins: Vec<P2>,
    pub round_d: f64,
}

pub fn build(joint: &str) -> (Vec<CPart>, Built) {
    let (b, l, pos, widths, n_round) = solve();
    let t = plate();
    let s = 2.0 * m::inradius(m::BORE_AREA, 4);
    let top = n_round as f64 * t;
    let ls = l - top;
    let x0 = (N_SEAT + n_round) as f64 * t;
    let holes: Vec<(&'static str, f64, f64)> =
        (0..8).map(|i| (if HOLES[i] == "hA" { "back" } else { "front" }, pos[i] - top, widths[i])).collect();
    let own = owners(ls, &holes, joint);
    let mut parts = Parts::default();
    for &(name, (lo, hi)) in STRIPS.iter() {
        let runs = |e: &str| -> Vec<(f64, f64, bool)> { own.get(e).iter().map(|&(a, c, w, _)| (a, c, w == name)).collect() };
        let o = strip_outline(&runs(lo), &runs(hi), s, t, None);
        let cuts: Vec<C> = holes.iter().filter(|h| h.0 == name).map(|&(_, x, d)| cc(x, t + s / 2.0, d / 2.0)).collect();
        let q = parts.add(&format!("{name} strip"), &o, cuts, x0, "strip", &format!("the {name} strip"), None);
        q.strip = name;
    }
    let outer = pc::ngon(4, COOKIE / 2.0, 0.0);
    let r_rod = rod_radius(s + 2.0 * t);
    let pins = vec![pc::polar(45.0, r_rod), pc::polar(225.0, r_rod)];
    let pin_cut = || pins.iter().map(|&(x, y)| cc(x, y, pc::ROD_HOLE / 2.0)).collect::<Vec<_>>();
    let round_d = 2.0 * (m::BORE_AREA / PI).sqrt();
    for k in 0..N_SEAT {
        let mut cuts = vec![cc(0.0, 0.0, SEAT_D / 2.0)];
        cuts.extend(pin_cut());
        parts.add("seat", &outer, cuts, k as f64 * t, "seat", "", None);
    }
    for k in 0..n_round {
        let mut cuts = vec![cc(0.0, 0.0, round_d / 2.0)];
        cuts.extend(pin_cut());
        parts.add("round bore", &outer, cuts, (N_SEAT + k) as f64 * t, "round", "", None);
    }
    let sock = pc::ngon(4, (s + 2.0 * t) / 2.0, 0.0);
    for k in 0..N_SOCK {
        let mut cuts = vec![C::Poly(sock.clone())];
        cuts.extend(pin_cut());
        parts.add("strip socket", &outer, cuts, x0 + k as f64 * t, "sock", "", None);
    }
    pc::cap(&mut parts, &outer, 4, None, Some(CHAMBER_D));
    (parts.v, Built { b, l, pos, widths, n_round, s, t, ls, x0, holes, own, pins, round_d })
}

fn checks(parts: &[CPart], info: &Built, sheets: &[Sheet], joint: &str) -> Checks {
    let mut out: Checks = vec![];
    let mut note = |ok: bool, what: &str, detail: String| out.push((ok, what.to_string(), detail));
    let (s, t, ls, l) = (info.s, info.t, info.ls, info.l);
    let rep = m::report(&info.b, l, &info.pos, &info.widths).unwrap_or_else(|e| die(e));
    let worst = sf::fmax(&rep.iter().map(|r| r.2.abs()).collect::<Vec<_>>());
    note(worst <= 5.0, "every note is in tune, in the model", format!("within {worst:.1} cents, low G among them: the strips are any length"));
    note(
        (s * s - m::BORE_AREA).abs() < 1e-6 && (PI * py::powf(info.round_d, 2.0) / 4.0 - m::BORE_AREA).abs() < 1e-6,
        "the bore is a 4mm round bore's area, square and round",
        format!("{:.2}mm^2: the strips' square {s:.2}mm, the cookies' circle {:.2}mm", s * s, info.round_d),
    );
    let body = N_SEAT as f64 * t + l;
    let start = info.x0 + N_SOCK as f64 * t;
    let up = (body - start) / body;
    let ha = N_SEAT as f64 * t + info.pos[m::hi("hA")];
    let switch = *SWITCH.lock().unwrap();
    let (ok, asked) = match switch {
        None => (THUMB_ROOM <= ha - start && ha - start < THUMB_ROOM + t, format!("just above high A: its centre {:.1}mm under them", ha - start)),
        Some(sw) => (
            (up - sw).abs() <= t / body / 2.0 + 1e-9 && ha - start >= THUMB_ROOM,
            format!("asked {}; high A's centre {:.1}mm under them", g(sw), ha - start),
        ),
    };
    note(
        ok,
        "the cookies start where asked",
        format!("{up:.3} of the way up the body, {:.0}mm of {body:.0}mm, {asked}; the strips run {}mm on into the socket", body - start, g(N_SOCK as f64 * t)),
    );
    note(
        (info.n_round as f64 * t + ls - l).abs() < 1e-9,
        "the bore is the cookies and the strips",
        format!("{} round plates, {}mm, and {ls:.1}mm of strips: {l:.1}mm", info.n_round, g(info.n_round as f64 * t)),
    );
    let hs = &info.holes;
    let ds: Vec<f64> = hs.iter().map(|h| h.2).collect();
    note(
        ds.iter().all(|&d| m::D_MIN - 1e-9 <= d && d <= m::D_MAX + 1e-9),
        "every hole cuts round, and fits the bore's face",
        format!("{:.2} to {:.2}mm across, the face {s:.2}mm", sf::fmin(&ds), sf::fmax(&ds)),
    );
    let socket = N_SOCK as f64 * t;
    note(
        hs.iter().all(|&(_, x, d)| x - d / 2.0 - pc::MIN_WEB >= socket),
        "every hole is below the socket",
        format!("the highest {:.1}mm under it", sf::fmin(&hs.iter().map(|&(_, x, d)| x - d / 2.0).collect::<Vec<_>>()) - socket),
    );
    let mut ok = true;
    for &(st, x, d) in hs {
        for c in corners_of(st) {
            for &(a, e, w, _) in info.own.get(c) {
                if a < x + d / 2.0 + pc::MIN_WEB && e > x - d / 2.0 - pc::MIN_WEB && w != st {
                    ok = false;
                }
            }
        }
    }
    let across = (2.0 * t + s - m::D_MAX) / 2.0;
    note(ok, "every hole keeps ply all round it", format!("its strip fingered both sides, {}mm past it along; {across:.2}mm across at the widest hole", g(pc::MIN_WEB)));
    let all: Vec<Vec<Run<&str>>> = CORNERS.iter().map(|c| info.own.get(c).clone()).collect();
    let free: Vec<f64> = all.iter().flatten().filter(|r| !r.3).map(|r| r.1 - r.0).collect();
    let cover = crate::standard_chanter::covered(&all, ls);
    let least = if free.is_empty() { f64::INFINITY } else { sf::fmin(&free) };
    note(
        cover && least >= MIN_FINGER - 1e-9,
        "every corner is one strip's, all the way",
        format!("{joint} joints; ")
            + &if joint == "finger" {
                format!("{} runs, the shortest unguarded {least:.1}mm, against {}", all.iter().map(|r| r.len()).sum::<usize>(), g(MIN_FINGER))
            } else {
                "front and back own every corner".into()
            },
    );
    let strips: Vec<&CPart> = parts.iter().filter(|p| p.kind == "strip").collect();
    let widest = sf::fmax(&strips.iter().map(|p| p.p.h).collect::<Vec<_>>());
    note(strips.len() == 4, "the four strips close the square", format!("{s:.2}mm bore, {:.2}mm outside; {ls:.1}mm long", s + 2.0 * t));
    let web = pc::pin_web(parts, Some(&["seat", "round", "sock"]), &[pc::ROD_HOLE / 2.0], true);
    note(web >= pc::MIN_WEB, "every cookie rod keeps ply between it and every cut", format!("{web:.2}mm at the least, against {}", g(pc::MIN_WEB)));
    let r = py::hypot(info.pins[0].0, info.pins[0].1);
    let face = COOKIE / 2.0 - r / 2f64.sqrt();
    let tube = r - (s + 2.0 * t) / 2.0 * 2f64.sqrt();
    note(
        py::min(face, tube) >= pc::WASHER_D / 2.0,
        "the nuts sit on the cookies' ends, clear of the strips",
        format!(
            "{}mm washers: {face:.2}mm to the outline, {tube:.2}mm to the strips' corner, against {}",
            g(pc::WASHER_D),
            g(pc::WASHER_D / 2.0)
        ),
    );
    note(
        SEAT_D >= pc::STAPLE_MAX && info.round_d <= pc::STAPLE_MIN + 1e-9,
        "the seat holds a staple, the round bore under it",
        format!(
            "seat {}mm over the largest staple, {}; the round bore {:.2}mm stops staples over that only: a {}mm staple is set by its hemp",
            g(SEAT_D),
            g(pc::STAPLE_MAX),
            info.round_d,
            g(pc::STAPLE_MIN)
        ),
    );
    let under = r + pc::NUT_AF / 2.0 - CHAMBER_D / 2.0;
    note(
        CHAMBER_D < COOKIE && under >= 1.0,
        "the cap slides over the cookies and stops on its nuts",
        format!(
            "socket {}mm clear all round; each top nut {under:.1}mm under the chamber's bottom plate, the chamber {}mm",
            g(pc::CAP_CLEAR),
            g(CHAMBER_D)
        ),
    );
    let (ok, d) = pc::mouthpiece_check(parts);
    note(ok, "the mouthpiece tapers as a real practice chanter's does", d);
    pc::sheet_checks(&mut note, sheets, parts.len(), &format!("; the strips {:.1} x {:.1}mm", ls + kerf(), widest + kerf()));
    out
}

pub fn label(joint: &str) -> String {
    format!("strips-{joint}-joint")
}

fn svg_sheet(k: usize, nsh: usize, sh: &Sheet, joint: &str) -> String {
    let txt = pc::svg_sheet(k, nsh, sh, 4);
    let switch = *SWITCH.lock().unwrap();
    let what = format!(
        "Practice chanter, square bore of four {joint}-jointed strips, cookies from {}",
        match switch {
            None => "just above high A".to_string(),
            Some(sw) => format!("{} of the way up", g(sw)),
        }
    );
    txt.replace("Practice chanter, 4-sided bore", &what)
        .replace("cuts the bores, seats and pin holes first; then black", "cuts the bores, seats, finger and pin holes first; then black")
}

/// Where a strip sits in the chanter: its x along it, u across, w through,
/// to the chanter's up, front and length.
pub fn placed(p: &CPart, s: f64, t: f64, x0: f64) -> Mat {
    let c = t + s / 2.0;
    let r = |v: [Num; 4]| v.to_vec();
    match p.strip {
        "front" => vec![r([I(1), I(0), I(0), F(x0)]), r([I(0), I(0), I(1), F(-s / 2.0 - t)]), r([I(0), I(-1), I(0), F(c)]), r([I(0), I(0), I(0), I(1)])],
        "back" => vec![r([I(1), I(0), I(0), F(x0)]), r([I(0), I(0), I(-1), F(s / 2.0 + t)]), r([I(0), I(1), I(0), F(-c)]), r([I(0), I(0), I(0), I(1)])],
        "left" => vec![r([I(1), I(0), I(0), F(x0)]), r([I(0), I(1), I(0), F(-c)]), r([I(0), I(0), I(1), F(s / 2.0)]), r([I(0), I(0), I(0), I(1)])],
        _ => vec![r([I(1), I(0), I(0), F(x0)]), r([I(0), I(-1), I(0), F(c)]), r([I(0), I(0), I(-1), F(-s / 2.0)]), r([I(0), I(0), I(0), I(1)])],
    }
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
        } else if x.starts_with("--switch=") {
            *SWITCH.lock().unwrap() = Some(pc::num_arg(&x));
        } else if x.starts_with("--plate=") {
            m::PLATE.set(pc::num_arg(&x));
        } else if x.starts_with("--reed=") {
            m::REED.set(pc::num_arg(&x));
        } else if x.starts_with("--low-a=") {
            m::LOW_A.set(pc::num_arg(&x));
        } else if x.starts_with("--kerf=") {
            sf::set_kerf(&x);
        } else {
            die(format!(
                "error: {x} is not a flag this generator reads. It takes --joint=, --switch=, --out=, --plate=, --reed=, --low-a=, --kerf= and --no-write."
            ));
        }
    }
    if write && out.is_none() {
        die("error: --out=DIR, or --no-write to run the checks only.");
    }
    let (parts, info) = build(&joint);
    let sheets = pc::pack(&parts);
    let res = checks(&parts, &info, &sheets, &joint);
    let t = info.t;
    let block = N_SEAT + info.n_round + N_SOCK;
    println!(
        "practice chanter, square bore of four {joint}-jointed strips, low A {}Hz: strips {:.1}mm long, {:.2}mm bore, {:.2}mm outside; {block} cookies ({}mm, {}mm square), body {:.1}mm; {} cap and {} mouthpiece plates; two M{} rods cut {:.0}mm long through the cookies",
        g(m::LOW_A.get()),
        info.ls,
        info.s,
        info.s + 2.0 * t,
        g(block as f64 * t),
        g(COOKIE),
        N_SEAT as f64 * t + info.l,
        pc::N_SOCKET + pc::N_CHAMBER + pc::N_TOP,
        pc::n_stem(),
        g(pc::ROD),
        block as f64 * t + 2.0 * (0.5 + pc::NUT_T) + pc::ROD_SPARE
    );
    for &(st, x, d) in &info.holes {
        let i = (0..8).find(|&i| (info.pos[i] - info.n_round as f64 * t - x).abs() < 1e-9).unwrap();
        println!("  {:<3} {st:<5} strip, {x:6.1}mm from its top ({:5.1}mm down the bore)  {d:.2}mm round", HOLES[i], info.pos[i]);
    }
    println!("  cookies, top down: {N_SEAT} seat, {} round, {N_SOCK} socket over the strips", info.n_round);
    if pc::print_checks(&res) {
        std::process::exit(1);
    }
    if write {
        let lb = label(&joint);
        pc::write_sheets(&out.unwrap(), &sheets, &|k| format!("practice-chanter-{lb}-sheet{k}-cut-files.svg"), &|k, n, sh| svg_sheet(k, n, sh, &joint));
    }
}
