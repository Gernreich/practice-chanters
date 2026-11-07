//! A Great Highland bagpipe practice chanter with a regular polygonal bore,
//! three to eight sides, as a transmission-matrix model: how long the bore
//! runs, and where its eight finger holes go and how wide each is cut.
//!
//!     target/release/chanter_model                 # every shape: the design, how it plays
//!     target/release/chanter_model --sides=5       # one shape
//!     target/release/chanter_model --plate=2.9     # ply as measured
//!     target/release/chanter_model --reed=25       # another reed correction, mm
//!     target/release/chanter_model --low-a=233     # another pitch
//!
//! THE SCALE is the bagpipe's nine notes, low G to high A, in just intonation
//! on the Mixolydian mode with G the harmonic seventh, as the pipes are
//! tuned. A practice chanter's cylindrical bore sounds an octave below the
//! pipe chanter's conical one, so low A is LOW_A, 240Hz, half the pipes'
//! 480. The notes are fingered as on the pipes, which is not one hole after
//! another: C and D keep the low A hole closed below the open ones, E opens
//! low A again, and high A closes E. FINGERING says which holes are open for
//! each note, and the model solves every note with its own fingering.
//!
//! THE BORE is a regular polygon of BORE_AREA, the area of a 4mm round bore,
//! the size makers bore practice chanters. Its wall loss is taken on the
//! hydraulic radius 2A/P, which for a regular polygon is its inradius: the
//! fewer the sides, the more wall for the area, and the more loss.
//! design(top=) makes the first top mm under the reed round, the same area,
//! above every hole.
//!
//! THE CHANTER is a stack of PLATE-thick cross-section plates, each with the
//! polygon cut through it. A finger hole is a slot cut in one plate from the
//! bore out through its face, PLATE long along the bore and as wide as the
//! note needs, so holes stand on the plates' grid and their widths are
//! solved. Its chimney is the wall, from the bore's face to the outside's.
//!
//! THE HOLES sit where hands reach them: the lower hand's four SPACING
//! apart, the upper hand's E and F the same, G G_GAP below F and the thumb's
//! hole THUMB above G. Their widths are solved, note by note. A hole under
//! W_SLOT wide is too narrow to cut, so it is drilled after assembly
//! instead, a round hole of the same area; one that would need more than
//! W_MAX is held at W_MAX and moved up the chanter until that is enough, to
//! the plate above where it would sound, and narrowed there to the note.
//!
//! The model, from the reed down:
//!
//!   * the reed and its staple are taken as REED mm more closed bore above
//!     the bore's top, where the staple sits; that is the uncalibrated part,
//!     as the embouchure is on a flute. A piper sets a chanter's pitch by how
//!     deep the reed sits, which moves exactly this;
//!   * the bore is a lossy tube;
//!   * each finger hole is Keefe's (1990) tone hole, open or closed, the
//!     round hole of the slot's area;
//!   * the bottom radiates as a flanged open end, the chanter's sole being
//!     far wider than its bore. The bore is the nearest whole number of
//!     plates, so low G comes out a few cents out either way, the one note
//!     with no hole of its own to tune; tape across part of the sole
//!     flattens it if sharp.
//!
//! A reed sounds where the impedance it sees peaks: where the admittance's
//! imaginary part passes through zero, rising. The admittance and the
//! search are bore-tmm's (src/models.rs, the reed kernel), the arithmetic
//! this model has always used.
use bore_tmm::models::{scan, Reed};
use lasermade_tools::py;
use std::collections::BTreeSet;
use std::f64::consts::PI;
use std::sync::atomic::{AtomicU64, Ordering};

// --- the settings a generator's flags change ----------------------------------
pub struct Setting(AtomicU64);
impl Setting {
    pub const fn new(bits: u64) -> Setting {
        Setting(AtomicU64::new(bits))
    }
    pub fn get(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }
    pub fn set(&self, x: f64) {
        self.0.store(x.to_bits(), Ordering::Relaxed)
    }
}
pub static PLATE: Setting = Setting::new(0x4008000000000000); // 3.0
pub static REED: Setting = Setting::new(0x4034000000000000); // 20.0: the reed and staple, as closed bore
pub static LOW_A: Setting = Setting::new(0x406e000000000000); // 240.0
// F to high G is the scale's smallest step, 84 cents, so at SPACING the G hole
// would have to be smaller than any drill; it stands G_GAP above F instead
pub static G_GAP: Setting = Setting::new(0x4035000000000000); // 21.0
pub fn plate() -> f64 {
    PLATE.get()
}

// --- air, warm from the breath ------------------------------------------------
pub const T_C: f64 = 25.0;
pub fn c() -> f64 {
    331.3 * (1.0 + T_C / 273.15).sqrt() // 346.1 m/s
}
pub const RHO: f64 = 1.184;

// --- the tune -----------------------------------------------------------------
pub const NOTES: [&str; 9] = ["low G", "low A", "B", "C", "D", "E", "F", "high G", "high A"];
pub const RATIOS: [f64; 9] = [7.0 / 8.0, 1.0, 9.0 / 8.0, 5.0 / 4.0, 4.0 / 3.0, 3.0 / 2.0, 5.0 / 3.0, 7.0 / 4.0, 2.0];
/// the holes from the bottom up, each named for the note it opens; high A is
/// the thumb's, on the back
pub const HOLES: [&str; 8] = ["A", "B", "C", "D", "E", "F", "G", "hA"];
pub fn fingering(note: &str) -> &'static [&'static str] {
    match note {
        "low G" => &[],
        "low A" => &["A"],
        "B" => &["B", "A"],
        "C" => &["C", "B"],
        "D" => &["D", "C", "B"],
        "E" => &["E", "A"],
        "F" => &["F", "E", "A"],
        "high G" => &["G", "F", "E", "A"],
        "high A" => &["hA", "G", "F", "A"],
        _ => panic!("{note}: not a note"),
    }
}
/// the note each hole is placed and sized for, in the order they are solved
pub const SETS: [&str; 8] = ["low A", "B", "C", "D", "E", "F", "high G", "high A"];
pub fn note_of(h: &str) -> &'static str {
    SETS[hi(h)]
}
/// a hole's index in HOLES
pub fn hi(h: &str) -> usize {
    HOLES.iter().position(|x| *x == h).unwrap()
}

// --- the bore and the plates (mm) ---------------------------------------------
pub const BORE_AREA: f64 = PI * 2.0 * 2.0; // a 4mm round bore's
pub const OUTER_AREA: f64 = PI * 10.0 * 10.0; // the outside: a 20mm round chanter's
pub const W_MAX: f64 = 5.0; // the widest slot a finger pad closes
pub const SIDES: [usize; 6] = [3, 4, 5, 6, 7, 8];
pub const SCAN: i64 = 30; // points a resonance is looked for across

pub const SPACING: f64 = 27.0; // centre to centre within a hand, 9 plates
pub const ANCHORS: (&str, &str) = ("C", "E"); // one hole in each hand, its plate solved
pub const W_ANCHOR: f64 = 4.0; // the anchors' width while their plates are solved
pub const THUMB: f64 = 9.0; // the high A hole above the G hole, on the back
pub const W_SLOT: f64 = 1.0; // narrower than this is drilled, not cut
pub const D_MIN: f64 = 1.0; // the smallest drill the model will ask for
// a bore of strips: its holes round, cut in the strip, and no wider than the
// bore's face leaves ply beside them; placed to GRID
pub const D_MAX: f64 = 3.2;
pub const D_ANCHOR: f64 = 2.5;
pub const GRID: f64 = 0.5;

extern "C" {
    fn log2(x: f64) -> f64;
}
/// math.log2, libm's
pub fn mlog2(x: f64) -> f64 {
    let f: unsafe extern "C" fn(f64) -> f64 = log2;
    unsafe { std::hint::black_box(f)(x) }
}

pub fn inradius(area: f64, n: usize) -> f64 {
    (area / (n as f64 * py::mtan(PI / n as f64))).sqrt()
}

/// The n-gon: its area, equivalent radius and loss radius (m), and its wall,
/// the chimney of a hole through a face (mm). n = 0 is a round bore. A bore
/// may have a round top: its first top mm, under the reed, round.
#[derive(Clone, Debug)]
pub struct Bore {
    pub n: usize,
    pub area: f64, // m^2
    pub a: f64,
    pub rh: f64,
    pub wall: f64,
    pub top: f64,
    /// the round top's loss radius and area, or the bore's own
    pub round_rh: f64,
    pub round_area: f64,
    pub strip: Option<f64>,
    pub grid: f64,
    pub w_lo: f64,
    pub w_max: f64,
    pub w_anchor: f64,
}

impl Bore {
    pub fn new(n: usize, top: f64, strip: Option<f64>) -> Bore {
        let area = BORE_AREA / 1e6;
        let a = (area / PI).sqrt();
        let rh = (if n > 0 { inradius(BORE_AREA, n) } else { (BORE_AREA / PI).sqrt() }) / 1000.0;
        let mut wall = if n > 0 { inradius(OUTER_AREA, n) - inradius(BORE_AREA, n) } else { f64::NAN };
        let (round_rh, round_area) = if top != 0.0 {
            let r = Bore::new(0, 0.0, None);
            (r.rh, r.area)
        } else {
            (rh, area)
        };
        if let Some(s) = strip.filter(|s| *s != 0.0) {
            wall = s;
        }
        let st = strip.filter(|s| *s != 0.0).is_some();
        let p = plate();
        Bore {
            n,
            area,
            a,
            rh,
            wall,
            top,
            round_rh,
            round_area,
            strip: strip.filter(|s| *s != 0.0),
            grid: if st { GRID } else { p },
            w_lo: if st { D_MIN } else { py::powf(D_MIN, 2.0) * PI / 4.0 / p },
            w_max: if st { D_MAX } else { W_MAX },
            w_anchor: if st { D_ANCHOR } else { W_ANCHOR },
        }
    }

    /// The hole's radius (m): a round hole w across, or a slot's area.
    pub fn hole_r(&self, w: f64) -> f64 {
        (if self.strip.is_some() { w / 2.0 } else { (w * plate() / PI).sqrt() }) / 1000.0
    }
}

/// The holes by name: (pos, width), in HOLES order.
pub type Holes = [(f64, f64); 8];

/// The holes in order down the bore, stable on ties, as sorted() leaves them.
fn down(pos: &[f64; 8]) -> Vec<usize> {
    let mut o: Vec<usize> = (0..8).collect();
    o.sort_by(|&a, &b| pos[a].partial_cmp(&pos[b]).unwrap());
    o
}

/// The note a fingering plays nearest `near`: a rising zero of Im(Y),
/// scanned then bisected. Err when there is none.
pub fn sounding(b: &Bore, l: f64, pos: &[f64; 8], widths: &[f64; 8], note: &str, near: Option<f64>, span: f64) -> Result<f64, String> {
    let near = near.filter(|x| *x != 0.0).unwrap_or_else(|| target(note));
    let opened = fingering(note);
    let (mut x, mut seg, mut r, mut t, mut op) = (b.top, vec![], vec![], vec![], vec![]);
    for i in down(pos) {
        seg.push((pos[i] - x) / 1000.0);
        r.push(b.hole_r(widths[i]));
        t.push(b.wall);
        op.push(opened.contains(&HOLES[i]) as u8);
        x = pos[i];
    }
    let reed = Reed {
        c: c(),
        rho: RHO,
        rh0: b.round_rh,
        area0: b.round_area,
        l0: (REED.get() + b.top) / 1000.0,
        rh: b.rh,
        area: b.area,
        a: b.a,
        seg: &seg,
        r: &r,
        t: &t,
        open: &op,
        last: (l - x) / 1000.0,
        q: 0.5,
        j: 0.8216,
    };
    let root = scan(near, span, SCAN, 32, |f| reed.y(f));
    if root.is_nan() {
        return Err(format!("{note}: no resonance within {}x of {near:.1}Hz", py::g(span)));
    }
    Ok(root)
}

pub fn target(note: &str) -> f64 {
    LOW_A.get() * RATIOS[NOTES.iter().position(|n| *n == note).unwrap()]
}

/// x in [lo, hi] with fn(x) == goal. fn falls as x grows, unless rising.
/// Err when the goal is out of reach.
pub fn bisect(f: &mut dyn FnMut(f64) -> Result<f64, String>, goal: f64, mut lo: f64, mut hi: f64, rising: bool, n: usize) -> Result<f64, String> {
    let mut g = |x: f64| -> Result<f64, String> { f(x).map(|v| if rising { -v } else { v }) };
    let t = if rising { -goal } else { goal };
    if !(g(hi)? <= t && { t <= g(lo)? }) {
        return Err(format!("out of reach between {lo:.1} and {hi:.1}"));
    }
    for _ in 0..n {
        let mid = (lo + hi) / 2.0;
        if g(mid)? > t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((lo + hi) / 2.0)
}

/// The centre of the plate x falls in, plates counted from the bore's top; on
/// a bore of strips, the nearest GRID.
pub fn snap(x: f64, b: Option<&Bore>) -> f64 {
    if b.is_some_and(|b| b.strip.is_some()) {
        return GRID * py::round(x / GRID) as f64;
    }
    let p = plate();
    ((x / p).floor() + 0.5) * p
}

/// The round hole of a w-wide slot's area.
pub fn drill(w: f64) -> f64 {
    (4.0 * w * plate() / PI).sqrt()
}

/// Laid out for the hands, then fitted to the plates (the Python
/// chanter_model.py's design(n, top=, strip=), with the default spacing and anchors). Returns
/// (bore, L, pos, widths), pos and widths in HOLES order.
pub fn design(n: usize, top: f64, strip: Option<f64>) -> Result<(Bore, f64, [f64; 8], [f64; 8]), String> {
    let b = Bore::new(n, top, strip);
    let whole = |d: f64| b.grid * py::round(d / b.grid) as f64;
    let (spacing, g_gap, thumb) = (whole(SPACING), whole(G_GAP.get()), whole(THUMB));
    let gap = if b.strip.is_some() { 2.0 * b.w_max } else { plate() };
    let lower = ["A", "B", "C", "D"];
    let upper = ["E", "F", "G"];
    let mut widths = [3.0; 8];
    let mut l = 400.0;
    let mut pos = [0.0; 8];
    for i in 0..8 {
        pos[i] = l * (0.85 - 0.08 * i as f64);
    }
    let mut moved: BTreeSet<usize> = BTreeSet::new();
    let est = |h: usize| 1000.0 * c() / (4.0 * target(SETS[h])) - REED.get();

    let place = |pos: &[f64; 8], widths: &[f64; 8], l: f64, h: usize, lo: f64, hi: f64, up: bool| -> Result<f64, String> {
        let nt = SETS[h];
        let mut f_pos = |x: f64| {
            let mut p = *pos;
            p[h] = x;
            sounding(&b, l, &p, widths, nt, None, 2.2)
        };
        let x = bisect(&mut f_pos, target(nt), lo, hi, false, 40)?;
        let mut s = snap(x, Some(&b));
        if up && s > x {
            s -= b.grid;
        }
        Ok(s)
    };
    let size = |pos: &[f64; 8], widths: &[f64; 8], l: f64, h: usize| -> Result<f64, String> {
        let nt = SETS[h];
        let mut f_w = |w: f64| {
            let mut ws = *widths;
            ws[h] = w;
            sounding(&b, l, pos, &ws, nt, None, 1.3)
        };
        let lo = b.w_lo;
        if f_w(lo)? > target(nt) {
            return Err(format!(
                "{n} sides: {nt} is sharp even with its hole the smallest drill, {}mm, {:.1}mm down the bore",
                py::g(D_MIN),
                pos[h]
            ));
        }
        if f_w(b.w_max)? < target(nt) {
            return Ok(f64::INFINITY);
        }
        bisect(&mut f_w, target(nt), lo, b.w_max, true, 40)
    };

    for it in 0..8 {
        let mut f_l = |x: f64| sounding(&b, x, &pos, &widths, "low G", None, 1.8);
        l = bisect(&mut f_l, target("low G"), 250.0, 600.0, false, 40)?;
        // a whole number of plates, the nearest
        l = b.grid * py::round(l / b.grid) as f64;
        if it < 3 {
            for (hand, a) in [(&lower[..], ANCHORS.0), (&upper[..], ANCHORS.1)] {
                let ai = hi(a);
                widths[ai] = b.w_anchor;
                pos[ai] = place(&pos, &widths, l, ai, 0.7 * est(ai), py::min(l - gap, 1.3 * est(ai)), false)?;
                let ia = hand.iter().position(|x| *x == a).unwrap() as i64;
                for (ih, h) in hand.iter().enumerate() {
                    let k = hi(h);
                    if !moved.contains(&k) {
                        pos[k] = pos[ai] + (ia - ih as i64) as f64 * spacing;
                    }
                }
            }
            if !moved.contains(&hi("G")) {
                pos[hi("G")] = pos[hi("F")] - g_gap;
            }
            if !moved.contains(&hi("hA")) {
                pos[hi("hA")] = pos[hi("G")] - thumb;
            }
        }
        for i in 0..8 {
            let below = if i > 0 { pos[i - 1] } else { l };
            let above = if i + 1 < 8 { pos[i + 1] } else { 0.0 };
            let w = size(&pos, &widths, l, i)?;
            if w <= b.w_max {
                widths[i] = w;
                continue;
            }
            moved.insert(i);
            widths[i] = b.w_max;
            let was = pos[i];
            pos[i] = place(&pos, &widths, l, i, py::max(above + gap, 0.6 * est(i)), below - gap, true)?;
            widths[i] = py::min(size(&pos, &widths, l, i)?, b.w_max);
            // an anchor takes its hand with it, so the hand's spacing holds
            if HOLES[i] == ANCHORS.0 || HOLES[i] == ANCHORS.1 {
                let hand: Vec<&str> = if lower.contains(&HOLES[i]) { lower.to_vec() } else { vec!["E", "F", "G", "hA"] };
                for f in hand {
                    let k = hi(f);
                    if !moved.contains(&k) {
                        pos[k] += pos[i] - was;
                    }
                }
            }
        }
    }
    Ok((b, l, pos, widths))
}

pub fn cents(f: f64, r: f64) -> f64 {
    1200.0 * mlog2(f / r)
}

/// Each note: (note, Hz, cents off its target).
pub fn report(b: &Bore, l: f64, pos: &[f64; 8], widths: &[f64; 8]) -> Result<Vec<(&'static str, f64, f64)>, String> {
    NOTES
        .iter()
        .map(|&nt| {
            let f = sounding(b, l, pos, widths, nt, None, 1.3)?;
            Ok((nt, f, cents(f, target(nt))))
        })
        .collect()
}

/// chanter_model's own report: every shape, the design and how it plays.
pub fn main() {
    let mut sides: Vec<usize> = SIDES.to_vec();
    for x in std::env::args().skip(1) {
        let v = x.split_once('=').map(|p| p.1).unwrap_or("");
        let num = || py::float(v).unwrap_or_else(|| lasermade_tools::die(format!("error: {x}: not a number.")));
        if x.starts_with("--sides=") {
            sides = vec![v.trim().parse().unwrap_or_else(|_| lasermade_tools::die(format!("error: {x}: not a number.")))];
        } else if x.starts_with("--plate=") {
            PLATE.set(num());
        } else if x.starts_with("--reed=") {
            REED.set(num());
        } else if x.starts_with("--low-a=") {
            LOW_A.set(num());
        } else {
            lasermade_tools::die(format!(
                "error: {x} is not a flag this model reads. It takes --sides=, --plate=, --reed= and --low-a."
            ));
        }
    }
    let g = py::g;
    println!(
        "low A {}Hz, air at {}C ({:.1} m/s); bore {:.2}mm^2, reed + staple as {}mm of closed bore; {}mm plates",
        g(LOW_A.get()),
        g(T_C),
        c(),
        BORE_AREA,
        g(REED.get()),
        g(plate())
    );
    for n in sides {
        let (b, l, pos, widths) = design(n, 0.0, None).unwrap_or_else(|e| lasermade_tools::die(e));
        println!("\n{n} sides: wall {:.2}mm, loss radius {:.2}mm; bore {l:.1}mm from its top to the sole", b.wall, 1000.0 * b.rh);
        let mut prev = l;
        for i in 0..8 {
            let how = if widths[i] >= W_SLOT { format!("slot {:4.2}mm wide", widths[i]) } else { format!("drill {:4.2}mm  ", drill(widths[i])) };
            println!(
                "  {:<3} plate {:3}, {:6.1}mm  {how}  {:5.1}mm above the one below",
                HOLES[i],
                (pos[i] / plate()).floor() as i64 + 1,
                pos[i],
                prev - pos[i]
            );
            prev = pos[i];
        }
        for (nt, f, cn) in report(&b, l, &pos, &widths).unwrap_or_else(|e| lasermade_tools::die(e)) {
            println!("    {nt:<7} {f:7.2}Hz {cn:+6.1} cents");
        }
    }
}
