use std::ops::{Deref, DerefMut};

/// This struct wraps an item with a function that returns the size of the item.
///
/// This is an alternative to implementing the [`crate::Pack`] trait on the type:
/// instead, you can transform your collection into a collection of [`SizedWrapper`]s
/// before passing it to one of the packing functions,
/// or else use the `by_key` versions of the packing functions
/// (which are implemented using this wrapper).
pub struct SizedWrapper<SizeFunc, T>
where
    SizeFunc: Fn(&T) -> usize,
{
    pub key_func: SizeFunc,
    pub item: T,
}

impl<SizeFunc, T> SizedWrapper<SizeFunc, T>
where
    SizeFunc: Fn(&T) -> usize,
{
    pub fn new(key_func: SizeFunc, item: T) -> Self {
        Self { key_func, item }
    }

    pub fn take(self) -> T {
        self.item
    }
}

impl<SizeFunc, T> crate::Pack for SizedWrapper<SizeFunc, T>
where
    SizeFunc: Fn(&T) -> usize,
{
    fn size(&self) -> usize {
        (self.key_func)(&self.item)
    }
}

impl<F, T> Deref for SizedWrapper<F, T>
where
    F: Fn(&T) -> usize,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<F, T> DerefMut for SizedWrapper<F, T>
where
    F: Fn(&T) -> usize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure that the wrapper struct does not introduce any memory overhead
    /// when the key function is a simple function pointer
    /// or a non-capturing closure.
    #[test]
    fn check_size_eq() {
        let original = vec![1, 2, 3];
        let wrapper = SizedWrapper::new(Vec::len, original.clone());

        assert_eq!(
            std::mem::size_of_val(&original),
            std::mem::size_of_val(&wrapper)
        );

        let wrapper_weird = SizedWrapper::new(|_| 1, original.clone());

        assert_eq!(
            std::mem::size_of_val(&original),
            std::mem::size_of_val(&wrapper_weird)
        );
    }

    /// Ensure that when the key function captures some data,
    /// that the wrapper struct's size is a simple combination of the size of the captured data
    /// and whatever it's wrapping.
    #[test]
    fn check_size_when_combining() {
        let original = vec![1, 2, 3];
        let (sender, _recv) = std::sync::mpsc::channel();
        let sender_size = std::mem::size_of_val(&sender);

        let key_function = move |item: &Vec<i32>| {
            // naughty: we're sending a copy of the vector into the channel
            sender.send(item.to_vec().clone()).unwrap();
            item.iter().map(|v| *v as usize).sum()
        };

        let wrapper = SizedWrapper::new(key_function, original.clone());

        // The sender has some size...
        assert_ne!(sender_size, 0);

        // and the wrapper's size is equal to the size of the sender, and the size of the `original` Vec.
        // This is because the wrapper does not actually contain the function pointer,
        // it only contains `sender` and `original`;
        // the function pointer is associated with it implicitly by the type system.
        assert_eq!(
            std::mem::size_of_val(&original) + sender_size,
            std::mem::size_of_val(&wrapper)
        );
    }
}
