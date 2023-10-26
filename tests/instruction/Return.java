// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Return | filecheck %s
class Return {
  public static native void print(byte b);
  public static native void print(char c);
  public static native void print(double d);
  public static native void print(float f);
  public static native void print(int i);
  public static native void print(long l);
  public static native void print(short s);
  public static native void print(boolean b);

  static byte byteReturn() {
    return 1;
  }

  static char charReturn() {
    return 'a';
  }

  static double doubleReturn() {
    return 1.0;
  }

  static float floatReturn() {
    return 0.0f;
  }

  static int intReturn() {
    return 2;
  }

  static long longReturn() {
    return 3;
  }

  static short shortReturn() {
    return 4;
  }

  static boolean booleanReturn() {
    return true;
  }

  public static void main(String[] args) {
    byte a = byteReturn();
    // CHECK: 1
    print(a);

    char b = charReturn();
    // CHECK: a
    print(b);

    double c = doubleReturn();
    // CHECK: 1.00
    print(c);
    
    float d = floatReturn();
    // CHECK: 0.00
    print(d);

    int e = intReturn();
    // CHECK: 2
    print(e);

    long f = longReturn();
    // CHECK: 3
    print(f);

    short g = shortReturn();
    // CHECK: 4
    print(g);

    boolean h = booleanReturn();
    // CHECK: true
    print(h);
  }
}