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
//! with (0,0) as the cutoff.  Or you could use i32 values with -1 as the cutoff.
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

use std::fmt::Debug;

const NUM: usize = 16; // number of elements and indices
const IX_BITS: u32 = 4; // bits to hold an index
const IX_MASK: u64 = (1 << IX_BITS) - 1; // mask for extracting an index, e.g. 0xF
const IXS_BITS: u32 = NUM as u32 * IX_BITS; // 64 bits for 16 indices

#[derive(Clone, Copy)]
pub struct Top16 {
    // A value must be larger than this to be included in the top list.
    // It is the smallest value in the list, or the cutoff value
    // if the list has not been filled yet.
    threshold: u32,
    // The cutoff value.  Only values larger than this will be considered,
    // or returned by the iterator.
    cutoff: u32,
    // The 4-bit indices of the top elements, packed in ascending order;
    // the least significant bits contain the index of the smallest, etc.
    sorted_ixs: u64,
    // The top elements, unordered.
    elements: [u32; NUM],
}

impl Top16 {
    /// Returns a new instance of Top16.
    /// Only values larger than the cutoff will be considered.
    pub fn new(cutoff: u32) -> Self {
        Self {
            elements: [cutoff; NUM],
            sorted_ixs: 0xFEDCBA9876543210,
            threshold: cutoff,
            cutoff,
        }
    }

    /// Changes the cutoff value to the specified new value.
    /// Note that this removes values that are smaller than the new cutoff.
    pub fn set_cutoff(&mut self, new_cutoff: u32) {
        // If the cutoff is being raised, then we need to set any values
        // that are smaller than the new cutoff to the new cutoff.
        // If the cutoff is being lowered, then we need to set any values
        // equal to the old cutoff to the new lower cutoff.
        // We can do both in one go.
        let cutoff = self.cutoff.max(new_cutoff - 1);
        let mut shift = 0u32;
        loop {
            if shift >= IXS_BITS {
                break; // We have processed all indices.
            }
            let ix = self.ix(shift);
            if self.elements[ix] > cutoff {
                break; // All remaining elements are larger; keep them.
            }
            self.elements[ix] = new_cutoff;
            shift += IX_BITS; // On to the next larger element's index.
        }
        self.threshold = self.element_at(0);
        self.cutoff = new_cutoff;
    }

    /// Returns the current cutoff value.
    #[inline]
    pub fn cutoff(&self) -> u32 {
        self.cutoff
    }

    /// Returns the largest element in the top 16.
    #[inline]
    pub fn max(&self) -> Option<u32> {
        let v = self.element_at(IXS_BITS - IX_BITS);
        (v > self.cutoff).then_some(v)
    }

    // Returns the index at the specified shift in the sorted indices.
    #[inline]
    fn ix(&self, shift: u32) -> usize {
        ((self.sorted_ixs >> shift) & IX_MASK) as usize
    }

    // Returns the element at the specified shift in the sorted indices.
    #[inline]
    fn element_at(&self, shift: u32) -> u32 {
        // TODO: check whether the optimizer can tell that this is always in bounds.
        self.elements[self.ix(shift)]
    }

    /// Considers a new value to see if is one of the top 16.
    /// If so, it is added to the list.  The return value is 0 if the value is not
    /// in the top 16, or its position in the top 16 if it is, 1 for the smallest
    /// element and 16 for the largest element.  That way you can, for example,
    /// easily trigger special behavior if the value is in the top 5.
    #[inline]
    pub fn rank(&mut self, value: u32) -> usize {
        // If the value is not greater than the threshold, then it is not in the top 16.
        // We separate this check from the rest of the logic so that it will be inlined.
        if value <= self.threshold {
            0
        } else {
            ((self.see_helper(value) >> 2) + 1) as usize
        }
    }

    /// Considers a new value to see if is one of the top 16.
    /// If so, it is added to the list.
    #[inline]
    pub fn see(&mut self, value: u32) {
        // If the value is not greater than the threshold, then it is not in the top 16.
        // We separate this check from the rest of the logic so that it will be inlined.
        if value > self.threshold {
            self.see_helper(value);
        }
    }

    fn see_helper(&mut self, value: u32) -> u32 {
        // Perform a binary search to find the bit position for the new value's index
        // among the sorted indices.  This diagram depicts the search pattern.
        // 0    4    8    12   16   20   24   28   32   36   40   44   48   52   56   60
        // xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx
        //                                         ^
        //                     ^                                       .
        //           ^                   .
        //      ^         .
        // ^    .
        //
        // Since we always have 16 elements, we can unroll the loop
        // and do log2(16) = 4 iterations of the binary search.
        // We avoid branches by using branchless programming techniques.
        // No += here because the RHS could be negative; we want to use u32s.
        let mut shift = 32u32;
        let le = |shift| (value <= self.element_at(shift)) as u32;
        // shift = shift + a.cmp(b) as u64 * 4 * IX_BITS;  // << 4;
        #[allow(clippy::identity_op, clippy::erasing_op)]
        {
            shift = shift + 4 * IX_BITS - (le(shift) << 5); //   - (0 | 8) * IX_BITS
            shift = shift + 2 * IX_BITS - (le(shift) << 4); //   - (0 | 4) * IX_BITS
            shift = shift + 1 * IX_BITS - (le(shift) << 3); //   - (0 | 2) * IX_BITS
            shift = shift + 0 * IX_BITS - (le(shift) << 2); //   - (0 | 1) * IX_BITS
        }

        // Insert the new value's index at the found shift.
        // E.g. if shift = 48 and sorted_ixs = 0xFEDCBA9876543210,
        // upper = 0xFEDCBA9876543210 >> 52       = 0x0000000000000FED
        // lower = 0xFEDCBA9876543210 << 12 >> 16 = 0x0000CBA987654321
        // sorted_ixs                             = 0xFED0CBA987654321
        //                        inserted value's index ^
        // Note that we have to include the index at shift in lower,
        // and we have to get rid of the smallest element's index,
        // which is in the least significant 4 bits of sorted_ixs.
        let lower =
            (self.sorted_ixs << (IXS_BITS - IX_BITS - shift)).unbounded_shr(IXS_BITS - shift);
        let upper = self.sorted_ixs.unbounded_shr(shift + IX_BITS);
        let old_min_ix = self.ix(0); // Save index of smallest element
        self.sorted_ixs = (((upper << IX_BITS) | (old_min_ix as u64)) << shift) | lower;

        // eprintln!("shift: {shift:2}, ixs: {:016X}", self.sorted_ixs);

        // Replace the smallest element with the new value and fix the threshold.
        self.elements[old_min_ix] = value;
        self.threshold = self.element_at(0); // always >= the previous value

        // dbg!(&self.elements[0..4]);
        shift
    }

    /// Returns an Iterator over the top 16 elements (or less if there are less), in descending order.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        self.make_iter(0)
    }

    /// Returns an Iterator over the top n elements (or less if there are less), in descending order.
    /// top16.take(n) is equivalent to top16.iter().take(n), but more efficient.
    #[inline]
    pub fn take(&self, n: u32) -> Iter<'_> {
        self.make_iter((16 - 16.min(n)) * IX_BITS)
    }

    // Does the actual work of creating an iterator.
    fn make_iter(&self, mut fwd_shift: u32) -> Iter<'_> {
        // Have to skip over any cutoff values (there shouldn't be anything lower).
        while fwd_shift < IXS_BITS && self.element_at(fwd_shift) <= self.cutoff {
            fwd_shift += IX_BITS;
        }
        Iter {
            top: self,
            fwd_shift,
            bwd_shift: IXS_BITS,
        }
    }
}

// Custom Debug implementation to show sorted_ixs as hex.
impl Debug for Top16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Top16 {{ cutoff: {}, threshold: {}, sorted_ixs: {:016X}, elements: [",
            self.cutoff, self.threshold, self.sorted_ixs
        )?;
        for (i, &v) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
                if i % 4 == 0 {
                    write!(f, " ")?;
                }
            }
            write!(f, "{v:?}")?;
        }
        write!(f, "]}}")
    }
}

/// Iterator for a Top16.  It returns the top 16 elements in descending order.
/// The iterator is double-ended, so you can use .rev() to get ascending order.
/// Note that the iterator will only return values larger than the cutoff value.
/// If the Top16 has not seen 16 values larger than the cutoff, the Iterator will
/// return less than 16 values.
pub struct Iter<'a> {
    // The Top16 instance to iterate over.
    top: &'a crate::Top16,
    // The bit position of the next element to return for next_back().
    fwd_shift: u32,
    // The bit position just past the next element to return for next().
    bwd_shift: u32,
}

impl Iterator for Iter<'_> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.fwd_shift == self.bwd_shift {
            None
        } else {
            self.bwd_shift -= IX_BITS;
            Some(self.top.element_at(self.bwd_shift))
        }
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.fwd_shift == self.bwd_shift {
            None
        } else {
            let ix = self.top.ix(self.fwd_shift);
            self.fwd_shift += IX_BITS;
            Some(self.top.elements[ix])
        }
    }
}
