// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2012, 2016, 2024  Neil Roberts
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
import android.content.res.Resources;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.BaseAdapter;
import android.widget.Filter;
import android.widget.Filterable;
import android.widget.TextView;
import java.util.Locale;

class SearchResultData
{
    public SearchResult[] results;
    public int count;
}

public class SearchAdapter extends BaseAdapter
    implements Filterable
{
    static private final int MAX_RESULTS = 128;

    private Context context;
    private SearchFilter filter;

    private SearchResult[] results;
    private int numResults = 0;

    public SearchAdapter(Context context)
    {
        this.context = context;

        setResultData(doSearch(""));
    }

    private void setResultData(SearchResultData resultData)
    {
        results = resultData.results;
        numResults = resultData.count;
    }

    private SearchResultData doSearch(CharSequence filterString)
    {
        SearchResultData resultData = new SearchResultData();

        resultData.results = new SearchResult[MAX_RESULTS];

        try {
            byte[] trie = TrieCache.getTrie(context);
            int numResults = Compiledb.search(trie,
                                              filterString,
                                              resultData.results);

            resultData.count = numResults;
            return resultData;
        } catch (java.io.IOException e) {
            throw new IllegalStateException("Error while loading " +
                                            "an asset");
        }
    }

    @Override
    public int getCount()
    {
        return numResults;
    }

    @Override
    public SearchResult getItem(int position)
    {
        return results[position];
    }

    @Override
    public long getItemId (int position)
    {
        // FIXME
        return position;
    }

    @Override
    public View getView(int position, View convertView, ViewGroup parent)
    {
        View layout;

        if (convertView == null) {
            LayoutInflater layoutInflater = LayoutInflater.from(context);
            int id;
            id = R.layout.search_item;
            layout = layoutInflater.inflate(id, parent, false);
        } else {
            layout = convertView;
        }

        SearchResult result = getItem(position);

        TextView word = layout.findViewById(R.id.word);
        word.setText(result.getWord());

        TextView translation = layout.findViewById(R.id.translation);
        translation.setText(result.getTranslation());

        TextView wordType = layout.findViewById(R.id.word_type);
        wordType.setText(PartOfSpeech.name(result.getType()));

        return layout;
    }

    @Override
    public boolean hasStableIds()
    {
        return false;
    }

    @Override
    public boolean isEmpty()
    {
        return numResults == 0;
    }

    @Override
    public boolean areAllItemsEnabled()
    {
        return false;
    }

    @Override
    public Filter getFilter()
    {
        if (filter == null)
            filter = new SearchFilter();

        return filter;
    }

    private static CharSequence normalizeFilter(CharSequence seq)
    {
        boolean spaceQueued = false;
        StringBuilder stringBuf = new StringBuilder();
        int length = seq.length();

        for (int i = 0;
             i < length;
             i = Character.offsetByCodePoints(seq, i, 1)) {
            int ch = Character.codePointAt(seq, i);

            if (Character.isWhitespace(ch)) {
                spaceQueued = true;
            } else {
                if (spaceQueued) {
                    if (stringBuf.length() > 0) {
                        stringBuf.append(' ');
                    }
                    spaceQueued = false;
                }

                if (ch == '’')
                    ch = '\'';

                stringBuf.appendCodePoint(Character.toLowerCase(ch));
            }
        }

        return stringBuf;
    }

    private class SearchFilter extends Filter
    {
        @Override
        public FilterResults performFiltering(CharSequence filter)
        {
            FilterResults ret = new FilterResults();
            SearchResultData resultData = doSearch(normalizeFilter(filter));

            ret.count = resultData.count;
            ret.values = resultData;

            return ret;
        }

        @Override
        public void publishResults(CharSequence filter, FilterResults res)
        {
            setResultData((SearchResultData) res.values);

            if (res.count > 0)
                notifyDataSetChanged();
            else
                notifyDataSetInvalidated();
        }
    }
}
