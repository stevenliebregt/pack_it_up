use crate::online::first_fit::__internal_first_fit;
use crate::wrapper::SizedWrapper;
use crate::{Bin, Pack};

/// Pack items in bins using the [First-fit-decreasing](https://en.wikipedia.org/wiki/First-fit-decreasing_bin_packing)
/// bin packing algorithm.
pub fn first_fit_decreasing<T>(bin_size: usize, mut items: Vec<T>) -> Vec<Bin<T>>
where
    T: Pack,
{
    assert!(bin_size > 0, "Bin size must be greater than 0");

    // Sort the items in decreasing order
    // TODO: the following line could have been possibly replaced by
    //   items.sort_unstable_by_key(Pack::size);
    // but doing that somehow breaks the ordering
    // that this function requires to give the correct answer?!?!
    #[allow(clippy::unnecessary_sort_by)]
    items.sort_unstable_by(|a, b| b.size().cmp(&a.size()));

    let lower_bound: usize = ((items.iter().map(|item| item.size()).sum::<usize>() as f64)
        / (bin_size as f64))
        .ceil() as usize;

    // Use the normal first fit implementation
    __internal_first_fit(bin_size, items, lower_bound)
}

/// Pack items in bins using the [First-fit-decreasing](https://en.wikipedia.org/wiki/First-fit-decreasing_bin_packing)
/// bin packing algorithm.
///
/// Unlike [`first_fit_decreasing`], the items don't have to implement [`Pack`].
/// Instead, you need to provide a function that returns the size of the item.
///
/// This function will be cloned for each item
/// (but if it's a simple function pointer or a non-capturing closure, then it is a no-op).
pub fn first_fit_decreasing_by_key<T, SizeFunc>(
    bin_size: usize,
    items: Vec<T>,
    key_func: SizeFunc,
) -> Vec<Bin<T>>
where
    SizeFunc: Fn(&T) -> usize + Clone,
{
    assert!(bin_size > 0, "Bin size must be greater than 0");

    // Wrap items in a SizedWrapper with the key function
    // This should be a low-to-no-impact operation if the key function is Copy
    // (because SizedWrapper is a zero-overhead struct in that case)
    let mut items: Vec<_> = items
        .into_iter()
        .map(|item| SizedWrapper::new(key_func.clone(), item))
        .collect();

    // Sort the items in decreasing order
    // TODO: the following line could have been possibly replaced by
    //   items.sort_unstable_by_key(Pack::size);
    // but doing that somehow breaks the ordering
    // that this function requires to give the correct answer?!?!
    #[allow(clippy::unnecessary_sort_by)]
    items.sort_unstable_by(|a, b| b.size().cmp(&a.size()));

    let lower_bound: usize = ((items.iter().map(|item| item.size()).sum::<usize>() as f64)
        / (bin_size as f64))
        .ceil() as usize;

    // Use the normal first fit implementation
    __internal_first_fit(bin_size, items, lower_bound)
        .into_iter()
        .map(|bin| bin.map(|item| item.take()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{generate_test_bins, generate_test_set_a};

    #[test]
    fn it_works() {
        let (test_data, bin_size) = generate_test_set_a();

        let result = first_fit_decreasing(bin_size, test_data);

        // First fit decreasing would result in the optimal solution

        let expected = generate_test_bins(
            20,
            vec![
                vec![19, 1],          // 20
                vec![19, 1],          // 20
                vec![10, 10],         // 20
                vec![10, 4, 3, 1, 1], //19
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

        let result = first_fit_decreasing_by_key(bin_size, test_data, |item| item.size);

        // First fit decreasing by key would result in the optimal solution

        let expected: Vec<_> = generate_test_bins(
            20,
            vec![
                vec![19, 1],          // 20
                vec![19, 1],          // 20
                vec![10, 10],         // 20
                vec![10, 4, 3, 1, 1], //19
            ],
        )
        .into_iter()
        .map(|bin| bin.map(|item| item.make_unpacked()))
        .collect();

        assert_eq!(expected, result)
    }
}
