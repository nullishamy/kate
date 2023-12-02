use std::{cell::RefCell, process::exit};

use anyhow::anyhow;
use args::Cli;
use clap::Parser;
use interpreter::{
    error::{Frame, Throwable},
    native::{DefaultNativeModule, NativeFunction},
    object::{
        builtins::{Array, Class},
        interner::{set_interner, StringInterner},
        layout::types::Byte,
        loader::ClassLoader,
        mem::{FieldRef, RefTo},
        runtime::RuntimeValue,
    },
    static_method, Context, VM, ThrownState,
};
use parse::attributes::CodeAttribute;
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

            let bytes = bytes.to_ref().unwrap_ref().slice().to_vec();

            let str =
                decode_string((CompactEncoding::Utf16, bytes)).expect("could not decode string");

            println!("{:#?}", str);
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

fn boot_system(vm: &mut VM, cls: RefTo<Class>) {
    info!("Booting system");

    let java_lang_system = vm
        .class_loader
        .for_name("java/lang/System".to_string())
        .unwrap();

    let init_phase_1 = java_lang_system
        .unwrap_ref()
        .class_file()
        .methods
        .locate("initPhase1".to_string(), "()V".to_string())
        .cloned()
        .unwrap();

    let code = init_phase_1
        .attributes
        .known_attribute::<CodeAttribute>(&cls.unwrap_ref().class_file().constant_pool)
        .unwrap();

    let ctx = Context {
        class: java_lang_system,
        code,
        operands: vec![],
        locals: vec![],
        pc: 0,
    };

    let res = vm.run(ctx);
    if let Err((e, _)) = res {
        println!("Uncaught exception when booting system: {}", e);

        if let Throwable::Runtime(err) = e {
            for source in err.sources.iter().rev() {
                println!("  {}", source);
            }
        } else if let Throwable::Internal(_) = e {
            for source in vm.frames.iter().rev() {
                println!("  {}", source);
            }
        }

        println!(
            "  {}",
            Frame {
                class_name: "java/lang/System".to_string(),
                method_name: "initPhase1".to_string()
            }
        );
        exit(1);
    } else {
        info!("Booted system")
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

    let source_root = env!("CARGO_MANIFEST_DIR");
    let mut class_loader = ClassLoader::new();

    class_loader
        .add_path(format!("{source_root}/../../std/java.base"))
        .add_path(format!("{source_root}/../../samples"));

    for cp in &args.classpath {
        class_loader.add_path(cp);
    }

    let bootstrapped_classes = class_loader.bootstrap().unwrap();

    let interner = StringInterner::new(
        bootstrapped_classes.java_lang_string.clone(),
        bootstrapped_classes.java_lang_object.clone(),
    );

    set_interner(interner);

    let mut vm = VM {
        class_loader,
        frames: Vec::new(),
        main_thread: RefTo::null(),
    };

    vm.bootstrap().unwrap();

    info!("Bootstrap complete");

    for class_name in &args.classes {
        let cls = vm.class_loader.for_name(class_name.clone()).unwrap();
        if args.has_option(opts::TEST_INIT) {
            test_init(cls.clone());
        }

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

        let ctx = Context {
            class: cls,
            code,
            operands: vec![],
            locals: vec![],
            pc: 0,
        };

        info!("Entering main");
        let res = if args.has_option(opts::TEST_THROW_INTERNAL) {
            Err((Throwable::Internal(anyhow!("testing, internal errors")), ThrownState { pc: -1 }))
        } else {
            vm.run(ctx)
        };

        if let Err((e, _)) = res {
            if let Throwable::Internal(_) = e {
                println!("/----------------------------------------------------------\\");
                println!("|The VM encountered an unrecoverable error and had to abort.|");
                println!("\\----------------------------------------------------------/");
            }

            println!("Uncaught exception in main: {}", e);

            if let Throwable::Runtime(err) = e {
                for source in err.sources.iter().rev() {
                    println!("  {}", source);
                }
            }

            println!(
                "  {}",
                Frame {
                    class_name: class_name.to_string(),
                    method_name: "main".to_string()
                }
            );
            exit(1);
        } else {
            info!("Execution concluded without error")
        }
    }
}
