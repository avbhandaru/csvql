// EvictingList (evl)
pub trait EvictingList {
  type Element;

  // Constructs a new EvictingList
  fn evl_new(n: usize) -> Self;

  // Adds a new element to an EvictingList
  fn evl_add(&mut self, elt: Self::Element);

  // Peeks the most recently added element of an EvictingList
  fn evl_peek(&self) -> Option<&Self::Element>;

  // Scoops the oldest element of an EvictingList
  fn evl_scoop(&mut self) -> Option<Self::Element>;
  // Gets an element of an EvictingList given its index
  fn evl_get(&self, index: usize) -> Option<&Self::Element>;
}

impl<T> EvictingList for std::collections::VecDeque<T> {
  type Element = T;

  fn evl_new(capacity: usize) -> Self {
    std::collections::VecDeque::with_capacity(capacity)
  }

  fn evl_add(&mut self, element: Self::Element) {
    if self.len() >= self.capacity() {
      self.pop_front();
    }
    self.push_back(element);
  }

  fn evl_peek(&self) -> Option<&Self::Element> {
    self.back()
  }

  fn evl_scoop(&mut self) -> Option<Self::Element> {
    self.pop_front()
  }

  fn evl_get(&self, index: usize) -> Option<&Self::Element> {
    self.get(index)
  }
}
