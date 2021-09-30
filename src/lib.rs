use std::sync::{Arc, Mutex};

struct Inner<T> {
    queue: Mutex<Vec<T>>,
}

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            inner: Arc::clone(&self.inner)
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) {
        self.inner.queue.lock().expect("Poison error").push(item);
    }
}


pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            inner: Arc::clone(&self.inner)
        }
    }
}

impl<T: std::cmp::Ord> Receiver<T> {
    pub fn recv_greatest(&self) -> Option<T> {
        let mut queue = self.inner.queue.lock().expect("Poison error");
        queue.sort();
        queue.pop()
    }

    pub fn recv_least(&self) -> Option<T> {
        let mut queue = self.inner.queue.lock().expect("Poison error");
        queue.sort();
        queue.reverse();
        queue.pop()
    }
}

pub fn pmpmc<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::new(Vec::new()),
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
    fn basic_functionality() {
        let (tx, rx) = pmpmc();
        assert_eq!(tx.send(3), ());
        assert_eq!(tx.send(1), ());
        assert_eq!(tx.send(2), ());
        assert_eq!(rx.recv_greatest(), Some(3));
        assert_eq!(rx.recv_greatest(), Some(2));
        assert_eq!(rx.recv_greatest(), Some(1));

        assert_eq!(tx.send(3), ());
        assert_eq!(tx.send(1), ());
        assert_eq!(tx.send(2), ());
        assert_eq!(rx.recv_least(), Some(1));
        assert_eq!(rx.recv_least(), Some(2));
        assert_eq!(rx.recv_least(), Some(3));
    }

    
    struct TestStruct {
        matters: u32,
        _does_not_matter: u32
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
        let first = TestStruct { matters: 1, _does_not_matter: 3};
        let second = TestStruct { matters: 2, _does_not_matter: 1};
        let third = TestStruct { matters: 3, _does_not_matter: 2};

        let (tx, rx) = pmpmc();
        assert_eq!(tx.send(second), ());
        assert_eq!(tx.send(third), ());
        assert_eq!(tx.send(first), ());

        assert_eq!(rx.recv_greatest().unwrap().matters, 3);
        assert_eq!(rx.recv_greatest().unwrap().matters, 2);
        assert_eq!(rx.recv_greatest().unwrap().matters, 1);

        let first = TestStruct { matters: 1, _does_not_matter: 3};
        let second = TestStruct { matters: 2, _does_not_matter: 1};
        let third = TestStruct { matters: 3, _does_not_matter: 2};

        let (tx, rx) = pmpmc();
        assert_eq!(tx.send(second), ());
        assert_eq!(tx.send(third), ());
        assert_eq!(tx.send(first), ());

        assert_eq!(rx.recv_least().unwrap().matters, 1);
        assert_eq!(rx.recv_least().unwrap().matters, 2);
        assert_eq!(rx.recv_least().unwrap().matters, 3);
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
        assert_eq!(rx.recv_least(), Some(4));
        assert_eq!(rx.recv_least(), Some(5));
        assert_eq!(rx.recv_least(), Some(6));
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
