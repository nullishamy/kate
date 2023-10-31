// Overloads for numerics so that we don't need to boot the system (for autoboxing to Object)
// When doing basic numeric assertions. These overloads use manual string builders to work around
// javac inserting "invokedynamic"s to make the strings.
// TODO: Remove these when we support invokedynamic

use std::path::PathBuf;

pub const HELPERS: &str = r#"
    public static native void print(byte b);
    public static native void print(char c);
    public static native void print(double d);
    public static native void print(float f);
    public static native void print(int i);
    public static native void print(long l);
    public static native void print(short s);
    public static native void print(boolean b);
    public static native void print(String s);

    public static void assertEqual(Object lhs, Object rhs) {
        if (!java.util.Objects.equals(lhs, rhs)) {
            throw new RuntimeException("(==) Assertion failed. lhs was '" + lhs + "', rhs was '" + rhs + "'");
        }
    }

    public static void assertEqual(long lhs, long rhs) {
        if (lhs != rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(==) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '" );
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertEqual(byte lhs, byte rhs) {
        if (lhs != rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(==) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '" );
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertEqual(double lhs, double rhs) {
        if (lhs != rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(==) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '" );
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertNotEqual(Object lhs, Object rhs) {
        if (java.util.Objects.equals(lhs, rhs)) {
            throw new RuntimeException("(!=) Assertion failed. lhs was '" + lhs + "', rhs was '" + rhs + "'");
        }
    }

    public static void assertNotEqual(long lhs, long rhs) {
        if (lhs == rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(!=) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '");
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertNotReached() {
        throw new RuntimeException("Unreachable statement reached");
    }
"#;

pub struct Class<'a> {
    pub name: &'a str,
    pub content: String,
}

pub fn using_helpers<'a>(class_name: &'a str, source: &'a str) -> Class<'a> {
    using_class(class_name, format!("{}\n{}", source, HELPERS))
}

pub fn using_main<'a>(class_name: &'a str, source: &'a str) -> Class<'a> {
    using_class(
        class_name,
        format!(
            r#"
                public static void main(String[] args) {{
                    {}
                }}

                {}
            "#,
            source, HELPERS
        ),
    )
}

pub fn using_relative<'a>(file: impl Into<PathBuf>, class_name: impl Into<PathBuf>) -> PathBuf {
    let file: PathBuf = file.into();
    let mut components = file.components();

    // Drop 'tests', runner executes from there
    let _ = components.next();

    components
        .map(|f| f.as_os_str())
        .map(Into::<PathBuf>::into)
        .rev()
        .map(|f| f.file_stem().map(|s| s.into()).unwrap_or(f))
        .rev()
        .chain(std::iter::once(class_name.into()))
        .collect()
}

pub fn direct<'a>(class_name: &'a str, source: &'a str) -> Class<'a> {
    Class {
        name: class_name,
        content: source.to_string(),
    }
}

pub fn using_class(class_name: &str, class_content: String) -> Class<'_> {
    Class {
        name: class_name,
        content: format!(
            r#"
                public class {} {{
                    {}
                }}
            "#,
            class_name, class_content
        ),
    }
}
