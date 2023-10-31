// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Maps | filecheck %s
import java.util.HashMap;
import java.util.Map;

class Maps {

    public static native void print(String s);
    public static native void print(int i);
    public static native void print(boolean b);

    public static void main(String[] args) {
        Map<String, String> map = new HashMap<>();
        // CHECK: 0
        print(map.size());
        // CHECK: true
        print(map.isEmpty());

        String oldValue = map.put("hello", "world");
        // Should be no value set previously
        // CHECK: true
        print(oldValue == null);
        // CHECK: 1
        print(map.size());

        // CHECK: true
        print(map.get("hello").equals("world"));
        // CHECK: false
        print(map.isEmpty());
    }
}

