
package kate;

public class Util {

    public static native void print(byte b);

    public static native void print(char c);

    public static native void print(double d);

    public static native void print(float f);

    public static native void print(int i);

    public static native void print(long l);

    public static native void print(short s);

    public static native void print(boolean b);

    public static native void print(String s);

    public static void assertEqual(Object lhs, Object rhs) {
        if (!java.util.Objects.equals(lhs, rhs)) {
            throw new RuntimeException("(==) Assertion failed. lhs was '" + lhs + "', rhs was '" + rhs + "'");
        }
    }

    public static void assertEqual(long lhs, long rhs) {
        if (lhs != rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(==) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '");
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertEqual(byte lhs, byte rhs) {
        if (lhs != rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(==) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '");
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertEqual(double lhs, double rhs) {
        if (lhs != rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(==) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '");
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertNotEqual(Object lhs, Object rhs) {
        if (java.util.Objects.equals(lhs, rhs)) {
            throw new RuntimeException("(!=) Assertion failed. lhs was '" + lhs + "', rhs was '" + rhs + "'");
        }
    }

    public static void assertNotEqual(long lhs, long rhs) {
        if (lhs == rhs) {
            StringBuilder sb = new StringBuilder();
            sb.append("(!=) Assertion failed. lhs was '");
            sb.append(lhs);
            sb.append("', rhs was '");
            sb.append(rhs);
            sb.append("'");

            throw new RuntimeException(sb.toString());
        }
    }

    public static void assertNotReached() {
        throw new RuntimeException("Unreachable statement reached");
    }
}
