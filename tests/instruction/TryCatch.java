// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t TryCatch | filecheck %s

class TryCatch {
    public static native void print(String s);

    private static void athrow() {
        throw new IllegalStateException("die");
    }

    private static void nested() {
        athrow();
    }

    private static void caughtInHere() {
        try {
           // CHECK-NOT: Uncaught exception in main: java/lang/IllegalStateException: die
           nested(); 
        } catch (IllegalStateException e) {
            // CHECK: caught
            print("caught");
        }
    }

    public static void main(String[] args) {
        caughtInHere();
    }
}
