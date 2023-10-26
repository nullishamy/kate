use std::process::exit;

use args::Cli;
use clap::Parser;
use interpreter_two::{object::{classloader::ClassLoader, RuntimeValue}, Context, VM, native::NativeFunction, static_method};
use parse::attributes::CodeAttribute;
use tracing::{error, info, Level};
use tracing_subscriber::fmt;

mod args;

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

    if args.test {
        format = format.with_ansi(false).with_source_location(false);
    }

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .event_format(format)
        .with_writer(std::io::stderr)
        .init();

    if args.classes.is_empty() {
        error!("No classes given.");
        return;
    }

    let source_root = env!("CARGO_MANIFEST_DIR");
    let mut vm = VM {
        class_loader: ClassLoader::new(),
    };

    vm.class_loader
        .add_path(format!("{source_root}/../../std/java.base").into())
        .add_path(".".into());

    for cp in args.classpath {
        vm.class_loader.add_path(cp.into());
    }

    vm.class_loader.bootstrap().unwrap();

    for class in args.classes {
        let _cls = vm.class_loader.load_class(class).unwrap();
        let mut cls = _cls.write();
        if args.test {
            macro_rules! printer {
                ($desc: expr, $printer: expr) => {
                    static_method!(name: "print", descriptor: $desc => |_, args, _| {
                        $printer(args[0].clone());
                        Ok(None)
                    })
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
                    println!("{}", char::from_u32(char_value as u32).expect(&format!("{} was not a char", char_value)))
                }),
                printer!("(J)V"),
                printer!("(D)V"),
                printer!("(F)V"),
                printer!("(S)V"),
                printer!("(B)V"),
            ] {
                cls.register_native(printer.0, printer.1);
            }

        }
        let method = cls
            .get_method(("main".to_string(), "([Ljava/lang/String;)V".to_string()))
            .unwrap();

        let code = method
            .attributes
            .known_attribute::<CodeAttribute>(cls.constant_pool())
            .unwrap();

        drop(cls);

        let ctx = Context {
            class: _cls,
            code,
            operands: vec![],
            locals: vec![],
            pc: 0,
        };

        let res = vm.run(ctx);

        if let Err(e) = res {
            error!("Execution error: {:?}", e);
            exit(1);
        } else {
            info!("Execution concluded without error")
        }
    }
}
