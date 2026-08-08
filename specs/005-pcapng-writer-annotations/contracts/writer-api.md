# Contract: pcapng Writer API

**Slice**: S06

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

This is the surface `fragcap-sink` exposes after this slice, and the on-disk
contract the written file honors. Two audiences depend on it: S07 and S08 as
callers, and every pcapng reader in existence as a consumer of the output.

## The Rust surface

```rust
pub struct PcapngWriter<W: Write> { /* ... */ }

impl<W: Write> PcapngWriter<W> {
    /// Begin a capture, writing the Section Header Block immediately.
    pub fn new(output: W) -> Result<Self, WriteError>;

    /// Declare an interface and return the identifier assigned to it.
    /// Identifiers are assigned in declaration order from zero.
    pub fn declare_interface(
        &mut self,
        decl: &InterfaceDeclaration,
    ) -> Result<u32, WriteError>;
}

impl<W: Write + Send> Sink for PcapngWriter<W> {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError>;
    fn flush(&mut self) -> Result<(), SinkError>;
    fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError>;
}
```

The annotation surface, separate so section 13.5's writer reuses the
derivation rather than restating the rules (FR-025):

```rust
pub struct Annotation { /* fields per data-model.md */ }

impl Annotation {
    /// Derive from a packet. This is the shared part: which keys are present
    /// and what they carry. Carries no pcapng knowledge.
    pub fn from_packet(
        packet: &CapturedPacket,
        interface: Option<&str>,
    ) -> Self;

    /// Render to the section 13.3 grammar.
    pub fn encode(&self) -> String;

    /// Parse the section 13.3 grammar.
    pub fn decode(s: &str) -> Result<Self, AnnotationError>;
}
```

### Contract obligations

| Obligation | Requirement |
| --- | --- |
| `new` writes the Section Header Block before returning | FR-001 |
| `declare_interface` returns identifier zero, once | FR-006 |
| A second `declare_interface` is an error | FR-006a |
| `write` against an undeclared identifier is an error | FR-033 |
| `write` never drops or skips a packet | FR-030 |
| `finish` writes one Interface Statistics Block per interface | FR-008 |
| `finish` consumes the writer | Plan D-4 |
| No method reads a clock | FR-008a |
| `encode` then `decode` returns the original value | FR-024 |
| `from_packet` decides key presence, not the caller | FR-025 |

### Error behavior

`WriteError` maps into core's `SinkError` at the trait boundary. The named
conditions this slice introduces:

| Condition | Behavior |
| --- | --- |
| Undeclared interface identifier | Error, no bytes written for that packet |
| A second interface declaration | Error, no bytes written |
| Timestamp predates the Unix epoch | Error, no bytes written for that packet |
| Annotation exceeds the 16-bit option length | Error, never silently truncated |
| Underlying writer fails | Error propagated, blocks already written stay valid |

No condition in this table is a discard. A packet that cannot be written is
reported to the caller, which is what lets the S08 pipeline count it. Silently
skipping any of them would be the P-4 defect this project treats as
unrecoverable.

## The on-disk contract

What a reader is entitled to assume about a file fragcap wrote.

### Structure

- Little-endian throughout, declared by the Section Header Block byte-order
  magic. Independent of the host that wrote it.
- Block order: Section Header first; every Interface Description Block before
  any Enhanced Packet Block that references it; Interface Statistics Blocks
  last.
- Every block's trailing total length equals its leading total length.
- Every block body and every option is padded to a 32-bit boundary, and no
  declared length counts its padding.
- Every option list ends with `opt_endofopt`.

### Declared values

| Where | What |
| --- | --- |
| `shb_userappl` | `fragcap/0.1.0` |
| Section Header `opt_comment` | `fragcap:profile=0.1.0` |
| `if_tsresol` | 6, meaning microseconds |
| `if_name` | The interface name as declared |
| Interface Statistics `opt_comment` | `fragcap:` counters with no standard field |

### The annotation grammar

```text
annotation  = "fragcap:" [ pair *( ";" pair ) ]
pair        = key "=" value
key         = 1*( %x61-7A / "_" )          ; lowercase ASCII
value       = *( unreserved / pct-encoded )
pct-encoded = "%" 2HEXDIG                  ; uppercase on output
```

Keys, in the order they appear:

| Key | Presence | Value |
| --- | --- | --- |
| `pid` | When attributed | Decimal process identifier |
| `proc` | When attributed | Executable file name |
| `role` | When a role is present | Profile role name |
| `stage` | When a stage is present | Profile stage identifier |
| `dir` | Always | `in`, `out`, `local`, or `unknown` |
| `attr` | Always | `live`, `retained`, or `none` |
| `iface` | When multi-interface | Capture interface name. Defined and round-tripped; not produced while the writer records one interface |

Encoded characters: `;`, `=`, `%`, every code point below 0x20, and 0x7F.

`attr=none` implies `pid`, `proc`, `role`, and `stage` are all absent.

### Compatibility guarantee

Every file fragcap writes is a valid pcapng file. A reader that has never heard
of fragcap opens it, reads every packet with correct timestamps and lengths,
and displays the annotation as an ordinary comment. This is verified against
Wireshark 4.6.3; see [research.md](../research.md) R-1 and R-6.

The `.fcapng` extension names this profile. It is not a distinct format, and
nothing in the file requires a plugin to read.

## Stability

The annotation grammar is versioned by the `fragcap:profile=` value in the
Section Header Block comment. A change to which keys exist, what they mean, or
how values are encoded bumps that version. Adding a key that consumers may
ignore does not.

The Rust surface is not stable in v0.1.0. `Annotation` is expected to move or
be re-exported when S07 arrives with a second consumer; the derivation rules it
encodes are the stable part, not its path.
