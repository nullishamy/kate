// RUN: javac %s -d %t
// RUN: kate --test --cp %t BiPush | filecheck %s
class BiPush {
  public static native void print(int i);

  public static void main(String[] args) {
    byte b = 6;
    // CHECK: 6
    print(b);
    byte b2 = -15;
    // CHECK: -15
    print(b2);
  }
}