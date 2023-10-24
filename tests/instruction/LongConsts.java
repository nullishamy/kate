// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t LongConsts | filecheck %s
class LongConsts {
  public static native void print(long l);

  public static void main(String[] args) {
    long zero = 0;
    long one = 1;

    // CHECK: 0
    print(zero);

    // CHECK: 1
    print(one);
  }
}