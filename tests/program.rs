use common::{exec_integration, input, join};
use proc::java;

mod common;

#[test]
pub fn hello_world() {
    let input = input().with_std();
    let class = java!(r#"
        public class HelloWorld {
            public static void main(String[] args) {
                System.out.println("Hello, World");
            }
        }
    "#);

    exec_integration(input, class)
        .stdout(join(["Hello, World"]))
        .success();
}

#[test]
pub fn anonymous_classes() {
    let input = input().with_std();
    let class = java!(r#"
        public class AnonymousClasses {
            static abstract class MakeMeAnonymous {
                int x;
                abstract void work();
            }

            public static void main(String[] args) {
                MakeMeAnonymous an = new MakeMeAnonymous() {
                    int x = 10;
                    void work() {
                        System.out.println("Hello from anonymous");
                    }
                };

                an.work();
            }
        }
    "#);

    exec_integration(input, class)
        .stdout(join(["Hello from anonymous"]))
        .success();
}