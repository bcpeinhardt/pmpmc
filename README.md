# pmpmc

Priority Multi Consumer Multi Producer Channel

This is a quick and dirty implementation of a multi producer multi consumer channel for elements that implement the Ord trait,
which gives the user two methods on the receiver: recv_greatest and recv_least.

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

let tx_new = tx.clone();
let rx_new = rx.clone();

let _ = std::thread::spawn(move || {
    tx_new.send(2);
    tx_new.send(1);
    tx_new.send(3);
})
.join();

assert_eq!(rx_new.recv_least(), Some(1));
assert_eq!(rx_new.recv_least(), Some(2));
assert_eq!(rx_new.recv_least(), Some(3));
assert_eq!(rx_new.recv_least(), None);
assert_eq!(rx_new.recv_least(), None);
```

I am using it for automatically scheduling tasks (workers now pull tasks off the channel in order of priority) but you can use it
any time you want to push to a list from one or more threads and sort the list before receiving on another thread.

This crate is not optimized in any way, please don't use it in production. When I have time I may come back and make it more efficient
and robust.

License: MIT
