# pmpmc

Priority Multi Consumer Multi Producer Channel

This is a simple implementation of a multi producer multi consumer channel for elements that implement the Ord trait,
which replaces the typical rx.recv() with rx.recv_greatest(). It is basically just using the typical Arc<Mutex<>> wrapper to push
to a std::collections::BinaryHeap from one thread and to pop from that heap in another thread.

## Example
```rust
use pmpmc::pmpmc;
let (tx, rx) = pmpmc();

let tx_new = tx.clone();
let rx_new = rx.clone();

let _ = std::thread::spawn(move || {
    tx_new.send(2);
    tx_new.send(1);
    tx_new.send(3);
})
.join();

assert_eq!(rx_new.recv_greatest(), Some(3));
assert_eq!(rx_new.recv_greatest(), Some(2));
assert_eq!(rx_new.recv_greatest(), Some(1));
assert_eq!(rx_new.recv_greatest(), None);
assert_eq!(rx_new.recv_greatest(), None);
```

I am using it for automatically scheduling tasks (workers now pull tasks off the channel in order of priority) but you can use it
any time you want to push to a list from one or more threads and sort the list before receiving on another thread.

License: MIT
