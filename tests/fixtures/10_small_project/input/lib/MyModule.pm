package MyModule;
use strict;
use warnings;

sub new {
    my ($class, %args) = @_;
    return bless { items => [] }, $class;
}

sub add_item {
    my ($self, $item) = @_;
    push @{$self->{items}}, $item;
}

sub count {
    my ($self) = @_;
    return scalar @{$self->{items}};
}

sub all_items {
    my ($self) = @_;
    return @{$self->{items}};
}

sub to_string {
    my ($self) = @_;
    return join(", ", @{$self->{items}});
}

1;
