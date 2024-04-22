// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2012, 2024  Neil Roberts
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

import java.io.InputStream;
import java.io.IOException;

public class Trie
{
    private static void readAll(InputStream stream,
                                byte[] data,
                                int offset,
                                int length)
        throws IOException
    {
        while (length > 0) {
            int got = stream.read(data, offset, length);

            if (got == -1) {
                throw new IOException("Unexpected end of file");
            } else {
                offset += got;
                length -= got;
            }
        }
    }

    private static final int extractInt(byte[] data,
                                        int offset)
    {
        return (((data[offset + 0] & 0xff) << 0) |
                ((data[offset + 1] & 0xff) << 8) |
                ((data[offset + 2] & 0xff) << 16) |
                ((data[offset + 3] & 0xff) << 24));
    }

    public static byte[] load(InputStream dataStream)
        throws IOException
    {
        byte lengthBytes[] = new byte[4];

        // Read 4 bytes to get the length of the file
        readAll(dataStream, lengthBytes, 0, lengthBytes.length);
        int length = extractInt(lengthBytes, 0);

        // Create a byte array big enough to hold that and the rest of
        // the file
        byte[] data = new byte[length + 4];

        // Copy the lengh bytes in
        System.arraycopy(lengthBytes, 0, data, 0, 4);

        // Read the rest of the data
        readAll(dataStream, data, 4, length);

        return data;
    }
}
