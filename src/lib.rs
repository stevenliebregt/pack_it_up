//! pack_it_up is a simple Rust library that implements various bin packing algorithms.
//!
//! # Example
//!
//! ```
//! use pack_it_up::Pack;
//! use pack_it_up::offline::first_fit_decreasing::first_fit_decreasing;
//!
//! /// This is the struct you want to pack into bins.
//! /// In this example, it only has two fields:
//! /// some content and the item's size.
//! #[derive(Debug, Eq, PartialEq)]
//! struct MyItem {
//!     some_content: i32,
//!     size: usize,
//! }
//!
//! /// This trait tells the algorithms how big the item is
//! impl Pack for MyItem {
//!     fn size(&self) -> usize {
//!         self.size
//!     }
//! }
//!
//! let my_items = vec![
//!     MyItem { some_content: 1, size: 1, },
//!     MyItem { some_content: 2, size: 2, },
//!     MyItem { some_content: 3, size: 19, },
//!     MyItem { some_content: 4, size: 17, },
//!     MyItem { some_content: 5, size: 1, },
//! ];
//!
//! // Packing the items into bins where the total size of each bin is at most 20
//! let mut bins = first_fit_decreasing(20, my_items);
//!
//! // The above will result in 2 full bins
//! assert_eq!(2, bins.len());
//!
//! let first_bin_contents = bins.remove(0).into_contents();
//! assert_eq!(vec![MyItem{ some_content: 3, size: 19 }, MyItem { some_content: 1, size: 1 }], first_bin_contents);
//!
//! let second_bin_contents = bins.remove(0).into_contents();
//! assert_eq!(vec![MyItem{ some_content: 4, size: 17 }, MyItem { some_content: 2, size: 2 }, MyItem { some_content: 5, size: 1 }], second_bin_contents);
//! ```

pub mod offline;
pub mod online;
pub mod wrapper;

/// Allows the bin packing algorithm to know how big an item is, which can then be used to
/// figure out in which bin it fits.
pub trait Pack {
    /// Get the size of the item to pack in bins.
    fn size(&self) -> usize;
}

#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Bin<T> {
    contents: Vec<T>,
    remaining_capacity: usize,
}

impl<T> Bin<T> {
    /// Create a new empty bin.
    #[doc(hidden)]
    pub(crate) const fn with_capacity(capacity: usize) -> Self {
        Self {
            contents: vec![],
            remaining_capacity: capacity,
        }
    }

    /// Create a new bin with a single item.
    #[doc(hidden)]
    pub(crate) fn with_item(capacity: usize, item: T) -> Self
    where
        T: Pack,
    {
        Self {
            remaining_capacity: capacity.saturating_sub(item.size()),
            contents: vec![item],
        }
    }

    /// Create a new bin with a single item, given its size.
    #[doc(hidden)]
    pub(crate) fn with_item_and_size(capacity: usize, item: T, size: usize) -> Self {
        Self {
            remaining_capacity: capacity.saturating_sub(size),
            contents: vec![item],
        }
    }

    /// Add an item to this bin, and update the remaining capacity.
    ///
    /// Uses saturating subtraction: if you push a too-big item,
    /// then the bin will have a remaining capacity of zero.
    #[doc(hidden)]
    pub(crate) fn add(&mut self, item: T)
    where
        T: Pack,
    {
        self.remaining_capacity = self.remaining_capacity.saturating_sub(item.size());
        self.contents.push(item);
    }

    /// Add an item to this bin (given its size) and update the remaining capacity.
    #[doc(hidden)]
    pub(crate) fn add_with_size(&mut self, item: T, size: usize) {
        self.remaining_capacity = self.remaining_capacity.saturating_sub(size);
        self.contents.push(item);
    }

    /// Get the contents of the bin.
    pub fn contents(&self) -> &[T] {
        &self.contents
    }

    /// Get the contents of the bin.
    pub fn into_contents(self) -> Vec<T> {
        self.contents
    }

    /// Transform the bin's contents into a different type.
    ///
    /// Note that the bin's capacity is simply copied from the old bin to the new one,
    /// and overflows are not considered
    /// if the transformation would increase the size of the bin's contents,
    /// that is allowed here.
    /// In fact, the new type doesn't even have to implement [`crate::Pack`],
    /// so it might not even have a reasonable notion of size.
    pub fn map<U>(self, transform_fn: impl Fn(T) -> U) -> Bin<U> {
        Bin {
            contents: self.contents.into_iter().map(transform_fn).collect(),
            remaining_capacity: self.remaining_capacity,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A dummy struct for testing
    /// Realistic structs would have more fields but that makes testing harder.
    #[allow(dead_code)]
    #[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
    pub struct MyItem {
        pub size: usize,
    }

    impl Pack for MyItem {
        fn size(&self) -> usize {
            self.size
        }
    }

    /// A dummy struct for testing.
    /// The same struct as [`MyItem`], but it doesn't implement [`Pack`].
    #[allow(dead_code)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct MyItemUnpacked {
        pub size: usize,
    }

    impl MyItem {
        pub fn make_unpacked(self) -> MyItemUnpacked {
            MyItemUnpacked { size: self.size }
        }
    }

    /// Generates a set with the following data and a bin size:
    ///   1,1,1,1,3,4,10,10,10,19,19
    /// Naive binning would result in 5 bins:
    /// ```
    ///   1,1,1,1,3,4   -> 11
    ///   10, 10        -> 20
    ///   10            -> 10
    ///   19            -> 19
    ///   19            -> 19
    /// ```
    /// Optimal binning would result in 4 bins:
    /// ```
    ///   19,1          -> 20
    ///   19,1          -> 20
    ///   10,10         -> 20
    ///   10,1,1,3,4    -> 19
    /// ```
    pub fn generate_test_set_a() -> (Vec<MyItem>, usize) {
        (
            vec![
                MyItem { size: 1 },
                MyItem { size: 1 },
                MyItem { size: 1 },
                MyItem { size: 1 },
                MyItem { size: 3 },
                MyItem { size: 4 },
                MyItem { size: 10 },
                MyItem { size: 10 },
                MyItem { size: 10 },
                MyItem { size: 19 },
                MyItem { size: 19 },
            ],
            20,
        )
    }

    pub fn generate_test_bins(bin_size: usize, data: Vec<Vec<usize>>) -> Vec<Bin<MyItem>> {
        data.into_iter()
            .map(|bin_data| expected_test_bin(bin_size, bin_data))
            .collect::<Vec<_>>()
    }

    pub fn expected_test_bin(bin_size: usize, data: Vec<usize>) -> Bin<MyItem> {
        Bin {
            contents: data.iter().map(|i| MyItem { size: *i }).collect::<Vec<_>>(),
            remaining_capacity: bin_size - data.iter().sum::<usize>(),
        }
    }
}
