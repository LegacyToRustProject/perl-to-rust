#!/usr/bin/perl
use strict;
use warnings;
use lib 'lib';
use MyModule;

my $list = MyModule->new();
$list->add_item("alpha");
$list->add_item("beta");
$list->add_item("gamma");

print "Count: " . $list->count() . "\n";
print "Items: " . $list->to_string() . "\n";

for my $item ($list->all_items()) {
    print "  - $item\n";
}
