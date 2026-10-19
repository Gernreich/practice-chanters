#!/usr/bin/env python3
"""A Great Highland bagpipe practice chanter with a regular polygonal bore,
three to eight sides, as a transmission-matrix model: how long the bore runs,
and where its eight finger holes go and how wide each is cut.

    python3 chanter_model.py                 # every shape: the design, how it plays
    python3 chanter_model.py --sides=5       # one shape
    python3 chanter_model.py --plate=2.9     # ply as measured
    python3 chanter_model.py --reed=25       # another reed correction, mm
    python3 chanter_model.py --low-a=233     # another pitch

THE SCALE is the bagpipe's nine notes, low G to high A, in just intonation on
the Mixolydian mode with G the harmonic seventh, as the pipes are tuned. A
practice chanter's cylindrical bore sounds an octave below the pipe chanter's
conical one, so low A is LOW_A, 240Hz, half the pipes' 480. The notes are
fingered as on the pipes, which is not one hole after another: C and D keep
the low A hole closed below the open ones, E opens low A again, and high A
closes E. FINGERING says which holes are open for each note, and the model
solves every note with its own fingering.

THE BORE is a regular polygon of BORE_AREA, the area of a 4mm round bore, the
size makers bore practice chanters. Its wall loss is taken on the hydraulic
radius 2A/P, which for a regular polygon is its inradius: the fewer the
sides, the more wall for the area, and the more loss. design(top=) makes
the first top mm under the reed round, the same area, above every hole.

THE CHANTER is a stack of PLATE-thick cross-section plates, each with the
polygon cut through it. A finger hole is a slot cut in one plate from the
bore out through its face, PLATE long along the bore and as wide as the note
needs, so holes stand on the plates' grid and their widths are solved.
Its chimney is the wall, from the bore's face to the outside's.

THE HOLES sit where hands reach them: the lower hand's four SPACING apart,
the upper hand's E and F the same, G G_GAP below F and the thumb's hole THUMB
above G. Their widths are solved, note by note. A hole under W_SLOT wide is
too narrow to cut, so it is drilled after assembly instead, a round hole of
the same area; one that would need more than W_MAX is held at W_MAX and
moved up the chanter until that is enough.

The model, from the reed down:

  * the reed and its staple are taken as REED mm more closed bore above the
    bore's top, where the staple sits; that is the uncalibrated part, as the
    embouchure is on a flute. A piper sets a chanter's pitch by how deep the
    reed sits, which moves exactly this;
  * the bore is a lossy tube;
  * each finger hole is Keefe's (1990) tone hole, open or closed, the round
    hole of the slot's area;
  * the bottom radiates as a flanged open end, the chanter's sole being far
    wider than its bore. The bore is the nearest whole number of plates, so
    low G comes out a few cents out either way, the one note with no hole of
    its own to tune; tape across part of the sole flattens it if sharp.

A reed sounds where the impedance it sees peaks: where the admittance's
imaginary part passes through zero, rising.
"""
import cmath
import math
import sys

# --- air, warm from the breath ----------------------------------------------
T_C = 25.0
C = 331.3 * math.sqrt(1 + T_C / 273.15)          # 346.1 m/s
RHO = 1.184

# --- the tune ----------------------------------------------------------------
LOW_A = 240.0
NOTES = ['low G', 'low A', 'B', 'C', 'D', 'E', 'F', 'high G', 'high A']
RATIOS = [7 / 8, 1, 9 / 8, 5 / 4, 4 / 3, 3 / 2, 5 / 3, 7 / 4, 2]
# the holes from the bottom up, each named for the note it opens; high A is
# the thumb's, on the back
HOLES = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'hA']
FINGERING = {
    'low G': (),
    'low A': ('A',),
    'B': ('B', 'A'),
    'C': ('C', 'B'),
    'D': ('D', 'C', 'B'),
    'E': ('E', 'A'),
    'F': ('F', 'E', 'A'),
    'high G': ('G', 'F', 'E', 'A'),
    'high A': ('hA', 'G', 'F', 'A'),
}
# the note each hole is placed and sized for, in the order they are solved
SETS = ['low A', 'B', 'C', 'D', 'E', 'F', 'high G', 'high A']

# --- the bore and the plates (mm) -------------------------------------------
BORE_AREA = math.pi * 2.0 ** 2     # a 4mm round bore's
OUTER_AREA = math.pi * 10.0 ** 2   # the outside: a 20mm round chanter's
PLATE = 3.0
REED = 20.0                        # the reed and staple, as closed bore
W_MAX = 5.0                        # the widest slot a finger pad closes
SIDES = (3, 4, 5, 6, 7, 8)
SCAN = 30                          # points a resonance is looked for across


def inradius(area, n):
    return math.sqrt(area / (n * math.tan(math.pi / n)))


class Bore:
    """The n-gon: its area, equivalent radius and loss radius (m), and its
    wall, the chimney of a hole through a face (mm). n=0 is a round bore.
    A bore may have a round top: its first top mm, under the reed, round."""

    def __init__(self, n, top=0.0):
        self.n = n
        self.A = BORE_AREA / 1e6
        self.a = math.sqrt(self.A / math.pi)
        self.rh = (inradius(BORE_AREA, n) if n else math.sqrt(BORE_AREA / math.pi)) / 1000
        self.wall = (inradius(OUTER_AREA, n) - inradius(BORE_AREA, n)) if n else None
        self.top = top
        self.round = Bore(0) if top else self

    def k(self, f):
        return 2 * math.pi * f / C - 1j * 3.0e-5 * math.sqrt(f) / self.rh

    def z0(self):
        return RHO * C / self.A


def tube(b, f, L):
    k, z = b.k(f), b.z0()
    c, s = cmath.cos(k * L), cmath.sin(k * L)
    return ((c, 1j * z * s), (1j * s / z, c))


def mul(m, n):
    return ((m[0][0] * n[0][0] + m[0][1] * n[1][0], m[0][0] * n[0][1] + m[0][1] * n[1][1]),
            (m[1][0] * n[0][0] + m[1][1] * n[1][0], m[1][0] * n[0][1] + m[1][1] * n[1][1]))


def hole(b, f, w, t, is_open):
    """Keefe's tone hole, for a slot w wide and PLATE long: series half,
    shunt, series half."""
    k = 2 * math.pi * f / C
    r, a = math.sqrt(w * PLATE / math.pi) / 1000, b.a
    t = t / 1000
    dl = r / a
    zh = RHO * C / (math.pi * r * r)
    if is_open:
        te = ((1 / k) * math.tan(k * t) + r * (1.40 - 0.58 * dl ** 2)) / \
             (1 - 0.61 * k * r * math.tan(k * t))
        zs = zh * (1j * k * te + 0.25 * (k * r) ** 2)
        ta = 0.47 * r * dl ** 4 / (math.tanh(1.84 * t / r) + 0.62 * dl ** 2 + 0.64 * dl)
    else:
        tc = t + r * dl * (1 + 0.207 * dl ** 3) / 8
        zs = -1j * zh / math.tan(k * tc)
        ta = 0.47 * r * dl ** 4 / (1 / math.tanh(1.84 * t / r) + 0.62 * dl ** 2 + 0.64 * dl)
    za = -1j * zh * k * ta
    half = ((1, za / 2), (0, 1))
    return mul(mul(half, ((1, 0), (1 / zs, 1))), half)


def admittance(b, f, L, pos, widths, opened):
    """Im(Y) the reed sees: REED mm of closed bore, then L mm of bore with
    the holes at pos (mm from the bore's top), the named ones open."""
    z = b.z0()
    ka = 2 * math.pi * f / C * b.a
    zr = z * (0.5 * ka ** 2 + 1j * 0.8216 * ka)
    # the reed's closed bore and the round top, if any, are one tube
    m = tube(b.round, f, (REED + b.top) / 1000)
    x = b.top
    for h in sorted(HOLES, key=lambda h: pos[h]):
        m = mul(m, tube(b, f, (pos[h] - x) / 1000))
        m = mul(m, hole(b, f, widths[h], b.wall, h in opened))
        x = pos[h]
    m = mul(m, tube(b, f, (L - x) / 1000))
    zin = (m[0][0] * zr + m[0][1]) / (m[1][0] * zr + m[1][1])
    return (1 / zin).imag


def sounding(b, L, pos, widths, note, near=None, span=1.3):
    """The note a fingering plays nearest `near`: a rising zero of Im(Y),
    scanned then bisected."""
    near = near or target(note)
    opened = FINGERING[note]
    lo, hi = near / span, near * span
    Y = lambda f: admittance(b, f, L, pos, widths, opened)
    prev_f, prev_y = lo, Y(lo)
    best = None
    for i in range(1, SCAN + 1):
        f = lo + (hi - lo) * i / SCAN
        y = Y(f)
        if prev_y < 0 <= y:
            a, c = prev_f, f
            for _ in range(32):
                mid = (a + c) / 2
                if Y(mid) < 0:
                    a = mid
                else:
                    c = mid
            root = (a + c) / 2
            if best is None or abs(root - near) < abs(best - near):
                best = root
        prev_f, prev_y = f, y
    if best is None:
        raise ValueError(f'{note}: no resonance within {span:g}x of {near:.1f}Hz')
    return best


def target(note):
    return LOW_A * RATIOS[NOTES.index(note)]


def bisect(fn, goal, lo, hi, rising=False, n=40):
    """x in [lo, hi] with fn(x) == goal. fn falls as x grows, unless rising.
    Raises when the goal is out of reach."""
    g = (lambda x: -fn(x)) if rising else fn
    t = -goal if rising else goal
    if not (g(hi) <= t <= g(lo)):
        raise ValueError(f'out of reach between {lo:.1f} and {hi:.1f}')
    for _ in range(n):
        mid = (lo + hi) / 2
        if g(mid) > t:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def snap(x):
    """The centre of the plate x falls in, plates counted from the bore's top."""
    return (math.floor(x / PLATE) + 0.5) * PLATE


SPACING = 27.0                     # centre to centre within a hand, 9 plates
ANCHORS = ('C', 'E')               # one hole in each hand, its plate solved
W_ANCHOR = 4.0                     # the anchors' width while their plates are solved
THUMB = 9.0                        # the high A hole above the G hole, on the back
# F to high G is the scale's smallest step, 84 cents, so at SPACING the G hole
# would have to be smaller than any drill; it stands G_GAP above F instead
G_GAP = 21.0
# A hole the note needs narrower than W_SLOT is drilled after assembly instead
# of cut, round, of the slot's area, at a V cut in its plate's edge; a hole
# wider than W_MAX is held there and moved up the bore instead.
W_SLOT = 1.0
D_MIN = 1.0                        # the smallest drill the model will ask for


def drill(w):
    """The round hole of a w-wide slot's area."""
    return math.sqrt(4 * w * PLATE / math.pi)


def design(n, spacing=None, anchors=ANCHORS, top=0.0):
    """Laid out for the hands, then fitted to the plates.

    In each hand one hole, the anchor, is W_ANCHOR wide while its plate is
    solved; the hand's other holes stand SPACING from it, and the thumb's
    hole THUMB above the G hole, the G hole G_GAP above F. Then every hole's width is solved where it
    stands, the anchors' too, so the snap to a plate costs nothing. A hole
    that would be wider than W_MAX is held at W_MAX and moved up instead.
    Repeated until the closed holes' effect on each other settles. Raises
    when a note cannot be reached. top is mm of round bore under the reed,
    above every hole."""
    spacing = spacing or SPACING
    b = Bore(n, top)
    lower, upper = ['A', 'B', 'C', 'D'], ['E', 'F', 'G']
    widths = {h: 3.0 for h in HOLES}
    note_of = dict(zip(HOLES, SETS))
    L = 400.0
    pos = {h: L * (0.85 - 0.08 * i) for i, h in enumerate(HOLES)}
    moved = set()

    def est(h):
        return 1000 * C / (4 * target(note_of[h])) - REED

    def place(h, lo, hi):
        nt = note_of[h]

        def f_pos(x):
            p = dict(pos)
            p[h] = x
            return sounding(b, L, p, widths, nt, span=2.2)
        return snap(bisect(f_pos, target(nt), lo, hi))

    def size(h):
        nt = note_of[h]

        def f_w(w):
            ws = dict(widths)
            ws[h] = w
            return sounding(b, L, pos, ws, nt)
        # past either end, say which way: 0 for a note sharp even at the
        # smallest hole, inf for one flat even at the largest
        lo = D_MIN ** 2 * math.pi / 4 / PLATE
        if f_w(lo) > target(nt):
            raise ValueError(f'{n} sides: {nt} is sharp even with its hole the '
                             f'smallest drill, {D_MIN:g}mm, {pos[h]:.1f}mm down the bore')
        if f_w(W_MAX) < target(nt):
            return math.inf
        return bisect(f_w, target(nt), lo, W_MAX, rising=True)

    for it in range(8):
        L = bisect(lambda x: sounding(b, x, pos, widths, 'low G', span=1.8),
                   target('low G'), 250, 600)
        # a whole number of plates, the nearest: low G comes out a few cents
        # out, half a plate's worth at most
        L = PLATE * round(L / PLATE)
        if it < 3:
            # the anchors' plates, at W_ANCHOR, only while the layout settles
            for hand, a in ((lower, anchors[0]), (upper, anchors[1])):
                widths[a] = W_ANCHOR
                pos[a] = place(a, 0.7 * est(a), min(L - PLATE, 1.3 * est(a)))
                for h in hand:
                    if h not in moved:
                        pos[h] = pos[a] + (hand.index(a) - hand.index(h)) * spacing
            if 'G' not in moved:
                pos['G'] = pos['F'] - G_GAP
            if 'hA' not in moved:
                pos['hA'] = pos['G'] - THUMB
        for h in HOLES:
            i = HOLES.index(h)
            below = pos[HOLES[i - 1]] if i else L
            above = pos[HOLES[i + 1]] if i + 1 < len(HOLES) else 0.0
            w = size(h)
            if w <= W_MAX:
                widths[h] = w
                continue
            moved.add(h)
            widths[h] = W_MAX
            pos[h] = place(h, max(above + PLATE, 0.6 * est(h)), below - PLATE)
            widths[h] = min(size(h), W_MAX)
    return b, L, pos, widths


def cents(f, ref):
    return 1200 * math.log2(f / ref)


def report(b, L, pos, widths):
    return [(nt, f, cents(f, target(nt)))
            for nt in NOTES for f in [sounding(b, L, pos, widths, nt)]]


def main():
    global REED, LOW_A, PLATE
    sides = SIDES
    for x in sys.argv[1:]:
        if x.startswith('--sides='):
            sides = (int(x.split('=', 1)[1]),)
        elif x.startswith('--plate='):
            PLATE = float(x.split('=', 1)[1])
        elif x.startswith('--reed='):
            REED = float(x.split('=', 1)[1])
        elif x.startswith('--low-a='):
            LOW_A = float(x.split('=', 1)[1])
        else:
            raise SystemExit(f'error: {x} is not a flag this model reads. It takes '
                             f'--sides=, --plate=, --reed= and --low-a=.')
    print(f'low A {LOW_A:g}Hz, air at {T_C:g}C ({C:.1f} m/s); bore {BORE_AREA:.2f}mm^2, '
          f'reed + staple as {REED:g}mm of closed bore; {PLATE:g}mm plates')
    for n in sides:
        b, L, pos, widths = design(n)
        print(f'\n{n} sides: wall {b.wall:.2f}mm, loss radius {1000 * b.rh:.2f}mm; '
              f'bore {L:.1f}mm from its top to the sole')
        prev = L
        for h in HOLES:
            how = (f'slot {widths[h]:4.2f}mm wide' if widths[h] >= W_SLOT
                   else f'drill {drill(widths[h]):4.2f}mm  ')
            print(f'  {h:3} plate {int(pos[h] // PLATE) + 1:3}, {pos[h]:6.1f}mm  '
                  f'{how}  {prev - pos[h]:5.1f}mm above the one below')
            prev = pos[h]
        for nt, f, c in report(b, L, pos, widths):
            print(f'    {nt:7} {f:7.2f}Hz {c:+6.1f} cents')
    return 0


if __name__ == '__main__':
    sys.exit(main())
