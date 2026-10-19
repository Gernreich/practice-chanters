#!/usr/bin/env python3
"""Cut files for a Great Highland bagpipe practice chanter whose bore is a
regular polygon, three to eight sides, built as a stack of cross-section
plates clamped on two threaded rods, with a stacked cap and mouthpiece.

    python3 practice_chanters.py --sides=5 --no-write      # the checks
    python3 practice_chanters.py --sides=5 --out=DIR       # the sheets
    python3 practice_chanters.py --sides=5 --plate=2.9 ... # for ply as measured
    python3 practice_chanters.py --kerf=0.18 ...           # for that kerf
    python3 practice_chanters.py --page-to=turn.html       # the 3D view, every shape

The acoustics are chanter_model.py's: where each hole goes and how wide it is
cut. This file turns them into plates, from the top down:

  * the CAP: a mouthpiece stem, two top plates, a chamber round the reed's
    blades, and a socket that slides over the body's top plates;
  * the SEAT: N_SEAT plates with a round hole for the reed's staple, which
    stops on the bore below, since the bore is narrower than any staple;
  * the BORE: one plate per PLATE of bore, the polygon through each, down to
    the open sole. A finger hole is a slot cut in its plate from the bore out
    through the face, which leaves the plate C-shaped; one too small to cut
    is drilled after assembly instead, at a V cut in its plate's edge.

Every body plate carries the same two rod holes, placed so that no turn or
flip of the polygon brings them back into line: a plate goes on only one
way, so no slot can face the wrong way. The rods are M2 threaded rod with a
nut and washer at each end, which clamp the stack: a press-fit pin cannot be
driven through 130 plates, and nothing is glued. Shellac seals the seams.
The cap comes off to set the reed, so it is not on the rods: it slides over
the body's top plates and rests on the two top nuts. The cap and the
mouthpiece, 19 and 14 plates, are short enough to press together on their
own pins.

A PLATE IS NOT ALWAYS 3MM. A stack multiplies the stock's error by every
plate in it: 2.9mm ply makes the bore 13mm short and every note 50 to 57
cents sharp, in the model.
Measure the ply and pass --plate=; the model then places every hole on that
grid, and the plate count follows.
"""
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
for sib in ('square-flute', 'swept-curve', 'four-hole-ocarinas', 'lasermade-tools'):
    sys.path.insert(0, os.path.join(HERE, '..', sib))
import chanter_model as M                                      # noqa: E402
import square_flute as SF                                      # noqa: E402
from four_hole_ocarinas import offset                          # noqa: E402
from square_flute import Part                            # noqa: E402
from ribbon_bore import path                                   # noqa: E402

KERF = SF.KERF
MIN_WEB = SF.MIN_WEB
BED, MARGIN = SF.BED, SF.MARGIN
ORANGE, BLACK = SF.ORANGE, SF.BLACK

# --- the reed and its seat (mm) ---------------------------------------------
# A practice-chanter reed's staple is 4 to 4.8mm across, by the reed makers'
# figures; pipers wrap it in hemp to fit the seat, so the seat is cut over the
# largest, and the bore's narrowness stops the smallest.
STAPLE_MIN, STAPLE_MAX = 4.0, 4.8
SEAT_D = 5.5
N_SEAT = 6                         # 18mm of seat: a staple is 22 to 24mm long
# --- the rods and pins --------------------------------------------------------
ROD = 2.0                          # M2 threaded rod, through the body
ROD_HOLE = 2.2                     # a sliding fit: the nuts clamp, not the holes
WASHER_D = 5.0                     # an M2 washer
NUT_AF, NUT_T = 4.0, 1.6           # an M2 nut: across its flats, and thick
ROD_SPARE = 3.0                    # rod left past the sole's nut; the top's is cut
                                   # flush, or the cap would land on the rods
PIN = 2.0                          # brass rod, the cap's and mouthpiece's pins
PIN_INTERF = 0.1                   # the hole this much under the rod, a press-fit
# --- the cap -----------------------------------------------------------------
CAP_CLEAR = 0.3                    # each side of the body, for hemp or tape
CAP_WEB = 2.0                      # ply each side of the cap's pins
N_SOCKET = 5                       # plates of socket over the body's top
CHAMBER_D = 14.0                   # round the reed's blades
N_CHAMBER = 12                     # 36mm: the reed stands about 30mm proud
MOUTH_D = 6.0                      # the blowing hole
N_TOP = 2
STEM_D = 17.0                      # the mouthpiece stem
N_STEM = 12
STEM_PIN_R = 5.75                  # the stem's pins, from its centre
# --- marks -------------------------------------------------------------------
V_W, V_D = 1.2, 0.6                # a drill mark: a V this wide and deep in the edge
STEP = 0.5                         # degrees, the boundary's sampling


def polar(t, r):
    """The point r from the centre at t degrees round from the front: the
    front is -v, the thumb's side of an odd polygon -u."""
    a = math.radians(t)
    return (r * math.sin(a), -r * math.cos(a))


def ngon(n, inr, rot=0.0):
    R = inr / math.cos(math.pi / n)
    return [polar(rot + (k + 0.5) * 360 / n, R) for k in range(n)]


def ray(poly, t):
    """How far the ray from the centre at t meets the convex polygon."""
    dx, dy = polar(t, 1.0)
    best = math.inf
    for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]):
        ex, ey = x1 - x0, y1 - y0
        den = dx * ey - dy * ex
        if abs(den) < 1e-12:
            continue
        s = (x0 * ey - y0 * ex) / den
        u = (x0 * dy - y0 * dx) / den
        if s > 1e-9 and -1e-9 <= u <= 1 + 1e-9:
            best = min(best, s)
    return best


def slot_r(t, theta, w, s0=0.0):
    """How far out the ray at t leaves a slot w wide running out along
    theta from s0 off the centre, or 0 if it meets only the slot's back,
    which lies inside the bore."""
    d = (t - theta + 180) % 360 - 180
    if abs(d) >= 90:
        return 0.0
    s = abs(math.sin(math.radians(d)))
    if s < 1e-12:
        return math.inf
    r = (w / 2) / s
    return r if r * math.cos(math.radians(d)) >= s0 - 1e-9 else 0.0


def side_meets(bore, theta, w):
    """Where each side of a slot w wide along theta meets the bore's edges:
    (side, distance out along the slot, the point)."""
    ux, uy = polar(theta, 1.0)
    out = []
    for sg in (1, -1):
        ox, oy = -uy * sg * w / 2, ux * sg * w / 2
        for (x0, y0), (x1, y1) in zip(bore, bore[1:] + bore[:1]):
            ex, ey = x1 - x0, y1 - y0
            den = ux * ey - uy * ex
            if abs(den) < 1e-12:
                continue
            k = ((x0 - ox) * ey - (y0 - oy) * ex) / den
            v = ((x0 - ox) * uy - (y0 - oy) * ux) / den
            if k > 0 and 0 <= v <= 1:
                out.append((sg, k, (ox + k * ux, oy + k * uy)))
    return out


def tidy(pts):
    """Drop repeated points, and points on a straight line between their
    neighbours."""
    pts = [q for i, q in enumerate(pts) if math.dist(q, pts[i - 1]) > 1e-6]
    out = []
    n = len(pts)
    for i in range(n):
        a, b, c = pts[i - 1], pts[i], pts[(i + 1) % n]
        if abs((b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0])) > 1e-7:
            out.append(b)
    return out


def c_plate(outer, bore, theta, w):
    """A plate with a slot w wide from the bore out through the face at
    theta: one outline, round the outside and back round the bore. The cut
    region is the bore joined to the slot, which is star-shaped about the
    centre, so its edge is the larger of the two at every angle. The slot
    starts where its sides first meet the bore, going in, so that below
    them it leaves no ledge where the bore narrows; a slot wider than the
    bore runs in to the centre and ends square there."""
    meets = side_meets(bore, theta, w)
    s0 = max(min((k for sg, k, q in meets if sg == side), default=0.0) for side in (1, -1))
    inner = lambda t: max(ray(bore, t), slot_r(t, theta, w, s0))
    cut = lambda t: inner(t) >= ray(outer, t)
    # the opening's edges: where the slot's side meets the outside
    ts = [theta + 180 + STEP * i for i in range(int(360 / STEP))]
    edges = []
    for a, b in zip(ts, ts[1:] + [ts[0] + 360]):
        if cut(a) != cut(b):
            lo, hi = a, b
            for _ in range(50):
                mid = (lo + hi) / 2
                if cut(mid) == cut(lo):
                    lo = mid
                else:
                    hi = mid
            edges.append(lo if not cut(lo) else hi)
    assert len(edges) == 2, edges
    # going round from the slot's middle, t1 closes the opening and t0
    # opens it again: the material runs from t1 round to t0
    t1, t0 = sorted(edges, key=lambda t: (t - theta) % 360)
    span = (t0 - t1) % 360
    angles = [t1 + span * i / int(span / STEP) for i in range(int(span / STEP) + 1)]
    for v in outer + bore:
        a = math.degrees(math.atan2(v[0], -v[1]))
        if 0 < (a - t1) % 360 < span:
            angles.append(t1 + (a - t1) % 360)
    # and where each side of the slot meets the bore, exactly
    for sg, k, q in meets:
        a = math.degrees(math.atan2(q[0], -q[1]))
        if 0 < (a - t1) % 360 < span:
            angles.append(t1 + (a - t1) % 360)
    # a slot wider than the bore at its centre ends in a straight ledge
    # there, from each back corner straight in to the bore
    for d in (90, -90):
        a = t1 + (theta + d - t1) % 360
        if 0 < a - t1 < span and slot_r(a - math.copysign(1e-7, d), theta, w, s0) > ray(bore, a):
            angles += [a - math.copysign(1e-7, d), a]
    angles = sorted(set(angles))
    out_pts = [polar(t, ray(outer, t)) for t in angles]
    in_pts = [polar(t, inner(t)) for t in reversed(angles)]
    # the slot's sides, straight from the outside to where they meet the bore
    return tidy(out_pts + in_pts)


def v_mark(outer, theta):
    """The outside with a V cut into it at the middle of the face at theta."""
    k = min(range(len(outer)), key=lambda i: abs(
        ((math.degrees(math.atan2((outer[i][0] + outer[i - 1][0]) / 2,
                                  -(outer[i][1] + outer[i - 1][1]) / 2)) - theta + 180) % 360) - 180))
    a, b = outer[k - 1], outer[k]
    mx, my = (a[0] + b[0]) / 2, (a[1] + b[1]) / 2
    L = math.hypot(b[0] - a[0], b[1] - a[1])
    ux, uy = (b[0] - a[0]) / L, (b[1] - a[1]) / L
    nx, ny = polar(theta, 1.0)
    v = [(mx - ux * V_W / 2, my - uy * V_W / 2), (mx - nx * V_D, my - ny * V_D),
         (mx + ux * V_W / 2, my + uy * V_W / 2)]
    return outer[:k] + v + outer[k:]


def circle(cx, cy, r, n=48):
    return [(cx + r * math.cos(2 * math.pi * i / n), cy + r * math.sin(2 * math.pi * i / n))
            for i in range(n)]


def seg_dist(p, a, b):
    ax, ay = b[0] - a[0], b[1] - a[1]
    L2 = ax * ax + ay * ay
    t = 0.0 if L2 == 0 else max(0.0, min(1.0, ((p[0] - a[0]) * ax + (p[1] - a[1]) * ay) / L2))
    return math.hypot(p[0] - a[0] - t * ax, p[1] - a[1] - t * ay)


def poly_dist(p, poly):
    return min(seg_dist(p, a, b) for a, b in zip(poly, poly[1:] + poly[:1]))


def inside(p, poly):
    c = False
    for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]):
        if (y0 > p[1]) != (y1 > p[1]) and p[0] < x0 + (p[1] - y0) * (x1 - x0) / (y1 - y0):
            c = not c
    return c


def faces(n):
    """The front face's angle and the thumb's: straight back on an even
    polygon, on an odd one the back face on the left hand's side."""
    thumb = 180.0 if n % 2 == 0 else 180.0 + 180.0 / n
    return 0.0, thumb


def symmetries(n):
    """The polygon's turns and flips, as maps of a point."""
    out = []
    for k in range(n):
        a = 2 * math.pi * k / n
        c, s = math.cos(a), math.sin(a)
        out.append(lambda p, c=c, s=s: (c * p[0] - s * p[1], s * p[0] + c * p[1]))
        out.append(lambda p, c=c, s=s: (c * -p[0] - s * p[1], s * -p[0] + c * p[1]))
    return out


def pins_for(n, outer, bore):
    """Two rod centres, as far from every cut as they can be, roughly across
    the plate, and placed so that no turn or flip but none at all brings
    both back into place. Every cut means the widest slot any hole could
    have, out through the front or the thumb's face; and a washer must sit
    on the end plates without overhanging."""
    front, thumb = faces(n)

    def clear(p):
        if not inside(p, outer) or inside(p, bore) or poly_dist(p, outer) < WASHER_D / 2:
            return -1.0
        c = min(poly_dist(p, outer), poly_dist(p, bore), math.hypot(*p) - SEAT_D / 2)
        t = math.degrees(math.atan2(p[0], -p[1]))
        for th in (front, thumb):
            d = (t - th + 180) % 360 - 180
            # off the slot's line if in front of the centre, else off its back
            c = min(c, (abs(math.sin(math.radians(d))) if abs(d) < 90 else 1) * math.hypot(*p)
                    - M.W_MAX / 2)
        return c - ROD_HOLE / 2

    best = {}
    R = max(math.hypot(*v) for v in outer)
    for i in range(0, 360, 2):
        cand = [(clear(polar(i, R * j / 60)), R * j / 60) for j in range(1, 60)]
        best[i] = max(cand)
    syms = symmetries(n)
    pairs = []
    for a in best:
        for b in best:
            sep = (b - a) % 360
            if not 130 <= sep <= 170:
                continue
            p, q = polar(a, best[a][1]), polar(b, best[b][1])
            fixed = 0
            for f in syms:
                fp, fq = f(p), f(q)
                if ({(round(x, 3), round(y, 3)) for x, y in (fp, fq)}
                        == {(round(x, 3), round(y, 3)) for x, y in (p, q)}):
                    fixed += 1
            if fixed == 1:
                pairs.append((min(best[a][0], best[b][0]), p, q))
    c, p, q = max(pairs)
    return [p, q]


def build(n):
    """The parts, and what the checks need, for an n-sided chanter."""
    b, L, pos, widths = M.design(n)
    bore = ngon(n, M.inradius(M.BORE_AREA, n))
    outer = ngon(n, M.inradius(M.OUTER_AREA, n))
    front, thumb = faces(n)
    pins = pins_for(n, outer, bore)
    pin_cut = [('circle', x, y, ROD_HOLE / 2) for x, y in pins]
    n_bore = int(round(L / M.PLATE))
    at = {int(pos[h] // M.PLATE): h for h in M.HOLES}
    num = iter(range(1, 9999))
    parts = []

    def part(name, outline, cuts, x, kind, note='', rnd=None):
        p = Part(name, next(num), outline, note)
        p.cuts = list(cuts)
        p.x, p.kind, p.round = x, kind, rnd
        parts.append(p)
        return p
    for k in range(N_SEAT):
        part('seat', outer, [('circle', 0.0, 0.0, SEAT_D / 2)] + pin_cut, k * M.PLATE, 'seat')
    for k in range(n_bore):
        x = (N_SEAT + k) * M.PLATE
        h = at.get(k)
        if h is None:
            part('bore', outer, [('poly', bore)] + pin_cut, x, 'bore')
            continue
        th = thumb if h == 'hA' else front
        w = widths[h]
        if w >= M.W_SLOT:
            p = part(f'{h} hole', c_plate(outer, bore, th, w), pin_cut, x, 'slot',
                     f'the {h} hole: a slot {w:.2f}mm wide')
        else:
            d = M.drill(w)
            p = part(f'{h} hole', v_mark(outer, th), [('poly', bore)] + pin_cut, x, 'drill',
                     f'the {h} hole: drill {d:.2f}mm through at the V, after assembly')
            p.drill = d
        p.finger, p.width, p.theta = h, w, th
    # the cap: socket, chamber, top, stem; its own pins in its wall
    sock = offset(outer, CAP_CLEAR)
    # the cap is the body's polygon grown round the socket, a pin in two of
    # its corners: CAP_WEB of ply each side of a pin, where the most is
    ring = offset(sock, CAP_WEB + PIN / 2)
    cap_pins = [ring[0], ring[n // 2]]
    cap_o = offset(sock, 2 * CAP_WEB + PIN)
    cap_pin_cut = [('circle', x, y, (PIN - PIN_INTERF) / 2) for x, y in cap_pins]
    stem_pins = [polar(front + 90, STEM_PIN_R), polar(front - 90, STEM_PIN_R)]
    stem_pin_cut = [('circle', x, y, (PIN - PIN_INTERF) / 2) for x, y in stem_pins]
    for k in range(N_SOCKET):
        part('cap socket', cap_o, [('poly', sock)] + cap_pin_cut,
             (N_SOCKET - 1 - k) * M.PLATE, 'socket')
    for k in range(N_CHAMBER):
        part('cap chamber', cap_o, [('circle', 0.0, 0.0, CHAMBER_D / 2)] + cap_pin_cut,
             -(k + 1) * M.PLATE, 'chamber')
    for k in range(N_TOP):
        part('cap top', cap_o, [('circle', 0.0, 0.0, MOUTH_D / 2)] + cap_pin_cut + stem_pin_cut,
             -(N_CHAMBER + k + 1) * M.PLATE, 'top')
    stem_o = circle(0, 0, STEM_D / 2, 72)
    for k in range(N_STEM):
        part('mouthpiece', stem_o, [('circle', 0.0, 0.0, MOUTH_D / 2)] + stem_pin_cut,
             -(N_CHAMBER + N_TOP + k + 1) * M.PLATE, 'stem', rnd=STEM_D / 2)
    info = dict(b=b, L=L, pos=pos, widths=widths, bore=bore, outer=outer, pins=pins,
                n_bore=n_bore, cap_o=cap_o, cap_pins=cap_pins, stem_pins=stem_pins,
                sock=sock, front=front, thumb=thumb)
    return parts, info


def turned(p):
    """The part turned half round, for the packer: a triangle beside one
    turned over takes half the room of two the same way up."""
    q = Part(p.name, p.num, [(-x, -y) for x, y in p.outline], p.note)
    q.cuts = [('poly', [(-x, -y) for x, y in c[1]]) if c[0] == 'poly'
              else ('circle', -c[1], -c[2], c[3]) for c in p.cuts]
    for k, v in vars(p).items():
        if k not in vars(q):
            setattr(q, k, v)
    return q


def apart(a, ax, ay, b, bx, by):
    """The least distance between two placed outlines, neither inside the
    other: a vertex of one against an edge of the other."""
    pa = [(x - a.x0 + ax, y - a.y0 + ay) for x, y in a.outline]
    pb = [(x - b.x0 + bx, y - b.y0 + by) for x, y in b.outline]
    if any(inside(v, pb) for v in pa) or any(inside(v, pa) for v in pb):
        return -1.0
    return min(min(poly_dist(v, pb) for v in pa), min(poly_dist(v, pa) for v in pb))


def pack(parts):
    """Rows across the bed, tallest parts first. Each part goes in the row
    both ways up and slides left until it comes GAP from the one before;
    whichever way ends further left is kept. A row full, the next starts
    below; a sheet full, the next sheet."""
    clear = SF.GAP + KERF
    order = sorted(parts, key=lambda p: (-p.h, p.kind, p.num))
    sheets, sheet, row, y, row_h = [], [], [], MARGIN, 0.0
    for p in order:
        best = None
        for q in (p, turned(p)):
            if not row:
                x = MARGIN
            else:
                last, lx, ly = row[-1]
                hi = lx + last.w + clear          # boxes apart: always clear
                lo = lx
                for _ in range(30):
                    mid = (lo + hi) / 2
                    if all(apart(o, ox, oy, q, mid, y) >= clear for o, ox, oy in row[-3:]):
                        hi = mid
                    else:
                        lo = mid
                x = hi
            if best is None or x + q.w < best[1] + best[0].w - 1e-9:
                best = (q, x)
        q, x = best
        if x + q.w + KERF > BED[0] - MARGIN or (row and q.h > row_h + 1e-9):
            y += row_h + clear
            row, row_h, x = [], 0.0, MARGIN
            q = p
            if y + q.h + KERF > BED[1] - MARGIN:
                sheets.append(sheet)
                sheet, y = [], MARGIN
        row.append((q, x, y))
        sheet.append((q, x, y))
        row_h = max(row_h, q.h)
    sheets.append(sheet)
    out = []
    for items in sheets:
        out.append({'parts': items,
                    'w': max(x + p.w + KERF for p, x, y in items) + MARGIN,
                    'h': max(y + p.h + KERF for p, x, y in items) + MARGIN})
    return out


def cut_polys(p):
    """Every cut a plate has, as polygons: its outline and its inner cuts."""
    out = [p.outline]
    for c in p.cuts:
        out.append(c[1] if c[0] == 'poly' else circle(c[1], c[2], c[3]))
    return out


def poly_area(pts):
    return abs(sum(x0 * y1 - x1 * y0 for (x0, y0), (x1, y1)
                   in zip(pts, pts[1:] + pts[:1]))) / 2


def stack_order(parts):
    """The body from the top down, as the builder threads it: runs of plain
    plates between the hole plates, which are the only ones that differ."""
    body = sorted((p for p in parts if p.kind in ('seat', 'bore', 'slot', 'drill')),
                  key=lambda p: p.x)
    out, run, kind = [], 0, None
    for p in body:
        k = p.kind if p.kind in ('seat', 'bore') else None
        if k and k == kind:
            run += 1
            continue
        if kind:
            out.append(f'{run} {kind}')
        if k:
            kind, run = k, 1
        else:
            kind, run = None, 0
            out.append(p.finger)
    if kind:
        out.append(f'{run} {kind}')
    return ', '.join(out)


def checks(n, parts, info, sheets):
    out = []

    def note(ok, what, detail):
        out.append((ok, what, detail))
    b, L, pos, widths = info['b'], info['L'], info['pos'], info['widths']
    rep = M.report(b, L, pos, widths)
    worst = max(abs(c) for nt, f, c in rep if nt != 'low G')
    low_g = next(c for nt, f, c in rep if nt == 'low G')
    # low G is the bore rounded to whole plates: one plate more or fewer
    # would put it further out
    near = [M.cents(M.sounding(b, L + d, pos, widths, 'low G', span=1.8), M.target('low G'))
            for d in (-M.PLATE, M.PLATE)]
    note(worst <= 5.0 and all(abs(low_g) < abs(c) for c in near), 'every note is in tune, in the model',
         f'within {worst:.1f} cents; low G {low_g:+.1f}, the bore the nearest whole plates '
         f'(a plate less {near[0]:+.1f}, more {near[1]:+.1f})')
    area = poly_area(info['bore'])
    note(abs(area - M.BORE_AREA) < 1e-6, 'the bore is a 4mm round bore\'s area',
         f'{area:.2f}mm^2, a {n}-gon {2 * M.inradius(M.BORE_AREA, n):.2f}mm across its '
         f'inscribed circle')
    bores = [p for p in parts if p.kind in ('bore', 'slot', 'drill')]
    note(len(bores) * M.PLATE == L, 'the bore is whole plates, down to the sole',
         f'{len(bores)} plates of {M.PLATE:g}mm, {L:g}mm')
    holes = [p for p in parts if hasattr(p, 'finger')]
    on = all(abs(p.x - (N_SEAT * M.PLATE + pos[p.finger] - M.PLATE / 2)) < 1e-6
             for p in holes)
    note(len(holes) == len(M.HOLES) and on, 'every finger hole is in its own plate, at its centre',
         ', '.join(f'{p.finger} {pos[p.finger]:.1f}' for p in sorted(holes, key=lambda p: -p.x)))
    slots = [p for p in holes if p.kind == 'slot']
    drills = [p for p in holes if p.kind == 'drill']
    note(all(M.W_SLOT <= p.width <= M.W_MAX + 1e-9 for p in slots)
         and all(p.drill >= M.D_MIN - 1e-9 for p in drills),
         'every hole is cuttable or drillable',
         f'{len(slots)} slots {min(p.width for p in slots):.2f} to '
         f'{max(p.width for p in slots):.2f}mm wide; '
         + (', '.join(f'{p.finger} drilled {p.drill:.2f}mm' for p in drills) or 'nothing to drill'))
    # the C plates really are open through the face, as wide as the slot: the
    # opening's two corners are the outline's points on the outside nearest
    # the slot's line, one each side of it
    worst_w = 0.0
    for p in slots:
        ux, uy = math.cos(math.radians(p.theta)), math.sin(math.radians(p.theta))
        rim = [q for q in p.outline if poly_dist(q, info['outer']) < 1e-6
               and abs(((math.degrees(math.atan2(q[0], -q[1])) - p.theta + 180) % 360) - 180) < 90]
        a = min((ux * q[0] + uy * q[1] for q in rim if ux * q[0] + uy * q[1] > 0), default=math.inf)
        b = min((-(ux * q[0] + uy * q[1]) for q in rim if ux * q[0] + uy * q[1] < 0),
                default=math.inf)
        worst_w = max(worst_w, abs(a + b - p.width))
    note(worst_w < 1e-6, 'every slot runs the width it is tuned for', f'{len(slots)} slots')
    # webs: every rod and pin hole clear of every other cut on its plate
    web = math.inf
    holes = (ROD_HOLE / 2, (PIN - PIN_INTERF) / 2)
    for p in parts:
        edges = [('circle', 0.0, 0.0, p.round) if p.round else ('poly', p.outline)] + p.cuts
        for c in p.cuts:
            if c[0] != 'circle' or min(abs(c[3] - r) for r in holes) > 1e-9:
                continue
            for e in edges:
                if e is c:
                    continue
                d = (poly_dist((c[1], c[2]), e[1]) if e[0] == 'poly'
                     else abs(math.hypot(c[1] - e[1], c[2] - e[2]) - e[3]))
                web = min(web, d - c[3])
    note(web >= MIN_WEB, 'every rod and pin keeps ply between it and every cut',
         f'{web:.2f}mm at the least, against {MIN_WEB:g}')
    # the nuts: a washer on each end plate, inside its face, and the top ones
    # in under the cap's socket
    room = min(poly_dist(q, info['outer']) for q in info['pins'])
    note(room >= WASHER_D / 2, 'the nuts sit on the end plates, inside the body',
         f'{WASHER_D:g}mm washers, {2 * room:.1f}mm of face round each rod')
    fixed = sum(1 for f in symmetries(n)
                if {(round(x, 3), round(y, 3)) for x, y in map(f, info['pins'])}
                == {(round(x, 3), round(y, 3)) for x, y in info['pins']})
    note(fixed == 1, 'every body plate goes on only one way',
         f'of the {2 * n} turns and flips of a {n}-gon, only leaving it be keeps both rods in place')
    inscribed = 2 * M.inradius(M.BORE_AREA, n)
    note(inscribed < STAPLE_MIN and SEAT_D >= STAPLE_MAX, 'the seat holds a staple, the bore stops it',
         f'seat {SEAT_D:g}mm over the largest staple, {STAPLE_MAX:g}; bore {inscribed:.2f}mm '
         f'inside, under the smallest, {STAPLE_MIN:g}')
    r_in = M.inradius(M.OUTER_AREA, n)
    under = min(math.hypot(*q) for q in info['pins']) + NUT_AF / 2 - CHAMBER_D / 2
    note(CHAMBER_D < 2 * r_in and under >= 1.0, 'the cap slides over the body and stops on its nuts',
         f'socket {CAP_CLEAR:g}mm clear all round; each top nut {under:.1f}mm under the '
         f'chamber\'s bottom plate at the least, the chamber {CHAMBER_D:g}mm')
    longest = max(max(p.w, p.h) for p in parts)
    note(all(s['w'] <= BED[0] and s['h'] <= BED[1] for s in sheets), 'every sheet fits the bed',
         f'{len(sheets)} sheet(s), largest {max(s["w"] for s in sheets):.0f} x '
         f'{max(s["h"] for s in sheets):.0f}mm, the largest plate {longest + KERF:.1f}mm')
    least = math.inf
    for s in sheets:
        its = s['parts']
        for i, (a, ax, ay) in enumerate(its):
            for c, cx, cy in its[i + 1:]:
                if (cx > ax + a.w + SF.GAP + KERF or ax > cx + c.w + SF.GAP + KERF
                        or cy > ay + a.h + SF.GAP + KERF or ay > cy + c.h + SF.GAP + KERF):
                    continue
                least = min(least, apart(a, ax, ay, c, cx, cy))
    note(least >= SF.GAP + KERF - 1e-3, 'no two parts on a sheet come closer than the gap',
         f'{least - KERF:.2f}mm of ply between cuts at the least, against {SF.GAP:g}')
    placed = sum(len(s['parts']) for s in sheets)
    note(placed == len(parts), 'every part is on a sheet', f'{placed} of {len(parts)}')
    return out


def svg_sheet(k, nsh, s, n):
    orange, black = [], []
    for p, sx, sy in s['parts']:
        dx, dy = sx - p.x0 + KERF / 2, sy - p.y0 + KERF / 2
        mv = lambda pts: [(x + dx, y + dy) for x, y in pts]
        if p.round:
            black.append(f'<circle cx="{dx:.3f}" cy="{dy:.3f}" r="{p.round + KERF / 2:.3f}"/>')
        else:
            black.append(f'<path d="{path(mv(offset(p.outline, KERF / 2)))}"/>')
        for c in p.cuts:
            if c[0] == 'poly':
                orange.append(f'<path d="{path(mv(offset(c[1], -KERF / 2)))}"/>')
            else:
                _, cx, cy, r = c
                orange.append(f'<circle cx="{cx + dx:.3f}" cy="{cy + dy:.3f}" '
                              f'r="{r - KERF / 2:.3f}"/>')
    desc = (f'1 user unit = 1mm. Practice chanter, {n}-sided bore, sheet {k} of {nsh}. '
            f'{M.PLATE:g}mm ply, drawn for a {KERF:g}mm kerf. Orange {ORANGE} cuts the bores, '
            f'seats and pin holes first; then black {BLACK} frees the plates.')
    g = lambda col, items: (
        f'  <g fill="none" stroke="{col}" stroke-width="0.2">\n'
        + ''.join(f'    {i}\n' for i in items) + '  </g>\n') if items else ''
    return (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{s["w"]:.2f}mm" '
            f'height="{s["h"]:.2f}mm" viewBox="0 0 {s["w"]:.2f} {s["h"]:.2f}">\n'
            f'<title>Practice chanter, {n}-sided bore, sheet {k} of {nsh}</title>\n'
            f'<desc>{desc}</desc>\n' + g(ORANGE, orange) + g(BLACK, black) + '</svg>\n')


NAMES = {3: 'triangle', 4: 'square', 5: 'pentagon', 6: 'hexagon', 7: 'heptagon', 8: 'octagon'}


def page(path_out):
    """The 3D view: every shape, lying along the page with its finger holes
    to the front and the cap on the left; Explode spreads the plates along
    the chanter, the cap a little further."""
    import turn3d as t
    designs = {}
    for n in M.SIDES:
        parts, info = build(n)
        mid = (min(p.x for p in parts) + max(p.x for p in parts)) / 2
        out = []
        for p in parts:
            holes = [c[1] if c[0] == 'poly' else circle(c[1], c[2], c[3], 20) for c in p.cuts]
            prt = {'o': p.outline, 'h': holes, 'e': []}
            # the plate's x across, y down and z through it go to the
            # chanter's up, front and length; a turn, not a mirror
            m = [[0, 0, 1, p.x], [0, 1, 0, 0], [-1, 0, 0, 0], [0, 0, 0, 1]]
            cap = p.kind in ('socket', 'chamber', 'top', 'stem')
            ex = ((p.x - mid) / 60 - (1.0 if cap else 0.0), 0, 0)
            wood = 'dark' if p.kind in ('slot', 'drill') else 'pale' if cap else 'ply'
            out.append(t.placed(prt, m, ex, wood))
        designs[f'{n} sides'] = {'parts': out, 'ex': 60, 'view': [-0.12, 0.32, 3.0]}
    t.write(path_out, 'Practice chanters in 3D', designs,
            'Drag to turn, scroll or pinch to zoom; draw Explode to spread the plates along '
            'the chanter. Every plate is the cut files\u2019 own outline. The finger-hole plates '
            'are darker: a slot through the face, or a V where a hole too small to cut is '
            'drilled. The cap, paler, is on, its socket over the body\u2019s top plates. The '
            'buttons switch between the bores, three sides to eight.',
            'practice_chanters.py')


def main():
    global KERF
    out, write, sides, page_to = None, True, None, None
    for x in sys.argv[1:]:
        if x == '--no-write':
            write = False
        elif x.startswith('--out='):
            out = x.split('=', 1)[1]
        elif x.startswith('--sides='):
            sides = int(x.split('=', 1)[1])
            if sides not in M.SIDES:
                raise SystemExit(f'error: --sides={sides}: the chanters have 3 to 8 sides.')
        elif x.startswith('--plate='):
            M.PLATE = float(x.split('=', 1)[1])
        elif x.startswith('--reed='):
            M.REED = float(x.split('=', 1)[1])
        elif x.startswith('--low-a='):
            M.LOW_A = float(x.split('=', 1)[1])
        elif x.startswith('--kerf='):
            KERF = SF.set_kerf(x)
        elif x.startswith('--page-to='):
            page_to = x.split('=', 1)[1]
        else:
            raise SystemExit(f'error: {x} is not a flag this generator reads. It takes --sides=, '
                             f'--out=, --plate=, --reed=, --low-a=, --kerf=, --page-to= and '
                             f'--no-write.')
    if page_to:
        page(page_to)
        print(f'  wrote {os.path.basename(page_to)}')
        return 0
    if sides is None:
        raise SystemExit('error: --sides=3 to 8, the bore\'s polygon.')
    if write and not out:
        raise SystemExit('error: --out=DIR, or --no-write to run the checks only.')
    parts, info = build(sides)
    sheets = pack(parts)
    res = checks(sides, parts, info, sheets)
    body = N_SEAT + info['n_bore']
    print(f'practice chanter, {sides} sides ({NAMES[sides]}), low A {M.LOW_A:g}Hz: '
          f'{body} body plates of {M.PLATE:g}mm ({body * M.PLATE:g}mm), '
          f'{N_SOCKET + N_CHAMBER + N_TOP + N_STEM} cap plates; two M{ROD:g} rods cut '
          f'{body * M.PLATE + 2 * (0.5 + NUT_T) + ROD_SPARE:.0f}mm long; {PIN:g}mm pins '
          f'{(N_SOCKET + N_CHAMBER + N_TOP) * M.PLATE:g}mm through the cap, '
          f'{(N_TOP + N_STEM) * M.PLATE:g}mm through the mouthpiece')
    for h in M.HOLES:
        w = info['widths'][h]
        k = int(info['pos'][h] // M.PLATE) + 1
        how = f'slot {w:.2f}mm wide' if w >= M.W_SLOT else f'drill {M.drill(w):.2f}mm at the V'
        print(f'  {h:3} bore plate {k:3} ({info["pos"][h]:5.1f}mm down the bore)  {how}')
    print('  stack, top down: ' + stack_order(parts))
    bad = 0
    for ok, what, detail in res:
        print(f'  {"pass" if ok else "FAIL"}  {what:52} {detail}')
        bad += not ok
    if bad:
        print(f'\n  {bad} check(s) failed. Nothing written.')
        return 1
    if write:
        os.makedirs(out, exist_ok=True)
        for k, s in enumerate(sheets, 1):
            f = os.path.join(out, f'practice-chanter-{sides}-sided-sheet{k}-cut-files.svg')
            open(f, 'w').write(svg_sheet(k, len(sheets), s, sides))
            print(f'  wrote {os.path.basename(f)}  {s["w"]:.0f} x {s["h"]:.0f}mm, '
                  f'{len(s["parts"])} parts')
    return 0


if __name__ == '__main__':
    sys.exit(main())
