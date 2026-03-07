#!/usr/bin/perl
use strict;
use warnings;
use JSON;
use MIME::Base64;

my $data = { name => "Alice", age => 30 };
my $json_str = encode_json($data);
print "JSON: $json_str\n";

my $decoded = decode_json($json_str);
print "Name: $decoded->{name}\n";

my $encoded = encode_base64("Hello, World!");
print "Base64: $encoded";
my $original = decode_base64($encoded);
print "Decoded: $original\n";
