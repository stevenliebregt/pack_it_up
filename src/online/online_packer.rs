use crate::Bin;

/// This trait is implemented by online packers.
/// These algorithms consume items one by one,
/// and at each step they decide whether to close any bins
/// (giving them away for later use) or not.
pub trait OnlinePacker<Item> {
    /// Try adding a new item to the packer.
    ///
    /// If this results in any bins getting closed, they will be returned in the `Ok(Vec)`;
    /// if no bins can be closed yet, the Vec will be empty.
    fn try_add(&mut self, item: Item) -> Result<Vec<Bin<Item>>, OnlinePackerError<Item>>;

    /// Add a new item to the packer.
    ///
    /// Like [`OnlinePacker::try_add`], but will panic if the item cannot be added.
    fn add(&mut self, item: Item) -> Vec<Bin<Item>> {
        match self.try_add(item) {
            Ok(bins) => bins,
            Err(_) => panic!("Could not add item to packer"),
        }
    }

    /// No new items will be coming in.
    /// If there were any bins still open,
    /// this will close and return them.
    fn finalize(self) -> Vec<Bin<Item>>;

    /// Helper function to process an entire sequence of items
    /// and return the bins in one go.
    ///
    /// It runs [`add`](OnlinePacker::add) on each item,
    /// and then calls [`finalize`](OnlinePacker::finalize),
    /// collecting the bins in the process.
    ///
    /// Note that there might have been some items in the packer before this function is called:
    /// these will be included in the final result, along with any that came in from the `items` parameter.
    ///
    /// If an [`OnlinePackerError`] occurs in the middle of the process,
    /// this will stop iterating over the items,
    /// and return all the state we have so far.
    fn pack_all<Iterable>(
        self,
        mut items: Iterable,
    ) -> Result<Vec<Bin<Item>>, (OnlinePackerError<Item>, Self, Iterable, Vec<Bin<Item>>)>
    where
        Self: Sized,
        Iterable: Iterator<Item = Item>,
    {
        let mut closed_bins = Vec::new();
        let mut packer = self;
        while let Some(item) = items.next() {
            match packer.try_add(item) {
                Ok(closed) => closed_bins.extend(closed),
                Err(err) => return Err((err, packer, items, closed_bins)),
            }
        }

        closed_bins.extend(packer.finalize());
        Ok(closed_bins)
    }
}

/// Error returned when an item cannot be added to an online packer.
#[derive(Debug)]
pub enum OnlinePackerError<T> {
    /// The item is too big to fit into any bin that this packer can make:
    /// for example, the bins are size 10 and you're trying to pack an item of size 50.
    ItemTooLarge(T),
}
