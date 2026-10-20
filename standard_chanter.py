#!/usr/bin/env python3
"""Cut files for a Great Highland bagpipe practice chanter with a real
chanter's holes: where they are and about how big, on a square bore of four
long strips of ply, glued, all the way up to the collar the top rests on.

    python3 standard_chanter.py --no-write                  # the checks
    python3 standard_chanter.py --out=DIR                   # the sheet
    python3 standard_chanter.py --joint=butt --out=DIR      # butt-jointed strips
    python3 standard_chanter.py --plate=2.9 ...             # for ply as measured
    python3 standard_chanter.py --kerf=0.18 ...             # for that kerf

THE REFERENCE is a practice chanter measured from aero1016's model of the one
they own (Thingiverse thing 6658761): a 4.87mm round bore 238.5mm from the
sole to the reed's seat, front holes 25mm apart, the seat 5.57mm across, and
a collar 10mm under the seat that the top rests on. Its holes are kept where
they are. The bore here is square and bigger, SIDE across, and the holes go
through a 3mm strip instead of the reference's 5 to 7mm of wall, so each
hole's size is solved for its note to sound where the reference's does, in
chanter_model.py's terms; the strips are lengthened at the sole for low G.
The model cannot reproduce the reference's own scale exactly (it is about
40 cents out at best, with REF_REED of reed), so it is used only for the
difference between the two chanters, not for the pitch.

From the sole up:

  * the STRIPS, front, back and two sides, from the sole into the collar,
    finger-jointed (--joint=finger, the default) or butt-jointed, glued;
  * the FOOT, N_FOOT layers of strips lying flat round the strips' bottom,
    four to a layer, each layer turned the other way to the one below so no
    seam runs through; each strip's outer edge an arc of the reference's
    round foot, so it is FOOT_W wide only mid-face and shorter to the corners;
  * the BAND, three 10mm strips glued round the strips' front and sides,
    under the collar, which thicken them where they meet it; none on the
    back, where the thumb reaches up past its hole, and the side pieces cut
    away there along a quarter circle up to the collar;
  * the COLLAR: a COLLAR_D plate with a square hole the strips' top stands
    in, glued to the strips and the band; then N_COLLAR plates with the
    reference's round bore, as wide as the cap, which rests on them;
  * the TENON: N_SEAT round plates with the reed's seat through them, which
    the cap slides over. The collar's round plates and the tenon press
    together on two pins;
  * the CAP, the reference's: a plain round tube, REF_CAP_D outside and
    REF_CAP_BORE inside, REF_CAP_LEN long, its top plate closed but for the
    blowing hole; glued, no pins. Then the mouthpiece, tapered as the
    reference's is (practice_chanters.mouthpiece).

And a JIG, not part of the chanter: a base plate and N_FOOT rings with the
foot's circle cut through them, pinned into a cup. The chanter stands in it on
its sole while the foot's layers are glued round it, one at a time; the
circle centres each and the base keeps the foot flush with the sole.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import practice_chanters as P                                  # noqa: E402
import strip_chanter as SC                                     # noqa: E402
from practice_chanters import M, SF, Part, circle              # noqa: E402

KERF = P.KERF
# --- the reference chanter, measured (mm) -------------------------------------
REF_BORE = 4.87                    # round, the sole to the seat
REF_TOP = 238.5                    # the seat's bottom, up from the sole
REF_SEAT_D, REF_SEAT_DEEP = 5.57, 25.5
REF_COLLAR = 228.0                 # the collar's bottom, up from the sole
# each hole: up from the sole, across, and the wall it goes through
REF_HOLES = {'A': (43.02, 4.15, 7.4), 'B': (75.02, 3.95, 7.0), 'C': (100.02, 3.95, 6.6),
             'D': (125.02, 3.95, 6.3), 'E': (150.02, 3.95, 5.9), 'F': (175.03, 3.35, 5.5),
             'G': (195.43, 3.05, 5.2), 'hA': (207.43, 2.95, 5.1)}
REF_REED = 130.0                   # the reed allowance that fits the reference's
                                   # scale best in the model, 41 cents RMS
# --- this chanter ---------------------------------------------------------------
SIDE = 5.5                         # the square bore: the holes the reference's size
N_COLLAR = 2                       # round-bore plates over the socket plate
N_SEAT = 9                         # the tenon: 27mm of seat, the reference's 25.5
BAND = 10.0                        # the band's length, and the side pieces' quarter
                                   # circle above the thumb
REF_FOOT_D, REF_FOOT_H = 25.4, 19.0
FOOT_W = 7.0                       # the foot's strips: 11.5 + 2 x 7 = 25.5mm, and cut to
                                   # the reference's 25.4mm round foot, 6.95mm mid-face
N_FOOT = 2                         # layers: 6mm tall, as asked; the reference's is 19mm
JIG = 35.0                         # the foot jig's square: a base plate, then N_FOOT
JIG_CLEAR = 0.3                    # rings with the foot's circle this much over
# the cap, the reference's: a plain round tube over a round tenon, glued, no
# pins; its top plate closes it, and the mouthpiece goes on above that
REF_CAP_D, REF_CAP_BORE, REF_CAP_LEN = 22.2, 17.6, 122.0
TENON_D = REF_CAP_BORE - 2 * P.CAP_CLEAR   # 17.0: the cap slides over it
COLLAR_D = 25.4                    # the collar's socket plate, over the band's corners;
                                   # its round plates are the cap's own 22.2mm
JOINT = 'finger'


class Bore:
    """A bore for chanter_model's tube and hole: round d across, or square."""

    def __init__(self, d, square=False):
        self.A = (d / 1000) ** 2 if square else math.pi * (d / 2000) ** 2
        self.a = math.sqrt(self.A / math.pi)
        self.rh = d / 2000
        self.strip = True

    def k(self, f):
        return 2 * math.pi * f / M.C - 1j * 3.0e-5 * math.sqrt(f) / self.rh

    def z0(self):
        return M.RHO * M.C / self.A

    def hole_r(self, w):
        return w / 2000


def admittance(f, b, L, holes, opened):
    """Im(Y) the reed sees: REF_REED of closed bore, then L of bore b down to
    the open sole; holes (name, mm down from the seat, across, wall)."""
    m = M.tube(b, f, REF_REED / 1000)
    x = 0.0
    for name, z, d, wall in sorted(holes, key=lambda h: h[1]):
        m = M.mul(m, M.tube(b, f, (z - x) / 1000))
        m = M.mul(m, M.hole(b, f, d, wall, name in opened))
        x = z
    m = M.mul(m, M.tube(b, f, (L - x) / 1000))
    ka = 2 * math.pi * f / M.C * b.a
    zr = b.z0() * (0.25 * ka ** 2 + 1j * 0.6133 * ka)
    zin = (m[0][0] * zr + m[0][1]) / (m[1][0] * zr + m[1][1])
    return (1 / zin).imag


def sounding(b, L, holes, note, near):
    """The fingering's resonance nearest `near`."""
    fing = M.FINGERING[note]
    lo, hi = near / 1.3, near * 1.3
    Y = lambda f: admittance(f, b, L, holes, fing)
    best, pf, py = None, lo, Y(lo)
    for i in range(1, 121):
        f = lo + (hi - lo) * i / 120
        y = Y(f)
        if py < 0 <= y:
            a, c = pf, f
            for _ in range(40):
                mid = (a + c) / 2
                if Y(mid) < 0:
                    a = mid
                else:
                    c = mid
            r = (a + c) / 2
            if best is None or abs(r - near) < abs(best - near):
                best = r
        pf, py = f, y
    if best is None:
        raise ValueError(f'{note}: no resonance near {near:.0f}Hz')
    return best


def reference():
    """The reference's notes, in the model."""
    b = Bore(REF_BORE)
    holes = [(h, REF_TOP - z, d, w) for h, (z, d, w) in REF_HOLES.items()]
    low_a = sounding(b, REF_TOP, holes, 'low A', 255.0)
    return {nt: sounding(b, REF_TOP, holes, nt, low_a * r) for nt, r in zip(M.NOTES, M.RATIOS)}


def bisect(fn, goal, lo, hi, rising):
    for _ in range(36):
        mid = (lo + hi) / 2
        if (fn(mid) < goal) == rising:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def solve():
    """The bore's length and each hole's size that sound the reference's
    notes, the holes where the reference's are, through a strip wall."""
    ref = reference()
    t = M.PLATE
    b = Bore(SIDE, square=True)
    note_of = dict(zip(M.HOLES, M.SETS))
    holes = {h: [h, REF_TOP - z, d, t] for h, (z, d, w) in REF_HOLES.items()}
    L = REF_TOP
    d_max = SIDE - 0.35
    for _ in range(3):
        L = bisect(lambda x: sounding(b, x, list(holes.values()), 'low G', ref['low G']),
                   ref['low G'], REF_TOP - 20, REF_TOP + 40, rising=False)
        for h in M.HOLES:
            nt = note_of[h]

            def f(d):
                holes[h][2] = d
                return sounding(b, L, list(holes.values()), nt, ref[nt])
            if f(d_max) < ref[nt]:
                raise SystemExit(f'error: {h} is flat even {d_max:.2f}mm across.')
            holes[h][2] = bisect(f, ref[nt], M.D_MIN, d_max, rising=True)
    got = {nt: sounding(b, L, list(holes.values()), nt, ref[nt]) for nt in M.NOTES}
    return ref, got, L, holes


def build():
    ref, got, L, holes = solve()
    t = M.PLATE
    s = SIDE
    num = iter(range(1, 9999))
    parts = []

    def part(name, outline, cuts, x, kind, note='', rnd=None):
        p = Part(name, next(num), outline, note)
        p.cuts = list(cuts)
        p.x, p.kind, p.round = x, kind, rnd
        parts.append(p)
        return p
    # down the body from the seat's top: tenon, collar's round plates, the
    # socket plate the strips' top stands in, then the strips to the sole
    x_collar = N_SEAT * t
    x0 = x_collar + N_COLLAR * t               # the strips' top, in the socket plate
    seat_bottom = N_SEAT * t
    Ls = L - (x0 - seat_bottom)
    strip_holes = [('back' if h == 'hA' else 'front', z - (x0 - seat_bottom), d)
                   for h, z, d, w in holes.values()]
    SC.JOINT = JOINT
    own = SC.owners(Ls, strip_holes)
    for name, (lo, hi) in SC.STRIPS.items():
        runs = [[(a, c, w == name) for a, c, w, g in own[e]] for e in (lo, hi)]
        o = SC.strip_outline(Ls, runs[0], runs[1], s, t)
        cuts = [('circle', x, t + s / 2, d / 2) for st, x, d in strip_holes if st == name]
        p = part(f'{name} strip', o, cuts, x0, 'strip', f'the {name} strip')
        p.strip = name
    # the band, front and sides: the front piece across both front corners,
    # the sides flush with the back, which stays clear for the thumb
    a = s / 2 + t
    band_x = x0 + t
    # each side piece loses its lower back corner along a quarter circle,
    # BAND across, from its bottom edge up to its top at the back, so the
    # band slopes away above the thumb instead of ending square; u runs
    # front to back on the left piece and back to front on the right
    w = 2 * a
    arc = [(BAND * math.sin(math.pi / 2 * i / 30), w - BAND + BAND * math.cos(math.pi / 2 * i / 30))
           for i in range(31)]
    side = [(0, 0), (0, w)] + arc[1:] + [(BAND, 0)]
    for name, o in (('front', [(0, 0), (BAND, 0), (BAND, 2 * (a + t)), (0, 2 * (a + t))]),
                    ('right', [(x, w - u) for x, u in side][::-1]), ('left', side)):
        p = part(f'{name} band', o, [], band_x, 'band', f'the {name} band piece')
        p.strip = name
    # the foot: a pinwheel of flat strips each layer, the other hand each
    # time, each cut to the round foot
    R = REF_FOOT_D / 2
    b = a + FOOT_W
    e = math.sqrt(R * R - a * a)                   # where the circle meets an inner edge
    t0, t1 = math.atan2(-a, -e), math.atan2(-e, a)
    arc = [(R * math.cos(t0 + (t1 - t0) * i / 60), R * math.sin(t0 + (t1 - t0) * i / 60))
           for i in range(61)]
    for k in range(N_FOOT):
        sg = 1 if k % 2 == 0 else -1
        for i in range(4):
            # one piece along the front, from the circle round past one
            # front corner to the inner corner beyond, turned a quarter
            # round for each next side
            pts = [(a * sg, -a), (-e * sg, -a)] + [(x * sg, y) for x, y in arc[1:-1]] + [(a * sg, -e)]
            c, sn = [(1, 0), (0, 1), (-1, 0), (0, -1)][i]
            pts = [(c * x - sn * y, sn * x + c * y) for x, y in pts]
            if sg < 0:
                pts = pts[::-1]
            part('foot', pts, [], x0 + Ls - (k + 1) * t, 'foot', f'foot layer {k + 1}')
    # the jig: a cup the foot is glued up in
    jo = P.ngon(4, JIG / 2)
    pin_d = P.PIN - P.PIN_INTERF
    jr = (R + JIG_CLEAR / 2 + pin_d / 2 + P.MIN_WEB
          + (JIG / 2 - pin_d / 2 - P.MIN_WEB) * math.sqrt(2)) / 2
    jig_pins = [('circle', x, y, pin_d / 2) for x, y in (P.polar(45, jr), P.polar(225, jr))]
    part('jig base', jo, list(jig_pins), 0.0, 'jig', 'the foot jig\'s base')
    for k in range(N_FOOT):
        part('jig ring', jo, [('circle', 0.0, 0.0, R + JIG_CLEAR / 2)] + jig_pins, 0.0, 'jig',
             f'the foot jig, ring {k + 1}')
    # the tenon and the collar, round, pinned together midway between the
    # seat and the tenon's edge
    tenon = P.circle(0, 0, TENON_D / 2, 72)
    r_pin = (REF_SEAT_D / 2 + TENON_D / 2) / 2
    pins = [P.polar(45, r_pin), P.polar(225, r_pin)]
    pin_cut = [('circle', x, y, (P.PIN - P.PIN_INTERF) / 2) for x, y in pins]
    for k in range(N_SEAT):
        part('tenon', tenon, [('circle', 0.0, 0.0, REF_SEAT_D / 2)] + pin_cut, k * t, 'seat',
             rnd=TENON_D / 2)
    collar = P.circle(0, 0, REF_CAP_D / 2, 72)
    for k in range(N_COLLAR):
        part('collar', collar, [('circle', 0.0, 0.0, REF_BORE / 2)] + pin_cut,
             x_collar + k * t, 'collar', rnd=REF_CAP_D / 2)
    tube = P.ngon(4, s / 2 + t)
    part('collar socket', P.circle(0, 0, COLLAR_D / 2, 72), [('poly', tube)], x0, 'sock',
         rnd=COLLAR_D / 2)
    # the cap: rings from the collar up, the last one closed but for the
    # blowing hole, then the mouthpiece
    n_cap = int(round(REF_CAP_LEN / t))
    ring = P.circle(0, 0, REF_CAP_D / 2, 72)
    for k in range(n_cap - 1):
        part('cap', ring, [('circle', 0.0, 0.0, REF_CAP_BORE / 2)], x_collar - (k + 1) * t,
             'socket' if k < N_SEAT else 'chamber', rnd=REF_CAP_D / 2)
    x_top = x_collar - n_cap * t
    part('cap top', ring, [('circle', 0.0, 0.0, P.MOUTH_D / 2)], x_top, 'top', rnd=REF_CAP_D / 2)
    P.mouthpiece(part, x_top)
    info = dict(ref=ref, got=got, L=L, holes=holes, s=s, t=t, Ls=Ls, x0=x0, own=own,
                strip_holes=strip_holes, seat_bottom=seat_bottom, tenon=tenon, pins=pins,
                collar=collar, a=a, b=b, R=R, band_x=band_x, n_cap=n_cap)
    return parts, info


def checks(parts, info, sheets):
    out = []

    def note(ok, what, detail):
        out.append((ok, what, detail))
    ref, got = info['ref'], info['got']
    s, t, Ls = info['s'], info['t'], info['Ls']
    off = {nt: 1200 * math.log2(got[nt] / ref[nt]) for nt in M.NOTES}
    worst = max(abs(c) for c in off.values())
    note(worst <= 1.0, 'every note sounds where the reference\'s does, in the model',
         f'within {worst:.2f} cents; the reference\'s low A {ref["low A"]:.0f}Hz in the model')
    moved = max(abs((REF_TOP - REF_HOLES[h][0]) - info['holes'][h][1]) for h in M.HOLES)
    note(moved < 1e-9, 'every hole is where the reference\'s is',
         ', '.join(f'{h} {REF_TOP - REF_HOLES[h][0]:.1f}' for h in M.HOLES) + 'mm under the seat')
    ds = [info['holes'][h][2] for h in M.HOLES]
    note(all(M.D_MIN <= d <= s - 0.35 + 1e-9 for d in ds), 'every hole cuts round, and fits the bore\'s face',
         ', '.join(f'{h} {info["holes"][h][2]:.2f}' for h in M.HOLES)
         + f'mm, against the reference\'s {min(v[1] for v in REF_HOLES.values()):.2f} to '
         f'{max(v[1] for v in REF_HOLES.values()):.2f}; the face {s:g}mm')
    # the thumb: nothing on the back between its hole and the collar, which
    # is at least as far above it as the reference's collar is
    hz, hd = next((x, d) for st, x, d in info['strip_holes'] if st == 'back')
    room = hz - t                                     # to the collar's bottom face
    ref_room = REF_COLLAR - REF_HOLES['hA'][0]
    backs = [p for p in parts if p.kind == 'band' and p.strip == 'back']
    note(not backs and room >= ref_room - 1e-9, 'the thumb has the reference\'s room above its hole',
         f'no band on the back; {room:.1f}mm from the hole\'s centre to the collar, the '
         f'reference {ref_room:.1f}mm')
    # the foot: each layer fills the frame round the strips once, and
    # stops short of the low A hole
    a, R = info['a'], info['R']
    feet = [p for p in parts if p.kind == 'foot']
    bad = 0
    for k in range(N_FOOT):
        layer = [p for p in feet if abs(p.x - (info['x0'] + Ls - (k + 1) * t)) < 1e-9]
        for i in range(50):
            for j in range(50):
                q = (-R + (i + 0.5) * 2 * R / 50, -R + (j + 0.5) * 2 * R / 50)
                r = math.hypot(*q)
                if abs(r - R) < 0.05:
                    continue                          # on the arc, between it and its chords
                want = 0 if max(abs(q[0]), abs(q[1])) < a or r > R else 1
                bad += sum(P.inside(q, p.outline) for p in layer) != want
    widest = max(math.hypot(*v) for p in feet for v in p.outline)
    mid = R - a
    lowest = max(x + d / 2 for st, x, d in info['strip_holes'])
    clear = (Ls - N_FOOT * t) - lowest
    note(bad == 0 and len(feet) == 4 * N_FOOT and clear >= P.MIN_WEB,
         'the foot is the reference\'s width, and clear of the low A hole',
         f'{2 * widest:.1f}mm round and {N_FOOT * t:g}mm tall, against the reference\'s '
         f'{REF_FOOT_D:g}mm and {REF_FOOT_H:g}mm; {N_FOOT} layers of 4 strips, {mid:.2f}mm wide '
         f'mid-face; {clear:.1f}mm under the low A hole')
    jig = [p for p in parts if p.kind == 'jig']
    rings = [c[3] for p in jig for c in p.cuts if c[0] == 'circle' and c[3] > 5]
    note(len(jig) == N_FOOT + 1 and len(rings) == N_FOOT
         and all(abs(r - (R + JIG_CLEAR / 2)) < 1e-9 for r in rings),
         'the foot jig holds every layer on the foot\'s circle',
         f'a base and {N_FOOT} rings, {N_FOOT * t:g}mm deep, the circle {2 * R + JIG_CLEAR:.1f}mm '
         f'for the foot\'s {2 * R:g}mm')
    # ply round each hole, and every corner one strip's
    ok = True
    for st, x, d in info['strip_holes']:
        for c in SC.STRIPS[st]:
            for a, e, w, g in info['own'][c]:
                if a < x + d / 2 + P.MIN_WEB and e > x - d / 2 - P.MIN_WEB and w != st:
                    ok = False
    note(ok, 'every hole keeps ply all round it',
         f'its strip fingered both sides, {P.MIN_WEB:g}mm past it along'
         if JOINT == 'finger' else 'the front and back are the full width')
    own = info['own']
    cover = all(abs(own[c][0][0]) < 1e-9 and abs(own[c][-1][1] - Ls) < 1e-9
                and all(abs(p[1] - q[0]) < 1e-9 for p, q in zip(own[c], own[c][1:]))
                for c in SC.CORNERS)
    runs = [e - a for c in SC.CORNERS for a, e, w, g in own[c] if not g]
    note(cover and (not runs or min(runs) >= SC.MIN_FINGER - 1e-9), 'every corner is one strip\'s, all the way',
         f'{JOINT} joints' + (f'; the shortest finger {min(runs):.1f}mm' if JOINT == 'finger' else ''))
    socket = [p for p in parts if p.kind == 'sock']
    into = len(socket) == 1 and abs(socket[0].x - info['x0']) < 1e-9
    note(into and abs(Ls + info['x0'] - info['seat_bottom'] - info['L']) < 1e-9,
         'the strips run from the sole into the collar',
         f'{Ls:.1f}mm, their top {t:g}mm into its socket plate; the bore {info["L"]:.1f}mm, '
         f'{info["L"] - REF_TOP:+.1f}mm on the reference\'s, at the sole, for low G')
    web = math.inf
    for p in parts:
        if p.kind not in ('seat', 'collar', 'sock', 'socket', 'chamber', 'top', 'stem', 'jig'):
            continue
        edges = [('circle', 0.0, 0.0, p.round) if p.round else ('poly', p.outline)] + p.cuts
        for c in p.cuts:
            if c[0] != 'circle' or abs(c[3] - (P.PIN - P.PIN_INTERF) / 2) > 1e-9:
                continue
            for e in edges:
                if e is c:
                    continue
                d = (P.poly_dist((c[1], c[2]), e[1]) if e[0] == 'poly'
                     else abs(math.hypot(c[1] - e[1], c[2] - e[2]) - e[3]))
                web = min(web, d - c[3])
    note(web >= P.MIN_WEB, 'every pin keeps ply between it and every cut',
         f'{web:.2f}mm at the least, against {P.MIN_WEB:g}')
    note(REF_SEAT_D >= P.STAPLE_MAX and REF_BORE >= P.STAPLE_MAX,
         'the seat is the reference\'s, and so is the bore under it',
         f'seat {REF_SEAT_D:g}mm, {N_SEAT * t:g}mm deep (the reference\'s {REF_SEAT_DEEP:g}); '
         f'the collar\'s bore {REF_BORE:g}mm stops a staple over that, as the reference\'s does')
    caps = [p for p in parts if p.kind in ('socket', 'chamber', 'top')]
    rings = [p for p in caps if p.kind != 'top']
    band_r = math.hypot(info['a'] + t, info['a'] + t)
    ok = (len(caps) == info['n_cap'] and all(abs(p.round - REF_CAP_D / 2) < 1e-9 for p in caps)
          and all(c[3] == REF_CAP_BORE / 2 for p in rings for c in p.cuts)
          and not any(len(p.cuts) > 1 for p in caps)
          and abs(max(p.x for p in rings) - (info['x0'] - (N_COLLAR + 1) * t)) < 1e-9
          and COLLAR_D / 2 > band_r)
    note(ok, 'the cap is the reference\'s, and slides over the tenon onto the collar',
         f'{len(caps)} plates, {len(caps) * t:g}mm long (the reference\'s {REF_CAP_LEN:g}), '
         f'{REF_CAP_D:g}mm round, {REF_CAP_BORE:g}mm bore, no pins; over a {TENON_D:g}mm tenon, '
         f'{P.CAP_CLEAR:g}mm clear all round; rests on the collar, as wide as it; the collar\'s '
         f'socket plate {COLLAR_D:g}mm, over the band\'s corners at {2 * band_r:.1f}mm')
    note(*P.mouthpiece_check(parts)[:1], 'the mouthpiece tapers as a real practice chanter\'s does',
         P.mouthpiece_check(parts)[1])
    note(all(sh['w'] <= P.BED[0] and sh['h'] <= P.BED[1] for sh in sheets), 'every sheet fits the bed',
         f'{len(sheets)} sheet(s), largest {max(sh["w"] for sh in sheets):.0f} x '
         f'{max(sh["h"] for sh in sheets):.0f}mm')
    least = math.inf
    for sh in sheets:
        its = sh['parts']
        for i, (a, ax, ay) in enumerate(its):
            for c, cx, cy in its[i + 1:]:
                if (cx > ax + a.w + SF.GAP + KERF or ax > cx + c.w + SF.GAP + KERF
                        or cy > ay + a.h + SF.GAP + KERF or ay > cy + c.h + SF.GAP + KERF):
                    continue
                least = min(least, P.apart(a, ax, ay, c, cx, cy))
    note(least >= SF.GAP + KERF - 1e-3, 'no two parts on a sheet come closer than the gap',
         f'{least - KERF:.2f}mm of ply between cuts at the least, against {SF.GAP:g}')
    placed = sum(len(sh['parts']) for sh in sheets)
    note(placed == len(parts), 'every part is on a sheet', f'{placed} of {len(parts)}')
    return out


def label():
    return f'standard-strips-{JOINT}-joint'


def svg_sheet(k, nsh, sh):
    txt = P.svg_sheet(k, nsh, sh, 4)
    what = (f'Practice chanter, a measured chanter\'s holes on a {SIDE:g}mm square bore of four '
            f'{JOINT}-jointed strips')
    return (txt.replace('Practice chanter, 4-sided bore', what)
            .replace('cuts the bores, seats and pin holes first; then black',
                     'cuts the bores, seats, finger and pin holes first; then black'))


def placed(p, info):
    """Where a part sits: strips as strip_chanter places them; the band's
    pieces round the front and sides."""
    s, t, a = info['s'], info['t'], info['a']
    if p.kind == 'strip':
        return SC.placed(p, dict(s=s, t=t, x0=info['x0']))
    if p.kind != 'band':
        return [[0, 0, 1, p.x], [0, 1, 0, 0], [-1, 0, 0, 0], [0, 0, 0, 1]]
    b, X = a + t, p.x
    return {'front': [[1, 0, 0, X], [0, 0, 1, -b], [0, -1, 0, b], [0, 0, 0, 1]],
            'right': [[1, 0, 0, X], [0, -1, 0, a], [0, 0, -1, -a], [0, 0, 0, 1]],
            'left': [[1, 0, 0, X], [0, 1, 0, -a], [0, 0, 1, a], [0, 0, 0, 1]]}[p.strip]


def design3d():
    """The chanter for practice_chanters.py's 3D page, finger-jointed."""
    global JOINT
    j, JOINT = JOINT, 'finger'
    parts, info = build()
    JOINT = j
    import turn3d as t3
    xs = [p.x for p in parts] + [info['x0'] + info['Ls']]
    mid = (min(xs) + max(xs)) / 2
    out = []
    for p in parts:
        if p.kind == 'jig':
            continue                                  # a tool, not the chanter
        holes = [c[1] if c[0] == 'poly' else circle(c[1], c[2], c[3], 20) for c in p.cuts]
        cap = p.kind in ('socket', 'chamber', 'top', 'stem')
        if p.kind in ('strip', 'band'):
            d = {'front': (0, -1, 0), 'back': (0, 1, 0), 'left': (0, 0, 1),
                 'right': (0, 0, -1)}[p.strip]
            ex = (0, d[1] * 0.12, d[2] * 0.12)
        else:
            ex = ((p.x - mid) / 60 - (1.0 if cap else 0.0), 0, 0)
        wood = 'dark' if p.kind == 'strip' else 'pale' if cap else 'ply'
        if p.kind == 'foot':
            ex = ((p.x - mid) / 60, 0, 0)
        out.append(t3.placed({'o': p.outline, 'h': holes, 'e': []}, placed(p, info), ex, wood))
    return {'parts': out, 'ex': 60, 'view': [-0.12, 0.32, 3.0]}


def main():
    global KERF, JOINT
    out, write = None, True
    for x in sys.argv[1:]:
        if x == '--no-write':
            write = False
        elif x.startswith('--out='):
            out = x.split('=', 1)[1]
        elif x.startswith('--joint='):
            JOINT = x.split('=', 1)[1]
            if JOINT not in ('finger', 'butt'):
                raise SystemExit(f'error: --joint={JOINT}: finger or butt.')
        elif x.startswith('--plate='):
            M.PLATE = float(x.split('=', 1)[1])
        elif x.startswith('--kerf='):
            KERF = P.KERF = SF.set_kerf(x)
        else:
            raise SystemExit(f'error: {x} is not a flag this generator reads. It takes --joint=, '
                             f'--out=, --plate=, --kerf= and --no-write.')
    if write and not out:
        raise SystemExit('error: --out=DIR, or --no-write to run the checks only.')
    parts, info = build()
    sheets = P.pack(parts)
    res = checks(parts, info, sheets)
    t = info['t']
    print(f'practice chanter, a measured chanter\'s holes on a {SIDE:g}mm square bore of four '
          f'{JOINT}-jointed strips: strips {info["Ls"]:.1f}mm long, {SIDE + 2 * t:g}mm outside; '
          f'body {info["Ls"] + (info["x0"]):.1f}mm, sole to tenon top; '
          f'{len([p for p in parts if p.kind in ("socket", "chamber", "top", "stem")])} cap plates')
    for h in M.HOLES:
        z_ref, d_ref, w = REF_HOLES[h]
        _, z, d, _ = info['holes'][h]
        up = info['L'] - z
        print(f'  {h:3} {"back " if h == "hA" else "front"} strip, {up:6.1f}mm up from the sole '
              f'({z:5.1f}mm under the seat, as the reference\'s), {d:.2f}mm round '
              f'(the reference {d_ref:.2f})')
    print(f'  top down: {N_SEAT} tenon, {N_COLLAR} collar, 1 collar socket over the strips; '
          f'the band, {BAND:g}mm, front and sides, under it; at the sole the round foot, '
          f'{N_FOOT} layers of 4 strips')
    bad = 0
    for ok, what, detail in res:
        print(f'  {"pass" if ok else "FAIL"}  {what:52} {detail}')
        bad += not ok
    if bad:
        print(f'\n  {bad} check(s) failed. Nothing written.')
        return 1
    if write:
        os.makedirs(out, exist_ok=True)
        for k, sh in enumerate(sheets, 1):
            f = os.path.join(out, f'practice-chanter-{label()}-sheet{k}-cut-files.svg')
            open(f, 'w').write(svg_sheet(k, len(sheets), sh))
            print(f'  wrote {os.path.basename(f)}  {sh["w"]:.0f} x {sh["h"]:.0f}mm, '
                  f'{len(sh["parts"])} parts')
    return 0


if __name__ == '__main__':
    sys.exit(main())
