#!/usr/bin/env python3
"""Cut files for a Great Highland bagpipe practice chanter whose square bore
is four long strips of ply, glued, for the bottom four fifths of the body,
and a stack of plates (the cookies) above that, under the plate chanters'
cap and mouthpiece.

    python3 strip_chanter.py --no-write                  # the checks
    python3 strip_chanter.py --out=DIR                   # the sheet
    python3 strip_chanter.py --joint=butt --out=DIR      # butt-jointed strips
    python3 strip_chanter.py --plate=2.9 ...             # for ply as measured
    python3 strip_chanter.py --kerf=0.18 ...             # for that kerf
    python3 strip_chanter.py --switch=0.75 ...           # cookies from 3/4 up instead

From the sole up:

  * the STRIPS: front, back and two sides, PLATE thick, round the square
    bore. The finger holes are round holes cut in the front strip, the
    thumb's in the back, anywhere along it, so no hole waits on a plate.
    --joint=finger (the default) joints every corner with fingers, which
    find their own places as the strips go together; each strip always has
    fingers along one edge or the other, and round a hole its strip has
    them along both, so the ply each side of the hole is never a notch.
    --joint=butt makes the front and back the full width and the sides as
    narrow as the bore, glued between them.
  * the COOKIES, from just above the high A hole (or --switch= of the way
    up the body): N_SOCK plates with a square hole the strips' outside,
    glued over the strips' top; then plates with a 4mm round hole, the
    square's area, so no note moves; then the reed's seat. Over 50 plates
    are too many to press onto pins, so they are clamped on two M2 rods,
    as the plate chanters' bodies are.
  * the CAP, the plate chanters' own, which slides over the cookies and
    rests on the two top nuts.

The strips are glued, and into the socket: four strips cannot be pressed
together the way plates can. The cookies are clamped, the cap press-fit,
and shellac seals every seam and the bore.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import practice_chanters as P                                  # noqa: E402
from practice_chanters import M, SF, Part, offset, circle      # noqa: E402

KERF = P.KERF
SWITCH = None                      # the cookies' start up the body; None: above high A
N_SOCK = 5                         # cookie plates glued over the strips' top
COOKIE = 20.0                      # the cookies' square, mm: room for the rods' nuts
FINGER = 10.0                      # a finger's length, about
MIN_FINGER = 4.0                   # no finger or gap shorter than this
THUMB_ROOM = 9.0                   # high A's centre to the cookies: the block stands
                                   # 5mm proud of the strips, and the thumb's pad
                                   # must seal the hole below that shoulder
G_GAP = 16.0                       # F to G: a thin wall wants G nearer F
JOINT = 'finger'

# the corners, each owned by one strip at a time: front-left, front-right,
# back-left, back-right; and each strip's two edges, its u = 0 edge first,
# as it lies in the chanter (see place())
CORNERS = ('FL', 'FR', 'KL', 'KR')
STRIPS = {'front': ('FL', 'FR'), 'back': ('KR', 'KL'),
          'left': ('FL', 'KL'), 'right': ('KR', 'FR')}


def solve():
    """The tune, with the cookies' round bore in place: its length depends
    on the body's, which the tune sets, so twice round."""
    t = M.PLATE
    top = 0.0
    for _ in range(4):
        g, M.G_GAP = M.G_GAP, G_GAP
        try:
            b, L, pos, widths = M.design(4, top=top, strip=t)
        finally:
            M.G_GAP = g
        body = P.N_SEAT * t + L
        if SWITCH is None:
            # as many round plates as leave the socket THUMB_ROOM above the
            # thumb's hole
            n_round = int((pos['hA'] - THUMB_ROOM) // t) - N_SOCK
        else:
            n_round = int(round((1 - SWITCH) * body / t)) - P.N_SEAT - N_SOCK
        if n_round < 1:
            raise SystemExit(f'error: --switch={SWITCH:g} leaves no room for round bore '
                             f'under the seat.')
        if n_round * t == top:
            return b, L, pos, widths, n_round
        top = n_round * t
    raise SystemExit('error: the cookies\' length did not settle.')


def owners(Ls, holes):
    """Each corner's owner along the strips, as [(x0, x1, strip, guarded)]
    from the strips' top. In turn each strip has fingers along one edge and
    the other; round each hole its strip owns both corners, out to MIN_WEB
    past the hole, and the strip opposite gives way."""
    if JOINT == 'butt':
        own = {'FL': 'front', 'FR': 'front', 'KL': 'back', 'KR': 'back'}
        return {c: [(0.0, Ls, own[c], False)] for c in CORNERS}
    n = max(1, int(round(Ls / FINGER)))
    edges = [Ls * i / n for i in range(n + 1)]
    base = {'FL': ('front', 'left'), 'FR': ('right', 'front'),
            'KL': ('left', 'back'), 'KR': ('back', 'right')}
    guard = []
    for strip, x, d in holes:
        a, b = x - d / 2 - P.MIN_WEB, x + d / 2 + P.MIN_WEB
        guard.append((a, b, strip))
    # the cut points: the fingers' and every guard's ends
    cuts = sorted(set(edges + [g for a, b, s in guard for g in (a, b)]))
    out = {}
    for c in CORNERS:
        iv = []
        for x0, x1 in zip(cuts, cuts[1:]):
            mid = (x0 + x1) / 2
            i = min(int(mid / Ls * n), n - 1)
            who = base[c][i % 2]
            g = next((s for a, b, s in guard if a <= mid <= b), None)
            if g == 'front':
                who = 'front' if c in ('FL', 'FR') else ('left' if c == 'KL' else 'right')
            elif g == 'back':
                who = 'back' if c in ('KL', 'KR') else ('left' if c == 'FL' else 'right')
            iv.append([x0, x1, who, g is not None])
        out[c] = tidy(iv)
    return out


def tidy(iv):
    """Joins runs with one owner, then gives every unguarded run shorter than
    MIN_FINGER to its neighbours, shortest first."""
    def join(iv):
        res = []
        for x0, x1, w, g in iv:
            if res and res[-1][2] == w:
                res[-1][1] = x1
                res[-1][3] = res[-1][3] or g
            else:
                res.append([x0, x1, w, g])
        return res
    iv = join(iv)
    while True:
        short = [k for k, (x0, x1, w, g) in enumerate(iv) if x1 - x0 < MIN_FINGER - 1e-9 and not g]
        if not short:
            return [tuple(r) for r in iv]
        k = min(short, key=lambda k: iv[k][1] - iv[k][0])
        nb = iv[k - 1] if k else iv[k + 1]
        iv[k][2] = nb[2]
        iv = join(iv)


def strip_outline(Ls, lo, hi, s, t):
    """A strip from its top, x = 0, to the sole, x = Ls; u across it, the
    bore's face from u = t to t + s, a finger out to 0 or s + 2t where the
    strip owns that corner. lo, hi: its edges' runs, [(x0, x1, owns)]."""
    def edge(runs, out_u, in_u):
        pts = []
        for x0, x1, owns in runs:
            u = out_u if owns else in_u
            pts += [(x0, u), (x1, u)]
        return pts
    left = edge(lo, 0.0, t)
    right = edge(hi, s + 2 * t, s + t)
    pts = left + right[::-1]
    out = []
    for p in pts:
        if out and abs(out[-1][0] - p[0]) < 1e-9 and abs(out[-1][1] - p[1]) < 1e-9:
            continue
        out.append(p)
    # drop points in the middle of a straight run
    clean = []
    for i, p in enumerate(out):
        a, c = out[i - 1], out[(i + 1) % len(out)]
        if abs((p[0] - a[0]) * (c[1] - p[1]) - (p[1] - a[1]) * (c[0] - p[0])) < 1e-9:
            continue
        clean.append(p)
    return clean


def build():
    """The parts, and what the checks need."""
    b, L, pos, widths, n_round = solve()
    t = M.PLATE
    s = 2 * M.inradius(M.BORE_AREA, 4)
    top = n_round * t
    Ls = L - top
    x0 = (P.N_SEAT + n_round) * t          # the strips' top, down the body
    holes = [('back' if h == 'hA' else 'front', pos[h] - top, widths[h]) for h in M.HOLES]
    own = owners(Ls, holes)
    num = iter(range(1, 9999))
    parts = []

    def part(name, outline, cuts, x, kind, note='', rnd=None):
        p = Part(name, next(num), outline, note)
        p.cuts = list(cuts)
        p.x, p.kind, p.round = x, kind, rnd
        parts.append(p)
        return p
    for name, (lo, hi) in STRIPS.items():
        runs = [[(a, c, w == name) for a, c, w, g in own[e]] for e in (lo, hi)]
        o = strip_outline(Ls, runs[0], runs[1], s, t)
        cuts = [('circle', x, t + s / 2, d / 2) for st, x, d in holes if st == name]
        p = part(f'{name} strip', o, cuts, x0, 'strip', f'the {name} strip')
        p.strip = name
    # the cookies, top down: seat, round bore, socket over the strips
    outer = P.ngon(4, COOKIE / 2)
    r_rod = rod_radius(s + 2 * t)
    pins = [P.polar(45, r_rod), P.polar(225, r_rod)]
    pin_cut = [('circle', x, y, P.ROD_HOLE / 2) for x, y in pins]
    round_d = 2 * math.sqrt(M.BORE_AREA / math.pi)
    for k in range(P.N_SEAT):
        part('seat', outer, [('circle', 0.0, 0.0, P.SEAT_D / 2)] + pin_cut, k * t, 'seat')
    for k in range(n_round):
        part('round bore', outer, [('circle', 0.0, 0.0, round_d / 2)] + pin_cut,
             (P.N_SEAT + k) * t, 'round')
    sock = P.ngon(4, (s + 2 * t) / 2)
    for k in range(N_SOCK):
        part('strip socket', outer, [('poly', sock)] + pin_cut, x0 + k * t, 'sock')
    cap_o, cap_pins, stem_pins, csock = P.cap(part, outer, 4, 0.0)
    info = dict(b=b, L=L, pos=pos, widths=widths, n_round=n_round, s=s, t=t, Ls=Ls, x0=x0,
                holes=holes, own=own, outer=outer, pins=pins, round_d=round_d, sock=sock,
                cap_o=cap_o)
    return parts, info


def rod_radius(tube):
    """The cookies' rods on their diagonal, midway between the nearest they
    can come to the socket's corner and to the outline's, MIN_WEB of ply
    each side; the washers then clear both (checked)."""
    inner = tube / 2 * math.sqrt(2) + P.ROD_HOLE / 2 + P.MIN_WEB
    outer = (COOKIE / 2 - P.ROD_HOLE / 2 - P.MIN_WEB) * math.sqrt(2)
    return (inner + outer) / 2


def checks(parts, info, sheets):
    out = []

    def note(ok, what, detail):
        out.append((ok, what, detail))
    b, L, pos, widths = info['b'], info['L'], info['pos'], info['widths']
    s, t, Ls = info['s'], info['t'], info['Ls']
    rep = M.report(b, L, pos, widths)
    worst = max(abs(c) for nt, f, c in rep)
    note(worst <= 5.0, 'every note is in tune, in the model',
         f'within {worst:.1f} cents, low G among them: the strips are any length')
    note(abs(s * s - M.BORE_AREA) < 1e-6 and abs(math.pi * info['round_d'] ** 2 / 4
                                                - M.BORE_AREA) < 1e-6,
         'the bore is a 4mm round bore\'s area, square and round',
         f'{s * s:.2f}mm^2: the strips\' square {s:.2f}mm, the cookies\' circle '
         f'{info["round_d"]:.2f}mm')
    body = P.N_SEAT * t + L
    start = info['x0'] + N_SOCK * t                # the cookies' bottom, down the body
    up = (body - start) / body
    ha = P.N_SEAT * t + pos['hA']
    if SWITCH is None:
        ok = THUMB_ROOM <= ha - start < THUMB_ROOM + t
        asked = f'just above high A: its centre {ha - start:.1f}mm under them'
    else:
        ok = abs(up - SWITCH) <= t / body / 2 + 1e-9 and ha - start >= THUMB_ROOM
        asked = f'asked {SWITCH:g}; high A\'s centre {ha - start:.1f}mm under them'
    note(ok, 'the cookies start where asked',
         f'{up:.3f} of the way up the body, {body - start:.0f}mm of {body:.0f}mm, {asked}; '
         f'the strips run {N_SOCK * t:g}mm on into the socket')
    note(abs(info['n_round'] * t + Ls - L) < 1e-9, 'the bore is the cookies and the strips',
         f'{info["n_round"]} round plates, {info["n_round"] * t:g}mm, and {Ls:.1f}mm of strips: '
         f'{L:.1f}mm')
    hs = info['holes']
    note(all(M.D_MIN - 1e-9 <= d <= M.D_MAX + 1e-9 for st, x, d in hs),
         'every hole cuts round, and fits the bore\'s face',
         f'{min(d for st, x, d in hs):.2f} to {max(d for st, x, d in hs):.2f}mm across, '
         f'the face {s:.2f}mm')
    socket = N_SOCK * t
    note(all(x - d / 2 - P.MIN_WEB >= socket for st, x, d in hs),
         'every hole is below the socket',
         f'the highest {min(x - d / 2 for st, x, d in hs) - socket:.1f}mm under it')
    # ply round each hole: its strip owns both corners there, MIN_WEB past it
    ok = True
    for st, x, d in hs:
        lo, hi = STRIPS[st]
        for c in (lo, hi):
            for a, e, w, g in info['own'][c]:
                if a < x + d / 2 + P.MIN_WEB and e > x - d / 2 - P.MIN_WEB and w != st:
                    ok = False
    across = (2 * t + s - M.D_MAX) / 2
    note(ok, 'every hole keeps ply all round it',
         f'its strip fingered both sides, {P.MIN_WEB:g}mm past it along; '
         f'{across:.2f}mm across at the widest hole')
    # every corner owned by one strip all the way, and no finger too short
    runs = [(e - a, g) for c in CORNERS for a, e, w, g in info['own'][c]]
    cover = all(abs(info['own'][c][0][0]) < 1e-9 and abs(info['own'][c][-1][1] - Ls) < 1e-9
                and all(abs(p[1] - q[0]) < 1e-9 for p, q in zip(info['own'][c], info['own'][c][1:]))
                for c in CORNERS)
    least = min(r for r, g in runs if not g) if any(not g for r, g in runs) else math.inf
    note(cover and least >= MIN_FINGER - 1e-9, 'every corner is one strip\'s, all the way',
         f'{JOINT} joints; ' + (f'{sum(len(info["own"][c]) for c in CORNERS)} runs, the shortest '
                                f'unguarded {least:.1f}mm, against {MIN_FINGER:g}'
                                if JOINT == 'finger' else 'front and back own every corner'))
    strips = [p for p in parts if p.kind == 'strip']
    widest = max(p.h for p in strips)
    note(len(strips) == 4, 'the four strips close the square',
         f'{s:.2f}mm bore, {s + 2 * t:.2f}mm outside; {Ls:.1f}mm long')
    # webs: the cookies' rods
    web = math.inf
    for p in parts:
        if p.kind not in ('seat', 'round', 'sock'):
            continue
        edges = [('poly', p.outline)] + p.cuts
        for c in p.cuts:
            if c[0] != 'circle' or abs(c[3] - P.ROD_HOLE / 2) > 1e-9:
                continue
            for e in edges:
                if e is c:
                    continue
                d = (P.poly_dist((c[1], c[2]), e[1]) if e[0] == 'poly'
                     else abs(math.hypot(c[1] - e[1], c[2] - e[2]) - e[3]))
                web = min(web, d - c[3])
    note(web >= P.MIN_WEB, 'every cookie rod keeps ply between it and every cut',
         f'{web:.2f}mm at the least, against {P.MIN_WEB:g}')
    # the washers: on the top plate inside its face, on the bottom one clear
    # of the strips coming through it
    r = math.hypot(*info['pins'][0])
    face = COOKIE / 2 - r / math.sqrt(2)
    tube = r - (s + 2 * t) / 2 * math.sqrt(2)
    note(min(face, tube) >= P.WASHER_D / 2, 'the nuts sit on the cookies\' ends, clear of the strips',
         f'{P.WASHER_D:g}mm washers: {face:.2f}mm to the outline, {tube:.2f}mm to the strips\' '
         f'corner, against {P.WASHER_D / 2:g}')
    note(P.SEAT_D >= P.STAPLE_MAX and info['round_d'] <= P.STAPLE_MIN + 1e-9,
         'the seat holds a staple, the round bore under it',
         f'seat {P.SEAT_D:g}mm over the largest staple, {P.STAPLE_MAX:g}; the round bore '
         f'{info["round_d"]:.2f}mm stops staples over that only: a {P.STAPLE_MIN:g}mm staple '
         f'is set by its hemp')
    under = r + P.NUT_AF / 2 - P.CHAMBER_D / 2
    note(P.CHAMBER_D < COOKIE and under >= 1.0, 'the cap slides over the cookies and stops on its nuts',
         f'socket {P.CAP_CLEAR:g}mm clear all round; each top nut {under:.1f}mm under the '
         f'chamber\'s bottom plate, the chamber {P.CHAMBER_D:g}mm')
    note(all(sh['w'] <= P.BED[0] and sh['h'] <= P.BED[1] for sh in sheets), 'every sheet fits the bed',
         f'{len(sheets)} sheet(s), largest {max(sh["w"] for sh in sheets):.0f} x '
         f'{max(sh["h"] for sh in sheets):.0f}mm; the strips {Ls + KERF:.1f} x {widest + KERF:.1f}mm')
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
    return f'strips-{JOINT}-joint'


def svg_sheet(k, nsh, sh):
    txt = P.svg_sheet(k, nsh, sh, 4)
    what = (f'Practice chanter, square bore of four {JOINT}-jointed strips, cookies from '
            + ('just above high A' if SWITCH is None else f'{SWITCH:g} of the way up'))
    return (txt.replace('Practice chanter, 4-sided bore', what)
            .replace('cuts the bores, seats and pin holes first; then black',
                     'cuts the bores, seats, finger and pin holes first; then black'))


def placed(p, info):
    """Where a part sits in the chanter: the plate frame's x across, y down
    (the front is -y), z through, to the chanter's up, front and length; a
    strip's x along it, u across, w through, to the same."""
    s, t = info['s'], info['t']
    if p.kind != 'strip':
        return [[0, 0, 1, p.x], [0, 1, 0, 0], [-1, 0, 0, 0], [0, 0, 0, 1]]
    X = info['x0']
    c = t + s / 2
    return {
        # front: w out to in is y -s/2-t to -s/2; u = 0 on the left, z = +c
        'front': [[1, 0, 0, X], [0, 0, 1, -s / 2 - t], [0, -1, 0, c], [0, 0, 0, 1]],
        # back: w out to in is y s/2+t to s/2; u = 0 on the right, z = -c
        'back': [[1, 0, 0, X], [0, 0, -1, s / 2 + t], [0, 1, 0, -c], [0, 0, 0, 1]],
        # left: w in to out is z s/2 to s/2+t; u = 0 at the front
        'left': [[1, 0, 0, X], [0, 1, 0, -c], [0, 0, 1, s / 2], [0, 0, 0, 1]],
        # right: w in to out is z -s/2 to -s/2-t; u = 0 at the back
        'right': [[1, 0, 0, X], [0, -1, 0, c], [0, 0, -1, -s / 2], [0, 0, 0, 1]],
    }[p.strip]


def design3d():
    """The chanter for practice_chanters.py's 3D page, finger-jointed."""
    global JOINT
    j, JOINT = JOINT, 'finger'
    parts, info = build()
    JOINT = j
    mid = (min(p.x for p in parts) + max(p.x + (info['Ls'] if p.kind == 'strip' else 0)
                                        for p in parts)) / 2
    out = []
    import turn3d as t3
    for p in parts:
        holes = [c[1] if c[0] == 'poly' else circle(c[1], c[2], c[3], 20) for c in p.cuts]
        cap = p.kind in ('socket', 'chamber', 'top', 'stem')
        if p.kind == 'strip':
            ex = {'front': (0, -1, 0), 'back': (0, 1, 0), 'left': (0, 0, 1),
                  'right': (0, 0, -1)}[p.strip]
            ex = (0, ex[1] * 0.12, ex[2] * 0.12)
        else:
            ex = ((p.x - mid) / 60 - (1.0 if cap else 0.0), 0, 0)
        wood = 'dark' if p.kind == 'strip' else 'pale' if cap else 'ply'
        out.append(t3.placed({'o': p.outline, 'h': holes, 'e': []}, placed(p, info), ex, wood))
    return {'parts': out, 'ex': 60, 'view': [-0.12, 0.32, 3.0]}


def main():
    global KERF, SWITCH, JOINT
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
        elif x.startswith('--switch='):
            SWITCH = float(x.split('=', 1)[1])
        elif x.startswith('--plate='):
            M.PLATE = float(x.split('=', 1)[1])
        elif x.startswith('--reed='):
            M.REED = float(x.split('=', 1)[1])
        elif x.startswith('--low-a='):
            M.LOW_A = float(x.split('=', 1)[1])
        elif x.startswith('--kerf='):
            KERF = P.KERF = SF.set_kerf(x)
        else:
            raise SystemExit(f'error: {x} is not a flag this generator reads. It takes --joint=, '
                             f'--switch=, --out=, --plate=, --reed=, --low-a=, --kerf= and '
                             f'--no-write.')
    if write and not out:
        raise SystemExit('error: --out=DIR, or --no-write to run the checks only.')
    parts, info = build()
    sheets = P.pack(parts)
    res = checks(parts, info, sheets)
    t = info['t']
    block = P.N_SEAT + info['n_round'] + N_SOCK
    print(f'practice chanter, square bore of four {JOINT}-jointed strips, low A {M.LOW_A:g}Hz: '
          f'strips {info["Ls"]:.1f}mm long, {info["s"]:.2f}mm bore, {info["s"] + 2 * t:.2f}mm '
          f'outside; {block} cookies ({block * t:g}mm, {COOKIE:g}mm square), body '
          f'{P.N_SEAT * t + info["L"]:.1f}mm; {P.N_SOCKET + P.N_CHAMBER + P.N_TOP + P.N_STEM} '
          f'cap plates; two M{P.ROD:g} rods cut {block * t + 2 * (0.5 + P.NUT_T) + P.ROD_SPARE:.0f}mm '
          f'long through the cookies')
    for st, x, d in info['holes']:
        h = next(h for h in M.HOLES if abs(info['pos'][h] - info['n_round'] * t - x) < 1e-9)
        print(f'  {h:3} {st:5} strip, {x:6.1f}mm from its top ({info["pos"][h]:5.1f}mm down '
              f'the bore)  {d:.2f}mm round')
    print(f'  cookies, top down: {P.N_SEAT} seat, {info["n_round"]} round, {N_SOCK} socket '
          f'over the strips')
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
