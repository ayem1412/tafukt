<div align="center">

# ⵜⴰⴼⵓⴽⵜ

### tafukt

**A BitTorrent library written from scratch in Rust.**

[![status](https://img.shields.io/badge/status-early%20development-orange)](#roadmap)
[![rust](https://img.shields.io/badge/rust-2024%20edition-000000?logo=rust)](https://www.rust-lang.org/)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

</div>

---

## The name

**Tafukt** (Tifinagh: **ⵜⴰⴼⵓⴽⵜ**) literally means _the sun_. She is a solar goddess, wife of the moon god Ayyur, mother of the stars. The root behind the word — `F(W)` — carries the ideas of _light_, _day_, and _fire_.

The name fits the library twice over. A torrent swarm is thousands of scattered strangers, each holding a fragment of the same thing, and the job of a client is to gather those fragments into something whole and legible. And like the sun, the library is meant to be something everything else can be built on top of — quiet infrastructure that other people's projects revolve around.

## What it is

tafukt is a **general-purpose BitTorrent library**, not an application. It parses torrents, speaks the peer wire protocol, finds peers, verifies data, and writes it to disk — and it hands all of that to you through a small API instead of deciding what your program should look like.

It is being written from scratch: no existing BitTorrent crate underneath, no protocol code borrowed. The point is both to have a library that does exactly what's needed and to understand every layer of it.

**Design principles:**

- **Layers stay separate.** `bencode` knows nothing about torrents. `bittorrent` knows nothing about networking. Dependencies point one direction only.
- **Nothing panics on hostile input.** Every byte the library reads may come from a stranger. Malformed data returns an error; it never crashes the process.
- **Few dependencies.** Only where the alternative is reimplementing a standard cryptographic or encoding algorithm.
- **The library decides nothing for you.** No global state, no logging framework, no opinions about your runtime beyond async I/O.

## Structure

```
tafukt/
└── crates/
    ├── bencode/       the .torrent data format — pure, no networking
    ├── bittorrent/    protocol types, metainfo, magnet links, piece math
```

`bencode` and `bittorrent` are pure libraries with no async runtime and no sockets. That constraint is what makes them reusable on their own.

---

## Building

Requires a recent Rust toolchain — the workspace uses the 2024 edition, so Rust 1.85 or newer.

```bash
git clone https://github.com/ayem1412/tafukt
cd tafukt
cargo build
```

Run the tests:

```bash
cargo test
cargo test -p bencode        # one crate only
```

<!-- Run the command-line downloader: -->
<!---->
<!-- ```bash -->
<!-- cargo run --release -p cli -->
<!-- ``` -->

Benchmarks and examples live in each crate. Always run these with `--release` — debug builds are an order of magnitude slower and the numbers are meaningless:

```bash
cargo run --release -p bencode --example bench
```

Before committing:

```bash
cargo fmt
cargo clippy --all-targets
```

### Test fixtures

Some tests read real `.torrent` files, which aren't committed to the repo. Drop your own into `crates/bencode/torrents/` — any Linux distribution image works, and a multi-file torrent with nested folders is worth having too, since it exercises paths that single-file torrents never reach. Tests that can't find a fixture skip rather than fail.

## Roadmap

**18 of 63 done — 29%**

`███████░░░░░░░░░░░░░░░░░`

### Foundations

<details open>
<summary><b>Bencode</b> — the format torrent files are written in · <b>10/16</b></summary>

<br>

- [x] Byte cursor with bounds-checked reads
- [x] Decoder — integers, byte strings, lists, dictionaries
- [x] Recursion depth limit (hostile input can't overflow the stack)
- [x] Strict validation — leading zeros, negative zero, trailing data
- [x] Byte-span tracking for the `info` dictionary
- [x] Human-readable `Display` for inspecting decoded values
- [x] Encoder — integers, byte strings, lists, dictionaries
- [x] Sorted-key output (required for round-trip fidelity)
- [ ] Decoder unit tests
- [ ] Encoder unit tests
- [ ] Round-trip tests on small inputs
- [ ] Round-trip test on a real torrent
- [ ] Truncation fuzzing — every prefix of a valid file
- [ ] Random-input fuzzing
- [x] Benchmark example
- [ ] Public API documentation

</details>

<details open>
<summary><b>Torrent files</b> — reading <code>.torrent</code> metadata · <b>8/12</b></summary>

<br>

- [x] Typed field accessors on decoded values
- [x] Announce URL
- [x] Announce list (BEP 12 tracker tiers)
- [x] Name, piece length, private flag
- [x] Piece hash list
- [x] Single-file layout
- [x] Multi-file layout, flattened with cumulative offsets
- [x] Path traversal rejection
- [ ] Piece-count validation against total size
- [ ] Tested against a published infohash
- [ ] Tested against a nested multi-file torrent
- [ ] Metainfo unit tests

</details>

<details open>
<summary><b>Magnet links</b> · <b>5/6</b></summary>

<br>

- [x] Parameter splitting and percent-decoding
- [x] Hex infohash (40 characters)
- [x] Base32 infohash (32 characters)
- [x] Display name and tracker list
- [x] Peer address hints (`x.pe`)
- [ ] Tests against known links

</details>

### Core protocol

<details>
<summary><b>Bitfield</b> — tracking which pieces exist · <b>0/5</b></summary>

<br>

- [ ] Packed bit storage with piece count
- [ ] Get and set (MSB-first bit order)
- [ ] Construction from peer-supplied bytes
- [ ] Spare-bit validation
- [ ] Set-bit counting

</details>

<details>
<summary><b>Piece math</b> — mapping bytes to pieces to files · <b>0/5</b></summary>

<br>

- [ ] `PieceMap` over the file list
- [ ] Byte position → piece index
- [ ] Piece index → byte range (with short final piece)
- [ ] Piece → the files and offsets it spans
- [ ] Tests against hand-computed cases

</details>

<details>
<summary><b>Peer connections</b> · <b>0/5</b></summary>

<br>

- [ ] TCP handshake with infohash verification
- [ ] Length-prefixed message framing
- [ ] Choke / interest state tracking
- [ ] Block request pipelining and piece assembly
- [ ] SHA-1 piece verification

</details>

<details>
<summary><b>Storage</b> · <b>0/3</b></summary>

<br>

- [ ] Sparse writes across file boundaries
- [ ] Partial reads from incomplete downloads
- [ ] Resume data — save and verify on load

</details>

### Finding peers

<details>
<summary><b>Trackers</b> · <b>0/2</b></summary>

<br>

- [ ] HTTP tracker client
- [ ] UDP tracker client

</details>

<details>
<summary><b>DHT</b> — peer discovery with no tracker · <b>0/4</b></summary>

<br>

- [ ] Kademlia routing table (XOR distance, k-buckets)
- [ ] KRPC messages over UDP
- [ ] Iterative lookup and bootstrap
- [ ] `ut_metadata` (BEP 9) — fetch torrent info from peers

</details>

### Running a download

<details>
<summary><b>Scheduling</b> · <b>0/5</b></summary>

<br>

- [ ] Extension protocol handshake (BEP 10)
- [ ] Availability tracking and rarest-first selection
- [ ] Peer pool management
- [ ] Choke algorithm and seeding
- [ ] Progress and rate reporting

</details>

<details>
<summary><b>Finishing touches</b> · <b>0/6</b></summary>

<br>

- [ ] Peer exchange (PEX)
- [ ] Endgame mode
- [ ] Rate limiting
- [ ] Multiple simultaneous torrents
- [ ] Sequential download mode (for streaming while downloading)
- [ ] End-to-end test against a large public torrent

</details>

---

## Status

Early. The parsing layer works — real torrent files and magnet links go in, structured data comes out — but nothing has touched the network yet. Not usable as a dependency.

The current milestone is verifying a computed infohash against a published one, which is the point at which the whole bencode and metainfo layer is proven correct.

## Specifications

Implemented against the official BEPs:

|                                                         |                         |
| ------------------------------------------------------- | ----------------------- |
| [BEP 3](https://www.bittorrent.org/beps/bep_0003.html)  | The BitTorrent Protocol |
| [BEP 5](https://www.bittorrent.org/beps/bep_0005.html)  | DHT Protocol            |
| [BEP 9](https://www.bittorrent.org/beps/bep_0009.html)  | Metadata Exchange       |
| [BEP 10](https://www.bittorrent.org/beps/bep_0010.html) | Extension Protocol      |
| [BEP 11](https://www.bittorrent.org/beps/bep_0011.html) | Peer Exchange           |
| [BEP 12](https://www.bittorrent.org/beps/bep_0012.html) | Multitracker Metadata   |
| [BEP 15](https://www.bittorrent.org/beps/bep_0015.html) | UDP Tracker Protocol    |
| [BEP 27](https://www.bittorrent.org/beps/bep_0027.html) | Private Torrents        |

## License

MIT.

---

<div align="center">
<sub><b>ⵜⴰⴼⵓⴽⵜ</b> · tafukt</sub>
</div>
