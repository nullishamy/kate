public class Signatures {
	public static void main(String[] args) {
		oneArrayParam(null);
		twoArrayParams(null, null);

		oneTwoDArrayParam(null);
		oneThreeDArrayParam(null);

		oneRefParam(null);
		twoRefParams(null, null);
	}
	static void oneArrayParam(String[] arr) { }
	static void twoArrayParams(String[] arr, String[] arrTwo) { }

	static void oneTwoDArrayParam(String[][] arr) { }
	static void oneThreeDArrayParam(String[][][] arr) { }

	static void oneRefParam(String arr) { }
	static void twoRefParams(String arr, String arrTwo) { }
}