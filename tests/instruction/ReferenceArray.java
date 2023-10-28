// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t ReferenceArray | filecheck %s

class ReferenceArray {

    public static native void print(String s);
    public static native void print(int i);
    public static native void print(boolean b);

    public static void main(String[] args) {
        String[] array = new String[]{
            "one",
            "two",
            "three",
            ""
        };

        // CHECK: 3
        print(array[0].length());
        // CHECK: false
        print(array[0].isEmpty());

        // CHECK: 3
        print(array[1].length());
        // CHECK: false
        print(array[1].isEmpty());

        // CHECK: 5
        print(array[2].length());
        // CHECK: false
        print(array[2].isEmpty());

        // CHECK: 0
        print(array[3].length());
        // CHECK: true
        print(array[3].isEmpty());
    }
}
