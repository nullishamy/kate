// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t NPE | filecheck %s

class NPE {

    public static native void print(int i);
    public static native void print(String s);

    static void throwme() {
        throw new NullPointerException();
    }

    public static void main(String[] args) {
        throwme();
    }
}

