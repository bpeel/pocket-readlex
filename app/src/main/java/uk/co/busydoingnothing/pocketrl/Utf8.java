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

public class Utf8
{
    // Gets the number of bytes needed for a UTF-8 sequence which
    // begins with the given byte.
    public static int getLength(byte firstByte)
    {
        if (firstByte >= 0)
            return 1;
        if ((firstByte & 0xe0) == 0xc0)
            return 2;
        if ((firstByte & 0xf0) == 0xe0)
            return 3;
        if ((firstByte & 0xf8) == 0xf0)
            return 4;
        if ((firstByte & 0xfc) == 0xf8)
            return 5;

        return 6;
    }

    public static int getCharacter(byte[] data, int offset)
    {
        byte firstByte = data[offset];

        if (firstByte >= 0)
            return firstByte;

        int nExtraBytes;
        int value;

        if ((firstByte & 0xe0) == 0xc0) {
            nExtraBytes = 1;
            value = firstByte & 0x1f;
        } else if ((firstByte & 0xf0) == 0xe0) {
            nExtraBytes = 2;
            value = firstByte & 0x0f;
        } else {
            nExtraBytes = 3;
            value = firstByte & 0x07;
        }

        for (int i = 0; i < nExtraBytes; i++) {
            value = (value << 6) | (data[offset + 1 + i] & 0x3f);
        }

        return value;
    }
}
