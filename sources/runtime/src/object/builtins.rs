use std::{
    alloc::Layout,
    cell::RefCell,
    collections::HashMap,
    fmt,
    marker::PhantomData,
    mem::{offset_of, size_of},
    sync::atomic::{AtomicU64, Ordering},
};

use parking_lot::RwLock;
use parse::{
    classfile::{ClassFile, Resolvable},
    flags::{ClassFileAccessFlag, ClassFileAccessFlags},
};
use support::{encoding::{decode_string, CompactEncoding}, types::FieldDescriptor};

use crate::{
    error::Throwable,
    internal,
    native::NativeModule,
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
    pub ref_count: AtomicU64,
    pub lock: RwLock<()>,
}

impl Object {
    pub fn new(class: RefTo<Class>, super_class: RefTo<Class>) -> Self {
        Self {
            class,
            super_class,
            ref_count: AtomicU64::new(0),
            lock: RwLock::new(()),
        }
    }

    pub fn field<T>(&self, field: &FieldDescriptor) -> Option<FieldRef<T>> {
        self.inc_count();

        let class = self.class();
        let layout = &class.unwrap_ref().instance_layout;
        let field_info = layout.field_info(field.name())?;

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
        self.ref_count.load(Ordering::SeqCst)
    }

    pub fn inc_count(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn dec_count(&self) {
        self.ref_count.fetch_sub(1, Ordering::SeqCst);
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

#[derive(Debug, Clone, PartialEq)]
pub enum ClassType {
    Class,
    Interface,
    Array,
    Primitive,
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
    ty: ClassType,
    component_type: RefTo<Class>,
    native_module: Option<Box<RefCell<dyn NativeModule>>>,
    classfile: Option<ClassFile>,
    is_initialised: bool,
    instance_layout: ClassFileLayout,
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

            ty: ClassType::Class,
            component_type: RefTo::null(),
            name,
            native_module: None,
            classfile: Some(class_file),
            is_initialised: false,
            instance_layout: layout,
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

            ty: ClassType::Primitive,
            component_type: RefTo::null(),
            name,
            native_module: None,
            classfile: None,
            is_initialised: false,
            instance_layout: layout,
        }
    }

    pub fn new_array(object: Object, array_ty: RefTo<Class>, layout: ClassFileLayout) -> Self {
        let name = &array_ty.unwrap_ref().name;
        let name = format!("[{}", name);

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

            ty: ClassType::Array,
            component_type: array_ty,
            name,
            native_module: None,
            classfile: None,
            is_initialised: false,
            instance_layout: layout,
        }
    }

    pub fn set_class_file(&mut self, class_file: ClassFile) {
        self.classfile = Some(class_file);
    }

    pub fn instance_layout(&self) -> &ClassFileLayout {
        &self.instance_layout
    }

    pub fn statics(&self) -> &RwLock<HashMap<String, FieldInfo>> {
        self.instance_layout.statics()
    }

    pub fn native_module(&self) -> &Option<Box<RefCell<dyn NativeModule>>> {
        &self.native_module
    }

    pub fn set_native_module(&mut self, module: Box<RefCell<dyn NativeModule>>) {
        self.native_module = Some(module);
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

    pub fn can_assign(_s: RefTo<Class>, _t: RefTo<Class>) -> bool {
        /*
            Taken from checkcast/instanceof (which have the same semantics at this stage of checking)
            Given S is the type of the object referred to by objectref, and T is the resolved class, array, or interface type;
        */

        let s = _s.unwrap_ref();
        let t = _t.unwrap_ref();

        // • If S is a class type, then:
        if s.is_class() {
            // Interface checking comes first, because interfaces are a form of class at the moment
            // – If T is an interface type, then S must implement interface T.
            if t.is_interface() {
                let mut super_interfaces = vec![];
                let mut _super = _s.clone();
                while let Some(sup) = _super.to_ref() {
                    super_interfaces.extend(sup.class_file().interfaces.clone().into_iter());
                    _super = sup.super_class();
                }

                return super_interfaces.iter().any(|i| {
                    let i = i.resolve();
                    let i_name = i.name.resolve().string();
                    t.name() == &i_name
                });
            }

            // – If T is a class type;
            if t.is_class() {
                // then S must be the same class as T
                if _s.as_ptr() == _t.as_ptr() {
                    return true;
                }

                // or S must be a subclass of T
                let mut _super = _s.clone();
                while let Some(sup) = _super.to_ref() {
                    // We found a match in the hierarchy
                    if _super.as_ptr() == _t.as_ptr() {
                        return true;
                    }

                    _super = sup.super_class();
                }
            }
        }

        // • If S is an array type SC[], that is, an array of components of type SC, then:
        if s.is_array() {
            let _component_ty = s.component_type();
            let component_ty = _component_ty.unwrap_ref();

            // – If T is a class type, then T must be Object.
            if t.is_class() {
                return component_ty.name() == "java/lang/Object";
            }

            // – If T is an interface type, then T must be one of the interfaces implemented by arrays (JLS §4.10.3).
            if t.is_interface() {
                todo!("don't know how to check these");
            }

            // – If T is an array type TC[], that is, an array of components of type TC, then one of the following must be true:
            if t.is_array() {
                let _other_component_ty = s.component_type();
                let other_component_ty = _other_component_ty.unwrap_ref();

                if component_ty.is_primitive() && other_component_ty.is_primitive() {
                    return component_ty.name() == other_component_ty.name();
                }

                return Class::can_assign(_component_ty, _other_component_ty);
            }
        }

        false
    }

    pub fn flags(&self) -> ClassFileAccessFlags {
        self.class_file().access_flags
    }

    pub fn is_interface(&self) -> bool {
        if self.is_array() || self.is_primitive() {
            false
        } else {
            self.flags().has(ClassFileAccessFlag::INTERFACE)
        }
    }

    pub fn is_array(&self) -> bool {
        self.ty == ClassType::Array
    }

    pub fn is_class(&self) -> bool {
        self.ty == ClassType::Class
    }

    pub fn is_primitive(&self) -> bool {
        self.ty == ClassType::Primitive
    }

    pub fn super_class(&self) -> RefTo<Class> {
        self.object.super_class.clone()
    }

    pub fn component_type(&self) -> RefTo<Class> {
        self.component_type.clone()
    }

    pub fn ty(&self) -> ClassType {
        self.ty.clone()
    }
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Class")
            .field("object", &self.object)
            .field("name", &self.name)
            .field("ty", &self.ty)
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
        let bytes = self.value.unwrap_ref().slice().to_vec();

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
    pub fn copy_from_slice(array_ty: RefTo<Class>, data: &[T]) -> RefTo<Array<T>> {
        let base_layout = Layout::new::<Array<T>>();
        let storage_layout = Layout::array::<T>(data.len()).unwrap();
        let (layout, _) = base_layout.extend(storage_layout).unwrap();

        let array_ref = unsafe {
            let mem = std::alloc::alloc_zeroed(layout).cast::<Array<T>>();
            (*mem).object = Object::new(array_ty, RefTo::null());

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
    /// Create an array from a vector of items. The type should be an "array type" (e.g [Ljava/lang/String;).
    pub fn from_vec(array_ty: RefTo<Class>, data: Vec<T>) -> RefTo<Array<T>> {
        let base_layout = Layout::new::<Array<T>>();
        let storage_layout = Layout::array::<T>(data.len()).unwrap();
        let (layout, _) = base_layout.extend(storage_layout).unwrap();

        let array_ref = unsafe {
            let mem = std::alloc::alloc_zeroed(layout).cast::<Array<T>>();
            (*mem).object = Object::new(array_ty, RefTo::null());

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

    fn type_name() -> &'static str {
        "array"
    }
}

impl HasObjectHeader<Class> for Class {
    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }

    fn header(&self) -> &Object {
        &self.object
    }

    fn type_name() -> &'static str {
        "java/lang/Class"
    }
}

impl HasObjectHeader<BuiltinString> for BuiltinString {
    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }

    fn header(&self) -> &Object {
        &self.object
    }

    fn type_name() -> &'static str {
        "java/lang/String"
    }
}

impl HasObjectHeader<Object> for Object {
    fn header_mut(&mut self) -> &mut Object {
        self
    }

    fn header(&self) -> &Object {
        self
    }

    fn type_name() -> &'static str {
        "java/lang/Object"
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BuiltinThread {
    pub object: Object,

    pub name: RefTo<BuiltinString>,
    pub priority: types::Int,
    pub daemon: types::Bool,
    pub interrupted: types::Bool,
    pub stillborn: types::Bool,
    pub eetop: types::Long,
    pub target: RefTo<Object>,
    pub thread_group: RefTo<BuiltinThreadGroup>,
    pub context_class_loader: RefTo<Object>,
    pub inherited_access_control_context: RefTo<Object>,
    pub thread_locals: RefTo<Object>,
    pub inheritable_thread_locals: RefTo<Object>,
    pub stack_size: types::Long,
    pub tid: types::Long,
    pub status: types::Int,
    pub park_blocker: RefTo<Object>,
    pub uncaught_exception_handler: RefTo<Object>,

    pub thread_local_random_seed: types::Int,
    pub thread_local_random_probe: types::Int,
    pub thread_local_random_secondary_seed: types::Int,
}

impl HasObjectHeader<BuiltinThread> for BuiltinThread {
    fn header(&self) -> &Object {
        &self.object
    }

    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }

    fn type_name() -> &'static str {
        "java/lang/Thread"
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct BuiltinThreadGroup {
    pub object: Object,

    pub parent: RefTo<BuiltinThreadGroup>,
    pub name: RefTo<BuiltinString>,
    pub max_priority: types::Int,
    pub destroyed: types::Bool,
    pub daemon: types::Bool,
    pub n_unstarted_threads: types::Int,

    pub n_threads: types::Int,
    pub threads: RefTo<Array<RefTo<Object>>>,

    pub n_groups: types::Int,
    pub groups: RefTo<Array<RefTo<Object>>>,
}

impl HasObjectHeader<BuiltinThreadGroup> for BuiltinThreadGroup {
    fn header(&self) -> &Object {
        &self.object
    }

    fn header_mut(&mut self) -> &mut Object {
        &mut self.object
    }

    fn type_name() -> &'static str {
        "java/lang/ThreadGroup"
    }
}