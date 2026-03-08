#!/usr/bin/perl
use strict;
use warnings;
use lib 'lib';
use Animal;
use Dog;

# Create base class instance
my $cat = Animal->new(name => 'Whiskers', sound => 'Meow', age => 3);
$cat->speak;
$cat->describe;
print $cat->to_string, "\n";

print "\n";

# Create subclass instance
my $dog = Dog->new(name => 'Rex', age => 2);
$dog->speak;
$dog->describe;

$dog->learn_trick('sit');
$dog->learn_trick('fetch');
$dog->learn_trick('roll over');
$dog->perform;
$dog->describe;

printf "Rex's trick count: %d\n", $dog->trick_count;

# Polymorphism via array of Animal refs
my @animals = ($cat, $dog);
print "\nAll animals:\n";
for my $animal (@animals) {
    $animal->speak;
}
