// Overloads for numerics so that we don't need to boot the system (for autoboxing to Object)
// When doing basic numeric assertions. These overloads use manual string builders to work around
// javac inserting "invokedynamic"s to make the strings.

use std::path::PathBuf;

pub struct Class<'a> {
    pub name: &'a str,
    pub content: String,
}

pub fn using_helpers<'a>(class_name: &'a str, source: &'a str) -> Class<'a> {
    let mut cls = using_class(class_name, source.to_string());
    cls.content = format!("import static kate.Util.*;\n{}", cls.content);

    cls
}

pub fn using_main<'a>(class_name: &'a str, source: &'a str) -> Class<'a> {
    let mut cls = using_class(
        class_name,
        format!(
            r#"
                public static void main(String[] args) {{
                    {}
                }}
            "#,
            source
        ),
    );

    cls.content = format!("import static kate.Util.*;\n{}", cls.content);
    cls
}

pub fn using_relative<'a>(file: impl Into<PathBuf>, class_name: impl Into<PathBuf>) -> PathBuf {
    let file: PathBuf = file.into();
    let mut components = file.components();

    // Drop 'tests' dir from path, runner executes from there
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
