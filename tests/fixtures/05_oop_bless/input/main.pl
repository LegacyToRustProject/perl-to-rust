#!/usr/bin/perl
use strict;
use warnings;
use lib 'lib';
use Dog;

my $dog = Dog->new(name => "Rex", breed => "Labrador");
print $dog->name() . "\n";
print $dog->breed() . "\n";
print $dog->speak() . "\n";

my $mutt = Dog->new(name => "Buddy");
print $mutt->breed() . "\n";
