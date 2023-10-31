// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t GetStatic | filecheck %s

class GetStatic {

    public static native void print(int i);
    public static native void print(boolean  b);

    static int x = 5;
    static Object y = new Object();

    public static void main(String[] args) {
        // CHECK: 5
        print(x);

        // CHECK: true
        print(y.equals(y));
    }
}

