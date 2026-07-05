public class concat {
    public static void main(String[] args) {
        String s = "";
        for (int i = 0; i < 50000; i++) {
            s = s + "x";
        }
        System.out.println(s.length());
    }
}
