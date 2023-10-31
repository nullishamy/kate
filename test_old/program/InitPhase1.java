// RUN: javac %s -d %t
// RUN: run-kate --test --boot-system --cp %t InitPhase1 | filecheck %s

class InitPhase1 {

    public static native void print(String s);

    public static void main(String[] args) {
        // CHECK: ok
        print("ok");
    }
}
