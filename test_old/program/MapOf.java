// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t MapOf | filecheck %s
import java.util.Map;

class MapOf {

    public static native void print(String s);
    public static native void print(int i);
    public static native void print(boolean b);

    public static void main(String[] args) {
        Map<String, String> map = Map.of(
            "hello", "world",
            "value", "other"
        );

        // CHECK: 2
        print(map.size());

        // CHECK: true
        print(map.containsKey("hello"));

        // CHECK: true
        print(map.containsKey("value"));

        // CHECK: false
        print(map.containsKey("not a key"));
    }
}

