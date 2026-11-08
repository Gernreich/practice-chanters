//! The practice chanters as parametric SolidCraft designs (2026-11-08): the
//! rules the generators follow, as expressions, so changing one number
//! rebuilds the rest. `practice_chanters --solidcraft=DIR` writes one script
//! per design; `solidcraft-cli run SCRIPT --save X.sc` makes the design,
//! `--out X.step` the STEP file.
//!
//! The frame is the 3D view's and the STEP files': X down the chanter from
//! the top of the body (the cap and mouthpiece at negative X), the finger
//! holes toward -Y, the thumb hole toward +Y.
//!
//! THE STACKED CHANTER. Each run of like plates (seat, bore, socket,
//! chamber, top) is one plate repeated, its count a parameter, so the plates
//! follow `ply` and the counts; the bore, the seat, the rods, the chamber,
//! the blowing hole and the cap's pins are each one cut through their run.
//! A finger hole is a cut one plate thick, in the bore plate its position
//! falls in: a slot `w_` wide from where its sides leave the bore out
//! through the face, or, narrower than `w_slot`, a V in the face to drill
//! at. Move a hole and its cut moves to its new plate. The mouthpiece's
//! plates each take their own diameter from the measured profile at their
//! mid-height, so they are a body each, as many as the profile's length
//! held at the shipped ply.
//!
//! What does not follow: the tuning (the bore plates' count and each hole's
//! place and width are the solver's, which no expression redoes; they are
//! parameters so a chanter tuned by hand can be written back), and the two
//! rods' places, which a search over the plate found.

use crate::chanter_model::{self as m, HOLES};
use crate::practice_chanters as pc;
use lasermade_tools::solidcraft::{n, Script};
use std::f64::consts::PI;

/// sum of coef * (expr), the zero terms dropped.
fn lin(terms: &[(f64, &str)]) -> String {
    let t: Vec<String> = terms.iter().filter(|(c, _)| c.abs() > 1e-12).map(|(c, e)| format!("{}*({e})", n(*c))).collect();
    if t.is_empty() { "0 mm".into() } else { t.join(" + ") }
}

/// A regular n-gon `flats` across, as the generators' ngon(n, flats/2, 0),
/// in the plate's (Y, Z): a corner at polar angle (k + 1/2) 360/n, and
/// polar(t, r) lands at Y = -r cos t, Z = -r sin t.
fn ngon_yz(nn: usize, flats: &str) -> Vec<[String; 2]> {
    let r = format!("({flats})/2/{}", n((PI / nn as f64).cos()));
    (0..nn)
        .map(|k| {
            let t = ((k as f64 + 0.5) * 360.0 / nn as f64).to_radians();
            [lin(&[(-t.cos(), &r)]), lin(&[(-t.sin(), &r)])]
        })
        .collect()
}

/// A sketch across the chanter at X = `x`, normal +X.
fn across(s: &mut Script, x: &str, name: &str) -> String {
    s.sketch_on([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], Some(x), name)
}

/// A polygon across the chanter at `x`, extruded `dist` down it.
fn prism(s: &mut Script, name: &str, x: &str, pts: &[[String; 2]], dist: &str, op: &str) {
    let sk = across(s, x, &format!("{name} sketch"));
    s.poly(pts);
    s.finish();
    s.extrude(&sk, dist, false, op, &[], name);
}

/// A cylinder along X from x, through every body it meets.
fn rod(s: &mut Script, name: &str, x: &str, y: &str, z: &str, d: &str, len: &str, op: &str) {
    s.cylinder(name, [x.to_string(), y.to_string(), z.to_string()], Some([1.0, 0.0, 0.0]), d, len, op, &[]);
}

/// The stacked octagon.
pub fn stacked() -> String {
    let nn = 8usize;
    let (_, info) = pc::build(nn);
    let p0 = m::plate();
    let mut s = Script::new();
    let t8 = (PI / 8.0).tan();
    // the sheet and the bore
    s.len("ply", format!("{} mm", n(p0)), "plate thickness: measure the ply, as the generator's --plate=");
    s.len("ref_bore", format!("{} mm", n(pc::REF_BORE)), "the reference chanter's bore, whose area the octagon keeps");
    s.len("bore_flats", format!("ref_bore*sqrt(pi/(8*{}))", n(t8)), "read only: the bore, an octagon, across its flats");
    s.len("outer_flats", format!("{} mm", n(pc::OUTER_FLATS)), "the body's outside across its flats");
    s.len("seat_d", format!("{} mm", n(pc::SEAT_D)), "the reed's seat, through the seat plates");
    s.num("n_seat", n(pc::N_SEAT as f64), "seat plates");
    s.num("n_bore", n(info.n_bore as f64), "bore plates, down to the open sole: the solver's whole number nearest the reference's low G");
    s.len("bore_len", "n_bore*ply", "read only: the bore below the seat");
    s.len("rod_hole", format!("{} mm", n(pc::ROD_HOLE)), "the two M2 rods' holes, through the seat and the bore");
    for (k, &(u, v)) in info.pins.iter().enumerate() {
        s.len(&format!("rod{}_y", k + 1), format!("{} mm", n(v)), &format!("rod {}'s place across the plate, toward the thumb (a search's)", k + 1));
        s.len(&format!("rod{}_z", k + 1), format!("{} mm", n(-u)), &format!("and up"));
    }
    // the holes, the solver's
    s.len("w_slot", format!("{} mm", n(m::W_SLOT)), "a hole narrower than this is drilled at a V, not cut as a slot");
    s.len("v_w", format!("{} mm", n(pc::V_W)), "the drill mark's V, across");
    s.len("v_d", format!("{} mm", n(pc::V_D)), "and deep");
    for i in 0..8 {
        let k = i + 1;
        s.len(&format!("pos_{k}"), format!("{} mm", n(info.pos[i])), &format!("the {} hole's centre, down the bore from the seat: the solver's", HOLES[i]));
        s.len(&format!("w_{k}"), format!("{} mm", n(info.widths[i])), &format!("its slot's width through one plate: the solver's"));
        s.len(&format!("drill_{k}"), format!("sqrt(4*w_{k}*ply/pi)"), &format!("read only: the drill that gives the same area, where it is drilled"));
        s.len(&format!("x_{k}"), format!("(n_seat + floor(pos_{k}/ply + 1e-9))*ply"), &format!("read only: the top of the plate it is cut in"));
    }
    // the cap
    s.len("cap_clear", format!("{} mm", n(pc::CAP_CLEAR)), "the socket's clearance each side of the body, for hemp or tape");
    s.len("cap_web", format!("{} mm", n(pc::CAP_WEB)), "ply each side of the cap's pins");
    s.len("pin", format!("{} mm", n(pc::PIN)), "the cap's brass pins");
    s.len("pin_interf", format!("{} mm", n(pc::PIN_INTERF)), "their holes this much under the pin, a press-fit");
    s.num("n_socket", n(pc::N_SOCKET as f64), "socket plates, over the body's top");
    s.num("n_chamber", n(pc::N_CHAMBER as f64), "chamber plates, round the reed's blades");
    s.num("n_top", n(pc::N_TOP as f64), "the cap's top plates");
    s.len("chamber_d", format!("{} mm", n(pc::CHAMBER_D)), "the chamber round the reed");
    s.len("mouth_d", format!("{} mm", n(pc::MOUTH_D)), "the blowing hole");
    s.len("sock_flats", "outer_flats + 2*cap_clear", "read only: the socket across its flats");
    s.len("cap_flats", "sock_flats + 2*(2*cap_web + pin)", "read only: the cap across its flats");
    s.len("pin_r", format!("(sock_flats/2 + cap_web + pin/2)/{}", n((PI / 8.0).cos())), "read only: the pins' distance from the axis, at two opposite corners");
    s.len("cap_top", "-(n_chamber + n_top)*ply", "read only: the cap's top, where the mouthpiece starts");
    // the bodies, in the generator's order
    let outer = ngon_yz(nn, "outer_flats");
    prism(&mut s, "seat plate", "0 mm", &outer, "ply", "new");
    s.pattern("seat plate", [1.0, 0.0, 0.0], "n_seat", "ply");
    prism(&mut s, "bore plate", "n_seat*ply", &outer, "ply", "new");
    s.pattern("bore plate", [1.0, 0.0, 0.0], "n_bore", "ply");
    let cap_o = ngon_yz(nn, "cap_flats");
    // a ring: the cap's outside, the socket inside it
    let sk = across(&mut s, "(n_socket - 1)*ply", "socket plate sketch");
    s.poly(&cap_o);
    s.poly(&ngon_yz(nn, "sock_flats"));
    s.finish();
    s.extrude(&sk, "ply", false, "new", &[], "socket plate");
    s.pattern("socket plate", [-1.0, 0.0, 0.0], "n_socket", "ply");
    prism(&mut s, "chamber plate", "-ply", &cap_o, "ply", "new");
    s.pattern("chamber plate", [-1.0, 0.0, 0.0], "n_chamber", "ply");
    prism(&mut s, "cap top plate", "-(n_chamber + 1)*ply", &cap_o, "ply", "new");
    s.pattern("cap top plate", [-1.0, 0.0, 0.0], "n_top", "ply");
    // the mouthpiece: the measured profile, diameter by height up from the cap
    let mp = pc::MOUTHPIECE;
    let mut prof = format!("{} mm", n(mp[0].1));
    for w in mp.windows(2) {
        let ((z0, d0), (z1, d1)) = (w[0], w[1]);
        prof += &format!(" + {} mm*min(1, max(0, (ZZ - {} mm)/{} mm))", n(d1 - d0), n(z0), n(z1 - z0));
    }
    s.len("stem_len", format!("{} mm", n(mp[mp.len() - 1].0)), "the reference's mouthpiece, up from the cap");
    let n_stem = pc::n_stem();
    for k in 0..n_stem {
        let d = prof.replace("ZZ", &format!("{}*ply", n(k as f64 + 0.5)));
        s.len(&format!("stem_d{}", k + 1), d, &format!("read only: mouthpiece plate {}'s diameter, the profile at its mid-height", k + 1));
    }
    for k in 0..n_stem {
        rod(&mut s, &format!("mouthpiece plate {}", k + 1), &format!("cap_top - {}*ply", k + 1), "0 mm", "0 mm", &format!("stem_d{}", k + 1), "ply", "new");
    }
    // the cuts through each run
    let body_len = "(n_seat + n_bore)*ply";
    rod(&mut s, "seat", "-1 mm", "0 mm", "0 mm", "seat_d", "n_seat*ply + 1 mm", "cut");
    prism(&mut s, "bore", "n_seat*ply", &ngon_yz(nn, "bore_flats"), "n_bore*ply + 1 mm", "cut");
    for k in 1..=2 {
        rod(&mut s, &format!("rod {k}"), "-1 mm", &format!("rod{k}_y"), &format!("rod{k}_z"), "rod_hole", &format!("{body_len} + 2 mm"), "cut");
    }
    rod(&mut s, "chamber", "-n_chamber*ply", "0 mm", "0 mm", "chamber_d", "n_chamber*ply", "cut");
    rod(&mut s, "blowing hole", &format!("cap_top - {}*ply - 1 mm", n_stem), "0 mm", "0 mm", "mouth_d", &format!("{}*ply + n_top*ply + 2 mm", n_stem), "cut");
    let t = (22.5f64).to_radians();
    for (k, sg) in [(1, 1.0), (2, -1.0)] {
        // ring_[0] and ring_[n/2]: the corners at 22.5 and 202.5 degrees
        rod(
            &mut s,
            &format!("cap pin {k}"),
            "cap_top - 1 mm",
            &lin(&[(-sg * t.cos(), "pin_r")]),
            &lin(&[(-sg * t.sin(), "pin_r")]),
            "pin - pin_interf",
            "(n_chamber + n_top + n_socket)*ply + 2 mm",
            "cut",
        );
    }
    // the finger holes, each in the plate its position falls in
    let (front, thumb) = pc::faces(nn);
    let rb = "bore_flats/2";
    for i in 0..8 {
        let k = i + 1;
        let th = if HOLES[i] == "hA" { thumb } else { front };
        let sgn = if th == 0.0 { -1.0 } else { 1.0 }; // toward -Y or +Y
        let slot_on = format!("min(1, floor(w_{k}/w_slot + 1e-9))");
        // the slot: from where its sides leave the bore, out past the face
        let s0 = format!("max(0 mm, min({rb}, {rb}*sqrt(2) - w_{k}/2))");
        let (y0, y1) = if sgn < 0.0 { (format!("-outer_flats/2 - 1 mm"), format!("-({s0})")) } else { (s0.clone(), "outer_flats/2 + 1 mm".to_string()) };
        s.bx(
            &format!("{} hole, slot", HOLES[i]),
            [format!("x_{k} + (1 - {slot_on})*1000 mm"), y0.clone(), format!("-w_{k}/2")],
            [String::from("ply"), format!("({y1}) - ({y0})"), format!("w_{k}")],
            "cut",
            &[],
        );
        // the V to drill at, when it is too narrow to cut
        let face = format!("{}*outer_flats/2", n(sgn));
        let tip = format!("{}*(outer_flats/2 - v_d)", n(sgn));
        let lift = format!("{slot_on}*1000 mm");
        let sk = across(&mut s, &format!("x_{k} + {lift}"), &format!("{} hole, V sketch", HOLES[i]));
        let out = format!("{}*(outer_flats/2 + 1 mm)", n(sgn));
        s.poly(&[[out.clone(), "-v_w/2".into()], [face.clone(), "-v_w/2".into()], [tip, "0 mm".into()], [face, "v_w/2".into()], [out, "v_w/2".into()]]);
        s.finish();
        s.extrude(&sk, "ply", false, "cut", &[], &format!("{} hole, V", HOLES[i]));
    }
    s.measure();
    s.text()
}


// --- the strip chanters --------------------------------------------------------

/// Where a part's own (p0, p1, p2) land in the world: for X, Y and Z, which
/// of the three, with what sign, plus an offset. Every placement in the strip
/// chanters is such a turn.
#[derive(Clone)]
struct Place([(usize, f64, String); 3]);

impl Place {
    /// plate_m(x): X = p2 + x, Y = p1, Z = -p0.
    fn plate(x: &str) -> Place {
        Place([(2, 1.0, x.to_string()), (1, 1.0, "0 mm".into()), (0, -1.0, "0 mm".into())])
    }
    /// A box from its ranges in the part's frame.
    fn bx(&self, s: &mut Script, name: &str, r: [(&str, &str); 3], op: &str, targets: &[&str]) {
        let mut corner = vec![];
        let mut size = vec![];
        for (k, sg, off) in &self.0 {
            let (lo, hi) = r[*k];
            if *sg > 0.0 {
                corner.push(format!("({lo}) + ({off})"));
            } else {
                corner.push(format!("-({hi}) + ({off})"));
            }
            size.push(format!("({hi}) - ({lo})"));
        }
        s.bx(name, [corner[0].clone(), corner[1].clone(), corner[2].clone()], [size[0].clone(), size[1].clone(), size[2].clone()], op, targets);
    }
    /// A world point from the part's.
    fn pt(&self, p: [&str; 3]) -> [String; 3] {
        let f = |j: usize| {
            let (k, sg, off) = &self.0[j];
            format!("{}({}) + ({off})", if *sg > 0.0 { "" } else { "-" }, p[*k])
        };
        [f(0), f(1), f(2)]
    }
    /// The world direction of the part's axis k.
    fn dir(&self, k: usize) -> [f64; 3] {
        let mut d = [0.0; 3];
        for (j, (kk, sg, _)) in self.0.iter().enumerate() {
            if *kk == k {
                d[j] = *sg;
            }
        }
        d
    }
    /// A cylinder along the part's axis 2 from (c0, c1, z0).
    fn cyl(&self, s: &mut Script, name: &str, c: [&str; 3], d: &str, h: &str, op: &str, targets: &[&str]) {
        s.cylinder(name, self.pt(c), Some(self.dir(2)), d, h, op, targets);
    }
}

/// The square-bore standard chanter, finger-jointed, as the 3D page has it.
/// Each hole's guard down the strips, g0_k to g1_k: MIN_WEB past its `len`
/// along. The generator's tidy gives a scrap shorter than min_finger beside a
/// guard to the guard, so a guard grows to the cell's edge over one. Between
/// two guards closer than that the scrap joins whichever shares its owner,
/// so it keeps its cell's unless both guards give it the other strip:
/// `gap_box` covers that case. Returns the holes in order, as pairs of
/// neighbours; the holes keep their order down the chanter.
fn guards(s: &mut Script, holes: &[crate::standard_chanter::Hole], len: &str) -> Vec<(usize, usize)> {
    let mut order: Vec<usize> = (1..=holes.len()).collect();
    order.sort_by(|&i, &j| holes[i - 1].z.partial_cmp(&holes[j - 1].z).unwrap());
    let a = |k: usize| format!("(xh_{k} - {len}_{k}/2 - min_web)");
    let b = |k: usize| format!("(xh_{k} + {len}_{k}/2 + min_web)");
    for (q, &k) in order.iter().enumerate() {
        let fl = format!("({} - floor({}/cell)*cell)", a(k), a(k));
        let fr = format!("(ceil({}/cell)*cell - {})", b(k), b(k));
        let mut g0 = format!("{} - (1 - min(1, floor({fl}/min_finger)))*{fl}", a(k));
        let mut g1 = format!("{} + (1 - min(1, floor({fr}/min_finger)))*{fr}", b(k));
        if q > 0 {
            let big = format!("min(1, floor(max(0, {} - {})/min_finger))", a(k), b(order[q - 1]));
            g0 = format!("{big}*({g0}) + (1 - {big})*{}", a(k));
            s.len(&format!("gap_{}_{k}", order[q - 1]), format!("{} - {}", a(k), b(order[q - 1])), "read only: the scrap between these two holes' guards");
        }
        if let Some(&nx) = order.get(q + 1) {
            let big = format!("min(1, floor(max(0, {} - {})/min_finger))", a(nx), b(k));
            g1 = format!("{big}*({g1}) + (1 - {big})*{}", b(k));
        }
        s.len(&format!("g0_{k}"), g0, "read only: where its guard starts down the strip");
        s.len(&format!("g1_{k}"), g1, "read only: and ends");
    }
    order.windows(2).map(|w| (w[0], w[1])).collect()
}

/// The (lo, hi) of the scrap between guards i and j down a strip. When it is
/// not there or not short the span is guard i's own, which the same join or
/// cut has already made, so it changes nothing (a join lifted away would add
/// a lump).
fn gap_span(i: usize, j: usize) -> (String, String) {
    let g = format!("gap_{i}_{j}");
    let on = format!("min(1, ceil(max(0, {g})/(0.001 mm)))*(1 - min(1, floor(max(0, {g})/min_finger)))");
    (format!("{on}*g1_{i} + (1 - {on})*g0_{i}"), format!("g1_{i} + {on}*max({g}, 0.01 mm)"))
}

/// The head's parameters, the reference's: the seat, the collar, the cap.
fn head_params(s: &mut Script, n_cap: usize) {
    use crate::standard_chanter as sd;
    s.num("n_seat", n(sd::N_SEAT as f64), "tenon plates, the reed's seat through them");
    s.num("n_collar", n(sd::N_COLLAR as f64), "the collar's round plates over its socket plate");
    s.len("seat_d", format!("{} mm", n(sd::REF_SEAT_D)), "the reed's seat, the reference's");
    s.len("ref_bore", format!("{} mm", n(sd::REF_BORE)), "the collar's round bore, the reference's");
    s.len("cap_d", format!("{} mm", n(sd::REF_CAP_D)), "the cap and the collar's round plates, the reference's");
    s.len("cap_bore", format!("{} mm", n(sd::REF_CAP_BORE)), "the cap's bore, the reference's");
    s.len("cap_clear", format!("{} mm", n(pc::CAP_CLEAR)), "the tenon's clearance each side, inside the cap");
    s.len("tenon_d", "cap_bore - 2*cap_clear", "read only: the tenon the cap slides over");
    s.num("n_cap", n(n_cap as f64), "the cap's plates, the reference's 122 mm of cap");
    s.len("pin", format!("{} mm", n(pc::PIN)), "the tenon's and collar's brass pins");
    s.len("pin_interf", format!("{} mm", n(pc::PIN_INTERF)), "their holes this much under the pin, a press-fit");
    s.len("r_pin", "(seat_d/2 + tenon_d/2)/2", "read only: the pins' distance from the axis");
    s.len("mouth_d", format!("{} mm", n(pc::MOUTH_D)), "the blowing hole");
}

/// The tenon, the collar's round plates, the socket plate (`socket` makes
/// it), the cap, its top and the mouthpiece, and the cuts through them.
fn head(s: &mut Script, socket: &dyn Fn(&mut Script)) {
    // the tenon, the collar and its socket plate
    let tenon = "tenon plate";
    rod(s, tenon, "0 mm", "0 mm", "0 mm", "tenon_d", "ply", "new");
    s.pattern(tenon, [1.0, 0.0, 0.0], "n_seat", "ply");
    rod(s, "collar plate", "n_seat*ply", "0 mm", "0 mm", "cap_d", "ply", "new");
    s.pattern("collar plate", [1.0, 0.0, 0.0], "n_collar", "ply");
    socket(s);
    // the cap, its top, and the mouthpiece
    // a ring, so its bore cuts nothing of the tenon inside it
    let sk = across(s, "n_seat*ply - ply", "cap plate sketch");
    for d in ["cap_d", "cap_bore"] {
        s.cmd("sketch.circle.center", format!("{{\"center\":[\"0 mm\",\"0 mm\"],\"diameter\":\"{d}\"}}"));
    }
    s.finish();
    s.extrude(&sk, "ply", false, "new", &[], "cap plate");
    s.pattern("cap plate", [-1.0, 0.0, 0.0], "n_cap - 1", "ply");
    rod(s, "cap top", "n_seat*ply - n_cap*ply", "0 mm", "0 mm", "cap_d", "ply", "new");
    let mp = pc::MOUTHPIECE;
    let mut prof = format!("{} mm", n(mp[0].1));
    for w in mp.windows(2) {
        let ((z0, d0), (z1, d1)) = (w[0], w[1]);
        prof += &format!(" + {} mm*min(1, max(0, (ZZ - {} mm)/{} mm))", n(d1 - d0), n(z0), n(z1 - z0));
    }
    let n_stem = pc::n_stem();
    for k in 0..n_stem {
        let d = prof.replace("ZZ", &format!("{}*ply", n(k as f64 + 0.5)));
        s.len(&format!("stem_d{}", k + 1), d, &format!("read only: mouthpiece plate {}'s diameter, the profile at its mid-height", k + 1));
    }
    for k in 0..n_stem {
        rod(s, &format!("mouthpiece plate {}", k + 1), &format!("n_seat*ply - n_cap*ply - {}*ply", k + 1), "0 mm", "0 mm", &format!("stem_d{}", k + 1), "ply", "new");
    }
    // the cuts through each run
    rod(s, "seat", "-1 mm", "0 mm", "0 mm", "seat_d", "n_seat*ply + 1 mm", "cut");
    rod(s, "collar bore", "n_seat*ply - 1 mm", "0 mm", "0 mm", "ref_bore", "n_collar*ply + 2 mm", "cut");
    for (k, ang) in [(1, 45.0f64), (2, 225.0)] {
        // polar(ang, r) lands at Y = -r cos, Z = -r sin
        let a = ang.to_radians();
        rod(s, &format!("pin {k}"), "-1 mm", &lin(&[(-a.cos(), "r_pin")]), &lin(&[(-a.sin(), "r_pin")]), "pin - pin_interf", "(n_seat + n_collar)*ply + 2 mm", "cut");
    }
    rod(s, "blowing hole", &format!("n_seat*ply - n_cap*ply - {n_stem}*ply - 1 mm"), "0 mm", "0 mm", "mouth_d", &format!("({n_stem} + 1)*ply + 2 mm"), "cut");
}

pub fn standard() -> String {
    use crate::standard_chanter as sd;
    use crate::strip_chanter as sc;
    let (_, info) = sd::build("finger");
    let mut s = Script::new();
    s.len("ply", format!("{} mm", n(m::plate())), "plate thickness: measure the ply, as the generator's --plate=");
    s.len("side", format!("{} mm", n(sd::SIDE)), "the square bore across, inside the four strips");
    s.len("a", "side/2 + ply", "read only: the strips' outside, half across");
    head_params(&mut s, info.n_cap);
    s.len("bore_l", format!("{} mm", n(info.l)), "the bore from the seat to the sole: the solver's, for the reference's low G");
    s.len("x0", "(n_seat + n_collar)*ply", "read only: the strips' top, in the collar's socket plate");
    s.len("ls", "bore_l - n_collar*ply", "read only: the strips' length");
    s.len("finger", format!("{} mm", n(sc::FINGER)), "a joint cell's length, about");
    s.len("min_finger", format!("{} mm", n(sc::MIN_FINGER)), "no finger or gap shorter than this: a scrap beside a hole's guard goes to the guard");
    s.len("min_web", format!("{} mm", n(pc::MIN_WEB)), "the guard round each hole, where its strip owns both corners");
    s.num("ncell", "max(1, round(ls/finger))", "read only: joint cells along each corner");
    s.len("cell", "ls/ncell", "read only: each cell's length");
    for (i, h) in info.holes.iter().enumerate() {
        let k = i + 1;
        s.len(&format!("z_{k}"), format!("{} mm", n(h.z)), &format!("the {} hole, down from the seat: the reference's", h.name));
        s.len(&format!("d_{k}"), format!("{} mm", n(h.d)), &format!("its diameter: the solver's, for the reference's note"));
        s.len(&format!("xh_{k}"), format!("z_{k} - n_collar*ply"), &format!("read only: its centre down the strip"));
    }
    let pairs = guards(&mut s, &info.holes, "d");
    s.len("band", format!("{} mm", n(sd::BAND)), "the band's length, and the side pieces' quarter circle");
    s.len("foot_d", format!("{} mm", n(sd::FOOT_D)), "the round foot");
    s.len("collar_d", format!("{} mm", n(sd::COLLAR_D)), "the collar's socket plate");
    let t = "ply";
    // the strips: lo and hi edge corners, and each one's place
    let strips: [(&str, &str, &str, Place); 4] = [
        ("front", "FL", "FR", Place([(0, 1.0, "x0".into()), (2, 1.0, "-a".into()), (1, -1.0, "side/2 + ply".into())])),
        ("back", "KR", "KL", Place([(0, 1.0, "x0".into()), (2, -1.0, "a".into()), (1, 1.0, "-(side/2 + ply)".into())])),
        ("left", "FL", "KL", Place([(0, 1.0, "x0".into()), (1, 1.0, "-(side/2 + ply)".into()), (2, 1.0, "side/2".into())])),
        ("right", "KR", "FR", Place([(0, 1.0, "x0".into()), (1, -1.0, "side/2 + ply".into()), (2, -1.0, "-side/2".into())])),
    ];
    let base = |c: &str| match c {
        "FL" => ["front", "left"],
        "FR" => ["right", "front"],
        "KL" => ["left", "back"],
        _ => ["back", "right"],
    };
    // who a hole's guard gives each corner to
    let forced = |hole: &str, c: &str| -> &'static str {
        if hole != "hA" {
            match c {
                "FL" | "FR" => "front",
                "KL" => "left",
                _ => "right",
            }
        } else {
            match c {
                "KL" | "KR" => "back",
                "FL" => "left",
                _ => "right",
            }
        }
    };
    for (name, _, _, pl) in &strips {
        pl.bx(&mut s, &format!("{name} strip"), [("0 mm", "ls"), ("0 mm", "side + 2*ply"), ("0 mm", t)], "new", &[]);
    }
    for (name, lo, hi, pl) in &strips {
        let body = format!("{name} strip");
        for (c, u) in [(*lo, ("0 mm", "ply")), (*hi, ("side + ply", "side + 2*ply"))] {
            let pair = base(c);
            let other = if pair[0] == *name { pair[1] } else { pair[0] };
            let k0 = pair.iter().position(|w| *w == other).unwrap();
            // the neighbour's cells, every other one
            let f = format!("{name} strip, {c} cells");
            let cnt = format!("floor((ncell - {k0} + 1)/2)");
            let lift = format!("(1 - min(1, {cnt}))*1000 mm");
            pl.bx(&mut s, &f, [(&format!("{k0}*cell + {lift}"), &format!("({k0} + 1)*cell + {lift}")), u, ("0 mm", t)], "cut", &[&body]);
            s.pattern(&f, pl.dir(0), format!("max(1, {cnt})"), "2*cell");
            // each hole's guard: given back to this strip, or taken from it
            for (i, h) in info.holes.iter().enumerate() {
                let k = i + 1;
                let op = if forced(h.name, c) == *name { "join" } else { "cut" };
                pl.bx(&mut s, &format!("{name} strip, {c} by {}", h.name), [(&format!("g0_{k}"), &format!("g1_{k}")), u, ("0 mm", t)], op, &[&body]);
            }
            for &(i, j) in &pairs {
                let (oi, oj) = (forced(info.holes[i - 1].name, c), forced(info.holes[j - 1].name, c));
                if oi == oj {
                    let (lo, hi) = gap_span(i, j);
                    let op = if oi == *name { "join" } else { "cut" };
                    pl.bx(&mut s, &format!("{name} strip, {c} between {} and {}", info.holes[i - 1].name, info.holes[j - 1].name), [(&lo, &hi), u, ("0 mm", t)], op, &[&body]);
                }
            }
        }
        for (i, h) in info.holes.iter().enumerate() {
            let on = if h.name == "hA" { "back" } else { "front" };
            if on == *name {
                pl.cyl(&mut s, &format!("{} hole", h.name), [&format!("xh_{}", i + 1), "ply + side/2", "-1 mm"], &format!("d_{}", i + 1), "ply + 2 mm", "cut", &[&body]);
            }
        }
    }
    // the band: front, right and left, under the collar
    let bx_ = "x0 + ply";
    let b = "(a + ply)";
    let front = Place([(0, 1.0, bx_.into()), (2, 1.0, format!("-{b}")), (1, -1.0, b.to_string())]);
    front.bx(&mut s, "front band", [("0 mm", "band"), ("0 mm", &format!("2*{b}")), ("0 mm", t)], "new", &[]);
    for (name, pl, flip) in [
        ("right band", Place([(0, 1.0, bx_.into()), (1, -1.0, "a".into()), (2, -1.0, "-a".into())]), true),
        ("left band", Place([(0, 1.0, bx_.into()), (1, 1.0, "-a".into()), (2, 1.0, "a".into())]), false),
    ] {
        // a rectangle and a quarter circle, its corner cut away to the back
        let (r0, r1, cv) = if flip { ("band", "2*a", "band") } else { ("0 mm", "2*a - band", "2*a - band") };
        pl.bx(&mut s, name, [("0 mm", "band"), (r0, r1), ("0 mm", t)], "new", &[]);
        pl.cyl(&mut s, &format!("{name}, round"), ["0 mm", cv, "0 mm"], "2*band", t, "join", &[name]);
        pl.bx(&mut s, &format!("{name}, trim"), [("-band - 1 mm", "0 mm"), (&format!("{cv} - band - 1 mm"), &format!("{cv} + band + 1 mm")), ("-1 mm", "ply + 1 mm")], "cut", &[name]);
        let (o0, o1) = if flip { ("2*a", "2*a + band + 1 mm") } else { ("-band - 1 mm", "0 mm") };
        pl.bx(&mut s, &format!("{name}, trim past"), [("-1 mm", "band + 1 mm"), (o0, o1), ("-1 mm", "ply + 1 mm")], "cut", &[name]);
    }
    // the foot: each layer a pinwheel of four, each the foot's disc cut back
    // to beyond the strips on one side and past them on the other
    for k in 0..sd::N_FOOT {
        let sg = if k % 2 == 0 { 1.0 } else { -1.0 };
        let x = format!("x0 + ls - {}*ply", k + 1);
        let pl = Place::plate(&x);
        for i in 0..4 {
            let name = format!("foot {}, piece {}", k + 1, i + 1);
            pl.cyl(&mut s, &name, ["0 mm", "0 mm", "0 mm"], "foot_d", t, "new", &[]);
            let phi = i as f64 * PI / 2.0;
            let rot = |nu: f64, nv: f64| ((phi.cos() * nu - phi.sin() * nv).round(), (phi.sin() * nu + phi.cos() * nv).round());
            // v <= -a, and sg*u <= a, turned by phi: keep n.p <= c
            for (j, (nu, nv, c)) in [(0.0, 1.0, "-a"), (sg, 0.0, "a")].iter().enumerate() {
                let (ru, rv) = rot(*nu, *nv);
                let big = "foot_d/2 + 1 mm";
                // remove n.p > c: along the part's axis the normal lies on
                let (axis, sgn) = if ru != 0.0 { (0usize, ru) } else { (1usize, rv) };
                let range = if sgn > 0.0 { (c.to_string(), big.to_string()) } else { (format!("-({big})"), format!("-({c})")) };
                let mut r: [(String, String); 3] = [("-(foot_d)".into(), "foot_d".into()), ("-(foot_d)".into(), "foot_d".into()), ("-1 mm".into(), "ply + 1 mm".into())];
                r[axis] = range;
                pl.bx(&mut s, &format!("{name}, edge {}", j + 1), [(&r[0].0, &r[0].1), (&r[1].0, &r[1].1), (&r[2].0, &r[2].1)], "cut", &[&name]);
            }
        }
    }
    // the tenon, the collar and its socket plate: a square through it
    head(&mut s, &|s: &mut Script| {
        rod(s, "collar socket", "x0", "0 mm", "0 mm", "collar_d", "ply", "new");
        Place::plate("x0").bx(s, "collar socket, square", [("-a", "a"), ("-a", "a"), ("-1 mm", "ply + 1 mm")], "cut", &["collar socket"]);
    });
    s.measure();
    s.text()
}

/// The standard chanter on eight sides: eight finger-jointed strips round an
/// octagonal bore, the holes cut as slots.
pub fn standard_octagon() -> String {
    use crate::standard_chanter as sd;
    use crate::standard_octagon as so;
    use crate::strip_chanter as sc;
    let (_, info) = so::build();
    let mut s = Script::new();
    s.len("ply", format!("{} mm", n(m::plate())), "plate thickness: measure the ply, as the generator's --plate=");
    s.len("flats", format!("{} mm", n(so::FLATS)), "the octagonal bore across its flats, inside the eight strips");
    s.len("fo", "flats + 2*ply", "read only: the octagon across its flats outside");
    s.len("face_out", format!("fo*{}", n((PI / 8.0).tan())), "read only: each strip's width");
    s.len("cut", format!("ply*{}", n((135f64).to_radians().sin())), "read only: how far a strip stops short of an edge it does not own");
    s.len("min_web", format!("{} mm", n(pc::MIN_WEB)), "the guard round each slot, and the ply each side of it");
    s.len("slot_w", "face_out - 2*min_web", "read only: the widest slot across");
    s.len("slot_max", format!("{} mm", n(so::SLOT_MAX)), "the longest slot, under a finger's pad");
    head_params(&mut s, info.n_cap);
    s.len("bore_l", format!("{} mm", n(info.l)), "the bore from the seat to the sole: the solver's, for the reference's low G");
    s.len("x0", "(n_seat + n_collar)*ply", "read only: the strips' top, in the collar's socket plate");
    s.len("ls", "bore_l - n_collar*ply", "read only: the strips' length");
    s.len("finger", format!("{} mm", n(sc::FINGER)), "a joint cell's length, about");
    s.len("min_finger", format!("{} mm", n(sc::MIN_FINGER)), "no finger or gap shorter than this: a scrap beside a slot's guard goes to the guard");
    s.num("ncell", "max(1, round(ls/finger))", "read only: joint cells along each edge");
    s.len("cell", "ls/ncell", "read only: each cell's length");
    let mut alongs = vec![];
    for (i, h) in info.holes.iter().enumerate() {
        let k = i + 1;
        s.len(&format!("z_{k}"), format!("{} mm", n(h.z)), &format!("the {} hole, down from the seat: the reference's", h.name));
        s.len(&format!("d_{k}"), format!("{} mm", n(h.d)), "the round hole it stands for: the solver's diameter, for the reference's note");
        s.len(&format!("across_{k}"), format!("min(d_{k}, slot_w)"), "read only: its slot across");
        s.len(&format!("along_{k}"), format!("across_{k} + max(0, pi*(d_{k}^2 - slot_w^2)/4/slot_w)"), "read only: its slot along, for the round hole's area");
        s.len(&format!("xh_{k}"), format!("z_{k} - n_collar*ply"), "read only: its centre down the strip");
        alongs.push(format!("along_{k}"));
    }
    let pairs = guards(&mut s, &info.holes, "along");
    let widest = alongs.iter().skip(1).fold(alongs[0].clone(), |acc, x| format!("max({acc}, {x})"));
    s.len("slot_room", format!("slot_max - {widest}"), "read only: room left under slot_max for the longest slot; below 0 fails");
    s.len("foot_d", format!("{} mm", n(so::FOOT_D)), "the round foot's plates");
    s.num("n_foot", n(so::N_FOOT as f64), "the foot's plates, slid onto the strips' bottom");
    // the strips: each sketched on its outside face, x down the chanter, y
    // across the strip, extruded inward
    let rect = |s: &mut Script, k: usize, name: &str, x0: &str, x1: &str, y0: &str, y1: &str, op: &str, targets: &[&str]| {
        let th = (45.0 * k as f64).to_radians();
        let u = [0.0, -th.sin(), -th.cos()];
        let sk = s.sketch_on([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], u, Some("-(flats/2 + ply)"), &format!("{name} sketch"));
        let p = |x: &str, y: &str| [format!("x0 + ({x})"), format!("({y}) - face_out/2")];
        s.poly(&[p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)]);
        s.finish();
        s.extrude(&sk, "ply", false, op, targets, name);
    };
    let slot_on = |h: &str| if h == "hA" { 4 } else { 0 };
    for k in 0..8 {
        let body = format!("{} strip", so::WALLS[k]);
        rect(&mut s, k, &body, "0 mm", "ls", "0 mm", "face_out", "new", &[]);
        // lo edge: edge k, the strip's own even cells; hi edge: edge k - 1, its odd
        for (k0, y0, y1, edge) in [(1, "0 mm", "cut", "near"), (0, "face_out - cut", "face_out", "far")] {
            let f = format!("{body}, {edge} edge cells");
            let cnt = format!("floor((ncell - {k0} + 1)/2)");
            let lift = format!("(1 - min(1, {cnt}))*1000 mm");
            rect(&mut s, k, &f, &format!("{k0}*cell + {lift}"), &format!("({k0} + 1)*cell + {lift}"), y0, y1, "cut", &[&body]);
            s.pattern(&f, [1.0, 0.0, 0.0], format!("max(1, {cnt})"), "2*cell");
            for (i, h) in info.holes.iter().enumerate() {
                let hs = slot_on(h.name);
                let keep = if k0 == 1 { (k + 8 - hs) % 8 < 4 } else { (k + 7 - hs) % 8 >= 4 };
                rect(&mut s, k, &format!("{body}, {edge} edge by {}", h.name), &format!("g0_{}", i + 1), &format!("g1_{}", i + 1), y0, y1, if keep { "join" } else { "cut" }, &[&body]);
            }
            let keeps = |h: &str| {
                let hs = slot_on(h);
                if k0 == 1 { (k + 8 - hs) % 8 < 4 } else { (k + 7 - hs) % 8 >= 4 }
            };
            for &(i, j) in &pairs {
                let (ki, kj) = (keeps(info.holes[i - 1].name), keeps(info.holes[j - 1].name));
                if ki == kj {
                    let (lo, hi) = gap_span(i, j);
                    rect(&mut s, k, &format!("{body}, {edge} edge between {} and {}", info.holes[i - 1].name, info.holes[j - 1].name), &lo, &hi, y0, y1, if ki { "join" } else { "cut" }, &[&body]);
                }
            }
        }
        // the slots, through the front and back strips
        for (i, h) in info.holes.iter().enumerate() {
            if slot_on(h.name) != k {
                continue;
            }
            let i = i + 1;
            let y0 = if k == 0 { "-(flats/2 + ply) - 1 mm" } else { "flats/2 - 1 mm" };
            let half = format!("(along_{i} - across_{i})/2");
            let x = format!("x0 + xh_{i}");
            for (e, sg) in [(1, "-"), (2, "+")] {
                s.cylinder(&format!("{} slot, end {e}", h.name), [format!("{x} {sg} {half}"), y0.into(), "0 mm".into()], Some([0.0, 1.0, 0.0]), &format!("across_{i}"), "ply + 2 mm", "cut", &[&body]);
            }
            let on = format!("min(1, floor((along_{i} - across_{i})/(0.001 mm)))");
            s.bx(
                &format!("{} slot, middle", h.name),
                [format!("{x} - {half} + (1 - {on})*1000 mm"), y0.into(), format!("-across_{i}/2")],
                [format!("max(2*{half}, 0.01 mm)"), "ply + 2 mm".into(), format!("across_{i}")],
                "cut",
                &[&body],
            );
        }
    }
    // the foot: round plates with the octagon through them
    let sk = across(&mut s, "x0 + ls - ply", "foot plate sketch");
    s.cmd("sketch.circle.center", "{\"center\":[\"0 mm\",\"0 mm\"],\"diameter\":\"foot_d\"}".to_string());
    s.poly(&ngon_yz(8, "fo"));
    s.finish();
    s.extrude(&sk, "ply", false, "new", &[], "foot plate");
    s.pattern("foot plate", [-1.0, 0.0, 0.0], "n_foot", "ply");
    head(&mut s, &|s: &mut Script| {
        let sk = across(s, "x0", "collar socket sketch");
        s.cmd("sketch.circle.center", "{\"center\":[\"0 mm\",\"0 mm\"],\"diameter\":\"cap_d\"}".to_string());
        s.poly(&ngon_yz(8, "fo"));
        s.finish();
        s.extrude(&sk, "ply", false, "new", &[], "collar socket");
    });
    let _ = sd::SIDE;
    s.measure();
    s.text()
}
