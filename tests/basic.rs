mod common;
use proc::java;
use common::{make_vm, load_test, attach_utils, execute_test, iassert_eq, dassert_eq, sassert_eq, assert_null};

#[test]
fn reference_arrays() {
    let compiled = java!(r#"
        public class ReferenceArrays {
            static native void capture(int i);
            static native void capture(String s);

            static void runTest() {
                // Creating arrays with initialiser
                String[] referenceArray = new String[]{
                    "hello",
                    "world"
                };

                // Getting length
                capture(referenceArray.length);

                // Getting elements
                capture(referenceArray[0]);
                capture(referenceArray[1]);

                // Mutating array
                referenceArray[0] = "foo";
                referenceArray[1] = "bar";

                capture(referenceArray[0]);
                capture(referenceArray[1]);

                // Mutating array with empty strings
                referenceArray[0] = "";
                referenceArray[1] = "";

                capture(referenceArray[0]);
                capture(referenceArray[1]);

                // Mutating array with nulls
                referenceArray[0] = null;
                referenceArray[1] = null;

                capture(referenceArray[0]);
                capture(referenceArray[1]);
            }
        }"#
    );

    let mut vm = make_vm();
    let cls = load_test(&mut vm, compiled);
    let capture_id = attach_utils(cls.clone());
    let mut captures = execute_test(&mut vm, cls, capture_id);

    iassert_eq(2, captures.next());

    sassert_eq("hello", captures.next());
    sassert_eq("world", captures.next());

    sassert_eq("foo", captures.next());
    sassert_eq("bar", captures.next());

    sassert_eq("", captures.next());
    sassert_eq("", captures.next());

    assert_null(captures.next());
    assert_null(captures.next());
}

#[test]
fn local_variables() {
    let compiled = java!(r#"
        public class LocalVariables {
            static native void capture(int i);
            static native void capture(String s);

            static void runTest() {
                // Including 5 to get the dynamic variant
                String s1 = "hello";
                String s2 = "world";
                String s3 = "foo";
                String s4 = "bar";
                String s5 = "baz";

                capture(s1);
                capture(s2);
                capture(s3);
                capture(s4);
                capture(s5);
            }
        }"#
    );

    let mut vm = make_vm();
    let cls = load_test(&mut vm, compiled);
    let capture_id = attach_utils(cls.clone());
    let mut captures = execute_test(&mut vm, cls, capture_id);

    sassert_eq("hello", captures.next());
    sassert_eq("world", captures.next());
    sassert_eq("foo", captures.next());
    sassert_eq("bar", captures.next());
    sassert_eq("baz", captures.next());
}

#[test]
fn static_functions() {
    let compiled = java!(r#"
        public class StaticFunctions {
            static native void capture(int i);
            static native void capture(String s);

            static String returnString() {
                return "hello world";
            }

            static int returnInt() {
                return 1;
            }

            static void runTest() {
                capture(returnString());
                capture(returnInt());
            }
        }"#
    );

    let mut vm = make_vm();
    let cls = load_test(&mut vm, compiled);
    let capture_id = attach_utils(cls.clone());
    let mut captures = execute_test(&mut vm, cls, capture_id);

    sassert_eq("hello world", captures.next());
    iassert_eq(1, captures.next());
}

#[test]
fn multidimensional_arrays() {
    let compiled = java!(r#"
        public class MultidimensionalArrays {
            static native void capture(int i);
            static native void capture(String s);

            static void runTest() {
                int[][] arr = new int[3][3];

                // Make sure all edges are 0 init
                capture(arr[0][0]);
                capture(arr[0][2]);
                capture(arr[2][0]);
                capture(arr[2][2]);

                // Make sure we can write into it
                arr[1][1] = 30; 
                capture(arr[1][1]);

                // But that it doesn't corrupt anything
                capture(arr[0][0]);
                capture(arr[0][2]);
                capture(arr[2][0]);
                capture(arr[2][2]);

                // Make sure the lengths match up
                capture(arr.length);
                capture(arr[0].length);
            }
        }"#
    );

    let mut vm = make_vm();
    let cls = load_test(&mut vm, compiled);
    let capture_id = attach_utils(cls.clone());
    let mut captures = execute_test(&mut vm, cls, capture_id);

    iassert_eq(0, captures.next());
    iassert_eq(0, captures.next());
    iassert_eq(0, captures.next());
    iassert_eq(0, captures.next());

    iassert_eq(30, captures.next());

    iassert_eq(0, captures.next());
    iassert_eq(0, captures.next());
    iassert_eq(0, captures.next());
    iassert_eq(0, captures.next());

    iassert_eq(3, captures.next());
    iassert_eq(3, captures.next());
}