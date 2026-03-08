#!/usr/bin/perl
use strict;
use warnings;

# Pure functional implementations of List::Util core functions

sub sum {
    my @nums = @_;
    my $total = 0;
    $total += $_ for @nums;
    return $total;
}

sub min {
    my $m = $_[0];
    for (@_) { $m = $_ if $_ < $m }
    return $m;
}

sub max {
    my $m = $_[0];
    for (@_) { $m = $_ if $_ > $m }
    return $m;
}

sub first (&@) {
    my $code = shift;
    for (@_) {
        return $_ if $code->($_);
    }
    return undef;
}

sub any (&@) {
    my $code = shift;
    return !!grep { $code->($_) } @_;
}

sub all (&@) {
    my $code = shift;
    return !grep { !$code->($_) } @_;
}

my @nums = (1..10);
printf "sum: %d\n", sum(@nums);
printf "min: %d\n", min(@nums);
printf "max: %d\n", max(@nums);
printf "first > 5: %d\n", first { $_ > 5 } @nums;
printf "any > 8: %s\n", (any { $_ > 8 } @nums) ? "true" : "false";
printf "all > 0: %s\n", (all { $_ > 0 } @nums) ? "true" : "false";
