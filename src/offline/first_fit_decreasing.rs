use crate::online::first_fit::__internal_first_fit;
use crate::{Bin, Pack};

/// Pack items in bins using the [First-fit-decreasing](https://en.wikipedia.org/wiki/First-fit-decreasing_bin_packing)
/// bin packing algorithm.
pub fn first_fit_decreasing<T>(bin_size: usize, mut items: Vec<T>) -> Vec<Bin<T>>
where
    T: Pack,
{
    assert!(bin_size > 0, "Bin size must be greater than 0");

    // Sort the items in decreasing order
    items.sort_unstable_by(|a, b| b.size().cmp(&a.size()));

    let lower_bound: usize = ((items.iter().map(|item| item.size()).sum::<usize>() as f64)
        / (bin_size as f64))
        .ceil() as usize;

    // Use the normal first fit implementation
    __internal_first_fit(bin_size, items, lower_bound)
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
}
