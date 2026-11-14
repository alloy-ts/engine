use super::value::StringHandle;

/// Per-Realm storage for string contents.
///
/// The MVP never frees strings while the Realm is alive (there is no GC); the
/// whole heap is released when its Realm is dropped or its context disposed.
#[derive(Debug, Default)]
pub struct Heap {
    strings: Vec<Box<str>>,
}

impl Heap {
    pub fn alloc_string(&mut self, contents: impl Into<Box<str>>) -> StringHandle {
        self.strings.push(contents.into());
        StringHandle(self.strings.len() - 1)
    }

    /// Contents of a string allocated by this heap.
    ///
    /// Handles are only created by [`Heap::alloc_string`] and Values never
    /// cross Realms, so indexing cannot fail for a handle from this heap.
    pub fn string(&self, handle: StringHandle) -> &str {
        &self.strings[handle.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_round_trip_through_handles() {
        let mut heap = Heap::default();
        let a = heap.alloc_string("héllo");
        let b = heap.alloc_string(String::new());
        assert_ne!(a, b);
        assert_eq!(heap.string(a), "héllo");
        assert_eq!(heap.string(b), "");
    }
}
