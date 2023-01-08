pub mod interpreter;
pub mod opcode;
pub mod runtime;

mod native;

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::runtime::{bootstrap::BootstrapClassLoader, classloader::ClassLoader};
    use parse::attributes::CodeAttribute;

    use crate::interpreter::{ExecutionContext, Interpreter};

    #[test]
    fn it_works() {
        let builder = thread::Builder::new();

        let handler = builder
            .stack_size(1024 * 1024 * 80)
            .spawn(move || {
                let source_root = env!("CARGO_MANIFEST_DIR");

                let classloader = BootstrapClassLoader::new(vec![
                    format!("{source_root}/../../std/java.base").into(),
                    format!("{source_root}/../../samples").into(),
                ]);

                let object = classloader.load_class("ExitCode".to_string());

                let mut interpreter = Interpreter::with_classloaders(vec![Box::new(classloader)]);

                assert!(object.is_ok(), "{:?}", object);

                let object = object.unwrap();
                let main = object
                    .lock()
                    .get_method("main".to_string(), "([Ljava/lang/String;)V".to_string());

                assert!(main.is_some(), "main method not found");

                let main = main.unwrap();
                let code = object
                    .lock()
                    .resolve_known_attribute::<CodeAttribute>(&main.attributes);
                assert!(code.is_ok(), "{:#?}", code);

                interpreter
                    .run_code(ExecutionContext::new(object, main))
                    .unwrap();
            })
            .unwrap();

        handler.join().unwrap();

        assert!(false);
    }
}
