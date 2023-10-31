// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t ArrayLength | filecheck %s

class ArrayLength {

    public static native void print(String s);

    public static native void print(int i);

    public static native void print(boolean b);

    public static void main(String[] args) {
        byte[] b = new byte[] { 1, 2, 3 };

        // CHECK: 3
        print(b.length);

        String[] s = new String[] { "one", "two", "three", "four", "five" };

        // CHECK: 5
        print(s.length);
    }
}
