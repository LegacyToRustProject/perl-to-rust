#!/usr/bin/perl
use strict;
use warnings;

my @items = ("apple", "banana", "cherry", "date");

# $_ in for loop
for (@items) {
    print "$_\n" if /^[abc]/;
}

# $_ in map
my @upper = map { uc($_) } @items;
print join(", ", @upper) . "\n";

# $_ in grep
my @long = grep { length($_) > 5 } @items;
print "Long items: " . join(", ", @long) . "\n";
