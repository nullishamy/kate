// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t PutField | filecheck %s

class PutField {

    public static native void print(int i);
    public static native void print(boolean  b);

    int x = 5;
    Object y = new Object();

    public static void main(String[] args) {
        PutField f = new PutField();

        int _x = f.x;
        // CHECK: 5
        print(_x);

        Object _y = f.y;

        // CHECK: true
        print(f.y.equals(_y));

        f.x = 10;
        // CHECK: 10
        print(f.x);

        f.y = new Object();

        // CHECK: false
        print(f.y.equals(_y));
    }
}

