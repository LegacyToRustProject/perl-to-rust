# perl-to-rust

**AI-powered Perl → Rust conversion agent.**

## Why Perl

- Perl powers critical infrastructure in telecom, finance, and bioinformatics
- Perl 5 codebases of 100K+ lines are common in enterprises, many unmaintained
- "Only Perl can parse Perl" — the language is notoriously context-sensitive
- Fewer developers every year. Legacy Perl code is becoming unmaintainable.
- AI doesn't care about Perl's syntactic complexity. It reads the intent, not the syntax.

## How It Works

```
Perl project (source + running instance)
    ↓ 1. Parse & analyze (modules, CPAN deps, regex patterns)
    ↓ 2. AI converts each module to Rust
    ↓ 3. cargo check (must compile)
    ↓ 4. Run both Perl & Rust with same inputs, compare outputs
    ↓ 5. Diff? → AI fixes → goto 3
    ↓ 6. Repeat until all outputs match
Verified Rust binary
```

## Version Compatibility

Perl 5 is the only target. Perl 6 (Raku) is a different language entirely.

| Perl Version | Priority | Notes |
|--------------|----------|-------|
| 5.26 - 5.40 | **First** | Modern Perl 5. Current baseline. |
| 5.16 - 5.24 | Second | Stable enterprise versions. Wide deployment. |
| 5.10 - 5.14 | Third | say, given/when, smart match. Legacy boundary. |
| 5.8 | Fourth | Pre-modern. Unicode support era. Still in production. |
| 5.6 | Fifth | Ancient but found in telecom systems. |

Older Perl is actually harder than newer Perl — not because of features, but because of coding style. Perl 5.6-era code tends to use extreme shortcuts, implicit variables ($\_), and dense regex. AI handles this better than humans because it doesn't need to "read" the syntax — it understands the behavior.

Auto-detection: `perl-to-rust analyze` detects version from `use v5.xx`, module requirements, and syntax patterns.

## Key Challenges

| Perl Feature | Conversion Strategy |
|---|---|
| Dynamic typing ($, @, %) | Rust enums + type inference |
| Regex as first-class | `regex` crate |
| CPAN modules | Map to Rust crate equivalents |
| Context sensitivity (scalar/list) | Explicit Rust types |
| Autovivification | Builder patterns |
| Tied variables | Custom trait implementations |
| One-liners / golf | Expand to readable Rust |

## Target Industries

- Telecom (call routing, billing systems)
- Finance (risk calculation, reporting)
- Bioinformatics (sequence analysis pipelines)
- System administration (legacy automation scripts)

## Status

**Concept.** Architecture design in progress.

## Part of [LegacyToRust Project](https://github.com/LegacyToRustProject)

## License

MIT
