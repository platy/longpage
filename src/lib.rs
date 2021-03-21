use std::{
    ops::{Range, RangeFrom},
    usize,
};

use sparse_vec::SparseVec;

mod sparse_vec;

/// Call this on a change to the viewed data or when ready to make a request. The response specifies which range of records should be requested next. Expects that any previous requests have completed.
/// Currently will aim to load 50% of the size of the view in either direction
pub fn next_request_for_view<T>(
    data: &SparseVec<T>,
    in_view: Range<usize>,
) -> Option<Range<usize>> {
    if in_view.len() == 0 {
        return None;
    }
    let extra_load = in_view.len() / 2;
    let should_load = in_view.start.checked_sub(extra_load).unwrap_or(0)
        ..(in_view.end + extra_load).min(data.len());

    let mut longest_empty: Option<Range<usize>> = None;
    let mut current_empty: Option<RangeFrom<usize>> = None;
    for (i, item) in data.iter_range(should_load.clone()).enumerate() {
        if item.is_some() {
            if let Some(current_empty) = current_empty.take() {
                let current_empty = current_empty.start..(should_load.start + i);
                if longest_empty.as_ref().map_or(true, |longest_empty| {
                    longest_empty.len() < current_empty.len()
                }) {
                    longest_empty = Some(current_empty);
                }
            }
        } else if current_empty.is_none() {
            current_empty = Some((should_load.start + i)..);
        }
        println!(
            "{}: longest: {:?} current: {:?}",
            i, &longest_empty, &current_empty
        )
    }
    if let Some(current_empty) = current_empty.take() {
        let current_empty = current_empty.start..(should_load.end);
        if longest_empty.as_ref().map_or(true, |longest_empty| {
            longest_empty.len() < current_empty.len()
        }) {
            longest_empty = Some(current_empty);
        }
    }
    longest_empty
}

#[test]
fn no_view_request_nothing() {
    let p = SparseVec::<u8>::with_len(20);
    assert_eq!(next_request_for_view(&p, 0..0), None);
}

#[test]
fn request_extra_half_after() {
    let p = SparseVec::<u8>::with_len(20);
    assert_eq!(next_request_for_view(&p, 0..10), Some(0..15));
}

#[test]
fn request_extra_half_before() {
    let p = SparseVec::<u8>::with_len(20);
    assert_eq!(next_request_for_view(&p, 10..20), Some(5..20));
}

#[test]
fn request_half_either_side() {
    let p = SparseVec::<u8>::with_len(100);
    assert_eq!(next_request_for_view(&p, 10..20), Some(5..25));
}

#[test]
fn full_view_request_all() {
    let p = SparseVec::<u8>::with_len(20);
    assert_eq!(next_request_for_view(&p, 0..20), Some(0..20));
}

#[test]
fn request_half_after() {
    let mut p = SparseVec::<u8>::with_len(20);
    p.insert_vec(0, (0..10).collect());
    assert_eq!(next_request_for_view(&p, 0..10), Some(10..15));
}

#[test]
fn request_half_before() {
    let mut p = SparseVec::<u8>::with_len(20);
    p.insert_vec(10, (10..20).collect());
    assert_eq!(next_request_for_view(&p, 10..20), Some(5..10));
}
