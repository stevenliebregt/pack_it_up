# pack_it_up

[![Crates.io](https://img.shields.io/crates/v/pack_it_up)](https://crates.io/crates/pack_it_up)
[![docs.rs](https://img.shields.io/docsrs/pack_it_up)](https://docs.rs/pack_it_up/)

pack_it_up is a simple Rust library that implements various bin packing algorithms

### Current implemented algorithms

- First-fit
- First-fit-decreasing

### Basic example

```rust
use pack_it_up::offline::first_fit_decreasing::first_fit_decreasing;

struct MyItem {
    some_content: i32,
    size: usize,
}

impl Pack for MyItem {
    fn size(&self) -> usize {
        self.size
    }
}

fn main() {
    let my_items = vec![
        MyItem { some_content: 1, size: 1, },
        MyItem { some_content: 2, size: 2, },
        MyItem { some_content: 3, size: 19, },
        MyItem { some_content: 4, size: 17, },
        MyItem { some_content: 5, size: 1, }, 
    ];
    
    let mut bins = first_fit_decreasing(20, my_items);
}
```

The above will result in 2 full bins, one with sizes 19 and 1, and the other with sizes 17, 2 and 1.

### Planned features

- Remaining algorithms
- Performance optimizations
- Simple derive for Pack if your struct already has a field called size