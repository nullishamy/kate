mod util;

use crate::util::{
    builder::{direct, using_main},
    compare, execute, expected, inline, state, TestResult,
};

#[test]
pub fn hello_world() -> TestResult {
    let state = state().init().init_std();

    let source = using_main(
        "HelloWorld",
        r#"
        System.out.print("Hello, World!\n");
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success().with_output("Hello, World!");

    compare(got, expected);

    Ok(())
}

#[test]
pub fn exit_success() -> TestResult {
    let state = state().init();

    let source = using_main(
        "ExitSuccess",
        r#"
        System.exit(0);
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}

#[test]
pub fn exit_error() -> TestResult {
    let state = state().init();

    let source = using_main(
        "ExitError",
        r#"
        System.exit(1);
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().with_exit(1);

    compare(got, expected);

    Ok(())
}

#[test]
#[ignore]
pub fn array_copy() -> TestResult {
    let state = state().init();

    let source = direct(
        "ArrayCopy",
        r#"
        import java.util.Arrays;

        class ArrayCopy {
            static native void print(String i);

            public static void main(String[] args) {
                // CHECK: Hello World!
                print("Hello".concat(" World!"));

                var bArr1 = new byte[]{1, 2, 3};
                var bArr2 = new byte[3];

                System.arraycopy(bArr1, 0, bArr2, 0, bArr1.length);
                // CHECK: [1, 2, 3]
                print(Arrays.toString(bArr2));

                var cArr1 = new char[]{'a', 'b', 'c'};
                var cArr2 = new char[]{'_', '_', '_', '_', '_',};

                System.arraycopy(cArr1, 0, cArr2, 1, cArr1.length);
                // CHECK: [_, a, b, c, _]
                print(Arrays.toString(cArr2));

                var dArr = new double[]{0, 1, 2, 3, 4};

                System.arraycopy(dArr, 0, dArr, 1, dArr.length - 1);
                // CHECK: [0.0, 0.0, 1.0, 2.0, 3.0]
                print(Arrays.toString(dArr));

                var fArr = new float[]{0, 1, 2, 3, 4};

                System.arraycopy(fArr, 1, fArr, 0, fArr.length - 1);
                // CHECK: [1.0, 2.0, 3.0, 4.0, 4.0]
                print(Arrays.toString(fArr));

                var iArr = new int[]{0, 1, 2, 3, 4};

                System.arraycopy(iArr, 0, iArr, 1, iArr.length - 1);
                // CHECK: [0, 0, 1, 2, 3]
                print(Arrays.toString(iArr));

                var lArr = new long[]{0, 1, 2, 3, 4};

                System.arraycopy(lArr, 1, lArr, 0, lArr.length - 1);
                // CHECK: [1, 2, 3, 4, 4]
                print(Arrays.toString(lArr));

                var sArr1 = new short[]{9, 8, 7};
                var sArr2 = new short[5];

                System.arraycopy(sArr1, 0, sArr2, 2, sArr1.length);
                // CHECK: [0, 0, 9, 8, 7]
                print(Arrays.toString(sArr2));

                var oArr = new String[]{"Foo", "Bar", null, null};

                System.arraycopy(oArr, 0, oArr, 1, 2);
                // CHECK: [Foo, Foo, Bar, null]
                print(Arrays.toString(oArr));

                var objects = new Object[]{"Foo", "Bar", new Object()};
                var strings = new String[4];

                System.arraycopy(objects, 0, strings, 0, 2);
                // CHECK: [Foo, Bar, null, null]
                print(Arrays.toString(strings));

                var zArr1 = new boolean[]{true, false, true, false};
                var zArr2 = new boolean[4];

                System.arraycopy(zArr1, 0, zArr2, 0, zArr1.length);
                // CHECK: [true, false, true, false]
                print(Arrays.toString(zArr2));
            }
        }
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success().with_output("Hello, World!");

    compare(got, expected);

    Ok(())
}
