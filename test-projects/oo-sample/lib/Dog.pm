package Dog;

use strict;
use warnings;
use parent 'Animal';   # inheritance: Dog ISA Animal

# SUPER:: call to parent constructor
sub new {
    my ($class, %args) = @_;
    $args{sound} //= 'Woof';
    my $self = $class->SUPER::new(%args);   # SUPER:: dispatch
    $self->{tricks} = [];
    return $self;
}

# New method: adds to tricks list
sub learn_trick {
    my ($self, $trick) = @_;
    push @{$self->{tricks}}, $trick;
}

# New method: uses tricks array
sub perform {
    my ($self) = @_;
    if (@{$self->{tricks}}) {
        printf "%s performs: %s\n", $self->name, join(', ', @{$self->{tricks}});
    } else {
        printf "%s knows no tricks yet\n", $self->name;
    }
}

# Override parent method + call SUPER::
sub describe {
    my ($self) = @_;
    $self->SUPER::describe();     # call parent's describe
    printf "Tricks learned: %d\n", scalar @{$self->{tricks}};
}

sub trick_count { return scalar @{$_[0]->{tricks}} }

1;
