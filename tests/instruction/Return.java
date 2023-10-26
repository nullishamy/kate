// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t Return | filecheck %s
class Return {
  public static native void print(int l);


  static int intReturn() {
    return 5;
  }

  public static void main(String[] args) {
    int x = intReturn();
    // CHECK: 5
    print(x);
  }
}