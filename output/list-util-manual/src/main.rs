// Converted from Perl List::Util core functions
// Original: Scalar-List-Utils-1.63/lib/List/Util.pm
// Conversion: Perl sub → pub fn, @_ → function parameters, $_ → explicit variable

/// Returns the sum of all numeric values in the slice.
/// Perl: sub sum { my $total = 0; $total += $_ for @_; $total }
pub fn sum(nums: &[f64]) -> f64 {
    nums.iter().sum()
}

/// Returns the minimum value.
/// Perl: sub min { my $m = $_[0]; for (@_) { $m = $_ if $_ < $m } $m }
pub fn min(nums: &[f64]) -> Option<f64> {
    nums.iter().cloned().reduce(f64::min)
}

/// Returns the maximum value.
/// Perl: sub max { my $m = $_[0]; for (@_) { $m = $_ if $_ > $m } $m }
pub fn max(nums: &[f64]) -> Option<f64> {
    nums.iter().cloned().reduce(f64::max)
}

/// Returns the first element for which predicate returns true.
/// Perl: sub first (&@) { my $code = shift; for (@_) { return $_ if $code->($_) } undef }
pub fn first<T, F>(slice: &[T], predicate: F) -> Option<&T>
where
    F: Fn(&T) -> bool,
{
    slice.iter().find(|x| predicate(x))
}

/// Returns true if predicate is true for any element.
/// Perl: sub any (&@) { my $code = shift; !!grep { $code->($_) } @_ }
pub fn any<T, F>(slice: &[T], predicate: F) -> bool
where
    F: Fn(&T) -> bool,
{
    slice.iter().any(predicate)
}

/// Returns true if predicate is true for all elements.
/// Perl: sub all (&@) { my $code = shift; !grep { !$code->($_) } @_ }
pub fn all<T, F>(slice: &[T], predicate: F) -> bool
where
    F: Fn(&T) -> bool,
{
    slice.iter().all(predicate)
}

/// Returns the product of all values.
/// Perl: sub product { my $p = 1; $p *= $_ for @_; $p }
pub fn product(nums: &[f64]) -> f64 {
    nums.iter().product()
}

/// Returns sum, or 0 if empty (unlike sum which returns undef).
/// Perl: sub sum0 { my $s = 0; $s += $_ for @_; $s }
pub fn sum0(nums: &[f64]) -> f64 {
    nums.iter().sum()
}

/// Returns (first N elements, remaining elements).
/// Perl: sub head { my $n = shift; @_[0..$n-1] }
pub fn head(n: usize, slice: &[f64]) -> &[f64] {
    &slice[..n.min(slice.len())]
}

/// Returns the last N elements.
/// Perl: sub tail { my $n = shift; @_[-$n..-1] }
pub fn tail(n: usize, slice: &[f64]) -> &[f64] {
    let len = slice.len();
    if n >= len {
        slice
    } else {
        &slice[len - n..]
    }
}

/// Returns unique elements (preserving first occurrence).
/// Perl: sub uniq { my %seen; grep { !$seen{$_}++ } @_ }
pub fn uniq(slice: &[f64]) -> Vec<f64> {
    let mut seen = std::collections::HashSet::new();
    slice
        .iter()
        .filter(|&x| {
            let bits = x.to_bits();
            seen.insert(bits)
        })
        .cloned()
        .collect()
}

fn main() {
    let nums: Vec<f64> = (1..=10).map(|x| x as f64).collect();

    println!("sum: {}", sum(&nums));
    println!("min: {}", min(&nums).unwrap_or(0.0));
    println!("max: {}", max(&nums).unwrap_or(0.0));
    println!(
        "first > 5: {}",
        first(&nums, |&x| x > 5.0).map(|x| *x).unwrap_or(0.0)
    );
    println!("any > 8: {}", any(&nums, |&x| x > 8.0));
    println!("all > 0: {}", all(&nums, |&x| x > 0.0));
    println!("product: {}", product(&nums));
    println!("sum0 of empty: {}", sum0(&[]));
    println!("head(3): {:?}", head(3, &nums));
    println!("tail(3): {:?}", tail(3, &nums));

    let with_dups = vec![1.0, 2.0, 1.0, 3.0, 2.0, 4.0];
    println!("uniq: {:?}", uniq(&with_dups));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        assert_eq!(sum(&[1.0, 2.0, 3.0, 4.0, 5.0]), 15.0);
        assert_eq!(sum(&[]), 0.0);
    }

    #[test]
    fn test_min_max() {
        let nums = vec![3.0, 1.0, 4.0, 1.0, 5.0];
        assert_eq!(min(&nums), Some(1.0));
        assert_eq!(max(&nums), Some(5.0));
        assert_eq!(min(&[]), None);
    }

    #[test]
    fn test_first() {
        let nums = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(first(&nums, |&x| x > 3.0), Some(&4.0));
        assert_eq!(first(&nums, |&x| x > 10.0), None);
    }

    #[test]
    fn test_any_all() {
        let nums = vec![1.0, 2.0, 3.0];
        assert!(any(&nums, |&x| x > 2.0));
        assert!(!any(&nums, |&x| x > 5.0));
        assert!(all(&nums, |&x| x > 0.0));
        assert!(!all(&nums, |&x| x > 1.0));
    }

    #[test]
    fn test_head_tail() {
        let nums = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(head(3, &nums), &[1.0, 2.0, 3.0]);
        assert_eq!(tail(2, &nums), &[4.0, 5.0]);
    }

    #[test]
    fn test_uniq() {
        let with_dups = vec![1.0, 2.0, 1.0, 3.0, 2.0];
        assert_eq!(uniq(&with_dups), vec![1.0, 2.0, 3.0]);
    }
}
