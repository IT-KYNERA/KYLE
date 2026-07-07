using System;
using System.Text;

class Concat {
    static void Main() {
        var sb = new StringBuilder(500000);
        for (int i = 0; i < 500000; i++)
            sb.Append('x');
        Console.WriteLine(sb.Length);
    }
}
