#!/usr/bin/env python3
"""Cut files for the standard chanter on eight sides: a Great Highland
practice chanter with the measured chanter's holes, where they are, on an
octagonal bore of eight long strips of ply, finger-jointed and glued, the
holes cut as slots.

    python3 standard_octagon.py --no-write                  # the checks
    python3 standard_octagon.py --out=DIR                   # the sheet
    python3 standard_octagon.py --plate=2.9 ...             # for ply as measured
    python3 standard_octagon.py --kerf=0.18 ...             # for that kerf

THE REFERENCE is standard_chanter.py's: aero1016's practice chanter, its
holes kept where they are, measured from the reed's seat, and the model used
only for the difference between the two chanters, not for the pitch.

THE BORE is a regular octagon FLATS across its flats. The strips meet at 135
degrees by the four-hole ocarinas' joint: in its own fingers a strip runs out
to the corner, in its neighbour's it stops CUT short, square to its face. So
every corner keeps a small triangular groove along the bore, CUT less the
ply's own bevel each way; the bore's area counts the eight of them.

THE SLOTS: an octagon's face is narrow, so a round hole big enough for its
note will not fit one strip with ply each side of it. Each hole is a slot
along the strip instead, SLOT_W across, as long as its note needs; it is
modelled as the round hole of its area, as the plate chanters' slots are. A
hole needing less than SLOT_W is cut round.

THE JOINTS: the fingers run as a pinwheel, each strip owning one of its two
edges at a time, so no strip is ever cut back on both. Round each slot its
strip owns both edges, MIN_WEB past it each way, and every strip round from
it gives up its near edge to own its far one. With eight strips and eight
edges that leaves the strip opposite owning neither there, narrowed to
FACE_OUT - 2 CUT for the slot's length: the back strip opposite the finger
holes, the front strip opposite the thumb's.

The model is standard_chanter.py's, the reed's allowance taken as more of
this chanter's own bore, as the square's is, so the two are alike. That is a
choice, not a measurement: taken instead as the reference's round bore above
the octagon, low G would want the bore 328mm long, not 242 (the square's 288).

From the sole up: the FOOT, N_FOOT round plates with the octagon through them,
slid onto the strips' bottom and glued flush with the sole; the STRIPS into
the collar's socket plate; then the collar, tenon, cap and mouthpiece,
standard_chanter.py's.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import practice_chanters as P                                  # noqa: E402
import standard_chanter as S                                   # noqa: E402
import strip_chanter as SC                                     # noqa: E402
from practice_chanters import M, SF, Part, circle              # noqa: E402

KERF = P.KERF
N = 8
FLATS = 8.0                        # the octagon across its flats, inside
TAN = math.tan(math.pi / N)
SLOT_MAX = 12.0                   # the longest slot, along: under a finger's pad
FOOT_D = S.FOOT_D                  # the foot, the square chanter's 50mm round
N_FOOT = S.N_FOOT
SOCK_D = S.REF_CAP_D               # the collar's socket plate, flush with the cap
WALLS = ['front', 'front left', 'left', 'back left', 'back', 'back right', 'right',
         'front right']


def dims(t=None):
    """The octagon's widths (mm): each strip's face inside and out, how far
    a strip stops short of a corner it does not own, the groove that
    leaves, and the slots' width."""
    t = M.PLATE if t is None else t
    face_in = FLATS * TAN
    face_out = (FLATS + 2 * t) * TAN
    cut = t * math.sin(math.radians(135))
    groove = cut - t * TAN
    slot_w = face_out - 2 * P.MIN_WEB
    return face_in, face_out, cut, groove, slot_w


class Bore(S.Bore):
    """The octagon, with its eight corner grooves, as chanter_model's tube."""

    def __init__(self):
        super().__init__(FLATS, square=True)
        g = dims()[3]
        self.area = N * TAN * (FLATS / 2) ** 2 + N * g * g / 2      # mm^2
        self.A = self.area / 1e6
        self.a = math.sqrt(self.A / math.pi)


def slot_len(d):
    """The slot SLOT_W across with a d round hole's area; d itself if a
    round hole that size fits."""
    w = dims()[4]
    if d <= w:
        return d
    area = math.pi * d * d / 4
    return (area - math.pi * w * w / 4) / w + w


def solve():
    """The bore's length and each hole's round equivalent that sound the
    reference's notes, the holes where the reference's are, through a strip."""
    ref = S.reference()
    t = M.PLATE
    b = Bore()
    w = dims()[4]
    note_of = dict(zip(M.HOLES, M.SETS))
    holes = {h: [h, S.REF_TOP - z, d, t] for h, (z, d, wl) in S.REF_HOLES.items()}
    d_max = 2 * math.sqrt((w * (SLOT_MAX - w) + math.pi * w * w / 4) / math.pi)
    L = S.REF_TOP
    for _ in range(3):
        L = S.bisect(lambda x: S.sounding(b, x, list(holes.values()), 'low G', ref['low G']),
                     ref['low G'], S.REF_TOP - 20, S.REF_TOP + 40, rising=False)
        for h in M.HOLES:
            nt = note_of[h]

            def f(d):
                holes[h][2] = d
                return S.sounding(b, L, list(holes.values()), nt, ref[nt])
            if f(d_max) < ref[nt]:
                raise SystemExit(f'error: {h} is flat even as a {SLOT_MAX:g}mm slot.')
            holes[h][2] = S.bisect(f, ref[nt], M.D_MIN, d_max, rising=True)
    got = {nt: S.sounding(b, L, list(holes.values()), nt, ref[nt]) for nt in M.NOTES}
    return ref, got, L, holes, b


def owners(Ls, slots):
    """Each edge's owner along the strips, as [(x0, x1, strip, guarded)]
    from the strips' top. Edge k is between strip k and k + 1. In turn the
    fingers go to one strip and the other, all eight edges together, a
    pinwheel; round each slot (strip, x, along) its strip owns both edges,
    MIN_WEB past it, and each strip round from it owns its far edge."""
    n = max(1, int(round(Ls / SC.FINGER)))
    marks = [Ls * i / n for i in range(n + 1)]
    guard = [(x - ln / 2 - P.MIN_WEB, x + ln / 2 + P.MIN_WEB, k) for k, x, ln in slots]
    cuts = sorted(set(marks + [g for a, b, k in guard for g in (a, b)]))
    out = {}
    for e in range(N):
        iv = []
        for x0, x1 in zip(cuts, cuts[1:]):
            mid = (x0 + x1) / 2
            i = min(int(mid / Ls * n), n - 1)
            who = e if i % 2 == 0 else (e + 1) % N
            h = next((k for a, b, k in guard if a <= mid <= b), None)
            if h is not None:
                j = (e - h) % N
                who = (h + j) % N if j < N // 2 else (e + 1) % N
            iv.append([x0, x1, who, h is not None])
        out[e] = SC.tidy(iv)
    return out


def stadium(cx, cy, along, across, steps=16):
    """A slot centred (cx, cy), `along` long in x and `across` wide in y;
    a circle when the two are equal."""
    r, s = across / 2, (along - across) / 2
    pts = [(cx + s + r * math.cos(-math.pi / 2 + math.pi * k / steps),
            cy + r * math.sin(-math.pi / 2 + math.pi * k / steps)) for k in range(steps + 1)]
    pts += [(cx - s + r * math.cos(math.pi / 2 + math.pi * k / steps),
             cy + r * math.sin(math.pi / 2 + math.pi * k / steps)) for k in range(steps + 1)]
    out = []
    for p in pts:
        if not out or math.hypot(p[0] - out[-1][0], p[1] - out[-1][1]) > 1e-9:
            out.append(p)
    if math.hypot(out[0][0] - out[-1][0], out[0][1] - out[-1][1]) < 1e-9:
        out.pop()
    return out


def build():
    ref, got, L, holes, b = solve()
    t = M.PLATE
    face_in, face_out, cut, groove, slot_w = dims()
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
    x_collar = S.N_SEAT * t
    x0 = x_collar + S.N_COLLAR * t
    seat_bottom = S.N_SEAT * t
    Ls = L - (x0 - seat_bottom)
    # the slots: (strip, along the strip from its top, round equivalent, length)
    slots = [(4 if h == 'hA' else 0, z - (x0 - seat_bottom), d, slot_len(d))
             for h, z, d, w in holes.values()]
    own = owners(Ls, [(k, x, ln) for k, x, d, ln in slots])
    for k in range(N):
        # u = 0 is edge k, toward strip k + 1; u = face_out edge k - 1
        lo = [(a, c, w == k) for a, c, w, g in own[k]]
        hi = [(a, c, w == k) for a, c, w, g in own[(k - 1) % N]]
        o = SC.strip_outline(Ls, lo, hi, face_out - 2 * t, t, cut)
        cuts = [('poly', stadium(x, face_out / 2, ln, min(slot_w, d)))
                for st, x, d, ln in slots if st == k]
        p = part(f'{WALLS[k]} strip', o, cuts, x0, 'strip', f'the {WALLS[k]} strip')
        p.strip = k
    # the foot: round plates with the octagon through them
    sock = P.ngon(N, FLATS / 2 + t)
    for k in range(N_FOOT):
        part('foot', P.circle(0, 0, FOOT_D / 2, 72), [('poly', sock)], x0 + Ls - (k + 1) * t,
             'foot', f'foot plate {k + 1}', rnd=FOOT_D / 2)
    tenon, pins, collar, n_cap = S.head(part, x_collar, x0, [('poly', sock)], SOCK_D)
    info = dict(ref=ref, got=got, L=L, holes=holes, b=b, t=t, Ls=Ls, x0=x0, own=own,
                slots=slots, seat_bottom=seat_bottom, pins=pins, n_cap=n_cap, sock=sock,
                face_in=face_in, face_out=face_out, cut=cut, groove=groove, slot_w=slot_w)
    return parts, info


def widths(info):
    """Each strip's outside width along it: [(x0, x1, width, guarded)]."""
    own, Ls, W, c = info['own'], info['Ls'], info['face_out'], info['cut']
    out = {}
    for k in range(N):
        lo, hi = own[k], own[(k - 1) % N]
        xs = sorted(set([r[0] for r in lo + hi] + [Ls]))
        runs = []
        for a, e in zip(xs, xs[1:]):
            m = (a + e) / 2
            rl = next(r for r in lo if r[0] <= m <= r[1])
            rh = next(r for r in hi if r[0] <= m <= r[1])
            w = W - c * (rl[2] != k) - c * (rh[2] != k)
            runs.append((a, e, w, rl[3] or rh[3]))
        out[k] = runs
    return out


def checks(parts, info, sheets):
    out = []

    def note(ok, what, detail):
        out.append((ok, what, detail))
    ref, got, t, Ls = info['ref'], info['got'], info['t'], info['Ls']
    off = {nt: 1200 * math.log2(got[nt] / ref[nt]) for nt in M.NOTES}
    worst = max(abs(c) for c in off.values())
    note(worst <= 1.0, 'every note sounds where the reference\'s does, in the model',
         f'within {worst:.2f} cents; the reference\'s low A {ref["low A"]:.0f}Hz in the model')
    moved = max(abs((S.REF_TOP - S.REF_HOLES[h][0]) - info['holes'][h][1]) for h in M.HOLES)
    note(moved < 1e-9, 'every hole is where the reference\'s is',
         ', '.join(f'{h} {S.REF_TOP - S.REF_HOLES[h][0]:.1f}' for h in M.HOLES) + 'mm under the seat')
    ref_area = math.pi * S.REF_BORE ** 2 / 4
    oct_area = N * TAN * (FLATS / 2) ** 2
    note(abs(info['b'].area - oct_area - N * info['groove'] ** 2 / 2) < 1e-9,
         'the bore is the octagon and its corner grooves',
         f'{FLATS:g}mm across the flats, {oct_area:.1f}mm^2 and {N} grooves of '
         f'{info["groove"]:.2f}mm, {info["b"].area:.1f}mm^2: {info["b"].area / ref_area:.1f} '
         f'times the reference\'s {S.REF_BORE:g}mm round')
    sl = info['slots']
    w, W, fin = info['slot_w'], info['face_out'], info['face_in']
    note(all(ln <= SLOT_MAX + 1e-9 for k, x, d, ln in sl) and w <= fin - 0.35 + 1e-9,
         'every slot opens into its face, and fits under a finger',
         ', '.join(f'{h} {ln:.1f}' for h, (k, x, d, ln) in zip(M.HOLES, sl))
         + f'mm long, {w:.2f}mm across, the face {fin:.2f}mm inside; {SLOT_MAX:g}mm at most')
    # ply round each slot: its strip owns both edges MIN_WEB past it, and
    # (W - w) / 2 of ply each side
    ok = True
    for k, x, d, ln in sl:
        for e in (k, (k - 1) % N):
            for a, b, who, g in info['own'][e]:
                if a < x + ln / 2 + P.MIN_WEB and b > x - ln / 2 - P.MIN_WEB and who != k:
                    ok = False
    side = (W - w) / 2
    note(ok and side >= P.MIN_WEB - 1e-9, 'every slot keeps ply all round it',
         f'its strip owns both edges, {P.MIN_WEB:g}mm past it along; {side:.2f}mm each side')
    own = info['own']
    cover = all(abs(own[e][0][0]) < 1e-9 and abs(own[e][-1][1] - Ls) < 1e-9
                and all(abs(p[1] - q[0]) < 1e-9 for p, q in zip(own[e], own[e][1:]))
                for e in range(N))
    runs = [b - a for e in range(N) for a, b, w_, g in own[e] if not g]
    note(cover and min(runs) >= SC.MIN_FINGER - 1e-9, 'every corner is one strip\'s, all the way',
         f'{sum(len(own[e]) for e in range(N))} fingers on {N} corners; the shortest unguarded '
         f'{min(runs):.1f}mm, against {SC.MIN_FINGER:g}')
    ws = widths(info)
    narrow = [(k, a, b, wd, g) for k in range(N) for a, b, wd, g in ws[k] if wd < W - info['cut'] - 1e-9]
    least = min(wd for k in range(N) for a, b, wd, g in ws[k])
    where = sorted(set(WALLS[k] for k, a, b, wd, g in narrow))
    note(least >= P.MIN_WEB and all(g for k, a, b, wd, g in narrow),
         'every strip keeps ply between its edges',
         f'{W - info["cut"]:.2f}mm wide outside but where a slot\'s strip owns both edges; '
         f'opposite it the {" and ".join(where)} strip {least:.2f}mm wide for '
         f'{sum(b - a for k, a, b, wd, g in narrow):.0f}mm in all, against {P.MIN_WEB:g}')
    # the thumb: nothing on the back between its slot and the collar
    k, hz, hd, hl = next(s for s in sl if s[0] == 4)
    room = hz - t
    ref_room = S.REF_COLLAR - S.REF_HOLES['hA'][0]
    note(room >= ref_room - 1e-9, 'the thumb has the reference\'s room above its hole',
         f'{room:.1f}mm from the slot\'s centre to the collar, the reference {ref_room:.1f}mm')
    socket = t
    note(all(x - ln / 2 - P.MIN_WEB >= socket for k, x, d, ln in sl),
         'every slot is below the collar',
         f'the highest {min(x - ln / 2 for k, x, d, ln in sl) - socket:.1f}mm under it')
    lowest = max(x + ln / 2 for k, x, d, ln in sl)
    clear = (Ls - N_FOOT * t) - lowest
    note(clear >= P.MIN_WEB, 'the foot is clear of the low A slot',
         f'{FOOT_D:g}mm round, {N_FOOT * t:g}mm tall; {clear:.1f}mm under the slot')
    note(abs(Ls + info['x0'] - info['seat_bottom'] - info['L']) < 1e-9,
         'the strips run from the sole into the collar',
         f'{Ls:.1f}mm, their top {t:g}mm into its socket plate; the bore {info["L"]:.1f}mm, '
         f'{info["L"] - S.REF_TOP:+.1f}mm on the reference\'s, at the sole, for low G')
    # the collar's pins land on a strip's top, inside the octagon's wall
    r = math.hypot(*info['pins'][0])
    pin_r = (P.PIN - P.PIN_INTERF) / 2
    inr = FLATS / 2
    note(inr + 0.2 <= r - pin_r and r + pin_r <= inr + t - 0.2,
         'the collar\'s pins land on the strips\' top',
         f'{r - pin_r:.2f} to {r + pin_r:.2f}mm out, the wall {inr:g} to {inr + t:g}mm')
    web = math.inf
    for p in parts:
        if p.kind not in ('seat', 'collar', 'sock', 'socket', 'chamber', 'top', 'stem'):
            continue
        edges = [('circle', 0.0, 0.0, p.round) if p.round else ('poly', p.outline)] + p.cuts
        for c in p.cuts:
            if c[0] != 'circle' or abs(c[3] - pin_r) > 1e-9:
                continue
            for e in edges:
                if e is c:
                    continue
                d = (P.poly_dist((c[1], c[2]), e[1]) if e[0] == 'poly'
                     else abs(math.hypot(c[1] - e[1], c[2] - e[2]) - e[3]))
                web = min(web, d - c[3])
    note(web >= P.MIN_WEB, 'every pin keeps ply between it and every cut',
         f'{web:.2f}mm at the least, against {P.MIN_WEB:g}')
    note(*S.seat_check(t))
    caps = [p for p in parts if p.kind in ('socket', 'chamber', 'top')]
    sock_r = max(math.hypot(*v) for v in info['sock'])
    note(len(caps) == info['n_cap'] and SOCK_D / 2 >= sock_r + P.MIN_WEB,
         'the cap is the reference\'s, and slides over the tenon onto the collar',
         f'{len(caps)} plates, {len(caps) * t:g}mm long, {S.REF_CAP_D:g}mm round; the socket '
         f'plate {SOCK_D:g}mm round over the octagon\'s {2 * sock_r:.1f}mm corners')
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
    return 'standard-octagon'


def svg_sheet(k, nsh, sh):
    txt = P.svg_sheet(k, nsh, sh, N)
    what = (f'Practice chanter, a measured chanter\'s holes as slots on a {FLATS:g}mm octagonal '
            f'bore of eight finger-jointed strips')
    return (txt.replace(f'Practice chanter, {N}-sided bore', what)
            .replace('cuts the bores, seats and pin holes first; then black',
                     'cuts the bores, seats, slots and pin holes first; then black'))


def placed(p, info):
    """Where a part sits: a plate as practice_chanters lays them; strip k
    with its outside facing out at k x 45 degrees from the front, -y, toward
    the left, +z; its x along the chanter, u across, w inward."""
    if p.kind != 'strip':
        return [[0, 0, 1, p.x], [0, 1, 0, 0], [-1, 0, 0, 0], [0, 0, 0, 1]]
    th = math.radians(45 * p.strip)
    n = (0.0, -math.cos(th), math.sin(th))
    u = (0.0, -math.sin(th), -math.cos(th))
    R, W = FLATS / 2 + info['t'], info['face_out']
    O = [info['x0'] if i == 0 else R * n[i] - W / 2 * u[i] for i in range(3)]
    return [[1, u[0], -n[0], O[0]], [0, u[1], -n[1], O[1]], [0, u[2], -n[2], O[2]],
            [0, 0, 0, 1]]


def design3d():
    """The chanter for practice_chanters.py's 3D page."""
    parts, info = build()
    import turn3d as t3
    xs = [p.x for p in parts] + [info['x0'] + info['Ls']]
    mid = (min(xs) + max(xs)) / 2
    out = []
    for p in parts:
        holes = [c[1] if c[0] == 'poly' else circle(c[1], c[2], c[3], 20) for c in p.cuts]
        cap = p.kind in ('socket', 'chamber', 'top', 'stem')
        if p.kind == 'strip':
            th = math.radians(45 * p.strip)
            ex = (0, -math.cos(th) * 0.12, math.sin(th) * 0.12)
        else:
            ex = ((p.x - mid) / 60 - (1.0 if cap else 0.0), 0, 0)
        wood = 'dark' if p.kind == 'strip' else 'pale' if cap else 'ply'
        out.append(t3.placed({'o': p.outline, 'h': holes, 'e': []}, placed(p, info), ex, wood))
    return {'parts': out, 'ex': 60, 'view': [-0.12, 0.32, 3.0]}


def main():
    global KERF
    out, write = None, True
    for x in sys.argv[1:]:
        if x == '--no-write':
            write = False
        elif x.startswith('--out='):
            out = x.split('=', 1)[1]
        elif x.startswith('--plate='):
            M.PLATE = float(x.split('=', 1)[1])
        elif x.startswith('--kerf='):
            KERF = P.KERF = SF.set_kerf(x)
        else:
            raise SystemExit(f'error: {x} is not a flag this generator reads. It takes '
                             f'--out=, --plate=, --kerf= and --no-write.')
    if write and not out:
        raise SystemExit('error: --out=DIR, or --no-write to run the checks only.')
    parts, info = build()
    sheets = P.pack(parts)
    res = checks(parts, info, sheets)
    t = info['t']
    print(f'practice chanter, a measured chanter\'s holes as slots on a {FLATS:g}mm octagonal '
          f'bore of eight finger-jointed strips: strips {info["Ls"]:.1f}mm long, '
          f'{FLATS + 2 * t:g}mm across the flats outside; body '
          f'{info["Ls"] + info["x0"]:.1f}mm, sole to tenon top')
    for h, (k, x, d, ln) in zip(M.HOLES, info['slots']):
        z_ref, d_ref, w = S.REF_HOLES[h]
        z = info['holes'][h][1]
        shape = (f'slot {ln:.2f} x {info["slot_w"]:.2f}mm' if ln > d - 1e-9 and d > info['slot_w']
                 else f'round {d:.2f}mm')
        print(f'  {h:3} {WALLS[k]:5} strip, {info["L"] - z:6.1f}mm up from the sole '
              f'({z:5.1f}mm under the seat), {shape}, as a {d:.2f}mm hole '
              f'(the reference {d_ref:.2f})')
    print(f'  top down: {S.N_SEAT} tenon, {S.N_COLLAR} collar, 1 collar socket over the strips; '
          f'at the sole {N_FOOT} foot plates')
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
