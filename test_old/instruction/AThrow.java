// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t AThrow | filecheck %s

class AThrow {

    private static void athrow() {
        throw new IllegalStateException("die");
    }

    private static void nested() {
        athrow();
    }

    public static void main(String[] args) {
        // CHECK: Uncaught exception in main: java/lang/IllegalStateException: die
        // CHECK: at AThrow.athrow
        // CHECK: at AThrow.nested
        // CHECK: at AThrow.main
        nested();
    }
}
