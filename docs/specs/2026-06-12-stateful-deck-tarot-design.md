---
title: Stateful-deck tarot for jianyi
status: design approved 2026-06-12; ready for implementation plan
component: jianyi (tarot)
created: 2026-06-12
updated: 2026-06-12
---

# Stateful-deck tarot for jianyi

Redesign jianyi's tarot from a stateless one-shot draw into a **deck a caller can
hold, shuffle, and draw from over time** — a deck primitive faithful enough that
the calculation genuinely *is* a deck being shuffled, while jianyi itself stays
stateless and pure.

## Why

The kairos.swift app wants a divination page; that page wants the experience of a
real deck (shuffle, draw the top card, the deck persists between draws), and it
will reach the engine over FFI. The I Ching needs none of this — a cast is a
single event — but tarot is inherently a *deck with continuity*. Today jianyi's
tarot is draw-only: it shuffles a fresh deck and returns N loose cards with no
persistent order. This redesign gives the deck a life across calls without making
jianyi hold any state.

## Principles (load-bearing, decided)

1. **jianyi stays stateless.** It holds no deck. Every operation is a pure
   function `(deck, args) → (deck, result)`. The **caller** holds the deck state
   and threads it into the next call. This is what makes the engine both
   CLI-replicable and FFI-clean — they are the same operations, two faces.
2. **Most operations are replicable by an AI using a CLI.** The CLI takes the
   deck state as an argument and prints the new one; the FFI takes it as a JSON
   string and returns one. An agent driving a reading by hand does *literally the
   same operations* the Swift app does over FFI.
3. **jianyi models the draw pile only (Option A).** The deck state is the ordered
   pile of undrawn cards and their orientations. Once a card is drawn it **leaves
   jianyi's model** — where it sits on the table, face up or face down, in which
   spread position, is the **caller's** concern (the app, or an interpretive
   layer above jianyi). This makes the "cards outside the deck, face up" case
   *disappear* rather than needing to be solved: that is the table, not the deck.
   (The richer table model — zones, positions, face-up/down, spreads — is a
   deliberate future build in the Swift app. The boundary drawn here makes that
   additive, not a rewrite.)
4. **Orientation is baked at shuffle, revealed at draw.** Whether a card is
   reversed is decided when the deck is shuffled/cut, not rolled when it is drawn.
   Drawing only *reveals* what was already true. (The current code rolls
   orientation at draw time; that is the behavior being corrected.)
5. **Two distinct "seeds."** They are not the same thing and conflating them
   breaks pop-from-top:
   - **Deck code** — the persistent state handed back and forth, mutated by every
     op. This is the deck itself (order + orientations).
   - **Shuffle entropy seed** — a separate input consumed only by `shuffle`, to
     make a given shuffle deterministic/replayable (a timestamp, or caller-
     supplied). It is *not* the deck state.
6. **Scope this round: mechanics only.** No card meanings, keywords, or `tarot
   card` lookup. Reference data (RWS keywords, suit/element/number, astrological
   correspondence) is a legitimate future addition to jianyi — mirroring
   `iching hexagram` — but is a separate content axis and a separate deliverable.
   A draw returns card *identity + orientation*; the caller supplies display text
   and art.

## Sequencing: Z first, then X

- **This round (Z):** build the stateful-deck engine *in jianyi*, as a library
  with a thin CLI over it. Provable entirely by CLI. No kairos involvement. No new
  drift.
- **Follow-on (X), separate deliverable:** wire kairos-ffi to depend on the jianyi
  crate, have `kairos-divination` drop its tarot copy and re-expose jianyi's deck,
  surface the ops over the C ABI, and build the Swift divination page.

### Context: the drift this avoids

Two near-identical tarot engines exist today — `repos/jianyi/src/tarot/` and
`repos/kairos.rs/crates/kairos-divination/src/{tarot,draw}.rs` (jianyi copied the
latter). The kairos FFI already depends on `kairos-divination` but does **not** yet
expose tarot (only planetary hours). Redesigning one copy and not the other would
re-create the "multiple casters disagreeing" problem. The end-state (X) is **one
engine**: jianyi is the base primitive, *depended on, not copied*. Z builds that
engine; X makes kairos depend on it. Making kairos.rs depend on jianyi is approved.
(Augur is **not** involved and is not a dependency for this work.)

## Architecture

### Crate / module structure

jianyi gains a **library target**; the binary becomes a thin CLI over it. The lib
is exactly what kairos.rs depends on in the X round, with zero rework.

```
repos/jianyi/
├── Cargo.toml              # add [lib] alongside the existing [[bin]]
├── src/
│   ├── lib.rs              # NEW — public API: re-exports the deck engine
│   ├── main.rs             # thin CLI dispatch
│   └── tarot/
│       ├── mod.rs
│       ├── cards.rs        # TarotCard enum — unchanged (stable indices 0..=77)
│       ├── deck.rs         # NEW — Deck, DeckCard { card, reversed }
│       ├── code.rs         # NEW — deck-code codec (encode / decode / inspect)
│       ├── ops.rs          # NEW — new() / shuffle() / draw(): pure functions
│       └── cli.rs          # rewritten: args + rendering for the new ops
```

- No new runtime dependencies beyond a base64 implementation for the codec
  (prefer hand-rolled or a tiny crate; jianyi values a small footprint).
- The old stateless `draw_cards` / `DrawResult` path is **retired** into the new
  ops. One-shot draw becomes sugar (`new` + `shuffle` + `draw`, discard the code).
- Pure functions only: no I/O, no globals, no hidden RNG. The RNG is seeded
  per-call from the entropy seed.

### The deck-code format (data contract)

The deck code is the canonical interchange value — one string to store and thread.

- **Logical model:** an ordered `Vec<DeckCard { card: 0..=77, reversed: bool }>`,
  **top of pile = index 0**. Length ≤ 78; shrinks as cards are drawn.
- **Wire format:** `v1.<base64>` where the payload is
  `[count: u8][card_byte: u8 × count]`, each `card_byte` packing the 7-bit card
  index plus a 1-bit `reversed` flag. A full deck ≈ 80 bytes → ~108-char token.
  Copy-pasteable; a single string over FFI / in SwiftData.
- **Versioned on purpose.** The `v1.` prefix lets the format evolve without
  silently misreading old decks. Decoders reject unknown versions with a clear
  error.
- **`inspect` is the transparency bridge** (hybrid encoding, Option 3): decode a
  code to the expanded pile (order + orientations + count), pretty or `--json`.
  Every op also speaks `--json` for the expanded form. Compact for round-trip,
  transparent on demand — which keeps it debuggable and CLI-replicable.
- **The code does not carry:** shuffle-seed history, drawn/table state (caller's),
  or card meanings (out of scope). Just the current pile.

## Operations

Four pure operations. Each takes explicit inputs and returns a new deck (and
whatever it reveals).

| Op | Signature (logical) | Returns |
|----|--------------------|---------|
| `new` | `new(arcana: Option<Filter>) -> Deck` | fresh deck in canonical order, all upright, unshuffled; `major`/`minor` sub-deck supported |
| `shuffle` | `shuffle(Deck, entropy_seed, no_reversals: bool) -> Deck` | reordered deck with orientations set; deterministic from `(deck, seed)`; default seed = now |
| `draw` | `draw(Deck, Select) -> (Vec<DrawnCard>, Deck)`, where `Select = Top(n) \| At(position)` | the selected card(s) (card + *revealed* orientation) and the shrunk deck |
| `inspect` | `inspect(Deck) -> View` | expanded pile; pure read, no mutation |

- `new` and `shuffle` are separate: a fresh deck is a real, nameable state (the
  app can show "new deck," then animate the shuffle).
- `draw` from an unshuffled deck is legal (canonical order) — correct behavior,
  not a guarded error.
- `draw` with `n > pile size` (or an `At(position)` out of range) is an error.
- **Two draw selections.** `Top(n)` is the off-the-top deal; `At(position)` is the
  fan-pick — remove the single card at pile position `position` (top of pile =
  position 0), splicing it out so later cards shift down one. They are mutually
  exclusive per call. *Only jianyi can resolve a position → card*, because the
  caller holds an opaque deck code and does not know the pile order — so the
  fan-pick must be an engine op, not something the app does itself.
- **On a well-shuffled deck, `At` and `Top` are statistically the same draw** (a
  fair shuffle already randomized position). `At` does not add randomness; it
  exists to *honor the user's physical pick* (they tapped that card in the fan,
  and the experience breaks if they're silently handed the top one) and to select
  a specific card from an *un*shuffled or deliberately-ordered pile.
- **Drawn cards leave the pile permanently; jianyi never re-adds them.** A card
  drawn before a `shuffle` is simply absent from that shuffle — `shuffle` only ever
  touches the pile it is handed and has no knowledge of cards already on the table.
  So "pull from the middle → shuffle → pull from the top" needs no special
  handling: the middle card is gone after the first draw and cannot reappear.
  Re-incorporating a drawn card is an explicit op we are *not* building (see Out
  of scope). Note: the `position` of an `At` draw is distinct from a card's
  `index` (its 0..=77 identity in the JSON shape) — different meanings, don't
  conflate.

### Shuffle model (3b — physical riffle + cut)

`shuffle` simulates a real deck, so that orientation is a *consequence* of the
shuffle rather than an independent coin flip:

- **Riffle:** split the pile near the middle with seeded jitter and interleave the
  two halves imperfectly. Repeat enough times to mix well (target ~7+ riffles for
  a 78-card deck; the exact count is a tuned constant, justified by the mixing
  test).
- **Cut + flip (reversals):** a cut where a segment is inverted introduces
  reversals the way real practice does. Orientation is set here, not at draw.
- **Deterministic:** all randomness derives from the entropy seed via a seeded
  PRNG; the same `(deck, seed)` always yields the same shuffled deck.
- **`no_reversals`:** the caller can request an upright-only deck (for readers who
  don't use reversals). The guarantee is simply *every card comes out upright* —
  it is **not** required to produce the same card *order* as a reversals-enabled
  shuffle of the same seed, because the cut-flip that creates reversals also moves
  cards (physically rotating a segment reverses its sequence as well as its
  orientation), so removing it legitimately changes the order. Still a valid full
  shuffle, just all-upright.

## CLI surface

```
jianyi tarot new      [--arcana major|minor]
jianyi tarot shuffle  <deck-code> [--seed <rfc3339>] [--no-reversals]
jianyi tarot draw     [<deck-code>] [-n N | --at <position>] [--seed <rfc3339>] [--emit-deck]
jianyi tarot inspect  <deck-code>
```

- **`draw` is the one-shot hinge:**
  - *with* `<deck-code>` → remove the selected card(s) — top `n` (`-n`) or a
    single fan-pick (`--at <position>`); **always** emit drawn cards **+ the new
    deck code** (else state is lost). `-n` and `--at` are mutually exclusive.
  - *without* `<deck-code>` → one-shot: `new` + `shuffle(--seed or now)` +
    `draw n`; print the cards; emit the deck code **only if `--emit-deck`**. This
    is the "works without shared state" path and is backward-compatible with
    today's `tarot draw -n 3`. `--emit-deck` is the "share the state back" toggle.
- Deck code is a positional arg; also accept `-` to read from stdin, so
  `new | shuffle - | inspect -` pipelines cleanly.
- `--seed` is RFC 3339 (matching `iching cast`); default is now.
- Errors: invalid deck code (bad version / base64 / length) → clear usage error;
  `n > pile` → error.

### Stable `--json` shapes (the FFI contract for X)

The lib returns these as structs; the CLI serializes them; the X-round FFI
serializes the *same structs* over the C ABI.

`new` / `shuffle`:
```json
{ "deck_code": "v1.AGsD…", "count": 78, "arcana": null }
```

`draw`:
```json
{ "drawn": [{ "card": "The Tower", "index": 16, "suit": "Major Arcana", "reversed": true }],
  "deck_code": "v1.AGsD…", "remaining": 75 }
```

`inspect`:
```json
{ "count": 75,
  "pile": [{ "card": "The Tower", "index": 16, "suit": "Major Arcana", "reversed": true }, … ] }
```

## Testing

Implementation is test-driven (RED → GREEN per op).

- **Op unit tests (pure / deterministic):** same `(deck, seed)` → identical
  shuffle; `draw` pops from the top in order; no duplicates / no loss; arcana
  filters; `n > pile` errors.
- **Draw-by-position:** `At(k)` removes exactly the card at pile position `k`;
  the remaining pile equals the original minus that card, order otherwise intact;
  `At` out of range errors; **a card drawn `At` before a `shuffle` does not
  reappear in the shuffled pile** (the persistence-across-shuffle guarantee).
- **Codec round-trip:** `encode → decode` identity for full *and* partial decks;
  bad version / base64 / length rejected with a clear error; `inspect` agrees with
  the logical pile.
- **Shuffle fidelity (3b):** result is a true permutation of the input;
  deterministic from seed; a coarse **mixing assertion** (post-riffle position
  displacement is not visibly clumped) so the riffle count is *proven* to mix;
  reversals occur and are seed-deterministic; `no_reversals` yields an all-upright
  deck.
- **CLI integration** (extend existing `tests/cli.rs`): a fixed-seed
  `new → shuffle → draw → inspect` pipeline produces stable, reproducible output.
- Existing I Ching tests remain untouched.

## Out of scope (named, not built)

- **Card meanings / reference data** (`tarot card` lookup, keywords) — separate
  content deliverable; jianyi may gain it later, mirroring `iching hexagram`.
- **`cut` / `peek` / `reset` as standalone CLI gestures** — easy additions; `new`
  covers reset, `draw` covers peek. The app can request `cut` when the UI wants a
  distinct "cut the deck" beat.
- **Returning a card to the deck** — some practices reincorporate a drawn card and
  reshuffle. Here, drawn cards leave the pile permanently; a `return` / `insert` op
  is acknowledged but not built this round.
- **The table model** (zones, spread positions, face-up/down placement) — lives in
  the Swift app (future), per Principle 3.
- **The X round** — kairos-ffi → jianyi dependency, `kairos-divination` reshape,
  Swift divination page. Separate plan after Z is proven.
