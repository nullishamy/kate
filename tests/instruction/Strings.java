// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Strings | filecheck %s

class Strings {

    public static native void print(String s);
    public static native void print(byte b);

    public static void main(String[] args) {
        String s = "hello";
        // CHECK: hello
        print(s);

        String a = s.substring(0, 3);
        // CHECK: hel
        print(a);
    }
}
