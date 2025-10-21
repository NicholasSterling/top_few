use proptest::prelude::*;
use std::collections::BinaryHeap;
use top_few::Top16;

#[test]
fn ascending() {
    let mut it = Top16::new(0);

    // Check that the iterator is empty at the start.
    let elements: Vec<u32> = it.iter().collect();
    assert_eq!(elements, [0u32; 0]);

    // Add some elements in ascending order.
    for i in 1..20 {
        it.see(i);
    }

    // Forward iterator.
    let elements: Vec<u32> = it.iter().collect();
    dbg!(&it);
    // dbg!(&elements);
    let expected: Vec<u32> = (4..20).rev().collect();
    assert_eq!(elements, expected);

    // Reverse iterator.
    let elements: Vec<u32> = it.iter().rev().collect();
    // dbg!(&elements);
    let expected: Vec<u32> = (4..20).collect();
    assert_eq!(elements, expected);

    // Forward iterator with take.
    let elements: Vec<u32> = it.iter().take(10).collect();
    // dbg!(&elements);
    let expected: Vec<u32> = (10..20).rev().collect();
    assert_eq!(elements, expected);

    // Reverse iterator with take.
    let elements: Vec<u32> = it.iter().rev().take(10).collect();
    // dbg!(&elements);
    let expected: Vec<u32> = (4..14).collect();
    assert_eq!(elements, expected);

    // Check the positions returned by see.
    assert_eq!(it.rank(0), 0); // 4 5 6 ...
    assert_eq!(it.rank(4), 0); // 4 5 6 ...
    assert_eq!(it.rank(5), 1); // 4 5 6 ...  => 5 5 6 ...
    assert_eq!(it.rank(5), 0); // 5 5 6 ...     ^
    assert_eq!(it.rank(6), 2); // 5 5 6 ...  => 5 6 6 ...
    assert_eq!(it.rank(30), 16); //                ^
}

#[test]
fn descending() {
    let mut it = Top16::new(0);
    for i in 1..20 {
        it.see(20 - i);
    }

    // Forward iterator.
    let elements: Vec<u32> = it.iter().collect();
    let expected: Vec<u32> = (4..20).rev().collect();
    assert_eq!(elements, expected);

    // Reverse iterator.
    let elements: Vec<u32> = it.iter().rev().collect();
    let expected: Vec<u32> = (4..20).collect();
    assert_eq!(elements, expected);
}

#[test]
fn higher_cutoff() {
    let mut it = Top16::new(10);
    for i in 1..20 {
        it.see(20 - i);
    }

    // Forward iterator.
    let elements: Vec<u32> = it.iter().collect();
    let expected: Vec<u32> = (11..20).rev().collect();
    assert_eq!(elements, expected);

    // Reverse iterator.
    let elements: Vec<u32> = it.iter().rev().collect();
    let expected: Vec<u32> = (11..20).collect();
    assert_eq!(elements, expected);

    // Raise the cutoff.
    it.set_cutoff(15);
    // Forward iterator after raising cutoff.
    let elements: Vec<u32> = it.iter().collect();
    let expected: Vec<u32> = (16..20).rev().collect();
    assert_eq!(elements, expected);
}

#[test]
fn peak() {
    let mut it = Top16::new(0);
    for i in 1..10 {
        it.see(i); // ascending
    }
    for i in 1..10 {
        it.see(10 - i); // descending
    }
    let elements: Vec<u32> = it.iter().rev().collect();
    let expected: Vec<u32> = vec![2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9];
    assert_eq!(elements, expected);
}

#[test]
fn take() {
    let mut it = Top16::new(0);

    // Check that the iterator is empty at the start.
    let elements: Vec<u32> = it.iter().collect();
    assert_eq!(elements, [0u32; 0]);

    // Add some elements in ascending order.
    for i in 1..20 {
        it.see(i);
    }

    // Forward iterator.
    let elements: Vec<u32> = it.take(5).collect();
    // dbg!(&elements);
    let expected: Vec<u32> = (15..20).rev().collect();
    assert_eq!(elements, expected);

    // Reverse iterator.
    let elements: Vec<u32> = it.take(5).rev().collect();
    // dbg!(&elements);
    let expected: Vec<u32> = (15..20).collect();
    assert_eq!(elements, expected);
}

fn get_top_16_via_heap<I>(iter: I) -> Vec<u32>
where
    I: Iterator<Item = u32>,
{
    // Create a binary heap and push all elements from the iterator into it.
    let mut heap = BinaryHeap::new();
    for x in iter {
        heap.push(x);
    }

    // Pop the top 16 elements from the heap.
    let mut result: Vec<u32> = Vec::with_capacity(16);
    for _ in 0..16 {
        if let Some(val) = heap.pop() {
            result.push(val);
        } else {
            break; // Less than 16 elements in the iterator
        }
    }
    result
}

#[test]
fn test_get_top_16_via_heap() {
    let top_16 = get_top_16_via_heap(1..=20);
    let expected = vec![20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5];
    assert_eq!(top_16, expected);
}

proptest! {
    #[test]
    fn proptest_top16_iterator_reversed_matches_heap(data in prop::collection::vec(any::<u32>(), 1..1000)) {
        let mut top16_instance = Top16::new(0);
        dbg!(&data);
        for &x in &data {
            top16_instance.see(x);
        }

        let top16_values: Vec<u32> = top16_instance.iter().collect();
        let heap_values = get_top_16_via_heap(data.into_iter());

        assert_eq!(top16_values, heap_values);
    }
}
