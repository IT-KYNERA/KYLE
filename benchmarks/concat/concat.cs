using System;

class Concat {
    static void Main() {
        string s = "";
        for (int i = 0; i < 50000; i++) {
            s = s + "x";
        }
        Console.WriteLine(s.Length);
    }
}
