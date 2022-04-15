public class Signatures {
	public static void main(String[] args) {
		one_array_param(null);
		two_array_params(null, null);

		one_two_darray_param(null);
		one_three_darray_param(null);

		one_ref_param(null);
		two_ref_params(null, null);
	}
	static void one_array_param(String[] arr) { }
	static void two_array_params(String[] arr, String[] arrTwo) { }

	static void one_two_darray_param(String[][] arr) { }
	static void one_three_darray_param(String[][][] arr) { }

	static void one_ref_param(String arr) { }
	static void two_ref_params(String arr, String arrTwo) { }
}