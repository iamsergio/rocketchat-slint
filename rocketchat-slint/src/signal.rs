// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use std::{cell::RefCell, rc::Rc};

type Slot = Box<dyn Fn()>;

pub type Signal = Rc<_Signal>;

pub struct _Signal {
    connected_slots: RefCell<Vec<Slot>>,
}

pub fn new() -> Rc<_Signal> {
    Rc::new(_Signal {
        connected_slots: RefCell::new(Vec::new()),
    })
}

impl _Signal {
    pub fn connect<F>(&self, slot: F)
    where
        F: Fn() + 'static,
    {
        self.connected_slots.borrow_mut().push(Box::new(slot));
    }

    pub fn emit(&self) {
        for slot in self.connected_slots.borrow_mut().iter_mut() {
            slot();
        }
    }

    #[allow(dead_code)]
    pub fn disconnect(&self) {
        self.connected_slots.borrow_mut().clear();
    }

    #[allow(dead_code)]
    fn count(&self) -> usize {
        self.connected_slots.borrow_mut().len()
    }
}

// tests:
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn test_signal() {
        let signal = crate::signal::new();
        assert_eq!(signal.count(), 0);

        let rc = Rc::new(RefCell::new(1));
        let rc_clone = Rc::clone(&rc);
        signal.connect(move || {
            let mut x = (*rc_clone).borrow_mut();
            *x = 2;
        });

        assert_eq!(signal.count(), 1);

        signal.emit();
        assert_eq!(signal.count(), 1);
        assert_eq!(*rc.borrow(), 2);

        signal.disconnect();
        assert_eq!(signal.count(), 0);
    }
}
