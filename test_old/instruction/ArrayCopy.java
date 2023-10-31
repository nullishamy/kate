// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t ArrayCopy | filecheck %s

class ArrayCopy {

    public static native void print(String s);

    public static native void print(int i);
    public static native void print(byte[] b);

    public static native void print(boolean b);

    public static void main(String[] args) {
        byte[] b = new byte[] { 1, 2, 3 };
        byte[] b_2 = new byte[b.length];

        // CHECK: [1, 2, 3]
        print(b);

        // CHECK: [0, 0, 0]
        print(b_2);

        // CHECK: 3
        print(b_2.length);

        // CHECK: 3
        print(b.length);

        System.arraycopy(b, 0, b_2, 0, 3);

        // CHECK: [1, 2, 3]
        print(b);

        // CHECK: [1, 2, 3]
        print(b_2);

        // CHECK: 3
        print(b_2.length);

        // CHECK: 3
        print(b.length);
    }
}

