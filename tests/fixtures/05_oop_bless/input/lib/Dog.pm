package Dog;
use strict;
use warnings;

sub new {
    my ($class, %args) = @_;
    return bless {
        name  => $args{name},
        breed => $args{breed} || "Mixed",
    }, $class;
}

sub name {
    my ($self) = @_;
    return $self->{name};
}

sub breed {
    my ($self) = @_;
    return $self->{breed};
}

sub speak {
    my ($self) = @_;
    return $self->{name} . " says: Woof!";
}

1;
