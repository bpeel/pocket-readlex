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

public class BitReader {
    private byte[] data;
    private int pos;
    private byte queue;
    private byte queueLength;
    private int startPos;

    public BitReader(byte[] data)
    {
        this.data = data;
    }

    public void resetPosition(int pos)
    {
        this.pos = pos;
        this.startPos = pos;
        this.queue = 0;
        this.queueLength = 0;
    }

    public int getBytesConsumed()
    {
        return this.pos - startPos;
    }

    private byte readByte()
    {
        return data[pos++];
    }

    public int readBits(int nBits)
    {
        int got = Math.min(nBits, queueLength);
        int result = ((int) queue) & (0xff >> (8 - got));

        queue >>= got;
        queueLength -= got;

        while (nBits - got >= 8) {
            result |= ((int) readByte()) << got;
            got += 8;
        }

        int remainder = nBits - got;

        if (remainder > 0) {
            int nextByte = readByte();
            queueLength = (byte) (8 - remainder);
            result |= ((nextByte & (0xff >> (int) queueLength))) << got;
            queue = (byte) (nextByte >> remainder);
        }

        return result;
    }
}
