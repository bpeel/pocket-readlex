// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2024  Neil Roberts
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

package uk.co.busydoingnothing.pocketrl;

import java.io.FileInputStream;
import java.io.InputStream;
import java.io.IOException;

public class Transliterater {
    static {
        System.loadLibrary("compiledb");
    }

    public static native void transliterate(byte[] dictionary,
                                            CharSequence input,
                                            StringBuilder output);

    // Test program
    public static void main(String[] args)
        throws IOException
    {
        if (args.length < 2) {
            System.err.println("Usage: java Transliterater <dictionary> " +
                               "<text>...");
            System.exit(1);
        }

        FileInputStream inputStream = new FileInputStream(args[0]);
        Trie trie = new Trie(inputStream);

        for (int i = 1; i < args.length; i++) {
            StringBuilder builder = new StringBuilder();

            transliterate(trie.getData(), args[i], builder);

            System.out.println(builder);
        }
    }
}
