# libperfusion

A pure Rust port of [libinjection](https://github.com/libinjection/libinjection),
which detects SQL injection and cross-site scripting by tokenizing the input
rather than matching regular expressions against it.

```rust
let fingerprint = libperfusion::sqli(b"1' OR '1'='1").unwrap();
assert_eq!(fingerprint, "s&sos");

assert!(libperfusion::xss(b"<script>alert('xss')</script>"));
```

## Status

The port is complete, and not published yet. It follows libinjection 4.0.0
(`d88a8f8`): the SQL tokenizer, folding, fingerprints and false-positive
checks, the HTML5 tokenizer, and the XSS checks. Its one dependency is
[memchr](https://crates.io/crates/memchr), for the byte searches. It has no
`unsafe` code of its own, and needs neither `std` nor an allocator. With the
`std` feature, which is off by default, memchr finds out at run time whether
an x86-64 processor has AVX2 and searches with it: [Performance](#performance)
has what that changes.

The name is a play on the original's: `libinjection` and `libinjection-rs` are
taken on crates.io, by bindings to the C library.

It is meant to give the same answer as the C library, as built on x86-64, on
every input, but for one bug it fixes: a 0xFF byte where an attribute's name
or value would start, which the C library takes for the end of the input,
hiding the rest of the tag from the XSS checks. `<img \xff onerror=alert(1)>`
is flagged here. Upstream has it as
[issue 92](https://github.com/libinjection/libinjection/issues/92), still
open.

Two things check that:

- libinjection's own tests: its 584 expected-output files and its sample
  corpora.
- A differential test, which builds the C library with that fix applied and
  compares it with the port on upstream's corpus and on fuzzed inputs: tokens,
  folded tokens, fingerprints and verdicts, in every context. A default run
  compares some 940,000 inputs. The longest so far compared 20 million and
  found no difference.

## Testing

`cargo test` runs everything. The differential tests need a C compiler (`CC`,
`CFLAGS`); the top of `tests/differential.rs` describes the longer run, and
the one with the C side under sanitizers.

`tests/upstream` holds libinjection's sources, tests and samples.
`tools/vendor_upstream.sh` refreshes it from a checkout and regenerates the
tables in `src` that are derived from it.

## Performance

`cargo bench` times the port next to the C library on the same inputs: those
of upstream's two speed tests, and its sample corpora. The SQL corpora go
through the XSS detection as well, as inputs that are not XSS: those cost the
most, since nothing ends the search early. All of these are short, a request
parameter rather than a request body, so three inputs made up for the
benchmark follow them, each at 4 KB and at 1 MB: prose, a JSON document and a
base64 blob, none of them an attack. On those, a search for a quote or for a
`<` can run to the end of the input.

Below is the time per input, in nanoseconds, of the C library, of the port as
it is by default, and of the port with its `std` feature. The times are
medians over five runs of `cargo bench` and five of `cargo bench --features
std`, pinned to a core with `taskset`, on an AMD Ryzen 9 7900, with
libinjection built by GCC 16.2 at `-O3` and the port by Rust 1.99.

| Workload                    | Inputs | Average bytes | libinjection |       Port | Port / C | With `std` | With `std` / C |
| --------------------------- | -----: | ------------: | -----------: | ---------: | -------: | ---------: | -------------: |
| sqli: upstream's speed test |      8 |            28 |        426.1 |      220.4 |    0.52x |      217.5 |          0.51x |
| sqli: attack samples        | 85,802 |           141 |       1068.2 |      562.5 |    0.53x |      570.2 |          0.53x |
| sqli: benign samples        |    423 |            34 |        609.7 |      332.3 |    0.55x |      334.1 |          0.55x |
| xss: upstream's speed test  |     26 |            30 |        233.8 |       95.7 |    0.41x |      104.6 |          0.45x |
| xss: attack samples         | 81,417 |            74 |        167.1 |       60.5 |    0.36x |       56.3 |          0.34x |
| xss: sqli attack samples    | 85,802 |           141 |       1566.5 |      673.6 |    0.43x |      725.2 |          0.46x |
| xss: benign samples         |    423 |            34 |        385.5 |      192.9 |    0.50x |      214.6 |          0.56x |
| sqli: 4 KB of prose         |      1 |         4,096 |        641.5 |      424.0 |    0.66x |      371.3 |          0.58x |
| sqli: 4 KB of JSON          |      1 |         4,096 |        416.7 |      321.5 |    0.77x |      288.2 |          0.69x |
| sqli: 4 KB of base64        |      1 |         4,096 |       8991.0 |     3845.5 |    0.43x |     3627.9 |          0.40x |
| xss: 4 KB of prose          |      1 |         4,096 |      54156.6 |    22735.7 |    0.42x |    26869.6 |          0.50x |
| xss: 4 KB of JSON           |      1 |         4,096 |      29500.4 |     5993.2 |    0.20x |     4251.7 |          0.14x |
| xss: 4 KB of base64         |      1 |         4,096 |      14010.2 |     3308.2 |    0.24x |     2381.7 |          0.17x |
| sqli: 1 MB of prose         |      1 |     1,048,576 |      11881.0 |    16098.5 |    1.35x |    11516.3 |          0.97x |
| sqli: 1 MB of JSON          |      1 |     1,048,576 |      11501.0 |    15998.8 |    1.39x |    11262.6 |          0.98x |
| sqli: 1 MB of base64        |      1 |     1,048,576 |      31048.0 |    35032.1 |    1.13x |    25885.3 |          0.83x |
| xss: 1 MB of prose          |      1 |     1,048,576 |   15453466.5 |  6838918.0 |    0.44x |  7756378.0 |          0.50x |
| xss: 1 MB of JSON           |      1 |     1,048,576 |    7474922.5 |  1501109.4 |    0.20x |  1281124.4 |          0.17x |
| xss: 1 MB of base64         |      1 |     1,048,576 |    3856476.7 |  1136360.1 |    0.29x |   896103.9 |          0.23x |

What `std` buys is AVX2 in the searches for a byte, and it is worth having
where those searches are most of the work: in the SQL detection, on long
inputs. That one reads only the start of a long input and searches the rest
for quotes, so that at 1 MB it is behind the C library without `std`, and
level with it or ahead with it. At 4 KB the gain is about a tenth, and on
short inputs there is none. The XSS detection reads all of its input, and the
searches are a small part of that: its rows differ between the two builds,
either way, by no more than they can between two builds of the same code.

Small differences are noise in any case. Rebuilding alone can move a time by
a tenth, an XSS row by a fifth, and the XSS rows on JSON and base64 by half:
where the compiler happens to put the code matters that much. Pinning does
not change that, nor the port's times. It does make the C library's steadier,
and up to 9% higher on the XSS rows, for a reason not found: without it, the
port's lead there is that much smaller.

## Missing quality of life APIs

The public API covers detection and, for each detector, the dialect or HTML
context that matched. Richer analysis found in comparable libraries is not yet
exposed:

- **Construct flags.** A bitset saying *which* constructs appeared in the
  match: for SQLi, things like union, tautology, stacked queries, comments,
  and functions; for XSS, script tags, event handlers, inline SVG, and URL
  schemes. All the data is in `State.tokens` after folding — it is a post-fold
  pass with no change to the hot path.

- **Verdict confidence.** A coarser classification than `Option<Fingerprint>`:
  something like `Decisive / Suspicious / Inconclusive` to let callers tune
  their response. Requires defining which fingerprints or false-positive paths
  map to each tier — new logic with no existing analogue in libinjection itself.

- **Evidence spans.** Byte offsets into the original input marking the tokens
  that made the call. Requires storing a start offset in `Token`, which the
  lexer does not currently track. Adding it is one write per token in the hot
  path and would need a fixed-size representation (no allocator) for the result.

- **SQLi quote context.** Which quote mode — raw input, single-quoted, or
  double-quoted — the matching parse was in. `sqli_with_dialect` returns the
  SQL dialect but drops this. It is already determined by the control flow in
  `detect_with_dialect` and costs nothing to surface.

- **Run metadata flags.** Bits describing the detection run itself: whether the
  input was truncated by a limit, whether multiple quote contexts were tried,
  whether the result came from a re-parse as MySQL. These fall out naturally
  from the existing control flow and pair well with the `_with_limit` variants.

- **`scan_prefix`.** A helper that truncates an input to a byte budget while
  respecting a token or character boundary, avoiding a cut mid-token. Useful
  for callers that want to bound scan cost without the `_with_limit` variants'
  hard slice.

## Open issues and decisions

- **`char` signedness.** The port behaves like the C library built with a
  signed `char`. Where `char` is unsigned, as on ARM Linux, the C library
  itself answers differently on some inputs with bytes above 127.
- **Manifest.** It declares no repository.
- **Public API.** Only `sqli` and `xss` are public. The tokenizers are behind
  `internals`, a hidden feature with no stability promise that the tests use.
  Upstream exposes them, along with a hook to replace the keyword lookup,
  which was not ported.
- **Copyright.** `LICENSE` carries libinjection's notice, as its license
  requires. No line was added for this port.
- **Minimum Rust version.** None is declared or tested.
- **Mutation testing** was done once, by hand: of 14 deliberately broken
  variants of the port, the tests caught 11, and the other 3 appear to be
  equivalent. None of that is in the repository.
- **Following upstream.** Only the tables are generated. Changes to the C code
  have to be ported by hand, and a fix that upstream makes dropped from `FIXES`
  in `tests/common/mod.rs`.

## License

BSD 3-Clause, the same as libinjection: see `LICENSE`.
