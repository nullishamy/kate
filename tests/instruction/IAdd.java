// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t IAdd | filecheck %s

class IAdd {

    public static native void print(int i);

    public static void main(String[] args) {
        int x = 3;
        int y = 2;
        int z = 1;

        // CHECK: 5
        print(x + y);

        // CHECK: 4
        print(x + 1);

        // CHECK: -2147483648
        print(Integer.MAX_VALUE + z);
    }
}
