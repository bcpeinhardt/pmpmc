//! Priority Multi Consumer Multi Producer Channel
//!
//! This is a simple implementation of a multi producer multi consumer channel for elements that implement the Ord trait,
//! which replaces the typical rx.recv() with rx.recv_greatest(). It is basically just using the typical Arc<Mutex<>> wrapper to push
//! to a std::collections::BinaryHeap from one thread and to pop from that heap in another thread. 
//!
//! # Example
//! ```
//! use pmpmc::pmpmc;
//! let (tx, rx) = pmpmc();
//! 
//! let tx_new = tx.clone();
//! let rx_new = rx.clone();
//!
//! let _ = std::thread::spawn(move || {
//!     tx_new.send(2);
//!     tx_new.send(1);
//!     tx_new.send(3);
//! })
//! .join();
//!
//! assert_eq!(rx_new.recv_greatest(), Some(3));
//! assert_eq!(rx_new.recv_greatest(), Some(2));
//! assert_eq!(rx_new.recv_greatest(), Some(1));
//! assert_eq!(rx_new.recv_greatest(), None);
//! assert_eq!(rx_new.recv_greatest(), None);
//! ```
//!
//! I am using it for automatically scheduling tasks (workers now pull tasks off the channel in order of priority) but you can use it
//! any time you want to push to a list from one or more threads and sort the list before receiving on another thread.

// Imports
use std::sync::{Arc, Mutex};
use std::collections::BinaryHeap;

// By convention, we call this struct Inner, but it's just a place to put the Mutex to the Heap
struct Inner<T> {
    queue: Mutex<BinaryHeap<T>>,
}

/// The sender struct allows for sending items across the channel.
/// For use, see method send.
pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

// We only want to clone the Arc, the inner needs to be shared
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: std::cmp::Ord> Sender<T> {
    /// The send method allows for sending items across the channel.
    ///
    /// # Example
    /// ```
    /// use pmpmc::pmpmc;
    /// let (tx, rx) = pmpmc();
    ///
    /// assert_eq!(tx.send(3), ());
    /// assert_eq!(tx.send(1), ());
    /// assert_eq!(tx.send(2), ());
    /// ```
    pub fn send(&self, item: T) {
        self.inner.queue.lock().expect("Poison error").push(item);
    }
}

/// The receiver struct allows for receiving items across the channel.
/// For use, see method recv_greatest().
pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: std::cmp::Ord> Receiver<T> {
    /// Sorts the elements in the channel and returns Some(greatest) or a None if the channel is empty
    /// # Example
    /// ```
    /// use pmpmc::pmpmc;
    /// let (tx, rx) = pmpmc();
    ///
    /// assert_eq!(tx.send(3), ());
    /// assert_eq!(tx.send(1), ());
    /// assert_eq!(tx.send(2), ());
    /// assert_eq!(rx.recv_greatest(), Some(3));
    /// assert_eq!(rx.recv_greatest(), Some(2));
    /// assert_eq!(rx.recv_greatest(), Some(1));
    /// ```
    pub fn recv_greatest(&self) -> Option<T> {
        let mut queue = self.inner.queue.lock().expect("Poison error");
        queue.pop()
    }
}

/// The pmpmc function works like most channel generating functions. It returns a tuple with a sender and a receiver
/// from which the user can clone as many copies as they need
/// # Example
/// ```
/// use pmpmc::pmpmc;
/// let (tx, rx) = pmpmc();
/// tx.send(3);
/// assert_eq!(rx.recv_greatest(), Some(3));
/// ```
/// The compiler will infer the type once an item is sent.
pub fn pmpmc<T: std::cmp::Ord>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::new(BinaryHeap::new()),
    };
    let shared_inner = Arc::new(inner);
    (
        Sender {
            inner: shared_inner.clone(),
        },
        Receiver {
            inner: shared_inner.clone(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn front_page_test() {
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
    }

    #[test]
    fn basic_functionality() {
        let (tx, rx) = pmpmc();
        assert_eq!(tx.send(3), ());
        assert_eq!(tx.send(1), ());
        assert_eq!(tx.send(2), ());
        assert_eq!(rx.recv_greatest(), Some(3));
        assert_eq!(rx.recv_greatest(), Some(2));
        assert_eq!(rx.recv_greatest(), Some(1));
    }

    struct TestStruct {
        matters: u32,
        _does_not_matter: u32,
    }
    impl PartialEq for TestStruct {
        fn eq(&self, other: &Self) -> bool {
            self.matters == other.matters
        }
    }
    impl Eq for TestStruct {}

    impl PartialOrd for TestStruct {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.matters.partial_cmp(&other.matters)
        }
    }
    impl Ord for TestStruct {
        fn cmp(&self, other: &Self) -> Ordering {
            self.matters.cmp(&other.matters)
        }
    }

    #[test]
    fn works_for_struct_with_custom_ordering() {
        let first = TestStruct {
            matters: 1,
            _does_not_matter: 3,
        };
        let second = TestStruct {
            matters: 2,
            _does_not_matter: 1,
        };
        let third = TestStruct {
            matters: 3,
            _does_not_matter: 2,
        };

        let (tx, rx) = pmpmc();
        assert_eq!(tx.send(second), ());
        assert_eq!(tx.send(third), ());
        assert_eq!(tx.send(first), ());

        assert_eq!(rx.recv_greatest().unwrap().matters, 3);
        assert_eq!(rx.recv_greatest().unwrap().matters, 2);
        assert_eq!(rx.recv_greatest().unwrap().matters, 1);
    }

    #[test]
    fn send_from_different_thread() {
        let (tx, rx) = pmpmc();
        let tx1 = tx.clone();
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let handle1 = std::thread::spawn(move || {
            tx1.send(5);
        });
        let handle2 = std::thread::spawn(move || {
            tx2.send(4);
        });
        let handle3 = std::thread::spawn(move || {
            tx3.send(6);
        });
        let _ = handle1.join();
        let _ = handle2.join();
        let _ = handle3.join();
        assert_eq!(rx.recv_greatest(), Some(6));
        assert_eq!(rx.recv_greatest(), Some(5));
        assert_eq!(rx.recv_greatest(), Some(4));
    }

    #[test]
    fn receive_in_different_threads() {
        let (tx, rx) = pmpmc();
        let rx1 = rx.clone();
        let rx2 = rx.clone();
        let rx3 = rx.clone();

        tx.send(5);
        tx.send(7);
        tx.send(6);

        let handle1 = std::thread::spawn(move || {
            assert_eq!(rx1.recv_greatest(), Some(7));
        });
        let _ = handle1.join();
        let handle2 = std::thread::spawn(move || {
            assert_eq!(rx2.recv_greatest(), Some(6));
        });
        let _ = handle2.join();
        let handle3 = std::thread::spawn(move || {
            assert_eq!(rx3.recv_greatest(), Some(5));
        });
        let _ = handle3.join();
    }
}
