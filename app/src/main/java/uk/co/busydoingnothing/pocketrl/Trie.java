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

import java.io.FileInputStream;
import java.io.InputStream;
import java.io.IOException;

class TrieStack
{
    private int[] data;
    private int size;

    public TrieStack()
    {
        this.data = new int[64];
        this.size = 0;
    }

    public int getTopPos()
    {
        return data[size - 2];
    }

    public int getTopStringLength()
    {
        return data[size - 1];
    }

    public void pop()
    {
        size -= 2;
    }

    public boolean isEmpty()
    {
        return size <= 0;
    }

    public void push(int pos,
                     int stringLength)
    {
        // If there isn’t enough space in the array then we’ll double
        // its size. The size of the array is initially chosen to be
        // quite large so this should probably never happen.
        if (size + 2 >= data.length) {
            int[] newData = new int[data.length * 2];
            System.arraycopy(data, 0, newData, 0, data.length);
            data = newData;
        }

        data[size++] = pos;
        data[size++] = stringLength;
    }
}

public class Trie
{
    private byte data[];

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

    public Trie(InputStream dataStream)
        throws IOException
    {
        byte lengthBytes[] = new byte[4];

        // Read 4 bytes to get the length of the file
        readAll(dataStream, lengthBytes, 0, lengthBytes.length);
        int length = extractInt(lengthBytes, 0);

        // Create a byte array big enough to hold that and the rest of
        // the file
        data = new byte[length + 4];

        // Copy the lengh bytes in
        System.arraycopy(lengthBytes, 0, data, 0, 4);

        // Read the rest of the data
        readAll(dataStream, data, 4, length);
    }

    private int findSiblingForCharacter(int ch, int pos) {
        while (true) {
            int siblingOffset = 0;

            for (int i = 0; ; i++) {
                siblingOffset |= (data[pos] & 0x7f) << (i * 7);

                if ((data[pos++] & 0x80) == 0) {
                    break;
                }
            }

            int nodeCh = Utf8.getCharacter(data, pos);

            if (nodeCh == ch) {
                // We find the character, return the position of the first child
                return pos + Utf8.getLength(data[pos]);
            } else {
                // Character doesn’t match, need to try the next
                // sibling. If there isn’t a next sibling then this
                // prefix isn’t in the trie.
                if (siblingOffset == 0)
                    return -1;

                pos += siblingOffset;
            }
        }
    }

    // Walks through the trie using the path for the given prefix. If
    // the path is found then it returns the offset of the first child
    // after the prefix. Otherwise it returns -1.
    private int findPrefix(CharSequence prefix)
    {
        int pos = 4;
        int length = prefix.length();

        for (int stringPos = 0;
             stringPos < length;
             stringPos = Character.offsetByCodePoints(prefix, stringPos, 1)) {
            int ch = Character.codePointAt(prefix, stringPos);

            pos = findSiblingForCharacter(ch, pos);

            if (pos == -1)
                return -1;
        }

        return pos;
    }

    private int countSiblings(int pos)
    {
        int nSiblings = 1;

        while (true) {
            int siblingOffset = 0;

            for (int i = 0; ; i++) {
                siblingOffset |= (data[pos] & 0x7f) << (i * 7);

                if ((data[pos++] & 0x80) == 0) {
                    break;
                }
            }

            if (siblingOffset == 0)
                break;

            pos += siblingOffset;
            nSiblings++;
        }

        return nSiblings;
    }

    private int skipSiblings(int pos, int skips)
    {
        for (int skipNum = 0; skipNum < skips; skipNum++) {
            int siblingOffset = 0;

            for (int i = 0; ; i++) {
                siblingOffset |= (data[pos] & 0x7f) << (i * 7);

                if ((data[pos++] & 0x80) == 0) {
                    break;
                }
            }

            if (siblingOffset == 0)
                break;

            pos += siblingOffset;
        }

        return pos;
    }

    private void walkPath(BitReader reader,
                          StringBuilder stringBuf)
    {
        int pos = 4;

        while (true) {
            int nSiblings = countSiblings(pos);
            int nBits = 32 - Integer.numberOfLeadingZeros(nSiblings - 1);
            int skips = reader.readBits(nBits);

            pos = skipSiblings(pos, skips);

            // Skip the sibling offset
            for (int i = 0; ; i++) {
                if ((data[pos++] & 0x80) == 0) {
                    break;
                }
            }

            int nodeCh = Utf8.getCharacter(data, pos);

            if (nodeCh == 0) {
                break;
            }

            stringBuf.appendCodePoint(nodeCh);

            pos += Utf8.getLength(data[pos]);
        }
    }

    private int getTranslations(SearchResult[] results,
                                int numResults,
                                String word,
                                int pos)
    {
        StringBuilder stringBuf = new StringBuilder();
        BitReader reader = new BitReader(data);

        while (numResults < results.length) {
            byte payload = data[pos];
            int articleNum =
                (data[pos + 1] & 0xff) |
                ((((int) data[pos + 2]) & 0xff) << 8);

            pos += 3;

            stringBuf.setLength(0);

            reader.resetPosition(pos);
            walkPath(reader, stringBuf);
            pos += reader.getBytesConsumed();

            results[numResults++] = new SearchResult(word,
                                                     stringBuf.toString(),
                                                     (byte) (payload & 0x7f),
                                                     articleNum);

            if ((payload & 0x80) == 0)
                break;
        }

        return numResults;
    }

    // Searches the trie for words that begin with ‘prefix’. The
    // results array is filled with the results. If more results are
    // available than the length of the results array then they are
    // ignored. If fewer are available then the remainder of the array
    // is untouched. The method returns the number of results found.
    public int search(CharSequence prefix,
                      SearchResult[] results)
    {
        int afterPrefix = findPrefix(prefix);

        if (afterPrefix == -1) {
            return 0;
        }

        StringBuilder stringBuf = new StringBuilder(prefix);

        // afterPrefix is now pointing at the first child after the
        // prefix. This node and all of its siblings are therefore
        // extensions of the prefix. We can now depth-first search the
        // tree to get them all in sorted order.

        TrieStack stack = new TrieStack();

        stack.push(afterPrefix, stringBuf.length());

        int numResults = 0;

        while (numResults < results.length && !stack.isEmpty()) {
            int pos = stack.getTopPos();

            stringBuf.setLength(stack.getTopStringLength());

            stack.pop();

            int siblingOffset = 0;

            for (int i = 0; ; i++) {
                siblingOffset |= (data[pos] & 0x7f) << (i * 7);

                if ((data[pos++] & 0x80) == 0) {
                    break;
                }
            }

            int nodeCh = Utf8.getCharacter(data, pos);

            // If there is a sibling then make sure we continue from
            // that after we’ve descended through the children of this
            // node.
            if (siblingOffset > 0) {
                stack.push(pos + siblingOffset, stringBuf.length());
            }

            int dataPos = pos + Utf8.getLength(data[pos]);

            if (nodeCh == 0) {
                // This is a complete word so add all of the
                // translations to the results
                numResults = getTranslations(results,
                                             numResults,
                                             stringBuf.toString(),
                                             dataPos);
            } else {
                // This isn’t the end so descend into the child nodes
                stringBuf.appendCodePoint(nodeCh);
                stack.push(dataPos, stringBuf.length());
            }
        }

        return numResults;
    }

    public byte[] getData()
    {
        return data;
    }

    // Test program
    public static void main(String[] args)
        throws IOException
    {
        if (args.length != 2) {
            System.err.println("Usage: java Trie <index> <prefix>");
            System.exit(1);
        }

        FileInputStream inputStream = new FileInputStream(args[0]);
        Trie trie = new Trie(inputStream);

        SearchResult result[] = new SearchResult[100];

        int numResults = trie.search(args[1], result);

        for (int i = 0; i < numResults; i++) {
            System.out.println(result[i].getWord() +
                               " (" +
                               result[i].getType() +
                               ", " +
                               result[i].getArticleNum() +
                               ", " +
                               result[i].getTranslation() +
                               ")");
        }
    }
}
