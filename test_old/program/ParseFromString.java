// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t ParseFromString | filecheck %s

class ParseFromString {

    public static native void print(String s);
    public static native void print(int i);
    public static native void print(boolean b);

    public static void main(String[] args) {
        String classFileVersion = "61.0";
        int index = classFileVersion.indexOf('.');

        // CHECK-NOT:-
        // CHECK: Index was -1
        if (index == -1) {
            throw new IllegalStateException("Index was -1");
        }

        // CHECK: 2
        print(index);

        int classFileMajorVersion;
        int classFileMinorVersion;

        // CHECK-NOT:-
        // CHECK: Number format exception
        try {
            classFileMajorVersion = Integer.valueOf(classFileVersion.substring(0, index));
            classFileMinorVersion = Integer.valueOf(classFileVersion.substring(index+1, classFileVersion.length()));
        } catch (NumberFormatException e) {
            throw new IllegalStateException("Number format exception", e);
        }

        // CHECK: 61
        print(classFileMajorVersion);

        // CHECK: 0
        print(classFileMinorVersion);
    }
}

