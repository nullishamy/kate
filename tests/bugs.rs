use common::{exec_integration, input, join};
use proc::java;

mod common;

#[test]
pub fn derived_class_with_field_caused_layout_bugs() {
    let input = input();
    let class = java!(
        r#"
        public class ClassLayoutBug1 {
            static class Parent {
                Object firstObject;
                Object secondObject;

                public Parent(Object firstObject) {
                    this.firstObject = firstObject;

                    if (this.firstObject == null) {
                        throw new NullPointerException("firstObject in constructor");
                    }

                    if (this.secondObject != null) {
                        throw new IllegalStateException("secondObject should be null at construction");
                    }
                }
            }

            static class Child extends Parent {
                private boolean dummyField;

                public Child(Object firstObject, Object secondObject) {
                    super(firstObject);

                    jdk.internal.misc.Unsafe unsafe = jdk.internal.misc.Unsafe.getUnsafe();
                    long offset = unsafe.objectFieldOffset(Parent.class, "secondObject");
                    unsafe.putReferenceVolatile(this, offset, secondObject);
                }
            }

            public static void main(String[] args) {
                Child child = new Child(new Object(), null);

                if (child.firstObject == null) {
                    throw new NullPointerException("firstObject was null. The inheritance layout logic must have broken.");
                }
            }
        }
    "#
    );

    exec_integration(input, class).success();
}

#[test]
pub fn new_thread_group() {
    let input = input();
    let class = java!(r#"
        public class NewThreadGroup {
            static native void print(String s);

            public static void main(String[] args) {
                var t = Thread.currentThread();
                print(t.getName());

                var g = t.getThreadGroup();
                print(g.getName());

                var ng = new ThreadGroup(g, "main2");
                print(ng.getName());
            }
        }
    "#);

    exec_integration(input, class)
        .stdout(join(["main", "main", "main2"]))
        .success();
}
