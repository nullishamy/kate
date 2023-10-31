// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Sets | filecheck %s
import java.util.HashSet;
import java.util.Set;

class Sets {

    public static native void print(String s);
    public static native void print(int i);
    public static native void print(boolean b);

    public static void main(String[] args) {
        Set<String> set = new HashSet<>();
        // CHECK: 0
        print(set.size());
        // CHECK: true
        print(set.isEmpty());

        // CHECK: true
        // (true if the set did not already contain the item)
        print(set.add("hello"));
        // CHECK: 1
        print(set.size());

        // CHECK: true
        print(set.contains("hello"));
        // CHECK: false
        print(set.isEmpty());
    }
}

