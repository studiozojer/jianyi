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

**Draw**

Draw one or more cards from the standard 78-card deck. Deterministic with `--seed`.

```
jianyi tarot draw
jianyi tarot draw -n 3
jianyi tarot draw -n 5 --seed 2026-06-02T11:00:00+00:00
jianyi tarot draw --arcana major
jianyi tarot draw --arcana minor -n 10
jianyi tarot draw --no-reversals
jianyi --json tarot draw -n 5 --seed 2026-06-02T11:00:00+00:00
```

`--arcana` accepts `major` (22 cards) or `minor` (56 cards). `--no-reversals` draws all cards upright without affecting which cards appear. `-n` defaults to 1.

## Text and sources

Hexagram judgments, images, and line texts are an original English rendering translated from the source Chinese. They are not the Wilhelm/Baynes translation. Interpretive "readings" (per-hexagram and per-line guidance) are reserved slots in the data schema; a corpus will fill them in a future release.

Tarot output is card identity and orientation only — no interpretive text is included.

## License

MIT OR Apache-2.0
