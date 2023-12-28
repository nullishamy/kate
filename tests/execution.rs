mod util;

// Test each instruction individually. Baseline regression tests
#[cfg(test)]
mod instruction {
    use crate::util::{
        builder::{using_helpers, using_main},
        compare, execute, expected, inline, state, TestResult,
    };

    #[test]
    fn aaload() -> TestResult {
        let state = state().init();

        let source = using_main(
            "AALoad",
            r#"
            String[] referenceArray = new String[]{
                "hello",
                "world"
            };

            assertEqual(referenceArray.length, 2);
            assertEqual(referenceArray[0], "hello");
            assertEqual(referenceArray[1], "world");
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn aastore() -> TestResult {
        let state = state().init();

        let source = using_main(
            "AAStore",
            r#"
            String[] referenceArray = new String[2];
            referenceArray[0] = "hello";
            referenceArray[1] = "world";

            assertEqual(referenceArray.length, 2);
            assertEqual(referenceArray[0], "hello");
            assertEqual(referenceArray[1], "world");
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn aconst_null() -> TestResult {
        let state = state().init();

        let source = using_main(
            "AConstNull",
            r#"
            String nullptr = null;

            assertEqual(nullptr, null);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn aload() -> TestResult {
        let state = state().init();

        let source = using_main(
            "ALoad",
            r#"
            // Including 5 to get the dynamic variant
            String s1 = "hello";
            String s2 = "world";
            String s3 = "foo";
            String s4 = "bar";
            String s5 = "baz";

            assertEqual(s1, "hello");
            assertEqual(s2, "world");
            assertEqual(s3, "foo");
            assertEqual(s4, "bar");
            assertEqual(s5, "baz");
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn anewarray() -> TestResult {
        let state = state().init();

        let source = using_main(
            "ANewArray",
            r#"
            String[] arr = new String[10];

            assertEqual(arr[0], null);
            assertEqual(arr[9], null);
            assertEqual(arr.length, 10);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[ignore = "not implemented"]
    #[test]
    fn two_d_arrays() -> TestResult {
        let state = state().init();

        let source = using_main(
            "TwoDArrays",
            r#"
            int[][] arr = new int[10][10];

            assertEqual(arr[0][0], 0);
            assertEqual(arr[9][0], 0);
            assertEqual(arr[9][9], 0);
            assertEqual(arr.length, 10);
            assertEqual(arr[0].length, 10);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn areturn() -> TestResult {
        let state = state().init();

        let source = using_helpers(
            "AReturn",
            r#"
            public static String areturn() {
                return "hello";
            }

            public static void main(String[] args) {
                String s = areturn();

                assertEqual(s, "hello");
            }
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn arraylength() -> TestResult {
        let state = state().init();

        let source = using_main(
            "ArrayLength",
            r#"
            String[] arr = new String[10];

            assertEqual(arr.length, 10);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn astore() -> TestResult {
        let state = state().init();

        let source = using_main(
            "AStore",
            r#"
            // Including 5 to get the dynamic variant
            String s1 = "hello";
            String s2 = "world";
            String s3 = "foo";
            String s4 = "bar";
            String s5 = "baz";

            assertEqual(s1, "hello");
            assertEqual(s2, "world");
            assertEqual(s3, "foo");
            assertEqual(s4, "bar");
            assertEqual(s5, "baz");
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn athrow() -> TestResult {
        let state = state().init();

        let source = using_helpers(
            "AThrow",
            r#"
            private static void throwException() {
                throw new IllegalStateException("die");
            }

            private static void nestedThrow() {
                throwException();
            }

            private static String thrownWithinMethod() {
                try {
                    throw new IllegalStateException("die");
                } catch (IllegalStateException e) {
                    return "Caught";
                }
            }

            private static String thrownOverMethod() {
                try {
                    throwException();
                    return "Not thrown";
                } catch (IllegalStateException e) {
                    return "Caught";
                }
            }

            private static String thrownOverManyMethods() {
                try {
                    nestedThrow();
                    return "Not thrown";
                } catch (IllegalStateException e) {
                    return "Caught";
                }
            }

            public static void main(String[] args) {
                String status = thrownWithinMethod();
                assertEqual(status, "Caught");

                // This is its own special case because the cli has to handle the catching 
                // of exceptions thrown / caught in main
                try {
                    status = "About to throw";
                    throw new IllegalStateException();
                } catch (IllegalStateException e) {
                    status = "Caught in main";
                }

                assertEqual(status, "Caught in main");

                // Just checking that it properly ignores exceptions that aren't caught
                throw new RuntimeException("thrown from main");
            }
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected()
            .has_error()
            .with_output("Uncaught exception in main: java/lang/RuntimeException: thrown from main")
            .with_output("  at AThrow.main");

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn baload() -> TestResult {
        let state = state().init();

        let source = using_main(
            "BALoad",
            r#"
            byte[] arr = new byte[]{1, 2, 3};

            assertEqual(arr[0], 1);
            assertEqual(arr[1], 2);
            assertEqual(arr[2], 3);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn bastore() -> TestResult {
        let state = state().init();

        let source = using_main(
            "BAStore",
            r#"
            byte[] arr = new byte[3];
            arr[0] = 1;
            arr[1] = 2;
            arr[2] = 3;

            assertEqual(arr[0], 1);
            assertEqual(arr[1], 2);
            assertEqual(arr[2], 3);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn bipush() -> TestResult {
        let state = state().init();

        let source = using_main(
            "BiPush",
            r#"
            byte b = 1;
            assertEqual(b, 1);

            byte b2 = -10;
            assertEqual(b2, -10);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn caload() -> TestResult {
        let state = state().init();

        let source = using_main(
            "CALoad",
            r#"
            char[] arr = new char[]{'a', 'b', 'c'};

            assertEqual(arr[0], 'a');
            assertEqual(arr[1], 'b');
            assertEqual(arr[2], 'c');
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn castore() -> TestResult {
        let state = state().init();

        let source = using_main(
            "CAStore",
            r#"
            char[] arr = new char[3];

            arr[0] = 'a';
            arr[1] = 'b';
            arr[2] = 'c';

            assertEqual(arr[0], 'a');
            assertEqual(arr[1], 'b');
            assertEqual(arr[2], 'c');
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn checkcast() -> TestResult {
        let state = state().init();

        let source = using_main(
            "CheckCast",
            r#"
                Object o = "string";
                String s = (String) o;
                assertEqual(s, "string");

                // We can re-erase it
                Object o2 = (Object) s;
                String s2 = (String) o2;
                assertEqual(s2, "string");

                // It doesn't mistake the component for the incoming type
                try {
                    String[] arr = (String[]) o;
                    assertNotReached();
                }
                catch (ClassCastException cce) {
                    print("Caught String[] cce");
                }

                // It doesn't get confused with other object types
                try {
                    Class cls = (Class) o;
                    assertNotReached();
                }
                catch (ClassCastException cce) {
                    print("Caught Class cce");
                }
                
                // It doesn't get confused with primitive arrays
                try {
                    byte[] arr = (byte[]) o;
                    assertNotReached();
                }
                catch (ClassCastException cce) {
                    print("Caught byte[] cce");
                }

                // It doesn't get confused with primitives
                try {
                    byte arr = (byte) o;
                    assertNotReached();
                }
                catch (ClassCastException cce) {
                    print("Caught byte cce");
                }

                int[] intarray = (int[]) o;
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected()
            .has_error()
            .with_output("Caught String[] cce")
            .with_output("Caught Class cce")
            .with_output("Caught byte[] cce")
            .with_output("Caught byte cce")
            .with_output("Uncaught exception in main: java/lang/ClassCastException: invalid cast from java/lang/String to [I")
            .with_output("  at CheckCast.main");

        compare(got, expected);

        Ok(())
    }

    #[test]
    #[ignore = "broken, we cannot apppend doubles to string builders, which our assertion relies on"]
    fn d2f() -> TestResult {
        let state = state().init();

        let source = using_main(
            "D2F",
            r#"
                double smallDouble = 1.2;
                float smallFloat = (float) smallDouble;
                assertEqual(smallFloat, 1.2);

                double largeDouble = 1.234567891011;
                float largeFloat = (float) smallDouble;
                assertEqual(largeFloat, 1.2);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn instanceof() -> TestResult {
        let state = state().init();

        let source = using_main(
            "InstanceOf",
            r#"
                Object o = "string";

                // It doesn't mistake the component for the incoming type
                if (o instanceof String[]) {
                    assertNotReached();
                }

                // It doesn't get confused with other object types
                if (o instanceof Class) {
                    assertNotReached();
                }

                // It doesn't get confused with primitive arrays
                if (o instanceof byte[]) {
                    assertNotReached();
                }

                if (!(o instanceof String)) {
                    assertNotReached();
                }
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    // TODO:
    #[ignore = "needs `wide` support which we do not have"]
    #[test]
    fn dadd() -> TestResult {
        let state = state().init();

        let source = using_main(
            "DAdd",
            r#"
            double nan = Double.NaN;
            double p_inf = Double.POSITIVE_INFINITY;
            double n_inf = Double.NEGATIVE_INFINITY;
            double p_zero = 0.0;
            double n_zero = -0.0;
            double p_123 = 123.456;
            double n_123 = -123.456;

            assertEqual(Double.isNaN(nan + 1), true);
            assertEqual(Double.isNaN(1 + nan), true);

            assertEqual(Double.isNaN(p_inf + n_inf), true);
            assertEqual(Double.isNaN(n_inf + p_inf), true);

            // TODO: Infinity checks
            // CHECK: inf
            // CHECK: -inf
            // print(p_inf + p_inf);
            // print(n_inf + n_inf);

            // CHECK: inf
            // CHECK: -inf
            // print(p_inf + 1);
            // print(n_inf + 1);

            assertEqual(p_zero + n_zero, 0);
            assertEqual(n_zero + p_zero, 0);

            assertEqual(p_zero + p_zero, 0);
            assertEqual(n_zero + n_zero, -0);

            assertEqual(p_zero + p_123, 123.456);
            assertEqual(n_zero + p_123, 123.456);

            assertEqual(n_123 + p_123, 123.456);
            assertEqual(p_123 + n_123, 123.456);

            // TODO: This
            // var x = -6.6057786;
            // var y = 1549700.4;
            // var z = -2.1339336E8;

            // CHECK: 1549693.7942214
            // print(x + y);
            // CHECK: -213393366.605779
            // print(x + z);
            // CHECK: -211843659.6
            // print(y + z);
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn getstatic() -> TestResult {
        let state = state().init();

        let source = using_helpers(
            "GetStatic",
            r#"
                static int x = 32;

                public static void main(String[] args) {
                    assertEqual(x, 32);
                }
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn putstatic() -> TestResult {
        let state = state().init();

        let source = using_helpers(
            "PutStatic",
            r#"
                static int x;

                public static void main(String[] args) {
                    // Should default to 0
                    assertEqual(x, 0);

                    x = 32;

                    assertEqual(x, 32);
                }
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn getfield() -> TestResult {
        let state = state().init();

        let source = using_helpers(
            "GetField",
            r#"
                Object y;
                int x = 32;

                public static void main(String[] args) {
                    GetField f = new GetField();
                    Object o = new Object();

                    assertEqual(f.x, 32);
                    assertEqual(f.y, null);

                    f.y = o;

                    assertEqual(f.y, o);
                }
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }

    #[test]
    fn putfield() -> TestResult {
        let state = state().init();

        let source = using_helpers(
            "PutField",
            r#"
                int x;
                Object y;

                public static void main(String[] args) {
                    PutField f = new PutField();
                    Object o = new Object();

                    assertEqual(f.x, 0);
                    assertEqual(f.y, null);

                    f.x = 32;
                    f.y = o;

                    assertEqual(f.x, 32);
                    assertEqual(f.y, o);
                }
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected().has_success();

        compare(got, expected);

        Ok(())
    }
}

// Test instructions to make sure they throw in exceptional cases
#[cfg(test)]
mod exceptions {
    use crate::util::{builder::using_main, compare, execute, expected, inline, state, TestResult};

    #[test]
    fn caload_oob() -> TestResult {
        let state = state().init();

        let source = using_main(
            "CALoadOOB",
            r#"
            char[] arr = new char[1];

            arr[0] = 'a';
            assertEqual(arr[0], 'a');

            char throwHere = arr[1];

            assertNotReached();
            "#,
        );

        let got = execute(state, inline(source)?)?;
        let expected = expected()
            .has_error()
            .with_output(
                "Uncaught exception in main: java/lang/ArrayIndexOutOfBoundsException: OOB @ 1",
            )
            .with_output("  at CALoadOOB.main");

        compare(got, expected);

        Ok(())
    }
}
