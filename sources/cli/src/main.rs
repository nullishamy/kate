use std::process::exit;

use args::Cli;
use clap::Parser;
use interpreter_two::{
    native::NativeFunction,
    object::{classloader::ClassLoader, string::Interner, RuntimeValue},
    static_method, Context, VM,
};
use parse::attributes::CodeAttribute;
use support::encoding::{decode_string, CompactEncoding};
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
    let mut class_loader = ClassLoader::new();

    class_loader
        .add_path(format!("{source_root}/../../std/java.base").into())
        .add_path(format!("{source_root}/../../samples").into());

    for cp in args.classpath {
        class_loader.add_path(cp.into());
    }

    let (_, jls) = class_loader.bootstrap().unwrap();

    let mut vm = VM {
        class_loader,
        interner: Interner::new(jls),
    };

    vm.class_loader.bootstrap().unwrap();

    for class in args.classes {
        let _cls = vm.class_loader.load_class(class).unwrap();
        let mut cls = _cls.write();
        if args.test {
            macro_rules! printer {
                ($desc: expr, $printer: expr) => {
                    static_method!(name: "print", descriptor: $desc => |_, args, _| {
                        let printer = $printer;
                        printer(args[0].clone());
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
                    println!(
                        "{}",
                        char::from_u32(char_value as u32)
                            .expect(&format!("{} was not a char", char_value))
                    )
                }),
                printer!("(J)V"),
                printer!("(D)V"),
                printer!("(F)V"),
                printer!("(S)V"),
                printer!("(B)V"),
                printer!("(Ljava/lang/String;)V", |a: RuntimeValue| {
                    let string = a.as_object().expect("was not an object (string)");
                    let string = string.read();
                    let bytes =
                        string.get_instance_field(("value".to_string(), "[B".to_string())).expect("could not locate value field");
                    let bytes = bytes.as_array().expect("bytes was not an array (byte[])");

                    let bytes = bytes
                        .values
                        .iter()
                        .map(|v| v.as_integral().expect("value was not an int (char)"))
                        .map(|v| v.value as u8)
                        .collect::<Vec<_>>();

                    let str = decode_string((CompactEncoding::Utf16, bytes)).expect("could not decode string");
                    println!("{}", str);
                }),
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
