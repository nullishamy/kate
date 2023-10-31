#![allow(clippy::mut_from_ref)]

use std::{marker::PhantomData, mem::ManuallyDrop};



use super::{builtins::Object, layout::FieldLocation};

#[derive(Debug)]
/// A handle to a field in an object.
/// Can be trivially cloned around for efficient references to fields.
pub struct FieldRef<T> {
    object: *mut Object,
    field: FieldLocation,
    phantom: PhantomData<T>,
}

impl<T> FieldRef<T> {
    pub fn new(object: *mut Object, field: FieldLocation) -> Self {
        Self {
            object,
            field,
            phantom: PhantomData,
        }
    }

    pub fn copy_out(&self) -> T {
        assert!(!self.object.is_null(), "cannot read from null");

        let offset = self.field.offset;
        let data_ptr = unsafe { self.object.byte_add(offset).cast::<T>() };

        unsafe { data_ptr.read() }
    }

    pub fn borrow(&self) -> &T {
        assert!(!self.object.is_null(), "cannot read from null");

        let offset = self.field.offset;
        let data_ptr = unsafe { self.object.byte_add(offset).cast::<T>() };
        unsafe { data_ptr.as_ref().unwrap() }
    }

    pub fn write(&self, value: T) {
        assert!(!self.object.is_null(), "cannot write to null");

        let offset = self.field.offset;
        let data_ptr = unsafe { self.object.byte_add(offset).cast::<T>() };
        unsafe { data_ptr.write(value) };
    }

    pub fn object(&self) -> Option<&Object> {
        assert!(!self.object.is_null(), "cannot ref to null");

        unsafe { self.object.as_ref() }
    }

    pub fn object_mut(&self) -> Option<&mut Object> {
        assert!(!self.object.is_null(), "cannot ref to null");

        unsafe { self.object.as_mut() }
    }
}

impl<T> Drop for FieldRef<T> {
    fn drop(&mut self) {
        let object = self.object_mut().unwrap();
        let binding = ManuallyDrop::new(object.class());
        let layout = ManuallyDrop::new(binding.borrow().instance_layout());

        // We are the last ref, deallocate the entire object we refer to
        if object.ref_count == 1 {
            unsafe { std::alloc::dealloc(self.object as *mut u8, layout.layout()) };
            return;
        }

        object.ref_count -= 1;
    }
}

impl<T> Clone for FieldRef<T> {
    fn clone(&self) -> Self {
        let object = self.object_mut().unwrap();
        object.ref_count += 1;

        Self {
            object: self.object,
            field: self.field,
            phantom: self.phantom,
        }
    }
}

#[derive(Debug)]
pub struct RefTo<T: HasObjectHeader<T>> {
    object: *mut Object,
    phantom: PhantomData<T>,
}

impl<T: HasObjectHeader<T>> PartialEq for RefTo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl<T: HasObjectHeader<T>> Eq for RefTo<T> {}

pub trait HasObjectHeader<T> {
    fn header(&self) -> &Object;
    fn header_mut(&mut self) -> &mut Object;
}

impl<T: HasObjectHeader<T>> RefTo<T> {
    pub fn new(object: impl HasObjectHeader<T> + 'static) -> Self {
        let b = Box::new(object);
        let leak = Box::leak::<'static>(b);
        let object = leak.header_mut();
        let ptr = object as *mut Object;

        Self {
            object: ptr,
            phantom: PhantomData,
        }
    }

    pub unsafe fn from_ptr(object_ptr: *mut Object) -> Self {
        Self {
            object: object_ptr,
            phantom: PhantomData,
        }
    }

    pub fn copy_out(&self) -> Option<T> {
        if self.object.is_null() {
            None
        } else {
            Some(unsafe { self.object.cast::<T>().read() })
        }
    }

    #[track_caller]
    pub fn borrow_mut(&self) -> &mut T {
        assert!(!self.object.is_null(), "ref was null");
        unsafe { self.object.cast::<T>().as_mut().unwrap() }
    }

    #[track_caller]
    pub fn borrow(&self) -> &T {
        assert!(!self.object.is_null(), "ref was null");
        unsafe { self.object.cast::<T>().as_ref().unwrap() }
    }

    pub fn as_ptr(&self) -> *const Object {
        self.object
    }

    pub fn is_null(&self) -> bool {
        self.object.is_null()
    }

    /// Safety: Caller must ensure object is of this type
    pub unsafe fn cast<U: HasObjectHeader<U>>(&self) -> RefTo<U> {
        RefTo {
            object: self.object,
            phantom: PhantomData,
        }
    }

    pub fn erase(self) -> RefTo<Object> {
        RefTo {
            object: self.object,
            phantom: PhantomData,
        }
    }

    pub fn null() -> Self {
        Self {
            object: std::ptr::null_mut(),
            phantom: PhantomData,
        }
    }

    pub fn into_option(&self) -> Option<&T> {
        if self.is_null() {
            None
        } else {
            Some(self.borrow())
        }
    }
}

impl<T: HasObjectHeader<T>> Clone for RefTo<T> {
    fn clone(&self) -> Self {
        // Null ptrs are all really the same.
        // They don't need ref counting.
        if self.is_null() {
            Self {
                object: self.object,
                phantom: PhantomData,
            }
        } else {
            let s = unsafe { self.object.as_mut().unwrap() };
            s.ref_count += 1;

            Self {
                object: self.object,
                phantom: PhantomData,
            }
        }
    }
}
