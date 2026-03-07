#!/usr/bin/perl
use strict;
use warnings;
use Getopt::Long;

# Typical Getopt::Long usage pattern
my $verbose  = 0;
my $output   = "out.txt";
my $count    = 1;
my $rate     = 1.0;
my @files;
my %options;
my $debug    = 0;

GetOptions(
    "verbose!"          => \$verbose,       # boolean negatable flag
    "output|o=s"        => \$output,        # string with alias
    "count|n=i"         => \$count,         # integer
    "rate=f"            => \$rate,          # float
    "file=s@"           => \@files,         # array: multiple values
    "define=s%"         => \%options,       # hash: key=value pairs
    "debug+"            => \$debug,         # incremental counter
) or die "Error in command line arguments\n";

if ($verbose) {
    print "Verbose mode enabled\n";
    printf "Output: %s\n", $output;
    printf "Count: %d\n", $count;
    printf "Rate: %.2f\n", $rate;
    printf "Debug level: %d\n", $debug;
    printf "Files: %s\n", join(", ", @files) if @files;
    for my $k (sort keys %options) {
        printf "  %s = %s\n", $k, $options{$k};
    }
}

# Process remaining arguments (ARGV)
for my $arg (@ARGV) {
    printf "Positional: %s\n", $arg;
}
