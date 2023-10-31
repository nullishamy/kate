// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t GetField | filecheck %s

class GetField {

    public static native void print(int i);
    public static native void print(boolean  b);

    int x = 5;
    Object y = new Object();

    public static void main(String[] args) {
        GetField f = new GetField();

        int _x = f.x;
        // CHECK: 5
        print(_x);

        Object _y = f.y;

        // CHECK: true
        print(f.y.equals(_y));
    }
}

