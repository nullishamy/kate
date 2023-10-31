// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t InheritedFields | filecheck %s

class Parent {
    int x;
}

class Child extends Parent {
    int y;
    int z;
}

class InheritedFields {

    public static native void print(int i);
    public static native void print(boolean  b);

    public static void main(String[] args) {
        Child c = new Child();
        // CHECK: 0
        print(c.z);

        // CHECK: 0
        print(c.y);

        c.z = 1234;
        c.y = 4321;

        // CHECK: 1234
        print(c.z);

        // CHECK: 4321
        print(c.y);

        // CHECK: 0
        print(c.x);

        c.x = 6789;

        // CHECK: 6789
        print(c.x);

        // CHECK: 1234
        print(c.z);

        // CHECK: 4321
        print(c.y);
    }
}


