// RUN: javac %s -d %t
// RUN: run-kate --test --cp %t NewException | filecheck %s

class NewException {

    public static native void print(String s);
    public static native void print(boolean b);

    public static void main(String[] args) {
        Throwable thNoMessage = new NullPointerException();
        print(thNoMessage.getMessage());

        Throwable thMessage = new NullPointerException("message");
        print(thNoMessage.getMessage());
    }
}


