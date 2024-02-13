use std::{collections::HashMap, sync::{Mutex, atomic::{AtomicUsize, Ordering}}, cell::RefCell};

use interpreter::{Interpreter, Context};
use runtime::{
    native::{NativeFunction, NativeModule},
    object::{
        builtins::Class,
        interner::{set_interner, StringInterner},
        loader::ClassLoader,
        mem::RefTo, value::RuntimeValue,
    },
    static_method,
    vm::VM, error::{Throwable, Frame},
};
use support::{
    descriptor::{FieldType, ObjectType},
    types::MethodDescriptor,
};

const SOURCE_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn make_vm() -> Interpreter {
    let mut class_loader = ClassLoader::new();

    class_loader.add_path(format!("{SOURCE_DIR}/../std/java.base"));
    let bootstrapped_classes = class_loader.bootstrap().unwrap();

    let interner = StringInterner::new(
        bootstrapped_classes.java_lang_string.clone(),
        bootstrapped_classes.java_lang_object.clone(),
        bootstrapped_classes.byte_array_ty.clone(),
    );

    set_interner(interner);

    let mut vm = Interpreter::new(
        VM::new(class_loader),
        interpreter::BootOptions { max_stack: 50 },
    );

    vm.bootstrap().unwrap();

    vm
}

pub fn load_test<const N: usize>(vm: &mut VM, class: (&[u8; N], String)) -> RefTo<Class> {
    vm.class_loader()
        .for_bytes(
            FieldType::Object(ObjectType {
                class_name: class.1,
            }),
            class.0,
        )
        .expect("class to load")
}

pub struct TestCaptures {
    methods: HashMap<MethodDescriptor, NativeFunction>,
    id: usize,
}

#[derive(Clone, Debug)]
pub struct CapturedOutput {
    values: Vec<RuntimeValue>
}

impl CapturedOutput {
    pub fn get(&self, index: usize) -> RuntimeValue {
        self.values.get(index).cloned().unwrap()
    }
}

static CAPTURE_COUNTER: AtomicUsize = AtomicUsize::new(0);
lazy_static::lazy_static! {
    static ref CAPTURE_STATE: Mutex<HashMap<usize, CapturedOutput>> = {
        Mutex::new(HashMap::new())
    };
}

pub fn get_captures(id: usize) -> CapturedOutput {
    let states = CAPTURE_STATE.lock().unwrap();
    let state = states.get(&id);
    state.unwrap().clone()
}

impl NativeModule for TestCaptures {
    fn classname(&self) -> &'static str {
        unimplemented!()
    }

    fn init(&mut self) {
        let id = self.id;

        let capture = move |
            _: RefTo<Class>,
            args: Vec<RuntimeValue>,
            _: &mut VM,
        | -> Result<Option<RuntimeValue>, Throwable> {
            let mut states = CAPTURE_STATE.lock().unwrap();
            let state = states.get_mut(&id).unwrap();
            state.values.push(args.get(0).unwrap().clone());

            Ok(None)
        };

        self.set_method(
            (
                "capture",
                "(D)V",
            ),
            static_method!(capture),
        );

        self.set_method(
            (
                "capture",
                "(I)V",
            ),
            static_method!(capture),
        );
    }

    fn methods(&self) -> &HashMap<MethodDescriptor, NativeFunction> {
        &self.methods
    }

    fn methods_mut(&mut self) -> &mut HashMap<MethodDescriptor, NativeFunction> {
        &mut self.methods
    }
}

pub fn attach_utils(class: RefTo<Class>) -> usize {
    let id = CAPTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut module = TestCaptures {
        id,
        methods: HashMap::new()
    };

    module.init();

    let mut states = CAPTURE_STATE.lock().unwrap();
    states.insert(id, CapturedOutput { values: vec![] });
    class.unwrap_mut().set_native_module(Box::new(RefCell::new(module)));

    id
}

pub fn execute_test(vm: &mut Interpreter, cls: RefTo<Class>, capture_id: usize) -> CapturedOutput {

    let main_ty: MethodDescriptor = ("runTest", "()V").try_into().unwrap();
    let ctx = Context::for_method(&main_ty, cls.clone());

    vm.push_frame(Frame {
        class_name: cls.unwrap_ref().name().to_string(),
        method_name: "runTest".to_string(),
    });

    let res = vm.run(ctx);
    res.unwrap();

    
    get_captures(capture_id)
}

pub fn iassert_eq(lhs: i64, rhs: RuntimeValue) {
    let val = rhs.as_integral().expect("was not an integral").value;
    assert_eq!(lhs, val);
}

pub fn dassert_eq(lhs: f64, rhs: RuntimeValue) {
    let val = rhs.as_floating().expect("was not a floating value").value;
    assert_eq!(lhs, val);
}