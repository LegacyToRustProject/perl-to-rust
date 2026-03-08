package Animal;

use strict;
use warnings;

# Perl bless-based OOP: constructor
sub new {
    my ($class, %args) = @_;
    return bless {
        name  => $args{name}  // 'Unknown',
        sound => $args{sound} // 'silence',
        _age  => $args{age}   // 0,
    }, $class;
}

# Read-only accessors
sub name  { return $_[0]->{name}  }
sub sound { return $_[0]->{sound} }

# Read-write accessor: $obj->age()  or  $obj->age(5)
sub age {
    my ($self, $val) = @_;
    $self->{_age} = $val if @_ > 1;
    return $self->{_age};
}

# Instance method using $self
sub speak {
    my ($self) = @_;
    printf "%s says %s\n", $self->name, $self->sound;
}

sub describe {
    my ($self) = @_;
    printf "I am %s, age %d\n", $self->name, $self->age;
}

# String overloading equivalent
sub to_string {
    my ($self) = @_;
    return sprintf "Animal(%s)", $self->name;
}

1;
