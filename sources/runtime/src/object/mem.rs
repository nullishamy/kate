#![allow(clippy::mut_from_ref)]

use std::{marker::PhantomData, mem::ManuallyDrop};



use super::{builtins::Object, layout::FieldLocation};

#[derive(Debug)]
/// A handle to a field in an object.
/// Can be trivially cloned around for efficient references to fields.
pub struct FieldRef<T> {
    object: *const Object,
    field: FieldLocation,
    phantom: PhantomData<T>,
}

impl<T: Copy> FieldRef<T> {
    pub fn copy_out(&self) -> T {
        *self.unwrap_ref()
    }
}

impl<T> FieldRef<T> {
    pub fn new(object: *const Object, field: FieldLocation) -> Self {
        Self {
            object,
            field,
            phantom: PhantomData,
        }
    }

    pub fn unwrap_ref(&self) -> &T {
        assert!(!self.object.is_null(), "cannot read from null");

        let offset = self.field.offset;
        let data_ptr = unsafe { self.object.byte_add(offset).cast::<T>() };
        unsafe { data_ptr.as_ref().unwrap() }
    }

    pub fn write(&self, value: T) {
        assert!(!self.object.is_null(), "cannot write to null");
        let object = self.object().unwrap();
        let _field_lock = object.lock.write();

        let offset = self.field.offset;
        let data_ptr = unsafe { self.object.byte_add(offset).cast::<T>() };

        // SAFETY:
        // No other aliases can exist at this time because we are holding the write lock
        unsafe { data_ptr.cast_mut().write(value) };
    }

    pub fn object(&self) -> Option<&Object> {
        unsafe { self.object.as_ref() }
    }
}

impl<T> Drop for FieldRef<T> {
    fn drop(&mut self) {
        let object = self.object().unwrap();
        let binding = ManuallyDrop::new(object.class());
        let layout = ManuallyDrop::new(binding.unwrap_ref().instance_layout());

        // We are the last ref, deallocate the entire object we refer to
        if object.ref_count() == 1 {
            unsafe { std::alloc::dealloc(self.object as *mut u8, layout.layout()) };
            return;
        }

        object.dec_count();
    }
}

impl<T> Clone for FieldRef<T> {
    fn clone(&self) -> Self {
        let object = self.object().unwrap();
        object.inc_count();

        Self {
            object: self.object,
            field: self.field,
            phantom: self.phantom,
        }
    }
}

#[derive(Debug)]
pub struct RefTo<T: JavaObject<T>> {
    object: *mut Object,
    phantom: PhantomData<T>,
}

unsafe impl <T : JavaObject<T>> Send for RefTo<T> {}

impl<T: JavaObject<T>> PartialEq for RefTo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl<T: JavaObject<T>> Eq for RefTo<T> {}

pub trait JavaObject<T> {
    fn header(&self) -> &Object;
    fn header_mut(&mut self) -> &mut Object;
    fn type_name() -> &'static str;
}

impl<T: JavaObject<T> + Copy> RefTo<T> {
    pub fn copy_out(&self) -> T {
        *self.unwrap_ref()
    }
}

impl<T: JavaObject<T>> RefTo<T> {
    pub fn new(object: impl JavaObject<T> + 'static) -> Self {
        let b = Box::new(object);
        let leak = Box::leak::<'static>(b);
        let object = leak.header_mut();
        let ptr = object as *mut Object;

        Self {
            object: ptr,
            phantom: PhantomData,
        }
    }

    /// ## Safety
    ///
    /// Caller must ensure the pointer points to a valid heap allocated object.
    /// The object must be brand new, not a pointer to an pre-existing allocation
    /// This function will take ownership of the pointer. It is up to callers not
    /// to use it after this function is called.
    pub unsafe fn from_ptr(object_ptr: *mut Object) -> Self {
        Self {
            object: object_ptr,
            phantom: PhantomData,
        }
    }

    pub fn with_lock<U>(&self, func: impl FnOnce(&mut T) -> U) -> U {
        if self.is_null() {
            panic!("attempted to lock null");
        }

        // FIXME: Verify that this actually obeys the strict aliasing rules. Not sure if `self.unwrap_mut()` is allowed here.
        let object = unsafe { self.as_ptr().as_ref().unwrap() };
        // FIXME: Don't immediately acquire once we go into MT land
        let _lock = object.lock.try_write().expect("could not acquire lock, presumed deadlock.");

        func(self.unwrap_mut())
    }

    #[track_caller]
    pub fn unwrap_mut(&self) -> &mut T {
        self.to_mut().expect("attempted to dereference null")
    }

    #[track_caller]
    pub fn unwrap_ref(&self) -> &T {
        self.to_ref().expect("attempted to dereference null")
    }

    pub fn to_ref(&self) -> Option<&T> {
        unsafe { self.object.cast::<T>().as_ref() }
    }

    pub fn to_mut(&self) -> Option<&mut T> {
        unsafe { self.object.cast::<T>().as_mut() }
    }

    pub fn as_ptr(&self) -> *const Object {
        self.object
    }

    pub fn is_null(&self) -> bool {
        self.object.is_null()
    }

    pub fn is_not_null(&self) -> bool {
        !self.is_null()
    }

    /// ## Safety
    ///
    /// Caller must ensure object is of this type
    #[track_caller]
    pub unsafe fn cast<U: JavaObject<U>>(&self) -> RefTo<U> {
        if let Some(obj) = self.to_ref() {
            // Type name must exactly match for builtins, for now. We don't want to deal with subclassing layout nonsense
            let to_type_name = &U::type_name().to_string();

            let from_cls = obj.header().class();

            // FIXME: Not sure if this is the right path if we cannot determine a source class? 
            if from_cls.is_null() {
                return RefTo {
                    object: self.object,
                    phantom: PhantomData,
                }
            }

            let from_type_name = from_cls.unwrap_ref().name();

            // TODO: Type check array components
            if to_type_name == "array" {
                if !from_cls.unwrap_ref().is_array() {
                    panic!(
                        "attempted to cast non array type {} to array",
                        from_type_name
                    );
                } else {
                    return RefTo {
                        object: self.object,
                        phantom: PhantomData,
                    }
                }
            }

            if to_type_name != from_type_name {
                panic!(
                    "attempted invalid cast from {} to {}",
                    from_type_name, to_type_name
                );
            }

            RefTo {
                object: self.object,
                phantom: PhantomData,
            }
        } else {
            RefTo {
                object: self.object,
                phantom: PhantomData,
            }
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

    pub fn garbage() -> Self {
        Self {
            object: 0xFFFFFFFFFFFF_u64 as *mut Object,
            phantom: PhantomData,
        }
    }

    pub fn into_option(&self) -> Option<&T> {
        if self.is_null() {
            None
        } else {
            Some(self.unwrap_ref())
        }
    }
}

impl<T: JavaObject<T>> Clone for RefTo<T> {
    fn clone(&self) -> Self {
        // Null ptrs are all really the same.
        // They don't need ref counting.
        if self.is_null() {
            Self {
                object: self.object,
                phantom: PhantomData,
            }
        } else {
            let s = unsafe { self.object.as_ref().unwrap() };
            s.inc_count();

            Self {
                object: self.object,
                phantom: PhantomData,
            }
        }
    }
}
