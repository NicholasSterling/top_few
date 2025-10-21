//! Top16 is a data structure that keeps track of the top 16 values seen so far
//! in a stream of values.  It is designed to be efficient in both time and space.
//! It takes advantage of the fact that 16 values can be indexed by four bits,
//! and 16 4-bit values can be stored in a single 64-bit word.  This means that
//! we can hold 16 values in an array and pack their indices into a single u64,
//! which we can search and keep sorted using fast shift operations.  And since
//! only four iterations of the binary search are needed to find the correct
//! position for a new value, we just unroll the loop with the four steps one
//! after another.  And finally, we use branchless programming techniques for
//! the search steps, avoiding branches to further improve performance.
//!
//! Note, though, that this does not quite return the top 16 values seen.
//! You must specify a cutoff value, and only values larger than that
//! will be considered.  So, for example, if you are using u32 values
//! and specify 0 as the cutoff, then 0s will never be included in the result,
//! even if all the values seen were 0.
//! If you really need to include 0s in the result,
//! you can use Option<u32> values with None as the cutoff value.
//! Or you could use (u32, u32) values, where the second u32 is a counter,
//! with (0,0) as the cutoff value.
//!
//! Having a cutoff value helps performance in a few ways.
//! We initialize the list to the cutoff value, so we always have 16 values,
//! which means that we don't have to have special handling for when we
//! have less than 16 values, e.g. during the binary search.
//! That further allows us to unroll the binary search loop.
//! And finally, if you were really only interested in values above some cutoff
//! in the first place, then you get that at no performance cost;
//! there is no need to filter them out before letting the Top16 see them.
//!
//! If a given value is seen multiple times, it can be included multiple times.
//! New values do not replace existing values; the oldest instances are retained,
//! and are reported first by iterators.
//!
//! Top16 is designed for streaming use cases, where you show it values as they
//! come in, and it keeps track of the top 16 values seen so far.
//! As you show it new values, it tells you if they are in the top 16,
//! and if so, at what position.
//!
//! At any time you can get an iterator over the top 16 values
//! which will efficiently return them in descending order.
//! It is a double-ended iterator, so you can use the rev method to
//! get them in ascending order.  Note that you will get less than 16 values
//! if it has not seen 16 values larger than the cutoff.

// TODO:
// - .max() should return an Option, right?
// - Extension method for Iterator, e.g. nums.iter().top16(cutoff).
// - But take IntoIterator.
// - Criterion benchmarks.
// - Try a.cmp(b); remember that 0 (equals) means that we do not know whether older or newer is kept.
// - Use usize instead of u64 for sorted_ixs.
// - #[cfg(target_pointer_width = "64")]
// - 32-bit version using two usizes.
// - generic T that is comparable, e.g. T: Ord + Copy
// - try Option<u32> with None as the cutoff value
// - faster than .take(): top(5) and bottom(5) methods.
// - doc tests
// - README.md and docs
// - Top8
// - API Guidelines Checklist
// - Check the assembly language.  Index unchecked?  Binary search?  max() doesn't mask?
// Godbolt: https://godbolt.org/z/7er6vYjax

pub mod top16;

pub use top16::{Iter, Top16};
