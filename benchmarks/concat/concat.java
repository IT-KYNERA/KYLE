class concat {
    public static void main(String[] args) {
        StringBuilder sb = new StringBuilder(500000);
        for (int i = 0; i < 500000; i++)
            sb.append('x');
        System.out.println(sb.length());
    }
}
