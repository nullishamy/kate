// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t PrimitiveArray | filecheck %s

class PrimitiveArray {

    public static native void print(long l);

    public static void main(String[] args) {
        long[] array = new long[3];
        array[0] = 1;
        array[1] = 2;
        array[2] = 3;

        // CHECK: 1
        print(array[0]);

        // CHECK: 2
        print(array[1]);

        // CHECK: 3
        print(array[2]);
    }
}
