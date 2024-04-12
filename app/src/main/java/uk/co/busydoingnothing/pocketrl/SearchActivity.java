// Pocket ReadLex – An offline app for ReadLex
// Copyright (C) 2012, 2013, 2016, 2024  Neil Roberts
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

import android.app.Dialog;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.res.Resources;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.text.Editable;
import android.text.TextWatcher;
import android.util.Log;
import android.view.inputmethod.InputMethodManager;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.view.View;
import android.widget.AdapterView;
import android.widget.ListView;
import android.widget.TextView;
import java.util.Vector;

public class SearchActivity extends AppCompatActivity
    implements TextWatcher
{
    public static final String EXTRA_SEARCH_TERM =
        "uk.co.busydoingnothing.pocketrl.SearchTerm";

    public static final String TAG = "pocketrlsearch";

    private SearchAdapter searchAdapter;

    @Override
    public void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.search);

        ListView lv = (ListView) findViewById(R.id.list);
        lv.setEmptyView(findViewById(R.id.empty));

        TextView tv = (TextView) findViewById(R.id.search_edit);
        tv.addTextChangedListener(this);

        searchAdapter = new SearchAdapter(this);

        lv.setAdapter(searchAdapter);

        Intent intent = getIntent();
        if (intent != null) {
            String searchTerm = intent.getStringExtra(EXTRA_SEARCH_TERM);

            if (searchTerm != null)
                tv.setText(searchTerm);
        }

        lv.setOnItemClickListener(new AdapterView.OnItemClickListener() {
            public void onItemClick (AdapterView<?> parent,
                                     View view,
                                     int position,
                                     long id)
            {
              SearchAdapter adapter =
                (SearchAdapter) parent.getAdapter();
              SearchResult result = adapter.getItem(position);
              Intent intent = new Intent(view.getContext(),
                                         ArticleActivity.class);
              intent.putExtra(ArticleActivity.EXTRA_ARTICLE_NUMBER,
                              result.getArticleNum());
              startActivity(intent);
            }
          });
    }

    @Override
    public void onStart()
    {
        super.onStart();

        View tv = findViewById(R.id.search_edit);

        tv.requestFocus();

        InputMethodManager imm =
            (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);

        if (imm != null) {
            imm.showSoftInput(tv,
                              0, /* flags */
                              null /* resultReceiver */);
        }
    }

    @Override
    public void afterTextChanged(Editable s)
    {
        searchAdapter.getFilter().filter(s);
    }

    @Override
    public void beforeTextChanged(CharSequence s,
                                  int start,
                                  int count,
                                  int after)
    {
    }

    @Override
    public void onTextChanged(CharSequence s,
                              int start,
                              int before,
                              int count)
    {
    }
}
