# Telecom Profile

Conversion profile for telecommunications Perl codebases.

## Common patterns:
- Socket programming (IO::Socket) → std::net / tokio
- Binary protocol parsing → nom / bytes
- ASN.1 handling → rasn
- SNMP (Net::SNMP) → snmp-rs
- CDR (Call Detail Record) processing → custom structs + CSV
- High-throughput message queues → tokio channels
