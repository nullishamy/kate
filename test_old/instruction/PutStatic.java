// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t PutStatic | filecheck %s

class PutStatic {

    public static native void print(int i);
    public static native void print(boolean  b);

    static int x = 5;
    static Object y = new Object();

    public static void main(String[] args) {
        int _x = x;
        // CHECK: 5
        print(_x);

        Object _y = y;

        // CHECK: true
        print(y.equals(_y));

        x = 10;
        // CHECK: 10
        print(x);

        y = new Object();

        // CHECK: false
        print(y.equals(_y));
    }
}

