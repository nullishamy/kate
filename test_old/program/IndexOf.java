// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t IndexOf | filecheck %s

class IndexOf {

    public static native void print(String s);

    public static native void print(int i);

    public static native void print(boolean b);

    public static native void print(char c);

    public static native void print(byte[] b);

    public static native byte[] getValue(String s);

    static final int HI_BYTE_SHIFT;
    static final int LO_BYTE_SHIFT;

    static {
        HI_BYTE_SHIFT = 8;
        LO_BYTE_SHIFT = 0;
    }

    public static int length(byte[] value) {
        return value.length >> 1;
    }

    static char getChar(byte[] val, int index) {
        assert index >= 0 && index < length(val) : "Trusted caller missed bounds check";
        index <<= 1;
        return (char) (((val[index++] & 0xff) << HI_BYTE_SHIFT)
                | ((val[index] & 0xff) << LO_BYTE_SHIFT));
    }

    static void checkBoundsBeginEnd(int begin, int end, int length) {
        if (begin < 0 || begin > end || end > length) {
            throw new StringIndexOutOfBoundsException(
                    "begin " + begin + ", end " + end + ", length " + length);
        }
    }

    private static int indexOfChar(byte[] value, int ch, int fromIndex, int max) {
        checkBoundsBeginEnd(fromIndex, max, length(value));
        return indexOfCharUnsafe(value, ch, fromIndex, max);
    }

    private static int indexOfCharUnsafe(byte[] value, int ch, int fromIndex, int max) {
        for (int i = fromIndex; i < max; i++) {
            char checking = getChar(value, i);
            if (checking == ch) {
                return i;
            }
        }
        return -1;
    }

    public static void main(String[] args) {
        String str = "61.0";
        byte[] bytes = getValue(str);
        print(bytes);

        int index = indexOfChar(bytes, '.', 0, length(bytes));
        // CHECK: 2
        print(index);

        index = str.indexOf('.');
        // CHECK: 2
        print(index);
    }
}
