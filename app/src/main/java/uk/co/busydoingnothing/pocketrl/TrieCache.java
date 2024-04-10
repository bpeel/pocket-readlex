// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2016, 2024  Neil Roberts
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

import android.content.Context;
import java.io.InputStream;
import java.io.IOException;
import java.util.Iterator;
import java.util.LinkedList;

public class TrieCache
{
    private static Trie cachedTrie;

    public static synchronized Trie getTrie(Context context)
        throws IOException
    {
        if (cachedTrie == null) {
            InputStream input = context.getAssets().open("dictionary.bin");
            cachedTrie = new Trie(input);
        }

        return cachedTrie;
    }
}
