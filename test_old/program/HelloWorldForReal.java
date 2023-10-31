// RUN: javac %s -d %t
// RUN: run-kate --test --boot-system --cp %t HelloWorldForReal | filecheck %s

class HelloWorldForReal {

    public static void main(String[] args) {
        // CHECK: Hello, World!
        System.out.print("Hello, World\n");
    }
}