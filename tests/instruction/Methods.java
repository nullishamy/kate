// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Methods | filecheck %s

class Methods {

    public static native void print(int i);
    public static native void print(String s);

    public static void main(String[] args) {
        // Testing instance methods
        String s = "hello";
        int l = s.length();
        // CHECK: hello
        print(s);

        // CHECK: 5
        print(l);
    }
}
