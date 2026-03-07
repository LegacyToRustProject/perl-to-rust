#!/usr/bin/perl
use strict;
use warnings;

my %colors = (
    red   => "#FF0000",
    green => "#00FF00",
    blue  => "#0000FF",
);

for my $name (sort keys %colors) {
    print "$name: $colors{$name}\n";
}

my $count = keys %colors;
print "Total colors: $count\n";

if (exists $colors{red}) {
    print "Red exists!\n";
}

delete $colors{green};
print "After delete: " . scalar(keys %colors) . " colors\n";
