// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t AThrow | filecheck %s

class AThrow {

    public static native void print(int i);

    private static void athrow() {
        throw new IllegalStateException("die");
    }

    private static void nested() {
        athrow();
    }

    public static void main(String[] args) {
        // CHECK: java/lang/IllegalStateException: die
        nested();
    }
}
