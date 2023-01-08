use core::panic;
use std::{collections::HashMap, rc::Rc, vec};

use crate::runtime::{
    classloader::ClassLoader,
    native::NativeFunction,
    object::{ClassObject, JavaObject, RuntimeObject, StringObject},
    stack::{
        Array, ArrayPrimitive, ArrayType, Floating, FloatingType, Integral, IntegralType,
        RuntimeValue,
    },
};
use crate::{native::NativeModule, opcode::Opcode};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use parking_lot::Mutex;
use parse::{
    attributes::CodeAttribute,
    classfile::{Addressed, Method, Resolvable},
    flags::MethodAccessFlag,
    pool::{ConstantClass, ConstantEntry},
};
use support::{
    descriptor::{self, FieldType, MethodType},
    encoding::encode_string,
};
use tracing::{info, debug};

pub struct Interpreter {
    classloaders: Vec<Box<dyn ClassLoader>>,
    primitive_classes: HashMap<String, Rc<Mutex<RuntimeObject>>>,
}

pub struct ExecutionContext {
    object: Rc<Mutex<ClassObject>>,
    method: Method,
    pc: i32,

    locals: Vec<RuntimeValue>,
    operands: Vec<RuntimeValue>,
}

impl ExecutionContext {
    pub fn new(class: Rc<Mutex<ClassObject>>, method: Method) -> Self {
        Self {
            object: class,
            method,
            pc: 0,
            operands: Vec::new(),
            locals: Vec::new(),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::with_classloaders(vec![])
    }

    pub fn load_primitive(&mut self, name: String) -> Result<Rc<Mutex<RuntimeObject>>> {
        if let Some(prim) = self.primitive_classes.get(&name) {
            Ok(Rc::clone(prim))
        } else {
            let string_class = self.load_class("java/lang/String".to_string())?;
            let java_class = self.load_class("java/lang/Class".to_string())?;

            let mut object = RuntimeObject::new(Rc::clone(&java_class));

            object.set_instance_field(
                ("name".to_string(), "Ljava/lang/String;".to_string()),
                RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(StringObject::new(
                    string_class,
                    encode_string(name.clone())?,
                ))))),
            );

            object.is_primitive = true;

            let jobj = Rc::new(Mutex::new(object));
            self.primitive_classes.insert(name, Rc::clone(&jobj));

            Ok(jobj)
        }
    }

    pub fn with_classloaders(classloaders: Vec<Box<dyn ClassLoader>>) -> Self {
        use crate::native::{jdk, lang};

        let mut s = Self {
            classloaders,
            primitive_classes: HashMap::new(),
        };

        lang::System::register(&s).unwrap();
        lang::Runtime::register(&s).unwrap();
        lang::Shutdown::register(&s).unwrap();
        lang::Object::register(&s).unwrap();
        lang::Class::register(&s).unwrap();
        lang::Throwable::register(&s).unwrap();
        lang::StringUTF16::register(&s).unwrap();
        lang::Float::register(&s).unwrap();
        lang::Double::register(&s).unwrap();

        jdk::VM::register(&s).unwrap();
        jdk::Cds::register(&s).unwrap();
        jdk::SystemPropsRaw::register(&s).unwrap();
        jdk::Unsafe::register(&s).unwrap();
        jdk::Reflection::register(&s).unwrap();

        let system = s.load_class("java/lang/System".to_string()).unwrap();
        s.initialise_class(Rc::clone(&system)).unwrap();

        let init = system
            .lock()
            .get_method("initPhase1".to_string(), "()V".to_string())
            .unwrap();

        let ctx = ExecutionContext {
            object: system,
            method: init,
            locals: vec![],
            operands: vec![],
            pc: 0,
        };

        if let Err(e) = s.run_code(ctx) {
            panic!("{:?}", e.context("failed to init runtime"))
        }

        s
    }

    pub(crate) fn load_class(&self, name: String) -> Result<Rc<Mutex<ClassObject>>> {
        for loader in self.classloaders.iter() {
            let res = loader.load_class(name.clone());
            if let Ok(class) = res {
                return Ok(class);
            } else if let Err(e) = res {
                info!("error loading {name}: {:#?}", e);
            }
        }

        Err(anyhow!("could not load class {name}"))
    }

    fn initialise_class(&mut self, class: Rc<Mutex<ClassObject>>) -> Result<Option<RuntimeValue>> {
        let mut locked_class = class.lock();
        let class_name = locked_class.get_class_name();

        if locked_class.is_initialised {
            info!(
                "Not initialising {}, class is already initialised",
                class_name
            );
            return Ok(None);
        }

        let clinit = locked_class.get_method("<clinit>".to_string(), "()V".to_string());

        if let Some(clinit) = clinit {
            let ctx = ExecutionContext::new(Rc::clone(&class), clinit);

            // Need to drop our lock on the class object before running the class initialiser
            // as it could call instructions which access class data
            locked_class.is_initialised = true;
            drop(locked_class);

            info!("Initialising {}", class_name);

            self.run_code(ctx)
        } else {
            info!("No clinit in {}", class_name);
            // Might as well mark this as initialised to avoid future
            // needless method lookups
            locked_class.is_initialised = true;
            Ok(None)
        }
    }

    fn insert_local(
        locals: &mut Vec<RuntimeValue>,
        index: usize,
        value: RuntimeValue,
        default: RuntimeValue,
    ) {
        while locals.len() <= index {
            locals.push(default.clone());
        }

        locals[index] = value;
    }

    pub fn run_code(&mut self, mut ctx: ExecutionContext) -> Result<Option<RuntimeValue>> {
        let mut operands = ctx.operands;
        let mut locals = ctx.locals;
        let pc = &mut ctx.pc;

        let root_cname = ctx.object.lock().get_class_name();

        let root_method_name = ctx.method.name.resolve().try_string()?;
        let root_method_descriptor = ctx.method.descriptor.resolve().try_string()?;

        info!(
            "Calling ({:?}) {} {} in {} {:?}",
            ctx.method.flags.flags, root_method_name, root_method_descriptor, root_cname, locals
        );

        if ctx.method.flags.has(MethodAccessFlag::NATIVE) {
            panic!("attempted to call native method {}", root_method_name)
        }

        let code = ctx
            .object
            .lock()
            .resolve_known_attribute::<CodeAttribute>(&ctx.method.attributes)?;
        let code = code.code;

        while *pc < code.len() as i32 {
            let slice = &code[*pc as usize..];
            let consumed_bytes_prev = slice.len();

            let mut code_bytes = BytesMut::new();
            code_bytes.extend_from_slice(slice);
            let opcode = Opcode::decode(&mut code_bytes, *pc)?;

            let consumed_bytes_post = code_bytes.len();
            let bytes_consumed_by_opcode = (consumed_bytes_prev - consumed_bytes_post) as i32;

            debug!(
                "Exec {:?} (@ {}, consumed {} bytes) with locals {:?}, operands {:?}, in {} / {}",
                opcode,
                *pc,
                bytes_consumed_by_opcode,
                locals,
                operands,
                root_cname,
                root_method_name
            );

            match opcode {
                Opcode::ICONST_M1 => operands.push(RuntimeValue::Integral((-1).into())),
                Opcode::ICONST_0 => operands.push(RuntimeValue::Integral(0.into())),
                Opcode::ICONST_1 => operands.push(RuntimeValue::Integral(1.into())),
                Opcode::ICONST_2 => operands.push(RuntimeValue::Integral(2.into())),
                Opcode::ICONST_3 => operands.push(RuntimeValue::Integral(3.into())),
                Opcode::ICONST_4 => operands.push(RuntimeValue::Integral(4.into())),
                Opcode::ICONST_5 => operands.push(RuntimeValue::Integral(5.into())),

                Opcode::LCONST_0 => operands.push(RuntimeValue::Integral(0.into())),
                Opcode::LCONST_1 => operands.push(RuntimeValue::Integral(1.into())),

                Opcode::FCONST_0 => operands.push(RuntimeValue::Floating(0.0.into())),
                Opcode::FCONST_1 => operands.push(RuntimeValue::Floating(1.0.into())),
                Opcode::FCONST_2 => operands.push(RuntimeValue::Floating(2.0.into())),
                Opcode::FCMPG => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v2 = v2.as_floating().ok_or(anyhow!("not floating"))?;
                    let v1 = v1.as_floating().ok_or(anyhow!("not floating"))?;

                    // TODO: Handle NaN

                    if v1 < v2 {
                        operands.push(RuntimeValue::Integral((-1).into()));
                    }

                    if v1 == v2 {
                        operands.push(RuntimeValue::Integral(0.into()));
                    }

                    if v1 > v2 {
                        operands.push(RuntimeValue::Integral(1.into()));
                    }
                }
                Opcode::FCMPL => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v2 = v2.as_floating().ok_or(anyhow!("not floating"))?;
                    let v1 = v1.as_floating().ok_or(anyhow!("not floating"))?;

                    // TODO: Handle NaN

                    if v1 < v2 {
                        operands.push(RuntimeValue::Integral((-1).into()));
                    }

                    if v1 == v2 {
                        operands.push(RuntimeValue::Integral(0.into()));
                    }

                    if v1 > v2 {
                        operands.push(RuntimeValue::Integral(1.into()));
                    }
                }
                Opcode::ACONST_NULL => {
                    operands.push(RuntimeValue::Null);
                }
                Opcode::ATHROW => {
                    let objectref = operands.pop().ok_or(anyhow!("no objectref"))?;
                    let objectref = objectref
                        .as_object()
                        .ok_or(anyhow!("not an object"))?
                        .clone();

                    let class = match objectref {
                        JavaObject::Runtime(data) => Rc::clone(&data.lock().class),
                        JavaObject::String(data) => Rc::clone(&data.lock().class),
                    };
                    let cname = class.lock().get_class_name();

                    return Err(anyhow!(
                        "exception thrown from {} in {}: {}",
                        root_method_name,
                        root_cname,
                        cname
                    ));
                }
                Opcode::ALOAD_0 => {
                    // TODO: Validate that this is RuntimeValue::Object
                    let local = locals.get(0).cloned().ok_or(anyhow!("no local @ {}", 0))?;

                    operands.push(local);
                }
                Opcode::ALOAD_1 => {
                    // TODO: Validate that this is RuntimeValue::Object
                    let local = locals.get(1).cloned().ok_or(anyhow!("no local @ {}", 1))?;

                    operands.push(local);
                }
                Opcode::ALOAD_2 => {
                    // TODO: Validate that this is RuntimeValue::Object
                    let local = locals.get(2).cloned().ok_or(anyhow!("no local @ {}", 2))?;

                    operands.push(local);
                }
                Opcode::ALOAD_3 => {
                    // TODO: Validate that this is RuntimeValue::Object
                    let local = locals.get(3).cloned().ok_or(anyhow!("no local @ {}", 3))?;

                    operands.push(local);
                }
                Opcode::ALOAD(index) => {
                    // TODO: Validate that this is RuntimeValue::Object
                    let local = locals
                        .get((index) as usize)
                        .cloned()
                        .ok_or(anyhow!("no local @ {}", index))?;
                    operands.push(local);
                }
                Opcode::FLOAD(index) => {
                    // TODO: Validate that this is RuntimeValue::Floating
                    let local = locals
                        .get((index) as usize)
                        .cloned()
                        .ok_or(anyhow!("no local @ {}", index))?;
                    operands.push(local);
                }
                Opcode::LDC2_W(index) => {
                    let num: Addressed<ConstantEntry> = ctx.object.lock().get_constant(index);

                    let num = num.resolve();
                    match num {
                        ConstantEntry::Long(data) => {
                            operands
                                .push(RuntimeValue::Integral(Integral::long(data.bytes as i64)));
                        }
                        ConstantEntry::Double(data) => {
                            operands.push(RuntimeValue::Floating(Floating::double(data.bytes)));
                        }
                        e => todo!("expected long or double, got {:#?}", e),
                    };
                }
                Opcode::LDC(index) => {
                    let num: Addressed<ConstantEntry> =
                        ctx.object.lock().get_constant(index.into());

                    let num = num.resolve();
                    let op = match num {
                        ConstantEntry::String(data) => {
                            let string_class = self.load_class("java/lang/String".to_string())?;
                            let value = data.string.resolve().try_string()?;
                            let value = encode_string(value)?;
                            info!("LDC: pushing string {:?}", value);
                            RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(
                                StringObject::new(string_class, value),
                            ))))
                        }
                        ConstantEntry::Integer(data) => {
                            RuntimeValue::Integral((data.bytes as i64).into())
                        }
                        ConstantEntry::Float(data) => {
                            RuntimeValue::Floating((data.bytes as f64).into())
                        }
                        ConstantEntry::Class(data) => {
                            let underlying_name = data.name.resolve().try_string()?;
                            let underlying_name = encode_string(underlying_name)?;
                            let java_class = self.load_class("java/lang/Class".to_string())?;
                            let string_class = self.load_class("java/lang/String".to_string())?;
                            let mut object = RuntimeObject::new(java_class);

                            object.set_instance_field(
                                ("name".to_string(), "Ljava/lang/String;".to_string()),
                                RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(
                                    StringObject::new(string_class, underlying_name),
                                )))),
                            );

                            let object = JavaObject::Runtime(Rc::new(Mutex::new(object)));

                            RuntimeValue::Object(object)
                        }
                        e => todo!("dont know how to push {:#?}", e),
                    };

                    operands.push(op);
                }
                Opcode::LDC_W(index) => {
                    let num: Addressed<ConstantEntry> = ctx.object.lock().get_constant(index);

                    let num = num.resolve();
                    let op = match num {
                        ConstantEntry::String(data) => {
                            let string_class = self.load_class("java/lang/String".to_string())?;
                            let value = data.string.resolve().try_string()?;
                            let value = encode_string(value)?;
                            info!("LDC: pushing string {:?}", value);
                            RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(
                                StringObject::new(string_class, value),
                            ))))
                        }
                        ConstantEntry::Integer(data) => {
                            RuntimeValue::Integral((data.bytes as i64).into())
                        }
                        ConstantEntry::Float(data) => {
                            RuntimeValue::Floating((data.bytes as f64).into())
                        }
                        ConstantEntry::Class(data) => {
                            let underlying_name = data.name.resolve().try_string()?;
                            let underlying_name = encode_string(underlying_name)?;
                            let java_class = self.load_class("java/lang/Class".to_string())?;
                            let string_class = self.load_class("java/lang/String".to_string())?;
                            let mut object = RuntimeObject::new(java_class);

                            object.set_instance_field(
                                ("name".to_string(), "Ljava/lang/String;".to_string()),
                                RuntimeValue::Object(JavaObject::String(Rc::new(Mutex::new(
                                    StringObject::new(string_class, underlying_name),
                                )))),
                            );

                            let object = JavaObject::Runtime(Rc::new(Mutex::new(object)));

                            RuntimeValue::Object(object)
                        }
                        e => todo!("dont know how to push {:#?}", e),
                    };

                    info!("LDC_W: Pushed {:?}", op);

                    operands.push(op);
                }
                Opcode::ARRAYLENGTH => {
                    let array = operands.pop().ok_or(anyhow!("no ref"))?;
                    let array = array.as_array().ok_or(anyhow!("not an array"))?;
                    let array = array.lock();

                    operands.push(RuntimeValue::Integral((array.values.len() as i64).into()))
                }
                Opcode::ISHR => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    let result = v1.checked_shr(v2.value as u32).ok_or(anyhow!("shr fail"))?;
                    operands.push(RuntimeValue::Integral(result.into()))
                }
                Opcode::IUSHR => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    let result = ((v1.value as u32) >> (v2.value as u32)) as i64;
                    operands.push(RuntimeValue::Integral(result.into()))
                }
                Opcode::ISHL => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    let result = v1.checked_shl(v2.value as u32).ok_or(anyhow!("shl fail"))?;
                    operands.push(RuntimeValue::Integral(result.into()))
                }
                Opcode::LSHL => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    let result = v1.checked_shl(v2.value as u32).ok_or(anyhow!("shl fail"))?;
                    operands.push(RuntimeValue::Integral(Integral::long(result)))
                }
                Opcode::LSHR => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not a long"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not a long"))?;

                    let result = v1.checked_shr(v2.value as u32).ok_or(anyhow!("shr fail"))?;
                    operands.push(RuntimeValue::Integral(Integral::long(result)))
                }
                Opcode::LUSHR => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not a long"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not a long"))?;

                    let result = ((v1.value as u64) >> (v2.value as u64)) as i64;
                    operands.push(RuntimeValue::Integral(Integral::long(result)))
                }
                Opcode::LAND => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not a long"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not a long"))?;

                    let result = v1 & v2;
                    operands.push(RuntimeValue::Integral(Integral::long(result)))
                }
                Opcode::LOR => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not a long"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not a long"))?;

                    let result = v1 | v2;
                    operands.push(RuntimeValue::Integral(Integral::long(result)))
                }
                Opcode::AASTORE => {
                    // TODO: Type checks
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    let index = operands.pop().ok_or(anyhow!("no index"))?;
                    let array = operands.pop().ok_or(anyhow!("no array"))?;

                    let index = index.as_integral().ok_or(anyhow!("not an int"))?;
                    let array = array.as_array().ok_or(anyhow!("not an array"))?;
                    let mut array = array.lock();

                    array.values[index.value as usize] = value;
                }
                Opcode::BASTORE => {
                    // TODO: Type checks
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    let index = operands.pop().ok_or(anyhow!("no index"))?;
                    let array = operands.pop().ok_or(anyhow!("no array"))?;

                    let index = index.as_integral().ok_or(anyhow!("not an int"))?;
                    let array = array.as_array().ok_or(anyhow!("not an array"))?;
                    let mut array = array.lock();

                    array.values[index.value as usize] = value;
                }
                Opcode::IASTORE => {
                    // TODO: Type checks
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    let index = operands.pop().ok_or(anyhow!("no index"))?;
                    let array = operands.pop().ok_or(anyhow!("no array"))?;

                    let index = index.as_integral().ok_or(anyhow!("not an int"))?;
                    let array = array.as_array().ok_or(anyhow!("not an array"))?;
                    let mut array = array.lock();

                    array.values[index.value as usize] = value;
                }
                Opcode::BALOAD => {
                    // TODO: Type checks (byte|bool)
                    let index = operands.pop().ok_or(anyhow!("no index"))?;
                    let array = operands.pop().ok_or(anyhow!("no array"))?;

                    let index = index.as_integral().ok_or(anyhow!("not an int"))?;
                    let array = array.as_array().ok_or(anyhow!("not an array"))?;
                    let array = array.lock();

                    let value = array
                        .values
                        .get(index.value as usize)
                        .cloned()
                        .ok_or(anyhow!("no value @ {}", index.value))?;
                    operands.push(value);
                }
                Opcode::AALOAD => {
                    // TODO: Type checks (refs;)
                    let index = operands.pop().ok_or(anyhow!("no index"))?;
                    let array = operands.pop().ok_or(anyhow!("no array"))?;

                    let index = index.as_integral().ok_or(anyhow!("not an int"))?;
                    let array = array.as_array().ok_or(anyhow!("not an array"))?;
                    let array = array.lock();

                    let value = array
                        .values
                        .get(index.value as usize)
                        .cloned()
                        .ok_or(anyhow!(
                            "no value @ {} (for len {})",
                            index.value,
                            array.values.len()
                        ))?;
                    info!(
                        "AALOAD: loaded {:?} from index {} (len: {})",
                        value,
                        index.value,
                        array.values.len()
                    );
                    operands.push(value);
                }
                Opcode::IAND => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;

                    let result = v1 & v2;
                    operands.push(RuntimeValue::Integral(result.into()))
                }
                Opcode::L2I => {
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not a long"))?;

                    operands.push(RuntimeValue::Integral(Integral::int(
                        v1.value as i32 as i64,
                    )))
                }
                Opcode::I2L => {
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral(Integral::long(v1.value)))
                }
                Opcode::I2C => {
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral((v1.value as u8 as i64).into()))
                }
                Opcode::I2B => {
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral((v1.value as i8 as i64).into()))
                }
                Opcode::I2F => {
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Floating((v1.value as f64).into()))
                }
                Opcode::F2I => {
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = *v1.as_floating().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral((v1.value as i64).into()))
                }
                Opcode::CASTORE => {
                    // TODO: Type checks
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    let index = operands.pop().ok_or(anyhow!("no index"))?;
                    let array = operands.pop().ok_or(anyhow!("no array"))?;

                    let index = index.as_integral().ok_or(anyhow!("not an int"))?;
                    let array = array.as_array().ok_or(anyhow!("not an array"))?;
                    let mut array = array.lock();
                    info!("CASTORE: {}", array.values.len());

                    array.values[index.value as usize] = value;
                }
                Opcode::IADD => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral((v1 + v2).into()))
                }
                Opcode::FADD => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_floating().ok_or(anyhow!("not an int"))?;
                    let v2 = *v2.as_floating().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Floating((v1 + v2).into()))
                }
                Opcode::LADD => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral(Integral::long(v1 + v2)))
                }
                Opcode::LSUB => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral(Integral::long(v1 - v2)))
                }
                Opcode::IOR => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral((v1 | v2).into()))
                }
                Opcode::ISUB => {
                    let v2 = operands.pop().ok_or(anyhow!("no value"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no value"))?;

                    let v1 = *v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = *v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral((v1 - v2).into()))
                }
                Opcode::BIPUSH(byte) => {
                    operands.push(RuntimeValue::Integral((byte as i64).into()));
                }
                Opcode::SIPUSH(byte) => {
                    info!("SIPUSH: {}", byte);
                    operands.push(RuntimeValue::Integral((byte as i64).into()));
                }
                Opcode::NEWARRAY(type_tag) => {
                    let arr_type = ArrayPrimitive::from_tag(type_tag);

                    let count = operands.pop().ok_or(anyhow!("no count"))?;
                    let count = count.as_integral().ok_or(anyhow!("not an int"))?;

                    let mut values = Vec::new();
                    values.resize(
                        count.value as usize,
                        match arr_type {
                            ArrayPrimitive::Bool => RuntimeValue::Integral(0.into()),
                            ArrayPrimitive::Char => RuntimeValue::Integral(0.into()),
                            ArrayPrimitive::Float => RuntimeValue::Floating(0.0.into()),
                            ArrayPrimitive::Double => RuntimeValue::Floating(0.0.into()),
                            ArrayPrimitive::Byte => RuntimeValue::Integral(0.into()),
                            ArrayPrimitive::Short => RuntimeValue::Integral(0.into()),
                            ArrayPrimitive::Int => RuntimeValue::Integral(0.into()),
                            ArrayPrimitive::Long => RuntimeValue::Integral(0.into()),
                        },
                    );
                    info!(
                        "NEWARRAY: new array count: {}, len: {}",
                        count.value,
                        values.len()
                    );

                    operands.push(RuntimeValue::Array(Rc::new(Mutex::new(Array {
                        ty: ArrayType::Primitive(arr_type),
                        values,
                    }))));
                }
                Opcode::ANEWARRAY(type_index) => {
                    let count = operands.pop().ok_or(anyhow!("no count"))?;
                    let count = count.as_integral().ok_or(anyhow!("not an int"))?;

                    let arr_type: Addressed<ConstantClass> =
                        ctx.object.lock().get_constant(type_index);

                    let arr_type = arr_type.resolve();

                    let class_name = arr_type.name.resolve().try_string()?;
                    let loaded_target = self.load_class(class_name)?;

                    let mut values = Vec::new();
                    values.resize(count.value as usize, RuntimeValue::Null);
                    info!(
                        "ANEWARRAY: new array count: {}, len: {}",
                        count.value,
                        values.len()
                    );

                    operands.push(RuntimeValue::Array(Rc::new(Mutex::new(Array {
                        ty: ArrayType::Object(loaded_target),
                        values,
                    }))));
                }
                Opcode::MONITORENTER | Opcode::MONITOREXIT => {
                    // TODO: This
                    operands.pop().unwrap();
                }
                Opcode::ASTORE_0 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 0, value, RuntimeValue::Null);
                }
                Opcode::ASTORE_1 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 1, value, RuntimeValue::Null);
                }
                Opcode::ASTORE_2 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 2, value, RuntimeValue::Null);
                }
                Opcode::ASTORE_3 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 3, value, RuntimeValue::Null);
                }
                Opcode::ASTORE(index) => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, index.into(), value, RuntimeValue::Null);
                }
                Opcode::FSTORE(index) => {
                    // TODO: Validate that this is RuntimeValue::Float
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, index.into(), value, RuntimeValue::Null);
                }
                Opcode::ISTORE_0 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 0, value, RuntimeValue::Null);
                }
                Opcode::ISTORE_1 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 1, value, RuntimeValue::Null);
                }
                Opcode::ISTORE_2 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 2, value, RuntimeValue::Null);
                }
                Opcode::ISTORE_3 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 3, value, RuntimeValue::Null);
                }
                Opcode::IINC(index, constant) => {
                    info!("LINC: Increment local @ {} by {}", index, constant);

                    let local = locals
                        .get_mut(index as usize)
                        .ok_or(anyhow!("no local @ {}", index))?;
                    let local = local.as_integral_mut().ok_or(anyhow!("not an int"))?;
                    local.value += constant as i64;
                }
                Opcode::LSTORE_0 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 0, value, RuntimeValue::Null);
                }
                Opcode::LSTORE_1 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 1, value, RuntimeValue::Null);
                }
                Opcode::LSTORE_2 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 2, value, RuntimeValue::Null);
                }
                Opcode::LSTORE_3 => {
                    // TODO: Validate that this is RuntimeValue::Object (or returnValue when implemented)
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, 3, value, RuntimeValue::Null);
                }
                Opcode::POP => {
                    operands.pop().unwrap();
                }
                Opcode::INEG => {
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    let value = value.as_integral().ok_or(anyhow!("not an int"))?;
                    let result = value.checked_neg().ok_or(anyhow!("negation failed"))?;

                    operands.push(RuntimeValue::Integral(result.into()))
                }
                Opcode::ISTORE(index) => {
                    // TODO: Validate that this is RuntimeValue::Integral
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(&mut locals, index.into(), value, RuntimeValue::Null);
                }
                Opcode::LSTORE(index) => {
                    // TODO: Validate that this is RuntimeValue::Integral
                    let value = operands.pop().ok_or(anyhow!("no value"))?;
                    Interpreter::insert_local(
                        &mut locals,
                        index.into(),
                        value.clone(),
                        RuntimeValue::Null,
                    );
                    Interpreter::insert_local(
                        &mut locals,
                        (index + 1).into(),
                        value,
                        RuntimeValue::Null,
                    );
                }
                Opcode::ILOAD_0 => {
                    let local = locals.get(0).cloned().ok_or(anyhow!("no local @ {}", 0))?;
                    operands.push(local);
                }
                Opcode::ILOAD_1 => {
                    let local = locals.get(1).cloned().ok_or(anyhow!("no local @ {}", 1))?;
                    operands.push(local);
                }
                Opcode::ILOAD_2 => {
                    let local = locals.get(2).cloned().ok_or(anyhow!("no local @ {}", 2))?;
                    operands.push(local);
                }
                Opcode::ILOAD_3 => {
                    let local = locals.get(3).cloned().ok_or(anyhow!("no local @ {}", 3))?;
                    operands.push(local);
                }
                Opcode::FLOAD_0 => {
                    let local = locals.get(0).cloned().ok_or(anyhow!("no local @ {}", 0))?;
                    operands.push(local);
                }
                Opcode::FLOAD_1 => {
                    let local = locals.get(1).cloned().ok_or(anyhow!("no local @ {}", 1))?;
                    operands.push(local);
                }
                Opcode::FLOAD_2 => {
                    let local = locals.get(2).cloned().ok_or(anyhow!("no local @ {}", 2))?;
                    operands.push(local);
                }
                Opcode::FLOAD_3 => {
                    let local = locals.get(3).cloned().ok_or(anyhow!("no local @ {}", 3))?;
                    operands.push(local);
                }
                Opcode::LLOAD_0 => {
                    let local = locals.get(0).cloned().ok_or(anyhow!("no local @ {}", 0))?;
                    operands.push(local);
                }
                Opcode::LLOAD_1 => {
                    let local = locals.get(1).cloned().ok_or(anyhow!("no local @ {}", 1))?;
                    operands.push(local);
                }
                Opcode::LLOAD_2 => {
                    let local = locals.get(2).cloned().ok_or(anyhow!("no local @ {}", 2))?;
                    operands.push(local);
                }
                Opcode::LLOAD_3 => {
                    let local = locals.get(0).cloned().ok_or(anyhow!("no local @ {}", 3))?;
                    operands.push(local);
                }
                Opcode::INSTANCEOF(index) => {
                    // TODO: Subclass support
                    // TODO: Null support

                    let object_to_check = operands.pop().ok_or(anyhow!("no object"))?;
                    if object_to_check.is_null() {
                        operands.push(RuntimeValue::Integral(0.into()))
                    } else {
                        let object_to_check = object_to_check
                            .as_object()
                            .ok_or(anyhow!("not an object"))?;

                        let target_class: Addressed<ConstantEntry> =
                            ctx.object.lock().get_constant(index);

                        let target_class = target_class.resolve();
                        let target_class = match target_class {
                            ConstantEntry::Class(data) => {
                                self.load_class(data.name.resolve().try_string()?)?
                            }
                            e => panic!("dont know how to type check {:#?}", e),
                        };

                        let class_to_check = match object_to_check {
                            JavaObject::Runtime(data) => {
                                let data = data.lock();
                                Rc::clone(&data.class)
                            }
                            JavaObject::String(data) => {
                                let data = data.lock();
                                Rc::clone(&data.class)
                            }
                        };

                        let matches = Rc::ptr_eq(&target_class, &class_to_check);
                        operands.push(RuntimeValue::Integral(if matches { 1 } else { 0 }.into()))
                    }
                }
                Opcode::CHECKCAST(index) => {
                    // TODO: Subclass support
                    // TODO: Null support
                    let target_class: Addressed<ConstantEntry> =
                        ctx.object.lock().get_constant(index);
                    let target_class = target_class.resolve();

                    let target_class_name = match target_class {
                        ConstantEntry::Class(data) => data.name.resolve().try_string()?,
                        e => panic!("dont know how to type check {:#?}", e),
                    };

                    // HACK: hacks for primitive type checking !! (fix when we have proper primitive class instances)
                    let object_to_check = operands.pop().ok_or(anyhow!("no object"))?;
                    match target_class_name.as_str() {
                        "[B" => {
                            let array =
                                object_to_check.as_array().ok_or(anyhow!("not an array"))?;
                            let array = array.lock();
                            let is_byte_array = match &array.ty {
                                ArrayType::Primitive(ty) => match ty {
                                    ArrayPrimitive::Byte => true,
                                    _ => return Err(anyhow!("not a byte")),
                                },
                                _ => return Err(anyhow!("not a primitive")),
                            };
                            drop(array);

                            if is_byte_array {
                                operands.push(object_to_check);
                            }
                        }
                        _ => {
                            let target_class = self.load_class(target_class_name)?;
                            if object_to_check.is_null() {
                                operands.push(object_to_check);
                            } else {
                                let object_to_check = object_to_check
                                    .as_object()
                                    .ok_or(anyhow!("not an object"))?;

                                let class_to_check = match object_to_check {
                                    JavaObject::Runtime(data) => {
                                        let data = data.lock();
                                        Rc::clone(&data.class)
                                    }
                                    JavaObject::String(data) => {
                                        let data = data.lock();
                                        Rc::clone(&data.class)
                                    }
                                };

                                let _matches = Rc::ptr_eq(&target_class, &class_to_check);
                                // TODO: If there's no match, throw an err
                                operands.push(RuntimeValue::Object(object_to_check.clone()));
                            }
                        }
                    };
                }
                Opcode::ILOAD(index) => {
                    // TODO: Validate that this is RuntimeValue::Integral
                    let local = locals
                        .get(index as usize)
                        .cloned()
                        .ok_or(anyhow!("no local @ {}", index))?;

                    operands.push(local);
                }
                Opcode::LLOAD(index) => {
                    // TODO: Validate that this is RuntimeValue::Integral
                    let local = locals
                        .get(index as usize)
                        .cloned()
                        .ok_or(anyhow!("no local @ {}", index))?;

                    operands.push(local);
                }
                Opcode::IF_ICMPEQ(branch_offset) => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    if v1 == v2 {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IF_ICMPLT(branch_offset) => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    info!("IF_ICMPLT: {:?} < {:?}: {}", v1, v2, v1 < v2);

                    if v1 < v2 {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IF_ACMPNE(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;

                    if v1.is_null() || v2.is_null() {
                        if v1.is_null() && !v2.is_null() {
                            *pc += branch_offset as i32;
                            continue;
                        }

                        if !v1.is_null() && v2.is_null() {
                            *pc += branch_offset as i32;
                            continue;
                        }
                    } else {
                        let v2 = v2.as_object().ok_or(anyhow!("not an object"))?;
                        let v1 = v1.as_object().ok_or(anyhow!("not an object"))?;

                        info!(
                            "IF_ACMPNE: {:?} != {:?}: {}",
                            v1.hash_code(),
                            v2.hash_code(),
                            v1 != v2
                        );

                        if v1 != v2 {
                            *pc += branch_offset as i32;
                            continue;
                        }
                    }
                }
                Opcode::IF_ACMPEQ(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;

                    if v1.is_null() || v2.is_null() {
                        if v1.is_null() && v2.is_null() {
                            *pc += branch_offset as i32;
                            continue;
                        }
                    } else {
                        let v2 = v2.as_object().ok_or(anyhow!("not an object"))?;
                        let v1 = v1.as_object().ok_or(anyhow!("not an object"))?;

                        info!(
                            "IF_ACMPEQ: {:?} == {:?}: {}",
                            v1.hash_code(),
                            v2.hash_code(),
                            v1 == v2
                        );

                        if v1 == v2 {
                            *pc += branch_offset as i32;
                            continue;
                        }
                    }
                }
                Opcode::IF_ICMPNE(branch_offset) => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    if v1 != v2 {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IF_ICMPGT(branch_offset) => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    if v1 > v2 {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IF_ICMPGE(branch_offset) => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    if v1 >= v2 {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IF_ICMPLE(branch_offset) => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    info!("IF_ICMPLE: {:?} <= {:?}: {}", v1, v2, v1 <= v2);

                    if v1 <= v2 {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::LCMP => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v2 = v2.as_integral().ok_or(anyhow!("not a long"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not a long"))?;

                    if v1 < v2 {
                        operands.push(RuntimeValue::Integral((-1).into()));
                    }

                    if v1 == v2 {
                        operands.push(RuntimeValue::Integral(0.into()));
                    }

                    if v1 > v2 {
                        operands.push(RuntimeValue::Integral(1.into()));
                    }
                }
                Opcode::LMUL => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v2 = v2.as_integral().ok_or(anyhow!("not a long"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not a long"))?;

                    operands.push(RuntimeValue::Integral(Integral::long(
                        v1.wrapping_mul(v2.value),
                    )));
                }
                Opcode::IMUL => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral(v1.wrapping_mul(v2.value).into()));
                }
                Opcode::IDIV => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral(v1.wrapping_div(v2.value).into()));
                }
                Opcode::FDIV => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_floating().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_floating().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Floating((*v1 / *v2).into()));
                }
                Opcode::FMUL => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_floating().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_floating().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Floating((*v1 * *v2).into()));
                }
                Opcode::IXOR => {
                    let v2 = operands.pop().ok_or(anyhow!("no rhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    operands.push(RuntimeValue::Integral((*v1 | *v2).into()));
                }
                Opcode::IREM => {
                    let v2 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v1 = operands.pop().ok_or(anyhow!("no rhs"))?;

                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;
                    let v2 = v2.as_integral().ok_or(anyhow!("not an int"))?;

                    // TODO: Figure out semantics
                    let result = v1.value - (v1.value / v2.value) * v2.value;
                    operands.push(RuntimeValue::Integral(result.into()));
                }
                Opcode::IFEQ(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    if *v1 == 0.into() {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IFLE(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    if *v1 <= 0.into() {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IFGT(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    if *v1 > 0.into() {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IFGE(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    info!("IFGE: {:?} >= 0: {}", *v1, *v1 >= 0.into());
                    if *v1 >= 0.into() {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IFNE(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    info!("IFNE: {:?} != 0: {}", *v1, *v1 != 0.into());
                    let zero = Integral::int(0);
                    if *v1 != zero {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IFLT(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;
                    let v1 = v1.as_integral().ok_or(anyhow!("not an int"))?;

                    if *v1 < 0.into() {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IFNULL(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    info!("IFNULL: {:#?} == null: {}", v1, v1.is_null());

                    if v1.is_null() {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::IFNONNULL(branch_offset) => {
                    let v1 = operands.pop().ok_or(anyhow!("no lhs"))?;

                    info!("IFNONNULL: {:#?} == null: {}", v1, v1.is_null());
                    if !v1.is_null() {
                        *pc += branch_offset as i32;
                        continue;
                    }
                }
                Opcode::GOTO(branch_offset) => {
                    info!("GOTO: Advancing by {} bytes", branch_offset);

                    *pc += branch_offset as i32;
                    continue;
                }
                Opcode::GETSTATIC(field_index) => {
                    let field_cp: ConstantEntry =
                        ctx.object.lock().get_constant(field_index).resolve();

                    let field_cp = field_cp.into_field();

                    if field_cp.is_err() {
                        return Err(anyhow!("value was not a field @ {field_index}"));
                    }

                    let field_cp = field_cp.unwrap();

                    let name_and_type = field_cp.name_and_type.resolve();
                    let descriptor = name_and_type.descriptor.resolve().try_string()?;
                    let parsed_descriptor = FieldType::parse(descriptor.clone())?;

                    let field_name = name_and_type.name.resolve().try_string()?;
                    let class = field_cp.class.resolve();

                    let class_name = class.name.resolve().try_string()?;

                    info!(
                        "GETSTATIC: field {} {} in {}",
                        field_name, descriptor, class_name
                    );
                    let loaded_target = self.load_class(class_name)?;

                    self.initialise_class(Rc::clone(&loaded_target))?;

                    let static_value = loaded_target
                        .lock()
                        .get_static_field(&(
                            name_and_type.name.resolve().try_string()?,
                            name_and_type.descriptor.resolve().try_string()?,
                        ))
                        .unwrap_or({
                            match parsed_descriptor {
                                // TODO: Type these properly
                                FieldType::Base(ty) => match ty {
                                    descriptor::BaseType::Boolean => RuntimeValue::Null,
                                    descriptor::BaseType::Char => RuntimeValue::Null,
                                    descriptor::BaseType::Float => RuntimeValue::Null,
                                    descriptor::BaseType::Double => RuntimeValue::Null,
                                    descriptor::BaseType::Byte => RuntimeValue::Null,
                                    descriptor::BaseType::Short => RuntimeValue::Null,
                                    descriptor::BaseType::Int => RuntimeValue::Integral(0.into()),
                                    descriptor::BaseType::Long => RuntimeValue::Null,
                                    descriptor::BaseType::Void => RuntimeValue::Null,
                                },
                                FieldType::Object(_) => RuntimeValue::Null,
                                FieldType::Array(_) => RuntimeValue::Null,
                            }
                        });

                    info!("GETSTATIC: Pushed {:?}", static_value);

                    operands.push(static_value)
                }
                Opcode::GETFIELD(field_index) => {
                    let field_cp: ConstantEntry =
                        ctx.object.lock().get_constant(field_index).resolve();

                    let field_cp = field_cp.into_field();

                    if field_cp.is_err() {
                        return Err(anyhow!("value was not a field @ {field_index}"));
                    }

                    let field_cp = field_cp.unwrap();

                    let name_and_type = field_cp.name_and_type.resolve();
                    let field_name = name_and_type.name.resolve().try_string()?;
                    let descriptor =
                        FieldType::parse(name_and_type.descriptor.resolve().try_string()?)?;
                    let class = field_cp.class.resolve();

                    let class_name = class.name.resolve().try_string()?;
                    info!("GETFIELD: field {} in {}", field_name, class_name);
                    let loaded_target = self.load_class(class_name)?;

                    self.initialise_class(Rc::clone(&loaded_target))?;

                    let objectref = operands
                        .pop()
                        .ok_or(anyhow!("no value on the operand stack to set"))?;
                    let objectref = objectref
                        .as_object()
                        .cloned()
                        .ok_or(anyhow!("not an object"))?;

                    // TODO: Type check the popped value

                    let field = (
                        name_and_type.name.resolve().try_string()?,
                        name_and_type.descriptor.resolve().try_string()?,
                    );

                    let field_value = match objectref {
                        JavaObject::Runtime(data) => data.lock().get_instance_field(&field),
                        JavaObject::String(data) => data.lock().get_instance_field(&field),
                    }
                    .unwrap_or({
                        match descriptor {
                            // TODO: Type these properly
                            FieldType::Base(ty) => match ty {
                                descriptor::BaseType::Boolean => RuntimeValue::Integral(0.into()),
                                descriptor::BaseType::Char => RuntimeValue::Null,
                                descriptor::BaseType::Float => RuntimeValue::Null,
                                descriptor::BaseType::Double => RuntimeValue::Null,
                                descriptor::BaseType::Byte => RuntimeValue::Null,
                                descriptor::BaseType::Short => RuntimeValue::Null,
                                descriptor::BaseType::Int => RuntimeValue::Integral(0.into()),
                                descriptor::BaseType::Long => RuntimeValue::Null,
                                descriptor::BaseType::Void => RuntimeValue::Null,
                            },
                            FieldType::Object(_) => RuntimeValue::Null,
                            FieldType::Array(_) => RuntimeValue::Null,
                        }
                    });

                    operands.push(field_value)
                }
                Opcode::TABLESWITCH(switch) => {
                    let index = operands
                        .pop()
                        .ok_or(anyhow!("no values on the operand stack"))?;

                    let index = *index.as_integral().ok_or(anyhow!("not an int"))?;

                    let branch_offset =
                        if index.value > switch.high as i64 || index.value < switch.low as i64 {
                            *pc as i64 + switch.default as i64
                        } else {
                            index.value - switch.low as i64
                        };

                    *pc += branch_offset as i32;
                }
                Opcode::NEW(class_index) => {
                    let class_cp: ConstantEntry =
                        ctx.object.lock().get_constant(class_index).resolve();

                    let class_cp = class_cp.into_class();

                    if class_cp.is_err() {
                        return Err(anyhow!("value was not a class @ {class_index}"));
                    }

                    let class_cp = class_cp.unwrap();

                    let class_name = class_cp.name.resolve().try_string()?;
                    info!("NEW: {}", class_name);
                    let loaded_target = self.load_class(class_name)?;

                    self.initialise_class(Rc::clone(&loaded_target))?;

                    let instance = RuntimeObject::new(Rc::clone(&loaded_target));
                    operands.push(RuntimeValue::Object(JavaObject::Runtime(Rc::new(
                        Mutex::new(instance),
                    ))));
                }
                Opcode::DUP => {
                    let value = operands
                        .pop()
                        .ok_or(anyhow!("no values on the operand stack"))?;

                    operands.push(value.clone());
                    operands.push(value);
                }
                Opcode::DUP_X1 => {
                    let v1 = operands
                        .pop()
                        .ok_or(anyhow!("no values on the operand stack"))?;
                    let v2 = operands
                        .pop()
                        .ok_or(anyhow!("no values on the operand stack"))?;

                    operands.push(v1.clone());
                    operands.push(v2);
                    operands.push(v1);
                }
                Opcode::DUP_X2 => {
                    let v1 = operands
                        .pop()
                        .ok_or(anyhow!("no values on the operand stack"))?;

                    let v2 = operands
                        .pop()
                        .ok_or(anyhow!("no values on the operand stack"))?;

                    if v1.category_type() == 1 && v2.category_type() == 2 {
                        info!("DUP_X2: form 2");
                        info!("DUP_X2: v1: {:?}, v2: {:?}", v1, v2);
                        operands.push(v1.clone());
                        operands.push(v2);
                        operands.push(v1);
                    } else {
                        info!("DUP_X2: form 1");
                        let v3 = operands
                            .pop()
                            .ok_or(anyhow!("no values on the operand stack"))?;
                        info!("DUP_X2: v1: {:?}, v2: {:?}, v3: {:?}", v1, v2, v3);

                        if v1.category_type() != 1
                            || v2.category_type() != 1
                            || v3.category_type() != 1
                        {
                            return Err(anyhow!(
                                "mismatched category types: v1: {}, v2: {}, v3: {}",
                                v1.category_type(),
                                v2.category_type(),
                                v3.category_type(),
                            ));
                        }

                        operands.push(v1.clone());
                        operands.push(v3);
                        operands.push(v2);
                        operands.push(v1);
                    }
                }
                Opcode::RETURN => {
                    info!("Returning from {} in {}", root_method_name, root_cname);
                    return Ok(None);
                }
                Opcode::ARETURN => {
                    // TODO: Type check for object
                    let return_value = operands.pop().ok_or(anyhow!("no return value"))?;
                    info!(
                        "AReturning from {} in {} with {:?}",
                        root_method_name, root_cname, return_value
                    );
                    return Ok(Some(return_value));
                }
                Opcode::IRETURN => {
                    // TODO: Type check for int & convert if needed
                    let return_value = operands.pop().ok_or(anyhow!("no return value"))?;
                    info!(
                        "IReturning from {} in {} with {:?}",
                        root_method_name, root_cname, return_value
                    );
                    return Ok(Some(return_value));
                }
                Opcode::LRETURN => {
                    // TODO: Type check for int & convert if needed
                    let return_value = operands.pop().ok_or(anyhow!("no return value"))?;
                    info!(
                        "LReturning from {} in {} with {:?}",
                        root_method_name, root_cname, return_value
                    );
                    return Ok(Some(return_value));
                }
                Opcode::DRETURN => {
                    // TODO: Type check for double & convert if needed
                    let return_value = operands.pop().ok_or(anyhow!("no return value"))?;
                    info!(
                        "DReturning from {} in {} with {:?}",
                        root_method_name, root_cname, return_value
                    );
                    return Ok(Some(return_value));
                }
                Opcode::PUTSTATIC(field_index) => {
                    let field_cp: ConstantEntry =
                        ctx.object.lock().get_constant(field_index).resolve();

                    let field_cp = field_cp.into_field();

                    if field_cp.is_err() {
                        return Err(anyhow!("value was not a field @ {field_index}"));
                    }

                    let field_cp = field_cp.unwrap();

                    let name_and_type = field_cp.name_and_type.resolve();
                    let field_name = name_and_type.name.resolve().try_string()?;
                    let descriptor = name_and_type.descriptor.resolve().try_string()?;
                    let class = field_cp.class.resolve();

                    let class_name = class.name.resolve().try_string()?;
                    let loaded_target = self.load_class(class_name.clone())?;

                    self.initialise_class(Rc::clone(&loaded_target))?;

                    let value_to_set = operands
                        .pop()
                        .ok_or(anyhow!("no value on the operand stack to set"))?;

                    // TODO: Type check the popped value
                    let mut loaded_target = loaded_target.lock();

                    info!(
                        "PUTSTATIC: field {} {} in {}",
                        field_name, descriptor, class_name
                    );
                    loaded_target.set_static_field(
                        (
                            name_and_type.name.resolve().try_string()?,
                            name_and_type.descriptor.resolve().try_string()?,
                        ),
                        value_to_set,
                    );
                }
                Opcode::PUTFIELD(field_index) => {
                    let field_cp: ConstantEntry =
                        ctx.object.lock().get_constant(field_index).resolve();

                    let field_cp = field_cp.into_field();

                    if field_cp.is_err() {
                        return Err(anyhow!("value was not a field @ {field_index}"));
                    }

                    let field_cp = field_cp.unwrap();

                    let name_and_type = field_cp.name_and_type.resolve();
                    let class = field_cp.class.resolve();

                    let class_name = class.name.resolve().try_string()?;
                    let loaded_target = self.load_class(class_name.clone())?;

                    self.initialise_class(Rc::clone(&loaded_target))?;

                    let value_to_set = operands
                        .pop()
                        .ok_or(anyhow!("no value on the operand stack to set"))?;

                    let objectref = operands
                        .pop()
                        .ok_or(anyhow!("no value on the operand stack to set"))?;
                    let objectref = objectref
                        .as_object()
                        .cloned()
                        .ok_or(anyhow!("not an object"))?;

                    // TODO: Type check the popped value

                    let field = (
                        name_and_type.name.resolve().try_string()?,
                        name_and_type.descriptor.resolve().try_string()?,
                    );
                    info!(
                        "PUTFIELD: {} in {} = {:?}",
                        field.0, class_name, value_to_set
                    );

                    match objectref {
                        JavaObject::Runtime(data) => {
                            data.lock().set_instance_field(field, value_to_set)
                        }
                        JavaObject::String(data) => {
                            data.lock().set_instance_field(field, value_to_set)
                        }
                    };
                }
                Opcode::INVOKESTATIC(index) => {
                    // The unsigned indexbyte1 and indexbyte2 are used to construct an
                    // index into the run-time constant pool of the current class
                    let target_method: Addressed<ConstantEntry> =
                        ctx.object.lock().get_constant(index);

                    // The run-time constant pool entry at the index must be a symbolic
                    // reference to a method or an interface method (§5.1), which gives
                    // the name and descriptor (§4.3.3) of the method or interface method
                    // as well as a symbolic reference to the class or interface in which
                    // the method or interface method is to be found.
                    let (method_name, method_descriptor, class_name) = match target_method.resolve()
                    {
                        ConstantEntry::Method(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        ConstantEntry::InterfaceMethod(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        e => {
                            return Err(anyhow!("expected interface method / method, got {:#?}", e))
                        }
                    };

                    // The named method is resolved (§5.4.3.3, §5.4.3.4).
                    let loaded_class = self.load_class(class_name.clone())?;

                    let loaded_method = loaded_class
                        .lock()
                        .get_method(method_name.clone(), method_descriptor.to_string())
                        .ok_or(anyhow!("no method {} in {}", method_name, class_name))?;

                    // The resolved method must not be an instance initialization method,
                    // or the class or interface initialization method (§2.9.1, §2.9.2).
                    if method_name == "<clinit>" || method_name == "<init>" {
                        return Err(anyhow!(
                            "cannot call static method {}, it is a class initialisation method",
                            method_name
                        ));
                    }

                    // On successful resolution of the method, the class or interface that
                    // declared the resolved method is initialized if that class or interface
                    // has not already been initialized (§5.5).
                    self.initialise_class(Rc::clone(&loaded_class))?;

                    // If the method is not native, the nargs argument values are popped
                    // from the operand stack. A new frame is created on the Java Virtual
                    // Machine stack for the method being invoked. The nargs argument
                    // values are consecutively made the values of local variables of the
                    // new frame, with arg1 in local variable 0 (or, if arg1 is of type
                    // long or double, in local variables 0 and 1) and so on.
                    let mut new_context =
                        ExecutionContext::new(Rc::clone(&loaded_class), loaded_method);

                    let mut reversed_descriptor = method_descriptor.clone();
                    reversed_descriptor.parameters.reverse();
                    for _arg in reversed_descriptor.parameters.iter() {
                        // TODO: Validate against FieldType in descriptor
                        let arg = operands.pop().ok_or(anyhow!("not enough args"))?;
                        if let Some(int) = arg.as_integral() {
                            if int.ty == IntegralType::Long {
                                new_context.locals.push(arg.clone());
                            }
                        }

                        if let Some(float) = arg.as_floating() {
                            if float.ty == FloatingType::Double {
                                new_context.locals.push(arg.clone());
                            }
                        }
                        new_context.locals.push(arg.clone())
                    }

                    new_context.locals.reverse();

                    let exec_result = if !new_context.method.flags.has(MethodAccessFlag::NATIVE) {
                        // The new frame is then made current, and the Java Virtual Machine pc is set
                        // to the opcode of the first instruction of the method to be invoked.
                        // Execution continues with the first instruction of the method.
                        self.run_code(new_context)
                    } else {
                        let lookup = new_context
                            .object
                            .lock()
                            .get_native_method(&(
                                method_name.clone(),
                                method_descriptor.to_string(),
                            ))
                            .ok_or(anyhow!(
                                "no native method {} {:?} {} / {}",
                                class_name,
                                new_context.method.flags.flags,
                                method_name,
                                method_descriptor.to_string()
                            ))?;

                        info!("INVOKESTATIC: invoking native method {}", method_name);

                        match lookup {
                            NativeFunction::Static(func) => {
                                func(Rc::clone(&new_context.object), new_context.locals, self)
                            }
                            _ => {
                                return Err(anyhow!(
                                    "attempted to INVOKESTATIC an instance native method"
                                ))
                            }
                        }
                    };

                    if let Err(e) = exec_result {
                        return Err(e.context(format!(
                            "when interpreting {} in {}",
                            method_name, class_name
                        )));
                    }

                    // Caller gave us a value, push it to our stack (Xreturn does this)
                    if let Some(return_value) = exec_result.unwrap() {
                        operands.push(return_value);
                    }
                }
                Opcode::INVOKEVIRTUAL(index) => {
                    // The unsigned indexbyte1 and indexbyte2 are used to construct an
                    // index into the run-time constant pool of the current class
                    let target_method: Addressed<ConstantEntry> =
                        ctx.object.lock().get_constant(index);

                    // The run-time constant pool entry at the index must be a symbolic
                    // reference to a method or an interface method (§5.1), which gives
                    // the name and descriptor (§4.3.3) of the method or interface method
                    // as well as a symbolic reference to the class or interface in which
                    // the method or interface method is to be found.
                    let (method_name, method_descriptor, class_name) = match target_method.resolve()
                    {
                        ConstantEntry::Method(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        ConstantEntry::InterfaceMethod(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        e => {
                            return Err(anyhow!("expected interface method / method, got {:#?}", e))
                        }
                    };

                    // The named method is resolved (§5.4.3.3, §5.4.3.4).
                    let loaded_class = self.load_class(class_name.clone())?;
                    let loaded_method = self
                        .resolve_method(
                            Rc::clone(&loaded_class),
                            method_name.clone(),
                            method_descriptor.to_string(),
                        )?
                        .ok_or(anyhow!(
                            "no method {} {} in {} or its parents",
                            method_name,
                            method_descriptor.to_string(),
                            class_name
                        ))?;

                    // If the resolved method is not signature polymorphic (§2.9.3), then
                    // the invokevirtual instruction proceeds as follows.
                    // TODO: Support signature polymorphic methods

                    // The objectref must be followed on the operand stack by nargs
                    // argument values, where the number, type, and order of the values
                    // must be consistent with the descriptor of the selected instance
                    // method.

                    let mut func_args = vec![];

                    let mut reversed_descriptor = method_descriptor.clone();
                    reversed_descriptor.parameters.reverse();
                    for _arg in reversed_descriptor.parameters.iter() {
                        // TODO: Validate against FieldType in descriptor
                        let arg = operands.pop().ok_or(anyhow!("not enough args"))?;
                        if let Some(int) = arg.as_integral() {
                            if int.ty == IntegralType::Long {
                                func_args.push(arg.clone());
                            }
                        }

                        if let Some(float) = arg.as_floating() {
                            if float.ty == FloatingType::Double {
                                func_args.push(arg.clone());
                            }
                        }
                        func_args.push(arg.clone())
                    }

                    let objectref = operands.pop().ok_or(anyhow!("no objectreference"))?;
                    if objectref.as_object().is_none() {
                        panic!(
                            "when caling {} ({}) in {} (from {} in {}), object was {:#?}",
                            method_name,
                            method_descriptor.to_string(),
                            class_name,
                            root_method_name,
                            root_cname,
                            objectref
                        )
                    }

                    // Let C be the class of objectref. A method is selected with respect
                    // to C and the resolved method (§5.4.6). This is the method to be invoked.
                    let objectclass = objectref.as_object().unwrap().clone().as_class_object();
                    let (selected_class, selected_method) = self
                        .select_method(objectclass, loaded_class, loaded_method)?
                        .ok_or(anyhow!(
                            "could not resolve method {} {}",
                            method_name,
                            method_descriptor.to_string()
                        ))?;

                    func_args.push(objectref.clone());
                    func_args.reverse();

                    let mut new_context = ExecutionContext::new(selected_class, selected_method);
                    new_context.locals = func_args;
                    let exec_result = if !new_context.method.flags.has(MethodAccessFlag::NATIVE) {
                        // The new frame is then made current, and the Java Virtual Machine pc is set
                        // to the opcode of the first instruction of the method to be invoked.
                        // Execution continues with the first instruction of the method.
                        self.run_code(new_context)
                    } else {
                        let lookup = new_context
                            .object
                            .lock()
                            .get_native_method(&(
                                method_name.clone(),
                                method_descriptor.to_string(),
                            ))
                            .ok_or(anyhow!(
                                "no native method {} {:?} {} / {}",
                                class_name,
                                new_context.method.flags.flags,
                                method_name,
                                method_descriptor.to_string()
                            ))?;
                        info!("INVOKEVIRTUAL: invoking native method {}", method_name);

                        match lookup {
                            NativeFunction::Static(func) => {
                                func(Rc::clone(&new_context.object), new_context.locals, self)
                            }
                            NativeFunction::Instance(func) => {
                                let obj = objectref.as_object().unwrap().clone();
                                func(obj, new_context.locals, self)
                            }
                        }
                    };

                    if let Err(e) = exec_result {
                        return Err(e.context(format!(
                            "when interpreting {} in {}",
                            method_name, class_name
                        )));
                    }

                    // Caller gave us a value, push it to our stack (Xreturn does this)
                    if let Some(return_value) = exec_result.unwrap() {
                        operands.push(return_value);
                    }
                }
                Opcode::INVOKESPECIAL(index) => {
                    // The unsigned indexbyte1 and indexbyte2 are used to construct an
                    // index into the run-time constant pool of the current class
                    let target_method: Addressed<ConstantEntry> =
                        ctx.object.lock().get_constant(index);

                    // The run-time constant pool entry at the index must be a symbolic
                    // reference to a method or an interface method (§5.1), which gives
                    // the name and descriptor (§4.3.3) of the method or interface method
                    // as well as a symbolic reference to the class or interface in which
                    // the method or interface method is to be found.
                    let (method_name, method_descriptor, class_name) = match target_method.resolve()
                    {
                        ConstantEntry::Method(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        ConstantEntry::InterfaceMethod(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        e => {
                            return Err(anyhow!("expected interface method / method, got {:#?}", e))
                        }
                    };

                    info!(
                        "INVOKESPECIAL: invoking {} {} in {}",
                        method_name,
                        method_descriptor.to_string(),
                        class_name
                    );

                    // The named method is resolved (§5.4.3.3, §5.4.3.4).
                    let loaded_class = self.load_class(class_name.clone())?;

                    let loaded_method = loaded_class
                        .lock()
                        .get_method(method_name.clone(), method_descriptor.to_string())
                        .ok_or(anyhow!("no method {} in {}", method_name, class_name))?;

                    // If all of the following are true, let C be the direct superclass of the current class:
                    //
                    // • The resolved method is not an instance initialization method
                    // (§2.9.1).
                    // • The symbolic reference names a class (not an interface), and that
                    // class is a superclass of the current class.
                    // • The ACC_SUPER flag is set for the class file (§4.1).
                    // Otherwise, let C be the class or interface named by the symbolic
                    // reference
                    // TODO: Proper resolution ^

                    // The objectref must be followed on the operand stack by nargs
                    // argument values, where the number, type, and order of the values
                    // must be consistent with the descriptor of the selected instance
                    // method.

                    let mut func_args = vec![];

                    let mut reversed_descriptor = method_descriptor.clone();
                    reversed_descriptor.parameters.reverse();
                    for _arg in reversed_descriptor.parameters.iter() {
                        // TODO: Validate against FieldType in descriptor
                        let arg = operands.pop().ok_or(anyhow!("not enough args"))?;
                        if let Some(int) = arg.as_integral() {
                            if int.ty == IntegralType::Long {
                                func_args.push(arg.clone());
                            }
                        }

                        if let Some(float) = arg.as_floating() {
                            if float.ty == FloatingType::Double {
                                func_args.push(arg.clone());
                            }
                        }
                        func_args.push(arg.clone());
                    }

                    let objectref = operands.pop().ok_or(anyhow!("no objectreference"))?;
                    if objectref.as_object().is_none() {
                        panic!(
                            "when caling {} ({}) in {} (from {} in {}), object was {:#?}",
                            method_name,
                            method_descriptor.to_string(),
                            class_name,
                            root_method_name,
                            root_cname,
                            objectref
                        )
                    }

                    // TODO: is this an acceptable resolution path (barring ^ superclass resolution)?
                    let (selected_method, selected_class) = (loaded_method, loaded_class);

                    func_args.push(objectref);
                    func_args.reverse();

                    let mut new_context = ExecutionContext::new(selected_class, selected_method);
                    new_context.locals = func_args;

                    let exec_result = if !new_context.method.flags.has(MethodAccessFlag::NATIVE) {
                        // The new frame is then made current, and the Java Virtual Machine pc is set
                        // to the opcode of the first instruction of the method to be invoked.
                        // Execution continues with the first instruction of the method.
                        self.run_code(new_context)
                    } else {
                        let lookup = new_context
                            .object
                            .lock()
                            .get_native_method(&(
                                method_name.clone(),
                                method_descriptor.to_string(),
                            ))
                            .ok_or(anyhow!(
                                "no native method {} {:?} {} / {}",
                                class_name,
                                new_context.method.flags.flags,
                                method_name,
                                method_descriptor.to_string()
                            ))?;
                        info!("INVOKESPECIAL: invoking native method {}", method_name);

                        match lookup {
                            NativeFunction::Static(func) => {
                                func(Rc::clone(&new_context.object), new_context.locals, self)
                            }
                            _ => {
                                return Err(anyhow!(
                                    "attempted to INVOKESTATIC an instance native method"
                                ))
                            }
                        }
                    };

                    if let Err(e) = exec_result {
                        return Err(e.context(format!(
                            "when interpreting {} in {}",
                            method_name, class_name
                        )));
                    }

                    // Caller gave us a value, push it to our stack (Xreturn does this)
                    if let Some(return_value) = exec_result.unwrap() {
                        operands.push(return_value);
                    }
                }
                Opcode::INVOKEINTERFACE(index, count, zero) => {
                    // The unsigned indexbyte1 and indexbyte2 are used to construct an
                    // index into the run-time constant pool of the current class
                    let target_method: Addressed<ConstantEntry> =
                        ctx.object.lock().get_constant(index);

                    // The count operand is an unsigned byte that must not be zero
                    if count == 0 {
                        return Err(anyhow!("count == 0, expected non zero"));
                    }

                    // The value of the fourth operand byte must always be zero.
                    if zero != 0 {
                        return Err(anyhow!("zero != 0, got {}", zero));
                    }

                    // The run-time constant pool entry at the index must be a symbolic
                    // reference to a method or an interface method (§5.1), which gives
                    // the name and descriptor (§4.3.3) of the method or interface method
                    // as well as a symbolic reference to the class or interface in which
                    // the method or interface method is to be found.
                    let (method_name, method_descriptor, class_name) = match target_method.resolve()
                    {
                        ConstantEntry::Method(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        ConstantEntry::InterfaceMethod(data) => {
                            let name_and_type = data.name_and_type.resolve();
                            let method_name = name_and_type.name.resolve().try_string()?;

                            let method_descriptor =
                                name_and_type.descriptor.resolve().try_string()?;
                            let method_descriptor = MethodType::parse(method_descriptor)?;

                            let class = data.class.resolve();
                            let class = class.name.resolve().try_string()?;

                            (method_name, method_descriptor, class)
                        }
                        e => {
                            return Err(anyhow!("expected interface method / method, got {:#?}", e))
                        }
                    };

                    // The named method is resolved (§5.4.3.3, §5.4.3.4).
                    let loaded_class = self.load_class(class_name.clone())?;

                    let loaded_method = loaded_class
                        .lock()
                        .get_method(method_name.clone(), method_descriptor.to_string())
                        .ok_or(anyhow!("no method {} in {}", method_name, class_name))?;

                    // The resolved interface method must not be an instance
                    // initialization method, or the class or interface initialization method
                    // (§2.9.1, §2.9.2).
                    if method_name == "<clinit>" || method_name == "<init>" {
                        return Err(anyhow!(
                            "cannot call interface method {}, it is an initialisation method",
                            method_name
                        ));
                    }

                    // The objectref must be followed on the operand stack by nargs
                    // argument values, where the number, type, and order of the values
                    // must be consistent with the descriptor of the selected instance
                    // method.
                    let mut func_args = vec![];

                    let mut reversed_descriptor = method_descriptor.clone();
                    reversed_descriptor.parameters.reverse();
                    for _arg in reversed_descriptor.parameters.iter() {
                        // TODO: Validate against FieldType in descriptor
                        let arg = operands.pop().ok_or(anyhow!("not enough args"))?;
                        if let Some(int) = arg.as_integral() {
                            if int.ty == IntegralType::Long {
                                func_args.push(arg.clone());
                            }
                        }

                        if let Some(float) = arg.as_floating() {
                            if float.ty == FloatingType::Double {
                                func_args.push(arg.clone());
                            }
                        }
                        func_args.push(arg.clone())
                    }

                    let objectref = operands.pop().ok_or(anyhow!("no objectreference"))?;
                    if objectref.as_object().is_none() {
                        panic!(
                            "when caling {} ({}) in {} (from {} in {}), object was {:#?}",
                            method_name,
                            method_descriptor.to_string(),
                            class_name,
                            root_method_name,
                            root_cname,
                            objectref
                        )
                    }

                    // Let C be the class of objectref. A method is selected with respect
                    // to C and the resolved method (§5.4.6). This is the method to be invoked.
                    let objectclass = objectref.as_object().unwrap().clone().as_class_object();
                    let (selected_class, selected_method) = self
                        .select_method(objectclass, loaded_class, loaded_method)?
                        .ok_or(anyhow!(
                            "could not resolve method {} {}",
                            method_name,
                            method_descriptor.to_string()
                        ))?;

                    func_args.push(objectref.clone());
                    func_args.reverse();

                    let mut new_context = ExecutionContext::new(selected_class, selected_method);
                    new_context.locals = func_args;
                    let exec_result = if !new_context.method.flags.has(MethodAccessFlag::NATIVE) {
                        // The new frame is then made current, and the Java Virtual Machine pc is set
                        // to the opcode of the first instruction of the method to be invoked.
                        // Execution continues with the first instruction of the method.
                        self.run_code(new_context)
                    } else {
                        let lookup = ctx
                            .object
                            .lock()
                            .get_native_method(&(
                                method_name.clone(),
                                method_descriptor.to_string(),
                            ))
                            .ok_or(anyhow!(
                                "no native method {} {:?} {} / {}",
                                class_name,
                                new_context.method.flags.flags,
                                method_name,
                                method_descriptor.to_string()
                            ))?;

                        info!("INVOKEINTERFACE: invoking native method {}", method_name);
                        match lookup {
                            NativeFunction::Static(func) => {
                                func(Rc::clone(&ctx.object), new_context.locals, self)
                            }
                            NativeFunction::Instance(func) => {
                                let obj = objectref.as_object().unwrap().clone();
                                func(obj, new_context.locals, self)
                            }
                        }
                    };

                    if let Err(e) = exec_result {
                        return Err(e.context(format!(
                            "when interpreting {} in {}",
                            method_name, class_name
                        )));
                    }

                    // Caller gave us a value, push it to our stack (Xreturn does this)
                    if let Some(return_value) = exec_result.unwrap() {
                        operands.push(return_value);
                    }
                }
                _ => todo!("{:#?} is not implemented", opcode),
            }

            *pc += bytes_consumed_by_opcode;
        }

        todo!("what should we do when execution finishes?");
    }

    fn resolve_method(
        &self,
        class: Rc<Mutex<ClassObject>>,
        method_name: String,
        method_descriptor: String,
    ) -> Result<Option<Method>> {
        // To resolve an unresolved symbolic reference from D to a method in a class C, the
        // symbolic reference to C given by the method reference is first resolved (§5.4.3.1).

        // When resolving a method reference:
        // 1. If C is an interface, method resolution throws an IncompatibleClassChangeError.
        if class.lock().is_interface() {
            return Err(anyhow!("cannot resolve method on interface"));
        }

        // 2. Otherwise, method resolution attempts to locate the referenced method in C and its superclasses:
        // • If C declares exactly one method with the name specified by the method
        // reference, and the declaration is a signature polymorphic method (§2.9.3),
        // then method lookup succeeds. All the class names mentioned in the
        // descriptor are resolved (§5.4.3.1).
        // The resolved method is the signature polymorphic method declaration. It is
        // not necessary for C to declare a method with the descriptor specified by the
        // method reference.
        // • Otherwise, if C declares a method with the name and descriptor specified by
        // the method reference, method lookup succeeds.

        let class_method = class
            .lock()
            .get_method(method_name.clone(), method_descriptor.clone());

        if let Some(class_method) = class_method {
            return Ok(Some(class_method));
        }

        // • Otherwise, if C has a superclass, step 2 of method resolution is recursively
        // invoked on the direct superclass of C.
        if let Some(super_class) = class.lock().get_super_class() {
            let class_name = super_class.name.resolve().try_string()?;
            let super_class = self.load_class(class_name)?;

            return self.resolve_method(super_class, method_name, method_descriptor);
        }

        // Otherwise, method resolution attempts to locate the referenced method in the
        // superinterfaces of the specified class C:
        // • If the maximally-specific superinterface methods of C for the name and
        // descriptor specified by the method reference include exactly one method that
        // does not have its ACC_ABSTRACT flag set, then this method is chosen and
        // method lookup succeeds.
        // • Otherwise, if any superinterface of C declares a method with the name and
        // descriptor specified by the method reference that has neither its ACC_PRIVATE
        // flag nor its ACC_STATIC flag set, one of these is arbitrarily chosen and method
        // lookup succeeds.

        // TODO: Interface resolution

        // • Otherwise, method lookup fails.

        Ok(None)
    }
    fn select_method(
        &self,
        class: Rc<Mutex<ClassObject>>,
        declared_class: Rc<Mutex<ClassObject>>,
        method: Method,
    ) -> Result<Option<(Rc<Mutex<ClassObject>>, Method)>> {
        // During execution of an invokeinterface or invokevirtual instruction, a method is
        // selected with respect to (i) the run-time type of the object on the stack, and (ii) a
        // method that was previously resolved by the instruction. The rules to select a method
        // with respect to a class or interface C and a method mR are as follows:

        let (method_name, method_descriptor) = (
            method.name.resolve().try_string()?,
            method.descriptor.resolve().try_string()?,
        );

        let class_name = class.lock().get_class_name();
        let declared_class_name = declared_class.lock().get_class_name();
        info!(
            "select_method: {} {} in {} (declared as {})",
            method_name, method_descriptor, class_name, declared_class_name
        );

        // 1. If mR is marked ACC_PRIVATE, then it is the selected method.
        if method.flags.has(MethodAccessFlag::PRIVATE) {
            info!("select_method: was private, using it");

            let method = declared_class
                .lock()
                .get_method(method_name, method_descriptor)
                .ok_or(anyhow!("could not resolve method"))?;

            return Ok(Some((declared_class, method)));
        }

        // 2. Otherwise, the selected method is determined by the following lookup procedure:
        // If C contains a declaration of an instance method m that can override mR (§5.4.5), then m is the selected method.
        let instance_method = class
            .lock()
            .get_method(method_name.clone(), method_descriptor.clone());

        if let Some(instance_method) = instance_method {
            info!("select_method: found instance method");
            if Interpreter::method_can_override(&method, &instance_method) {
                info!("select_method: could override, using it");

                return Ok(Some((class, instance_method)));
            }
            info!("select_method: could not override, skipping it");
        }

        // Otherwise, if C has a superclass, a search for a declaration of an instance
        // method that can override mR is performed, starting with the direct superclass
        // of C and continuing with the direct superclass of that class, and so forth, until
        // a method is found or no further superclasses exist. If a method is found, it
        // is the selected method.
        let super_class = class.lock().get_super_class();
        if let Some(super_class) = super_class {
            let class_name = super_class.name.resolve().try_string()?;
            info!(
                "select_method: attempting resolution in super class {}",
                class_name
            );
            let super_class = self.load_class(class_name)?;

            return self.select_method(super_class, declared_class, method);
        }

        // Otherwise, the maximally-specific superinterface methods of C are
        // determined (§5.4.3.3). If exactly one matches mR's name and descriptor and
        // is not abstract, then it is the selected method.
        // TODO: This, once we figure out a better way to handle super classes

        Ok(None)
    }

    fn method_can_override(base: &Method, derived: &Method) -> bool {
        // An instance method mC can override another instance method mA iff all of the following are true:
        let (base_name, base_descriptor) = (
            base.name.resolve().string(),
            base.descriptor.resolve().string(),
        );

        let (derived_name, derived_descriptor) = (
            derived.name.resolve().string(),
            derived.descriptor.resolve().string(),
        );

        // mC has the same name and descriptor as mA.
        if base_name != derived_name && base_descriptor != derived_descriptor {
            return false;
        }

        // mC is not marked ACC_PRIVATE.
        let flags = &base.flags;
        if flags.has(MethodAccessFlag::PRIVATE) {
            return false;
        }

        // One of the following is true:
        // – mA is marked ACC_PUBLIC.
        // – mA is marked ACC_PROTECTED.
        // – mA is marked neither ACC_PUBLIC nor ACC_PROTECTED nor ACC_PRIVATE, and
        // either (a) the declaration of mA appears in the same run-time package as the
        // declaration of mC, or (b) if mA is declared in a class A and mC is declared in a class
        // C, then there exists a method mB declared in a class B such that C is a subclass
        // of B and B is a subclass of A and mC can override mB and mB can override mA.

        if flags.has(MethodAccessFlag::PUBLIC) || flags.has(MethodAccessFlag::PROTECTED) {
            return true;
        }

        // TODO: Try resolving package information before returning true
        true
    }
}
