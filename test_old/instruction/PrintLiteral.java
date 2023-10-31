// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t PrintLiteral | filecheck %s

class PrintLiteral {

    public static native void print(String s);
    public static native void print(byte b);

    public static void main(String[] args) {
        String s = "hello";
        // CHECK: hello
        print(s);
    }
}
