use std::{cell::RefCell, process::exit};

use anyhow::anyhow;
use args::Cli;
use clap::Parser;

use interpreter::{Context, Interpreter};
use parse::{
    attributes::CodeAttribute,
    classfile::{Method, Resolvable},
};
use runtime::{
    error::{Frame, Throwable, ThrownState},
    native::{DefaultNativeModule, NativeFunction},
    object::{
        builtins::{Array, BuiltinString, Class},
        interner::{set_interner, StringInterner},
        layout::types::Byte,
        loader::ClassLoader,
        mem::{FieldRef, RefTo},
        value::RuntimeValue,
    },
    vm::VM,
};
use runtime::{object::interner::intern_string, static_method};
use support::encoding::{decode_string, CompactEncoding};
use tracing::{error, info, Level};
use tracing_subscriber::fmt;

use crate::args::opts;

mod args;

fn test_init(cls: RefTo<Class>) {
    let cls = cls.unwrap_mut();
    macro_rules! printer {
        ($desc: expr, $printer: expr) => {
            (
                $desc,
                static_method!(|_, args, _| {
                    let printer = $printer;
                    printer(args[0].clone());
                    Ok(None)
                }),
            )
        };
        ($desc: expr) => {
            printer!($desc, |a| {
                println!("{}", a);
            })
        };
    }

    for printer in [
        printer!("(I)V"),
        printer!("(Z)V", |a: RuntimeValue| {
            let int_value = a.as_integral().expect("was not an int (bool)").value;
            if int_value == 0 {
                println!("false")
            } else {
                println!("true")
            }
        }),
        printer!("(C)V", |a: RuntimeValue| {
            let char_value = a.as_integral().expect("was not an int (char)").value;
            println!(
                "{}",
                char::from_u32(char_value as u32).unwrap_or_else(|| {
                    panic!("{} was not a char", char_value);
                })
            )
        }),
        printer!("(J)V"),
        printer!("(D)V"),
        printer!("(F)V"),
        printer!("(S)V"),
        printer!("(B)V"),
        printer!("(Ljava/lang/String;)V", |a: RuntimeValue| {
            let string = a.as_object().expect("was not an object (string)");
            if string.is_null() {
                println!("null");
                return;
            }

            let string = string.unwrap_ref();
            let bytes: FieldRef<RefTo<Array<u8>>> = string
                .field(("value".to_string(), "[B".to_string()))
                .expect("could not locate value field");

            let bytes = bytes.unwrap_ref().unwrap_ref().slice().to_vec();

            let str =
                decode_string((CompactEncoding::Utf16, bytes)).expect("could not decode string");

            println!("{}", str);
        }),
        printer!("([B)V", |arr: RuntimeValue| {
            let arr = arr.as_object().expect("not an object (byte[])");
            let arr = unsafe { arr.cast::<Array<Byte>>() };

            println!(
                "[{}]",
                arr.unwrap_ref()
                    .slice()
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }),
    ] {
        if cls.native_module().is_none() {
            cls.set_native_module(Box::new(RefCell::new(DefaultNativeModule::new(
                // Leaking is fine. The class that spawned the test will live for the whole test.
                // We need this because the class name is required to be a &'static str, just so we can use
                // string literals without worrying about lifetimes
                cls.name().clone().leak(),
            ))));
        }

        let module = cls.native_module().as_ref().unwrap();
        let mut module = module.borrow_mut();
        module.set_method("print", printer.0, printer.1);
    }
}

fn boot_system(vm: &mut Interpreter, cls: RefTo<Class>) {
    info!("Booting system");

    let java_lang_system = vm
        .class_loader()
        .for_name("Ljava/lang/System;".into())
        .unwrap();

    let ip1 = java_lang_system
        .unwrap_ref()
        .class_file()
        .methods
        .locate("initPhase1".to_string(), "()V".to_string())
        .cloned()
        .unwrap();

    let code = ip1
        .attributes
        .known_attribute::<CodeAttribute>(&cls.unwrap_ref().class_file().constant_pool)
        .unwrap();

    let ctx = Context {
        class: java_lang_system,
        code,
        operands: vec![],
        is_reentry: false,
        locals: vec![],
        pc: 0,
    };

    vm.push_frame(Frame {
        class_name: "java/lang/System".to_string(),
        method_name: "initPhase1".to_string(),
    });

    let res = vm.run(ctx);
    if let Err((e, _)) = res {
        println!("Uncaught exception when booting system: {}", e);

        if let Throwable::Runtime(err) = e {
            for source in err.sources.iter().rev() {
                println!("  {}", source);
            }
        } else if let Throwable::Internal(_) = e {
            for source in vm.frames().iter().rev() {
                println!("  {}", source);
            }
        }
        exit(1);
    } else {
        info!("Booted system")
    }
}

fn run_method(vm: &mut Interpreter, ctx: Context, method: &Method, args: &Cli) {
    let code = ctx.code.clone();
    let cls = ctx.class.clone();

    let res = if args.has_option(opts::TEST_THROW_INTERNAL) {
        Err((
            Throwable::Internal(anyhow!("testing, internal errors")),
            ThrownState { pc: -1 },
        ))
    } else {
        vm.run(ctx)
    };

    if let Err((err, state)) = res {
        if let Throwable::Internal(ite) = &err {
            println!("/----------------------------------------------------------\\");
            println!("|The VM encountered an unrecoverable error and had to abort.|");
            println!("\\----------------------------------------------------------/");
            println!("Uncaught exception in main: {}", ite);

            for source in vm.frames().iter().rev() {
                println!("  {}", source);
            }
        }

        if let Throwable::Runtime(rte) = &err {
            if let Some(entry) = err.caught_by(vm, &code, &state).unwrap() {
                let method_name = method.name.resolve().string();
                let class_name = cls.unwrap_ref().name();

                // We should consider the performed call "fully returned" because we are now back
                // As such, we should clear frames that came after ours
                let our_frame_index = vm
                    .frames()
                    .iter()
                    .position(|f| &f.class_name == class_name && f.method_name == method_name)
                    .unwrap();

                let len = vm.frames().len();
                drop(vm.frames_mut().drain(our_frame_index + 1..len));

                let re_enter_context = Context {
                    code: code.clone(),
                    class: cls,
                    is_reentry: true,
                    pc: entry.handler_pc as i32,
                    // Push the exception object as the first operand
                    operands: vec![rte.obj.clone()],
                    locals: vec![],
                };

                info!("Re-entering main at {}", re_enter_context.pc);

                return run_method(vm, re_enter_context, method, args);
            } else {
                println!("Uncaught exception in main: {}", rte);
                for source in rte.sources.iter().rev() {
                    println!("  {}", source);
                }
            }
        }

        exit(1);
    } else {
        info!("Execution concluded without error")
    }
}

fn main() {
    let args = Cli::parse();

    let mut format = fmt::format()
        .with_ansi(true)
        .without_time()
        .with_level(true)
        .with_target(false)
        .with_thread_names(false)
        .with_source_location(true)
        .compact();

    if args.has_option(opts::TEST_INIT) {
        format = format.with_ansi(false).with_source_location(false);
    }

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .event_format(format)
        .with_writer(std::io::stderr)
        .init();

    info!("System starting up");

    if args.classes.is_empty() {
        error!("No classes given.");
        return;
    }

    let mut class_loader = ClassLoader::new();

    if let Some(std) = &args.std {
        class_loader.add_path(std);
    }

    for cp in &args.classpath {
        class_loader.add_path(cp);
    }

    // Init the natives that we declare in kate/Util
    if args.has_option(opts::TEST_INIT) {
        let kate_util = class_loader.for_name("Lkate/Util;".into()).unwrap();
        test_init(kate_util)
    }

    let bootstrapped_classes = class_loader.bootstrap().unwrap();

    let interner = StringInterner::new(
        bootstrapped_classes.java_lang_string.clone(),
        bootstrapped_classes.java_lang_object.clone(),
        bootstrapped_classes.byte_array_ty.clone(),
    );

    set_interner(interner);

    let max_stack = args
        .get_option(opts::MAX_STACK)
        .and_then(|f| f.parse::<u64>().ok())
        .unwrap_or(128);

    let mut vm = Interpreter::new(
        VM::new(class_loader),
        interpreter::BootOptions { max_stack },
    );

    vm.bootstrap().unwrap();

    info!("Bootstrap complete");

    for class_name in &args.classes {
        let cls = vm
            .class_loader()
            .for_name(format!("L{};", class_name).into())
            .unwrap();
        if args.has_option(opts::TEST_BOOT) {
            boot_system(&mut vm, cls.clone());
        }

        let method = cls
            .unwrap_ref()
            .class_file()
            .methods
            .locate("main".to_string(), "([Ljava/lang/String;)V".to_string())
            .unwrap();

        let code = method
            .attributes
            .known_attribute::<CodeAttribute>(&cls.unwrap_ref().class_file().constant_pool)
            .unwrap();

        let string_array_ty = vm
            .class_loader()
            .for_name("[Ljava/lang/String;".into())
            .unwrap();

        let cli_args = args
            .extras
            .iter()
            .map(|s| intern_string(s.to_string()))
            .collect::<Result<_, Throwable>>()
            .unwrap();

        let cli_args: RefTo<Array<RefTo<BuiltinString>>> =
            Array::from_vec(string_array_ty, cli_args);

        let main_ctx = Context {
            class: cls.clone(),
            code: code.clone(),
            is_reentry: false,
            operands: vec![],
            locals: vec![RuntimeValue::Object(cli_args.erase())],
            pc: 0,
        };

        info!("Entering main");
        vm.push_frame(Frame {
            class_name: cls.unwrap_ref().name().to_string(),
            method_name: "main".to_string(),
        });
        run_method(&mut vm, main_ctx, method, &args);
    }
}
