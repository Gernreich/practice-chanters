# Practice chanters

<!-- readme-only -->
**[Read the writeup](https://gernreich.github.io/practice-chanters/)** ·
**[Turn them in 3D](https://gernreich.github.io/practice-chanters/#turn-them-in-3d)**

<!-- page-only **[Read the README](https://github.com/Gernreich/practice-chanters)** -->

Six Great Highland bagpipe practice chanters in 3mm plywood, alike except for their
bores. One has a triangular bore, the others square, pentagonal, hexagonal, heptagonal and octagonal.
A seventh is the square with its top fifth bored round: see
[The square with a round top](#the-square-with-a-round-top).
Each is a stack of thin cross-section plates with the bore cut through every one, clamped
on two threaded rods, under a stacked cap and mouthpiece. The only part you buy is a
plastic practice-chanter reed.

Output is millimetre-true — `1 user unit = 1mm` with a physical `width`/`height` — so
it prints and cuts at real size.

**None has been cut or played yet.** The holes come from an acoustic model that has
not been checked against a real chanter, and the reed's length inside the cap is a guess.
[Before you cut](#before-you-cut) says what that costs, and [Tuning](#tuning) how to
take it up.

<table>
<tr>
<td align="center"><a href="cut-files/practice-chanter-5-sided-sheet1-cut-files.svg"><img src="previews/practice-chanter-5-sided-sheet1-cut-files.svg" alt="A sheet of parts: rows of small pentagons, each with a little pentagonal hole in the middle and two pin holes, alternate ones turned upside down; a few have a slot from the hole out through one face, and two have a small V in one edge; at the top, larger pentagons with round holes for the cap and the reed's seat, and small discs for the mouthpiece" width="100%"></a></td>
</tr>
<tr>
<td align="center"><sub>the pentagonal chanter: every plate on one sheet</sub></td>
</tr>
</table>

Built for **[LaserMadeMusic](https://www.youtube.com/@LaserMadeMusic)**.

**[The rest of the build files](https://gernreich.github.io/)** — every instrument,
generator and tool, indexed.

**[Download everything as a ZIP](https://github.com/Gernreich/practice-chanters/archive/refs/heads/main.zip)**

## Where the cut-off is

**Built as stacked plates, there is no cut-off between three sides and eight.** Every
shape builds, fits one sheet and tunes in the model. What a bore's cross-section
needs is the right area. Its shape matters far less:
[a square-bored chanter](https://www.daye1.com/bagpage/makepipe/squint1.html)
was built on exactly that principle, a square of the round bore's area at every point.
Here every bore is the area of a 4mm circle, 12.57mm², the bore most practice-chanter makers drill.
A plate is a slice across the chanter, so the polygon is simply cut out of it, and
any polygon can be.

Two things change with the number of sides, and neither stops the build:

- **The faces shrink.** Each face of the bore is 5.39mm wide on the triangle, 3.54mm
  on the square, 2.70mm on the pentagon, 2.20mm on the hexagon, 1.86mm on the
  heptagon and 1.61mm on the octagon. By eight sides the laser's own kerf rounds the
  corners and the bore is all but round, so **eight is where adding sides stops
  meaning anything**, not where it stops working.
- **The triangle has the most wall.** For the same area it has 29% more wall than a
  round bore, against 13% for the square and 3% for the octagon. More wall means more
  friction losses, so the triangle's notes should be a little less bright and
  responsive. The model counts that loss.

**There is a cut-off if the bore is built the other way**, from long strips jointed
round it as the [octagonal flute](https://github.com/Gernreich/octagonal-flute) is.
Each strip is a face of the bore, and its finger joints need ply between them. In
3mm ply at a 4mm bore, **only the square works.** The pentagon leaves 1.36mm between
joints where 3mm is needed. The hexagon and up leave less, and the octagon none.
The triangle cannot be finger-jointed at all, since its corners are too sharp for
the joint. Thinner 1.5mm stock buys the pentagon. A wider bore would buy more shapes, but
then it is no longer a practice chanter. That is why these are stacks.

## The chanters

| | |
|---|---|
| Pitch | low A at 240Hz, an octave below a pipe chanter's 470–480Hz; `--low-a=` sets it |
| Scale | low G to high A, the [Highland scale](https://en.wikipedia.org/wiki/Great_Highland_bagpipe) in just intonation, G the harmonic seventh |
| Fingering | the [standard Highland fingering](https://bagpipejourney.com/articles/finger_positions.shtml): seven finger holes on the front, the high A thumb hole on the back |
| Bore | the area of a 4mm circle, 378mm long (381mm on the triangle), open at the sole |
| Body | the area of a 20mm circle on the outside, 396mm long (399mm on the triangle): 132 or 133 plates |
| Cap | 31 plates: a socket over the body, a chamber round the reed, and a mouthpiece |
| Joints | clamped on two M2 threaded rods; the cap and mouthpiece press-fit on 2mm pins; nothing glued, the seams sealed with shellac |
| Sheets | one 600 × 305mm sheet of 3mm ply each |

| Sides | Bore, across its flats | Model's worst note | Sheet |
|---|---|---|---|
| 3, triangle | 3.11mm | in tune | 584 × 190mm |
| 4, square | 3.54mm | in tune | 599 × 183mm |
| 5, pentagon | 3.72mm | 1.6 cents | 598 × 230mm |
| 6, hexagon | 3.81mm | 2.5 cents | 598 × 217mm |
| 7, heptagon | 3.86mm | 3.6 cents | 599 × 225mm |
| 8, octagon | 3.89mm | 4.0 cents | 596 × 196mm |

Low G is left out of the worst note: it sounds from the whole bore, and the bore is
a whole number of plates, so it lands up to a few cents out. The generator prints
how far for each shape.

**The holes are not all the same size.** Each is the width that puts its note in
tune, in a fixed hand layout: the lower hand's four holes 27mm apart, the upper
hand's three 27mm and then 21mm apart, the thumb 9mm above G. A hole is a slot cut
across one plate, 3mm tall and as wide as it needs. A slot under 1mm wide is too
narrow to cut cleanly. Such a hole is drilled through instead, after assembly, at a
small V cut into its plate's edge. The drill is the slot's area as a circle, so it
tunes the same. On every shape that is G, about 1.3mm; on the square, pentagon and
hexagon also D, about 1.9mm.

On the pentagon to the octagon the thumb's slot wants to be wider than 5mm. It is
held at 5mm and moved up the chanter until 5mm is right, to the nearest plate, which
is why those four are a few cents out. A slot wider than the bore runs in to the
bore's middle and ends square there.

## The square with a round top

The second design is the square chanter with the top of its bore round. The bore is square
from the sole to 4/5 of the way up the body, 318mm of 396mm. Above that, the 20 bore
plates under the seat have a 4mm round hole instead of the square. The seat's 6 plates
were round already, so the top 26 plates all have round holes.

- **Nothing moves.** The round hole has the square's area, 12.57mm², and every finger
  hole is below the switch: the thumb's is 31 plates under it. The model re-solves
  the holes with the round top in place, and they come out the same to a hundredth of
  a millimetre; no note moves by even 0.0001 cents. The bore's shape changes only a
  hole's chimney and the wall's friction, and there are no holes up there.
- **Same outline, same sheet.** The round plates are the square body's outline with
  the same rod holes, so the cap fits as before and the sheet is the same, 599 ×
  183mm. A round plate is told from a square one by its hole.
- **The staple stops on its hemp, not the bore.** On the square, the bore is 3.54mm
  across under the seat, narrower than the smallest staple, 4mm, so the bore stops
  the staple. The round bore is 4mm, so a 4mm staple could slide on into it, and only
  its hemp sets its depth. A staple over 4mm still stops on the bore.
- **Where the two meet,** each corner of the square stands 0.5mm outside the circle,
  and the circle stands 0.23mm outside each flat. The areas are equal, so the model
  sees no step there.

`--round-top=0.75` moves the switch, to the nearest whole plate; it must stay above the thumb's hole.

## How it is built

From the top down:

- **The mouthpiece**, 12 round plates 17mm across with a 6mm blowing hole, pressed
  together on two 2mm pins.
- **The cap**: two top plates, twelve plates round a 14mm chamber for the reed's
  blades, and five plates whose hole is the body's own polygon, 0.3mm larger, to
  slide over the top of the body. It is the body's shape grown 6.3mm outward, pressed
  together on two pins in its corners.
- **The seat**: six plates with a 5.5mm round hole that takes the reed's staple.
  Practice-chanter staples run 4 to 4.8mm across. The bore below is narrower than
  any of them, so the staple stops on it.
- **The bore**: one plate for every 3mm, the polygon through each, down to the open
  sole. Eight of them carry the finger holes.

Every body plate has the same two rod holes, placed so that turning or flipping the
plate puts them somewhere else: a plate threads on only one way, so no slot can end
up facing the wrong way. The rods are clamped by a nut and washer at each end. A
pin pressed through 130 plates would never go in, and nothing is glued. The cap is
not on the rods, since it comes off to set the reed. It slides over the top five
plates of the body and rests on the two top nuts.

## Turn them in 3D

<!-- readme-only -->
The chanters turn in 3D on the [published page](https://gernreich.github.io/practice-chanters/#turn-them-in-3d),
which GitHub does not run: drag to turn them, pull them apart along their length,
and switch between the six bores and the square with a round top.

<!-- page-only <iframe src="turn.html" title="A practice chanter as solid plates you can drag to turn and pull apart: a long stack of thin polygonal plates with darker finger-hole plates along the front, a wider cap at one end and a round mouthpiece beyond it, with a button for each of the six bores and the square with a round top" style="width:100%;height:560px;border:0"></iframe> -->

## Before you cut

**Measure the ply.** The stack multiplies the stock's error by every plate in it. 2.9mm ply
makes the bore 13mm short, and every note 50 to 57 cents sharp in the model, which
is a quarter-tone. Measure a few places on the sheet with calipers, then pass the
average as `--plate=`. The generator re-solves every hole on that grid, and the plate
count follows.

**The reed's length is a guess.** The model treats the reed and staple as 20mm of
extra closed bore, which is not a measured figure. A reed pushed deeper into its seat
plays sharper, and one pulled out plays flatter. That is how every practice chanter is
pitched, and it shifts the whole scale together, but it cannot fix one note against
another. `--reed=` sets the figure if you measure your own.

**The model has not met a real chanter.** It is a transfer-matrix model with textbook
tone-hole and end corrections. Its holes are smaller than a turned chanter's, which
are drilled about 2.4 to 4mm. Expect individual notes to be out by tens of cents
until [Tuning](#tuning) has been done.

**The slots are cut exactly to width,** so a kerf other than the default 0.13mm
changes every hole. Pass yours with `--kerf=`.

## Building it

You need two lengths of M2 threaded rod, four M2 nuts and washers, a few centimetres
of 2mm brass rod for the pins, high-cut shellac, and a plastic practice-chanter reed.

1. Cut the sheet. The generator prints the stacking order for its shape. For the
   square it is: 6 seat, 51 bore, hA, 2 bore, G, 6 bore, F, 8 bore, E, 5 bore, D,
   8 bore, C, 8 bore, B, 8 bore, A, 22 bore. Here "seat" and "bore" are runs of the
   plain plates, and the letters are the hole plates. The square with a round top
   starts 6 seat, 20 round, 31 bore and goes on the same.
2. Cut the rods to the length the generator prints: 403mm, or 406mm for the
   triangle. Put a nut and washer on one end of each.
3. Thread the plates on from the sole up, in the reverse of that order. Brush thin
   shellac on each face as it goes on. The slots all face the same way because the
   rods only let them.
4. Put the top washers and nuts on and tighten them evenly, just enough to close every
   seam. Cut the rods off flush with the top nuts, or the cap will rest on the rods
   instead of the nuts. Let the shellac dry.
5. Drill the small holes at their V marks, starting a size under the printed figure.
   Then seal the bore: draw a pipe cleaner wet with shellac through it a few
   times, and let it dry. Brush the inside of the cap the same way.
6. Press the cap's plates together on two pins through its corners. Press the mouthpiece's plates on two
   pins, which go on into the cap's top plates.
7. Push the reed's staple into the seat. Wrap it with hemp if it is loose. Put the cap
   on.

## Tuning

Tune low A first, by moving the reed in or out of its seat. Then go up the scale:

- **A note that is flat** wants its hole bigger. File the slot wider a little at a
  time, or open the drill up a size.
- **A note that is sharp** wants its hole smaller. Line part of the slot with tape,
  or with a sliver of veneer and shellac.
- **Low G** has no hole. Tape across part of the open sole flattens it.

Widening a hole moves its own note most, about 10 cents for a fifth wider in the
model. The notes either side of it move a cent or two, so go up the scale once,
then check every note again and touch up.

## Regenerating it

```sh
python3 practice_chanters.py --sides=5 --no-write          # the checks, writing nothing
python3 practice_chanters.py --sides=5 --out=cut-files     # its sheet
python3 practice_chanters.py --sides=5 --plate=2.9 --no-write   # for ply as measured
python3 practice_chanters.py --sides=5 --reed=24 --no-write     # a different reed allowance
python3 practice_chanters.py --sides=5 --low-a=233 --no-write   # a different pitch
python3 practice_chanters.py --sides=5 --kerf=0.18 --out=mine   # for a laser with a 0.18mm kerf
python3 practice_chanters.py --sides=4 --round-top --out=cut-files   # the square with a round top
python3 practice_chanters.py --page-to=turn.html           # the 3D view, all seven
python3 chanter_model.py --sides=5                         # the acoustic model alone
```

`practice_chanters.py` draws nothing until every check passes:

- every note is in tune in the model, and low G is as near as whole plates allow;
- the bore is a 4mm round bore's area;
- the bore is whole plates, down to the sole;
- every finger hole is in its own plate, at its centre;
- every hole is cuttable or drillable;
- every slot runs the width it is tuned for;
- every rod and pin keeps ply between it and every cut;
- the nuts sit on the end plates, inside the body;
- every body plate goes on only one way;
- the seat holds a staple, and the bore stops it (with a round top, the bore stops staples over 4mm only, and the check says so);
- the cap slides over the body and stops on its nuts;
- every sheet fits the bed;
- no two parts on a sheet come closer than the gap;
- every part is on a sheet.

`practice_chanters.py` imports the kerf and the `Part` class from the square flute's
`square_flute.py`, the mitred offset from the four-hole ocarinas' `four_hole_ocarinas.py`,
the path writer from swept-curve and the 3D page from lasermade-tools. So
[square-flute](https://github.com/Gernreich/square-flute),
[four-hole-ocarinas](https://github.com/Gernreich/four-hole-ocarinas),
[swept-curve](https://github.com/Gernreich/swept-curve) and
[lasermade-tools](https://github.com/Gernreich/lasermade-tools) must be checked out
beside this repository.

## Files

Every chanter is one sheet of 3mm ply.

| | |
|---|---|
| `cut-files/practice-chanter-3-sided-sheet1-cut-files.svg` | the triangular chanter, every plate |
| `cut-files/practice-chanter-4-sided-sheet1-cut-files.svg` | the square chanter, every plate |
| `cut-files/practice-chanter-5-sided-sheet1-cut-files.svg` | the pentagonal chanter, every plate |
| `cut-files/practice-chanter-6-sided-sheet1-cut-files.svg` | the hexagonal chanter, every plate |
| `cut-files/practice-chanter-7-sided-sheet1-cut-files.svg` | the heptagonal chanter, every plate |
| `cut-files/practice-chanter-8-sided-sheet1-cut-files.svg` | the octagonal chanter, every plate |
| `cut-files/practice-chanter-4-sided-round-top-sheet1-cut-files.svg` | the square chanter with a round top, every plate |
| `previews/practice-chanter-3-sided-sheet1-cut-files.svg` · `previews/practice-chanter-4-sided-sheet1-cut-files.svg` · `previews/practice-chanter-5-sided-sheet1-cut-files.svg` · `previews/practice-chanter-6-sided-sheet1-cut-files.svg` · `previews/practice-chanter-7-sided-sheet1-cut-files.svg` · `previews/practice-chanter-8-sided-sheet1-cut-files.svg` · `previews/practice-chanter-4-sided-round-top-sheet1-cut-files.svg` | a display rendering of each sheet — **not** cut files |
| `turn.html` | the 3D view, all six bores, on a page of its own |
| `practice_chanters.py` | the generator, its checks, the cut files and the 3D view |
| `chanter_model.py` | the acoustic model: where each hole goes and how wide |
| `README.md` · `index.html` | this writeup and its published page; the markdown is the source |
| `.repro` | the command that draws each generated file |

Released under [CC0 1.0](LICENSE).
