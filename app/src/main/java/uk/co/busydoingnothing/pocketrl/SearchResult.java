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

public class SearchResult
{
    private String word;
    private String translation;
    private byte type;
    private int articleNum;

    public SearchResult(String word,
                        String translation,
                        byte type,
                        int articleNum)
    {
        this.word = word;
        this.translation = translation;
        this.type = type;
        this.articleNum = articleNum;
    }

    public String getWord()
    {
        return word;
    }

    public String getTranslation()
    {
        return translation;
    }

    public byte getType()
    {
        return type;
    }

    public int getArticleNum()
    {
        return articleNum;
    }
}
