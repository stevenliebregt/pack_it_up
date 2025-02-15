use crate::{wrapper::SizedWrapper, Bin, Pack};

/// Pack items in bins using the [First-fit](https://en.wikipedia.org/wiki/First-fit_bin_packing)
/// bin packing algorithm.
pub fn first_fit<T>(bin_size: usize, items: impl IntoIterator<Item = T>) -> Vec<Bin<T>>
where
    T: Pack,
{
    assert!(bin_size > 0, "Bin size must be greater than 0");

    __internal_first_fit(bin_size, items, 1)
}

/// Pack items in bins using the [First-fit](https://en.wikipedia.org/wiki/First-fit_bin_packing)
/// bin packing algorithm.
///
/// Unlike [`first_fit`], the items don't have to implement [`Pack`].
/// Instead, you need to provide a function that returns the size of the item.
///
/// This function will be cloned for each item
/// (but if it's a simple function pointer or a non-capturing closure, then it is a no-op).
pub fn first_fit_by_key<T, SizeFunc>(
    bin_size: usize,
    items: impl IntoIterator<Item = T>,
    key_func: SizeFunc,
) -> Vec<Bin<T>>
where
    SizeFunc: Fn(&T) -> usize + Clone,
{
    assert!(bin_size > 0, "Bin size must be greater than 0");

    __internal_first_fit(
        bin_size,
        items
            .into_iter()
            .map(|item| SizedWrapper::new(key_func.clone(), item)),
        1,
    )
    .into_iter()
    .map(|bin| bin.map(|item| item.take()))
    .collect()
}

#[doc(hidden)]
pub(crate) fn __internal_first_fit<T>(
    bin_size: usize,
    items: impl IntoIterator<Item = T>,
    lower_bound: usize,
) -> Vec<Bin<T>>
where
    T: Pack,
{
    // Initialize bins
    let mut bins = Vec::<Bin<T>>::with_capacity(lower_bound);
    bins.push(Bin::with_capacity(bin_size));

    for item in items.into_iter() {
        // Find the first bin that the item fits in
        match bins
            .iter_mut()
            .find(|bin| item.size() <= bin.remaining_capacity)
        {
            Some(bin) => bin.add(item),
            None => bins.push(Bin::with_item(bin_size, item)),
        }

        // TODO: Should be move bins that are full to a new vector to avoid having to iterate them?
    }

    bins
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{generate_test_bins, generate_test_set_a};

    #[test]
    fn it_works() {
        let (test_data, bin_size) = generate_test_set_a();

        let result = first_fit(bin_size, test_data);

        // First fit does not give an optimal solution

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

        assert_eq!(expected, result)
    }

    #[test]
    fn it_works_by_key() {
        let (test_data, bin_size) = generate_test_set_a();

        let test_data = test_data
            .into_iter()
            .map(|item| item.make_unpacked())
            .collect::<Vec<_>>();

        let result = first_fit_by_key(bin_size, test_data, |item| item.size);

        // First fit does not give an optimal solution

        let expected: Vec<_> = generate_test_bins(
            20,
            vec![
                vec![1, 1, 1, 1, 3, 4], // 11
                vec![10, 10],           // 20
                vec![10],               // 10
                vec![19],               // 19
                vec![19],               // 19
            ],
        )
        .into_iter()
        .map(|bin| bin.map(|item| item.make_unpacked()))
        .collect();

        assert_eq!(expected, result)
    }
}
