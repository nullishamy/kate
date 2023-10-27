// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Conditional | filecheck %s

class Conditional {

    public static native void print(int i);

    public static native void print(boolean i);

    public static void main(String[] args) {
        int x = 3;
        int y = 2;
        int z = 3;

        if (x == y) {
            // CHECK-NOT: 1
            print(1);
        } else {
            // CHECK: 2
            print(2);
        }

        if (x == z) {
            // CHECK: 3
            print(3);
        } else {
            // CHECK-NOT: 4
            print(4);
        }

        if (x != y) {
            // CHECK: 5
            print(5);
        } else {
            // CHECK-NOT: 6
            print(6);
        }

        if (x > y) {
            // CHECK: 7
            print(7);
        } else {
            // CHECK-NOT: 8
            print(8);
        }

        if (x < y) {
            // CHECK-NOT: 9
            print(9);
        } else {
            // CHECK: 10
            print(10);
        }
    }
}
