// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t ByteType | filecheck %s

class ByteType {

    public static native void print(String s);
    public static native void print(boolean b);

    public static void main(String[] args) {
        Class<Byte> byteType = Byte.TYPE;
        // CHECK: byte
        print(byteType.getName());

        // CHECK: true
        print(byteType.isPrimitive());
    }
}

