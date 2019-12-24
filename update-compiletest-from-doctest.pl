#!/usr/bin/perl -w

# This extracts the `compile_fail` tests from the doctests and puts
# them in a folder for `trybuild` to test.  `trybuild` lets us verify
# that the errors generated are what are expected.  This should be
# re-run after any change to the doctests, and the results should be
# checked in along with any new `.stderr` files.

die "Running in wrong directory" unless -f "../qcell/Cargo.toml";

$/ = undef;

for my $in (<src/doctest_*.rs>) {
    my $prefix = $in;
    $prefix =~ s|^.*doctest_(.*?)\.rs$|src/compiletest/$1|;

    die "Can't read $in" unless open IN, "<$in";
    my $data = <IN>;
    die "Can't read $in" unless close IN;

    my @tests = ();
    my %index = ();
    while (1) {
        last unless $data =~ s/```compile_fail(.*?)```//s;
        my $test = $1;

        $test =~ s|//!#?||sg;
        $test =~ s/^/   /mg;
        $test =~ s/^[ \t]*\n//s;  # Remove initial empty line
        $test =~ s/\s*$//s;

        my $testdata = <<"EOF";
extern crate qcell;

#[allow(warnings)]
fn main() {
$test
}
EOF
        push @tests, $testdata;
        $index{$testdata} = @tests-1;
    }

    for my $fnam (<$prefix-*.rs>) {
        die "Can't read $in" unless open IN, "<$fnam";
        my $data = <IN>;
        die "Can't read $in" unless close IN;

        # Keep the file if it is still identical to a test still
        # required, else delete it
        my $match = $index{$data};
        if (defined $match) {
            $tests[$match] = '';
        } else {
            print "Deleting $fnam ...\n";
            die "Can't delete file: $fnam" unless unlink $fnam;

            # Also delete error file if present
            my $stderr = $fnam;
            $stderr =~ s/\.rs$/.stderr/;
            unlink $stderr;
        }
    }

    # Output the rest in a sequential order
    my $cnt = 0;
    for $data (@tests) {
        next if $data eq '';

        while (1) {
            my $out = sprintf("%s-%02d.rs", $prefix, $cnt++);
            next if -f $out;
            print "Writing $out ...\n";
            die "Can't create file: $out" unless open OUT, ">$out";
            print OUT $data;
            die "Can't write file: $out" unless close OUT;
            last;
        }
    }
}
