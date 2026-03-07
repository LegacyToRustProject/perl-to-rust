#!/usr/bin/perl
use strict;
use warnings;

my @arr = (1, 2, 3, 4, 5);

# Scalar context: count elements
my $count = @arr;
print "Count: $count\n";

# List context: copy
my @copy = @arr;
push @copy, 6;
print "Original: @arr\n";
print "Copy: @copy\n";

# Scalar context in string interpolation
print "Array has " . scalar(@arr) . " elements\n";

# Wantarray
sub context_test {
    if (wantarray()) {
        return (1, 2, 3);
    } else {
        return "scalar";
    }
}

my @list_result = context_test();
my $scalar_result = context_test();
print "List: @list_result\n";
print "Scalar: $scalar_result\n";
