#!/usr/bin/perl
use strict;
use warnings;

# Equivalent of: perl -ne 'print if /error/i' logfile.txt
# Expanded into a full script that reads from stdin or a file

my @lines = (
    "INFO: Server started",
    "ERROR: Connection failed",
    "WARN: Timeout detected",
    "error: disk full",
    "INFO: Request processed",
);

# Print lines matching /error/i
for (@lines) {
    print "$_\n" if /error/i;
}

# Equivalent of: perl -pe 's/foo/bar/g'
my @texts = ("foo and foo", "no match", "foofoo");
for (@texts) {
    s/foo/bar/g;
    print "$_\n";
}

# Equivalent of: perl -ane 'print "$F[0]\n"'
my @records = ("Alice 30 Engineer", "Bob 25 Designer");
for (@records) {
    my @fields = split /\s+/;
    print "$fields[0]\n";
}
