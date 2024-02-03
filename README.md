# Kate

Kate is a JVM written in Rust, it's a passion project that aims to improve my
knowledge of systems development and JVM internals.

Contributions are welcome (see CONTRIBUTING.md). Check the issues for any
"up for grabs" issues.

The project can now run basic Java code, boot the JDK, and do primitive file IO (including writing to stdout etc).
For the most part, it assumes well-formedness of classfiles, and will not handle invalid classfiles gracefully.

## External resources

- [ASM Tools](https://wiki.openjdk.java.net/display/CodeTools/asmtools)
- [jasm](https://github.com/roscopeco/jasm)
- [assert_cmd](https://docs.rs/crate/assert_cmd/latest)
