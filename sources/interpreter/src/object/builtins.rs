use std::{
    alloc::Layout,
    collections::HashMap,
    fmt,
    marker::PhantomData,
    mem::{offset_of, size_of},
};

use parse::{
    classfile::ClassFile,
    flags::{ClassFileAccessFlag, ClassFileAccessFlags},
};
use support::encoding::{decode_string, CompactEncoding};

use crate::{
    error::Throwable,
    internal,
    native::{NameAndDescriptor, NativeFunction},
};

use super::{
    layout::{types, ClassFileLayout, FieldInfo},
    mem::{FieldRef, HasObjectHeader, RefTo},
};

#[repr(C)]
#[derive(Debug)]
pub struct Object {
    pub class: RefTo<Class>,
    pub super_class: RefTo<Class>,
    pub ref_count: u64,
}

impl Object {
    pub fn field<T>(&mut self, field: NameAndDescriptor) -> Option<FieldRef<T>> {
        self.ref_count += 1;
        let class = self.class();
        let layout = &class.borrow().layout_for_instances_of_this_class;
        let field_info = layout.field_info(&field.0)?;

        let location = field_info.location;

        Some(FieldRef::new(self, location))
    }

    pub fn class(&self) -> RefTo<Class> {
        self.class.clone()
    }

    pub fn super_class(&self) -> RefTo<Class> {
        self.super_class.clone()
    }

    pub fn ref_count(&self) -> u64 {
        self.ref_count
    }

    pub fn inc_count(&mut self) {
        self.ref_count += 1;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayPrimitive {
    Bool,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
}

impl ArrayPrimitive {
    pub fn from_tag(tag: u8) -> Result<Self, Throwable> {
        Ok(match tag {
            4 => Self::Bool,
            5 => Self::Char,
            6 => Self::Float,
            7 => Self::Double,
            8 => Self::Byte,
            9 => Self::Short,
            10 => Self::Int,
            11 => Self::Long,
            _ => return Err(internal!("unknown array type {}", tag)),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayType {
    Object(RefTo<Class>),
    Primitive(ArrayPrimitive),
}

#[repr(C)]
pub struct Class {
    object: Object,

    // Java's fields. For now we do not use them, so they will all be null
    cached_constructor: RefTo<Object>,
    class_name: RefTo<Object>,
    module: RefTo<Object>,
    class_loader: RefTo<Object>,
    class_data: RefTo<Object>,
    package_name: RefTo<Object>,
    _component_type: RefTo<Object>,
    reflection_data: RefTo<Object>,
    class_redefined_count: i32,
    generic_info: RefTo<Object>,
    enum_constants: RefTo<Array<RefTo<Object>>>,
    enum_constant_directory: RefTo<Object>,
    annotation_data: RefTo<Object>,
    annotation_type: RefTo<Object>,
    class_value_map: RefTo<Object>,

    // Our extra fields
    name: String,
    component_type: Option<ArrayType>,
    native_methods: HashMap<NameAndDescriptor, NativeFunction>,
    classfile: Option<ClassFile>,
    is_initialised: bool,
    layout_for_instances_of_this_class: ClassFileLayout,
}

impl Class {
    pub fn new(
        object: Object,
        name: String,
        class_file: ClassFile,
        layout: ClassFileLayout,
    ) -> Self {
        Self {
            object,

            cached_constructor: RefTo::null(),
            class_name: RefTo::null(),
            module: RefTo::null(),
            class_loader: RefTo::null(),
            class_data: RefTo::null(),
            package_name: RefTo::null(),
            _component_type: RefTo::null(),
            reflection_data: RefTo::null(),
            class_redefined_count: 0,
            generic_info: RefTo::null(),
            enum_constants: RefTo::null(),
            enum_constant_directory: RefTo::null(),
            annotation_data: RefTo::null(),
            annotation_type: RefTo::null(),
            class_value_map: RefTo::null(),

            component_type: None,
            name,
            native_methods: HashMap::new(),
            classfile: Some(class_file),
            is_initialised: false,
            layout_for_instances_of_this_class: layout,
        }
    }

    pub fn new_primitive(object: Object, name: String, layout: ClassFileLayout) -> Self {
        Self {
            object,

            cached_constructor: RefTo::null(),
            class_name: RefTo::null(),
            module: RefTo::null(),
            class_loader: RefTo::null(),
            class_data: RefTo::null(),
            package_name: RefTo::null(),
            _component_type: RefTo::null(),
            reflection_data: RefTo::null(),
            class_redefined_count: 0,
            generic_info: RefTo::null(),
            enum_constants: RefTo::null(),
            enum_constant_directory: RefTo::null(),
            annotation_data: RefTo::null(),
            annotation_type: RefTo::null(),
            class_value_map: RefTo::null(),

            component_type: None,
            name,
            native_methods: HashMap::new(),
            classfile: None,
            is_initialised: false,
            layout_for_instances_of_this_class: layout,
        }
    }

    pub fn new_array(
        object: Object,
        ty_name: String,
        ty: ArrayType,
        layout: ClassFileLayout,
    ) -> Self {
        Self {
            object,

            cached_constructor: RefTo::null(),
            class_name: RefTo::null(),
            module: RefTo::null(),
            class_loader: RefTo::null(),
            class_data: RefTo::null(),
            package_name: RefTo::null(),
            _component_type: RefTo::null(),
            reflection_data: RefTo::null(),
            class_redefined_count: 0,
            generic_info: RefTo::null(),
            enum_constants: RefTo::null(),
            enum_constant_directory: RefTo::null(),
            annotation_data: RefTo::null(),
            annotation_type: RefTo::null(),
            class_value_map: RefTo::null(),

            component_type: Some(ty),
            name: ty_name,
            native_methods: HashMap::new(),
            classfile: None,
            is_initialised: false,
            layout_for_instances_of_this_class: layout,
        }
    }

    pub fn instance_layout(&self) -> &ClassFileLayout {
        &self.layout_for_instances_of_this_class
    }

    pub fn static_field_info(&self, field: NameAndDescriptor) -> Option<&FieldInfo> {
        self.layout_for_instances_of_this_class
            .static_field_info(&field.0)
    }

    pub fn static_field_info_mut(&mut self, field: NameAndDescriptor) -> Option<&mut FieldInfo> {
        self.layout_for_instances_of_this_class
            .static_field_info_mut(&field.0)
    }

    pub fn native_methods(&self) -> &HashMap<NameAndDescriptor, NativeFunction> {
        &self.native_methods
    }

    pub fn native_methods_mut(&mut self) -> &mut HashMap<NameAndDescriptor, NativeFunction> {
        &mut self.native_methods
    }

    pub fn class_file(&self) -> &ClassFile {
        self.classfile.as_ref().unwrap()
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn is_initialised(&self) -> bool {
        self.is_initialised
    }

    pub fn set_initialised(&mut self, value: bool) {
        self.is_initialised = value
    }

    pub fn is_assignable_to(&self, _other: &Class) -> bool {
        // warn!("Need to implement is_assignable_to");
        true
    }

    pub fn flags(&self) -> ClassFileAccessFlags {
        self.class_file().access_flags
    }

    pub fn is_interface(&self) -> bool {
        self.flags().has(ClassFileAccessFlag::INTERFACE)
    }

    pub fn is_array(&self) -> bool {
        self.component_type.is_some()
    }

    pub fn is_primitive(&self) -> bool {
        // Not an array and not a class, must be primitive
        self.component_type.is_none() && self.classfile.is_none()
    }

    pub fn super_class(&self) -> RefTo<Class> {
        self.object.super_class.clone()
    }

    pub fn component_type(&self) -> Option<ArrayType> {
        self.component_type.clone()
    }
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Class")
            .field("object", &self.object)
            .field("name", &self.name)
            .field("is_initialised", &self.is_initialised)
            .finish_non_exhaustive()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BuiltinString {
    pub object: Object,

    pub value: RefTo<Array<u8>>,
    pub coder: u8,
    pub hash: i32,
    pub hash_is_zero: i32,
}

impl BuiltinString {
    pub fn string(&self) -> Result<String, Throwable> {
        let encoding = CompactEncoding::from_coder(self.coder);
        let bytes = self.value.borrow().slice().to_vec();

        decode_string((encoding, bytes)).map_err(|f| internal!(f))
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Array<T> {
    object: Object,

    capacity: usize,
    count: usize,
    phantom: PhantomData<T>,
}

impl<T: Copy> Array<T> {
    pub fn copy_from_slice(ty: ArrayType, ty_name: String, data: &[T]) -> RefTo<Array<T>> {
        let base_layout = Layout::new::<Array<T>>();
        let storage_layout = Layout::array::<T>(data.len()).unwrap();
        let (layout, _) = base_layout.extend(storage_layout).unwrap();

        let array_ref = unsafe {
            let mem = std::alloc::alloc_zeroed(layout).cast::<Array<T>>();
            (*mem).object = Object {
                class: RefTo::new(Class::new_array(
                    Object {
                        class: RefTo::null(),
                        super_class: RefTo::null(),
                        ref_count: 0,
                    },
                    ty_name,
                    ty,
                    ClassFileLayout::from_java_type(types::ARRAY_BASE),
                )),
                super_class: RefTo::null(),
                ref_count: 0,
            };

            (*mem).capacity = data.len();
            (*mem).count = data.len();

            let end_offset = offset_of!(Array<T>, count);
            let count_ptr = mem.byte_add(end_offset + size_of::<usize>()).cast::<T>();
            count_ptr.copy_from(data.as_ptr(), data.len());

            mem.as_mut().unwrap()
        };

        unsafe { RefTo::from_ptr(&mut array_ref.object) }
    }
}

impl<T> Array<T> {
    pub fn from_vec(ty: ArrayType, ty_name: String, data: Vec<T>) -> RefTo<Array<T>> {
        let base_layout = Layout::new::<Array<T>>();
        let storage_layout = Layout::array::<T>(data.len()).unwrap();
        let (layout, _) = base_layout.extend(storage_layout).unwrap();

        let array_ref = unsafe {
            let mem = std::alloc::alloc_zeroed(layout).cast::<Array<T>>();
            (*mem).object = Object {
                class: RefTo::new(Class::new_array(
                    Object {
                        class: RefTo::null(),
                        super_class: RefTo::null(),
                        ref_count: 0,
                    },
                    ty_name,
                    ty,
                    ClassFileLayout::from_java_type(types::ARRAY_BASE),
                )),
                super_class: RefTo::null(),
                ref_count: 0,
            };
            (*mem).capacity = data.len();
            (*mem).count = data.len();

            let end_offset = offset_of!(Array<T>, count);
            let count_ptr = mem.byte_add(end_offset + size_of::<usize>()).cast::<T>();
            count_ptr.copy_from(data.as_ptr(), data.len());

            mem.as_mut().unwrap()
        };

        unsafe { RefTo::from_ptr(&mut array_ref.object) }
    }
}

impl<T> Array<T> {
    pub fn elements_offset() -> usize {
        size_of::<Array<T>>()
    }

    pub fn element_scale() -> usize {
        size_of::<T>()
    }

    pub fn new(object: Object) -> Self {
        Self {
            object,
            count: 0,
            capacity: 0,
            phantom: PhantomData,
        }
    }

    pub fn slice(&self) -> &[T] {
        if self.count == 0 {
            return &[];
        }

        let elements_start = self.data_ptr();
        let len = self.count;

        let slice_ptr = std::ptr::slice_from_raw_parts(elements_start, len);
        unsafe { &*slice_ptr }
    }

    pub fn data_ptr(&self) -> *const T {
        let start = self as *const Array<T>;
        let end_offset = offset_of!(Array<T>, count);

        unsafe { start.byte_add(end_offset + size_of::<usize>()).cast::<T>() }
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        if self.count == 0 {
            return &mut [];
        }

        let start = self as *mut Array<T>;
        let end_offset = offset_of!(Array<T>, count);
        let elements_start = unsafe { start.byte_add(end_offset + size_of::<usize>()).cast::<T>() };

        let len = self.count;

        let slice_ptr = std::ptr::slice_from_raw_parts_mut(elements_start, len);
        unsafe { &mut *slice_ptr }
    }

    pub fn push(&mut self, value: T) {
        if self.count == self.capacity {
            panic!("out of storage, {} == {}", self.count, self.capacity)
        }

        self.count += 1;

        let slice = self.slice_mut();
        let end = slice.as_mut_ptr_range().end;

        // Write from the end of the buffer by T bytes with the value
        unsafe { end.write(value) };
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> HasObjectHeader<Array<T>> for Array<T> {
    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }

    fn header(&self) -> &Object {
        &self.object
    }
}

impl HasObjectHeader<Class> for Class {
    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }

    fn header(&self) -> &Object {
        &self.object
    }
}

impl HasObjectHeader<BuiltinString> for BuiltinString {
    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }

    fn header(&self) -> &Object {
        &self.object
    }
}

impl HasObjectHeader<Object> for Object {
    fn header_mut(&mut self) -> &mut Object {
        self
    }

    fn header(&self) -> &Object {
        self
    }
}
