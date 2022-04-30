public class Args {
	public static void main(String[] args) {
		takesAnInt(1);
		takesNone();
		proxiesAnInt(1);
		returnsNull();
	}

	public static Object returnsNull() { return null; }
	public static void takesAnInt(int i) { }
	public static void takesNone() { }

	public static void proxiesAnInt(int i) { takesAnInt(i); }
}