use crate::{Bin, Pack};

use super::OnlinePacker;

/// This implements the [Next-K-fit](https://en.wikipedia.org/wiki/Next-fit_bin_packing)
/// bin packing algorithm.
///
/// A total of `K` bins are kept open.
/// When a new item arrives, we attempt to put it into any one of the open bins.
/// If none of the open bins are big enough, the most-filled bin is closed,
/// and a new bin is opened to hold the new item.
#[derive(Debug)]
pub struct NextKFitPacker<Item, SizeFn> {
    bins: Vec<Bin<Item>>,
    max_bin_size: usize,
    size_fn: SizeFn,
}

impl<Item, SizeFn> NextKFitPacker<Item, SizeFn> {
    /// Create a new NextKFitPacker.
    ///
    /// It will keep open `k` bins,
    /// each of which will fit a maximum of `size`.
    ///
    /// The size of a single element is determined by the `size_fn`.
    ///
    /// Panics if `k` or `size` is 0.
    pub fn new_with_key(k: usize, size: usize, size_fn: SizeFn) -> Self {
        assert_ne!(k, 0, "k must be greater than 0");
        assert_ne!(size, 0, "size must be greater than 0");

        Self {
            bins: (0..k).map(|_| Bin::with_capacity(size)).collect::<Vec<_>>(),
            max_bin_size: size,
            size_fn,
        }
    }
}

impl<Item> NextKFitPacker<Item, fn(&Item) -> usize> {
    /// Create a new NextKFitPacker.
    ///
    /// This function requires that `Item` implements [`Pack`].
    /// If your type doesn't, consider using [`new_with_key`](NextKFitPacker::new_with_key).
    pub fn new(k: usize, size: usize) -> NextKFitPacker<Item, fn(&Item) -> usize>
    where
        Item: Pack,
    {
        fn pack_size(item: &impl Pack) -> usize {
            item.size()
        }

        NextKFitPacker::<Item, _>::new_with_key(k, size, pack_size)
    }
}

impl<Item, SizeFn> OnlinePacker<Item> for NextKFitPacker<Item, SizeFn>
where
    SizeFn: Fn(&Item) -> usize,
{
    fn try_add(
        &mut self,
        item: Item,
    ) -> Result<Vec<Bin<Item>>, super::online_packer::OnlinePackerError<Item>> {
        let item_size = (self.size_fn)(&item);
        if item_size > self.max_bin_size {
            return Err(super::online_packer::OnlinePackerError::ItemTooLarge(item));
        }

        // See if the item fits in any of the open bins.
        // At the same time, keep track of the most-filled bin.
        let mut most_filled_bin_idx = 0;
        let mut most_filled_bin_capacity = usize::MAX;
        for (bin_idx, bin) in self.bins.iter_mut().enumerate() {
            if bin.remaining_capacity < most_filled_bin_capacity {
                most_filled_bin_idx = bin_idx;
                most_filled_bin_capacity = bin.remaining_capacity;
            }

            if item_size <= bin.remaining_capacity {
                bin.add_with_size(item, item_size);
                return Ok(Vec::new());
            }
        }

        // The item didn't fit into any of the bins,
        // so we need to:
        // - open a new bin
        // - put the new item in it
        // - close the most-filled bin (and return it)
        let mut bin = Bin::with_item_and_size(self.max_bin_size, item, item_size);

        std::mem::swap(&mut self.bins[most_filled_bin_idx], &mut bin);

        Ok(vec![bin])
    }

    fn finalize(mut self) -> Vec<Bin<Item>> {
        // TODO: maybe the remaining bins could be packed more efficiently?
        // Right now, we just return all the bins we have that aren't empty.
        self.bins.retain(|bin| !bin.contents.is_empty());
        self.bins
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{generate_test_bins, generate_test_set_a, MyItem};

    use super::*;

    #[test]
    fn empty_input_returns_no_bins() {
        let packer: NextKFitPacker<MyItem, _> = NextKFitPacker::new(3, 10);
        assert_eq!(packer.finalize(), vec![]);

        let packer: NextKFitPacker<MyItem, _> = NextKFitPacker::new(3, 10);
        assert_eq!(packer.pack_all(vec![].into_iter()).unwrap(), vec![]);
    }

    #[test]
    fn test_dataset_a_k1() {
        let (test_data, bin_size) = generate_test_set_a();
        let packer = NextKFitPacker::new(1, bin_size);

        let bins = packer.pack_all(test_data.into_iter()).unwrap();

        // With one bin lookahead, NextKFit does not give an optimal solution
        let expected = generate_test_bins(
            20,
            vec![
                vec![1, 1, 1, 1, 3, 4], // 11
                vec![10, 10],           // 20
                vec![10],               // 10
                vec![19],               // 19
                vec![19],               // 19
            ],
        );

        assert_eq!(expected, bins);
    }

    #[test]
    fn test_dataset_a_k1_seq() {
        let (test_data, bin_size) = generate_test_set_a();
        let mut packer = NextKFitPacker::new(1, bin_size);

        // This contains the expected bin outputs at every step:
        // if None, expecting zero bins,
        // if Some, expecting one bin whose contents is the MyItems with the given sizes.
        //
        // e.g: On the 7th invocation of packer.add(),
        // we expect to get Bin {contents: [MyItem(1), MyItem(1), MyItem(1), MyItem(1), MyItem(3), MyItem(4)], remaining_capacity: 9}.
        let mut expected = [
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![1, 1, 1, 1, 3, 4]),
            None,
            Some(vec![10, 10]),
            Some(vec![10]),
            Some(vec![19]),
            // This bin is only produced after finalizing
            Some(vec![19]),
        ]
        .map(|opt| {
            opt.map(|vec| Bin {
                contents: vec.iter().map(|i| MyItem { size: *i }).collect::<Vec<_>>(),
                remaining_capacity: bin_size - vec.iter().sum::<usize>(),
            })
        })
        .into_iter();

        for item in test_data {
            let produced_bin = packer.add(item).into_iter().next();
            let expected_bin = expected.next().unwrap();
            match (&expected_bin, &produced_bin) {
                (Some(expected), Some(actual)) => assert_eq!(expected, actual),
                (None, None) => (),
                _ => panic!("Expected {:?}, got {:?}", expected_bin, produced_bin),
            }
        }

        let final_bins = packer.finalize();
        for (bin, expected) in final_bins.iter().zip(expected) {
            assert_eq!(bin.contents, expected.unwrap().into_contents());
        }
    }

    #[test]
    fn test_dataset_a_k2() {
        let (test_data, bin_size) = generate_test_set_a();
        let packer = NextKFitPacker::new(2, bin_size);

        let mut bins = packer.pack_all(test_data.into_iter()).unwrap();

        // With two bin lookahead, NextKFit does not give an optimal solution;
        // however, it emits the same solution as Next1Fit, above,
        // but in a different order.
        let expected = generate_test_bins(
            20,
            vec![
                vec![10, 10],           // 20
                vec![1, 1, 1, 1, 3, 4], // 11
                vec![19],               // 19
                vec![19],               // 19
                vec![10],               // 10
            ],
        );

        println!("{:#?}", bins);

        assert_eq!(expected, bins);
    }
}
