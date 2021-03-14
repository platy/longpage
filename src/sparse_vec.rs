use std::{ops::Range, slice};

#[derive(Debug)]
pub struct SparseVec<T> {
    len: usize,
    /// Each block starts from an offset within the SparseVec range and proceeds to the end of it's Vec
    blocks: Vec<(usize, Vec<T>)>,
}

impl<T> SparseVec<T> {
    pub fn with_len(len: usize) -> Self {
        SparseVec {
            len,
            blocks: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn range_iter(&self, idxs: Range<usize>) -> Iter<'_, T> {
        Iter {
            len: idxs.end,
            position: idxs.start,
            blocks_iter: self.blocks.iter(),
            block_iter: None,
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.range_iter(0..self.len)
    }

    /// Insert data into empty space
    // Panics if space is occupied
    pub fn insert_vec(&mut self, start: usize, vec: Vec<T>) {
        let insert_pos = self
            .blocks
            .iter()
            .position(|(offset, _)| *offset >= start)
            .unwrap_or(self.blocks.len());
        assert!(
            insert_pos == 0
                || start >= (self.blocks[insert_pos - 1].0 + self.blocks[insert_pos - 1].1.len()),
            "Inserted vec overlaps existing block"
        );
        assert!(
            start + vec.len()
                <= self
                    .blocks
                    .get(insert_pos)
                    .map(|(offset, _)| *offset)
                    .unwrap_or(usize::MAX),
            "Inserted vec overlaps existing block"
        );
        self.blocks.insert(insert_pos, (start, vec));
    }
}

impl<T> From<Vec<T>> for SparseVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            len: vec.len(),
            blocks: vec![(0, vec)],
        }
    }
}

pub struct Iter<'i, T> {
    len: usize,
    position: usize,
    blocks_iter: slice::Iter<'i, (usize, Vec<T>)>,
    block_iter: Option<(usize, slice::Iter<'i, T>)>,
}

impl<'i, T> Iter<'i, T> {
    fn next_block(&mut self) {
        self.block_iter = loop {
            let block = self
                .blocks_iter
                .next();
            if let Some((offset, vec)) = block {
                if self.position < offset + vec.len() {
                    break Some((*offset, vec.iter()))
                }
            } else {
                break None
            }
        };
    }
}

impl<'i, T> Iterator for Iter<'i, T> {
    type Item = Option<&'i T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.len {
            return None;
        }
        if self.block_iter.is_none() {
            self.next_block();
        }
        let result = if let Some((offset, block_iter)) = &mut self.block_iter {
            if self.position < *offset {
                // in gap before block
                Some(None)
            } else if let Some(next) = block_iter.next() {
                Some(Some(next))
            } else {
                // after block
                self.next_block();
                if let Some((offset, block_iter)) = &mut self.block_iter {
                    if self.position < *offset {
                        // in gap before block
                        Some(None)
                    } else if let Some(next) = block_iter.next() {
                        Some(Some(next))
                    } else {
                        // after block
                        Some(None)
                    }
                } else {
                    // iter is empty
                    Some(None)
                }
            }
        } else {
            // iter is empty
            Some(None)
        };
        self.position += 1;
        result
    }
}

#[test]
fn create_large_empty() {
    SparseVec::<String>::with_len(usize::MAX);
}

#[test]
fn iterate_empty() {
    assert_eq!(
        SparseVec::<u32>::with_len(5).iter().collect::<Vec<_>>(),
        vec![None, None, None, None, None]
    );
}

#[test]
fn iterate_full() {
    assert_eq!(
        SparseVec::<u32>::from(vec![1, 2, 3, 4, 5])
            .iter()
            .map(|o| o.copied())
            .collect::<Vec<_>>(),
        (1..=5).map(Some).collect::<Vec<_>>()
    );
}

#[test]
fn gapped_blocks() {
    let mut vec: SparseVec<u8> = SparseVec::with_len(5);
    println!("{:?}", &vec);
    vec.insert_vec(0, vec![1, 2]);
    println!("{:?}", &vec);
    vec.insert_vec(3, vec![4, 5]);
    println!("{:?}", &vec);
    assert_eq!(
        vec.iter().map(|o| o.copied()).collect::<Vec<_>>(),
        vec![Some(1), Some(2), None, Some(4), Some(5)]
    );
}

#[test]
fn following_blocks() {
    let mut vec: SparseVec<u8> = SparseVec::with_len(5);
    vec.insert_vec(0, vec![1, 2, 3]);
    vec.insert_vec(3, vec![4, 5]);
    assert_eq!(
        vec.iter().map(|o| o.copied()).collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
    );
}

#[test]
fn empty_blocks() {
    let mut vec: SparseVec<u8> = SparseVec::with_len(5);
    vec.insert_vec(0, vec![]);
    vec.insert_vec(0, vec![]);
    assert_eq!(
        vec.iter().map(|o| o.copied()).collect::<Vec<_>>(),
        vec![None, None, None, None, None]
    );
}

#[test]
#[should_panic]
fn overlap_insert_before() {
    let mut vec: SparseVec<u8> = SparseVec::with_len(5);
    vec.insert_vec(2, vec![3, 4]);
    vec.insert_vec(0, vec![1, 2, 3]);
}

#[test]
#[should_panic]
fn overlap_insert_after() {
    let mut vec: SparseVec<u8> = SparseVec::with_len(5);
    vec.insert_vec(0, vec![1, 2, 3]);
    vec.insert_vec(2, vec![3, 4]);
}
