// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t NewThread | filecheck %s

class NewThread {

    public static native void print(String s);

    public static void main(String[] args) {
        var t = Thread.currentThread();
        print(t.getName());
    }
}

