// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Shift | filecheck %s

class Shift {

    public static native void print(int i);
    public static native void print(long l);

    public static void main(String[] args) {
        long i = 0xFFFFFF;
        long x = 1;

        // CHECK: 8388607
        print(i >> x);

        // CHECK: 33554430
        print(i << x);
    }
}
