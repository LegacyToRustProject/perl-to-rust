#!/usr/bin/perl
use strict;
use warnings;

my $line = "2024-03-15";

if ($line =~ /^(\d{4})-(\d{2})-(\d{2})$/) {
    my ($year, $month, $day) = ($1, $2, $3);
    print "Year: $year, Month: $month, Day: $day\n";
}

my $text = "foo bar foo baz foo";
$text =~ s/foo/qux/g;
print "$text\n";
