#!/usr/bin/perl
use strict;
use warnings;
use CGI;
use CGI::Session;
use DBI;

my $q = CGI->new;

# Route dispatch
my $action = $q->param('action') || 'index';

if ($action eq 'index') {
    # Endpoint 1: Hello page
    my $name = $q->param('name') || 'World';
    print $q->header('text/html');
    print "<h1>Hello, $name!</h1>";
    print "<form method='post' action='?action=greet'>";
    print "  Name: <input name='name' value='$name'>";
    print "  <button type='submit'>Greet</button>";
    print "</form>";

} elsif ($action eq 'greet') {
    # Endpoint 2: POST form handler
    my $name = $q->param('name') || 'Anonymous';
    print $q->header(-type => 'text/html', -charset => 'UTF-8');
    print "<h2>Greetings, $name!</h2>";
    print "<a href='?action=index'>Back</a>";

} elsif ($action eq 'users') {
    # Endpoint 3: JSON API (list users from DB)
    my $dbh = DBI->connect($ENV{DATABASE_URL}, '', '', { RaiseError => 1 });
    my $sth = $dbh->prepare("SELECT id, name, email FROM users ORDER BY id LIMIT 20");
    $sth->execute();
    my @users;
    while (my $row = $sth->fetchrow_hashref()) {
        push @users, { id => $row->{id}, name => $row->{name}, email => $row->{email} };
    }
    print $q->header('application/json');
    # Would use JSON module in real code
    print "{\"users\":[]}";

} elsif ($action eq 'user') {
    # Endpoint 4: Single user lookup
    my $id = $q->param('id');
    unless ($id && $id =~ /^\d+$/) {
        print $q->header(-status => '400 Bad Request', -type => 'text/plain');
        print "Invalid user ID";
        exit;
    }
    my $dbh = DBI->connect($ENV{DATABASE_URL}, '', '', { RaiseError => 1 });
    my $user = $dbh->selectrow_hashref(
        "SELECT id, name, email FROM users WHERE id = ?",
        undef, $id
    );
    if ($user) {
        print $q->header('application/json');
        printf '{"id":%d,"name":"%s","email":"%s"}', $user->{id}, $user->{name}, $user->{email};
    } else {
        print $q->header(-status => '404 Not Found', -type => 'text/plain');
        print "User not found";
    }

} elsif ($action eq 'redirect') {
    # Endpoint 5: Redirect
    print $q->redirect('?action=index');

} else {
    print $q->header(-status => '404 Not Found', -type => 'text/plain');
    print "Unknown action: $action";
}
