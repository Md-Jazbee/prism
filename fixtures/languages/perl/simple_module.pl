# Minimal Perl fixture for T1 golden conformance.
#
# Covers: package, use imports, sub defs, same-file call, unresolved call.

package Sample;
use strict;
use warnings;
use File::Basename;

sub helper {
    my ($x) = @_;
    return $x + 1;
}

sub main {
    helper(2);
    missing_fn();
}

1;
