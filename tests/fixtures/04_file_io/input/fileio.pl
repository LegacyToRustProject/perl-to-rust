#!/usr/bin/perl
use strict;
use warnings;

# Write to a file
open(my $fh, '>', '/tmp/perl_test.txt') or die "Cannot open: $!";
print $fh "Line 1\n";
print $fh "Line 2\n";
print $fh "Line 3\n";
close($fh);

# Read from the file
open(my $in, '<', '/tmp/perl_test.txt') or die "Cannot open: $!";
my $line_num = 0;
while (my $line = <$in>) {
    chomp $line;
    $line_num++;
    print "[$line_num] $line\n";
}
close($in);

# Clean up
unlink '/tmp/perl_test.txt';
