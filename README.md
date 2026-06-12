# jianyi

簡儀 — a portable, deterministic divination calculator. Think pocket calculator or swiss army knife: `jianyi` is stripped to base functions — deterministic casting from a seed, structured text output, and a stable JSON schema for scripting — so a model can fold it into its own toolset as one primitive in a larger whole. No network connection, no account, no external data sources. (The name is Guo Shoujing's 簡儀 (1276): the observational instrument pared down to its portable working essence — the same gesture, in software.)

## Install

Build from source:

```
cargo build --release
```

Or install from crates.io (once published):

```
cargo install jianyi
```

## Usage

All commands accept `--json` as a global flag to emit JSON instead of pretty output.

### I Ching

**Cast**

Cast the I Ching. Omit `--question` for an open casting; omit `--seed` to use the current time.

```
jianyi iching cast
jianyi iching cast --question "What should I pay attention to?"
jianyi iching cast --seed 2026-06-02T11:00:00+00:00
```

**Hexagram lookup**

Look up a hexagram by number (1–64).

```
jianyi iching hexagram 1
jianyi iching hexagram 1 --full        # include all six line texts
jianyi iching hexagram 1 --glyph       # include the Unicode hexagram glyph
jianyi --json iching hexagram 1
```

**Trigram lookup**

Look up a trigram by English name, pinyin, Unicode symbol, or binary string.

```
jianyi iching trigram heaven
jianyi iching trigram qian
jianyi iching trigram ☰
jianyi iching trigram 111
```

**Compose**

Resolve a compose expression to a trigram, hexagram, or transition. Expressions combine trigram names or hexagram numbers using `over`, `under`, and `->`.

```
jianyi iching compose "heaven over earth"
jianyi iching compose "1 -> 2"
jianyi iching compose "thunder under wind"
```

### Tarot

Tarot is modeled as a **deck the caller holds**. jianyi stays stateless: each
command takes a deck code and prints a new one. A *deck code* (`v1.…`) is the
deck's order and orientations, compact enough to store and pass around.

**New deck** — a fresh deck in canonical order, all upright, unshuffled:

```
jianyi tarot new
jianyi tarot new --arcana major     # 22-card major arcana only
jianyi --json tarot new
```

**Shuffle** — riffle + cut; reversals enter via the cut. Deterministic with `--seed`:

```
jianyi tarot shuffle <deck-code>
jianyi tarot shuffle <deck-code> --seed 2026-06-02T11:00:00+00:00
jianyi tarot shuffle <deck-code> --no-reversals
```

**Draw** — the top `n` (`-n`) or a single fan-pick at a position (`--at`).
A stateful draw always returns the new deck code; omit the deck code for a
one-shot fresh draw (add `--emit-deck` to also get the resulting code):

```
jianyi tarot draw <deck-code> -n 3
jianyi tarot draw <deck-code> --at 39
jianyi tarot draw -n 1                       # one-shot
```

**Inspect** — expand a deck code to its ordered pile:

```
jianyi tarot inspect <deck-code>
```

Deck code can be `-` to read from stdin, so commands pipeline:

```
jianyi tarot new | sed -n 's/^  Code: //p' | xargs jianyi tarot shuffle
```

## Text and sources

Hexagram judgments, images, and line texts are an original English rendering translated from the source Chinese. They are not the Wilhelm/Baynes translation. Interpretive "readings" (per-hexagram and per-line guidance) are reserved slots in the data schema; a corpus will fill them in a future release.

Tarot output is card identity and orientation only — no interpretive text is included.

## License

MIT OR Apache-2.0
