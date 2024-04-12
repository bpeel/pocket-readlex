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

import android.app.AlertDialog;
import android.app.Dialog;
import android.content.DialogInterface;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.content.res.Resources;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.text.Editable;
import android.text.SpannableStringBuilder;
import android.text.TextWatcher;
import android.text.method.LinkMovementMethod;
import android.text.style.URLSpan;
import android.util.Log;
import android.view.inputmethod.InputMethodManager;
import android.view.LayoutInflater;
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
    private static final String LICENSE_URL =
        "https://www.gnu.org/licenses/gpl-3.0-standalone.html";
    private static final String READ_LEXICON_URL =
        "https://readlex.pythonanywhere.com/";
    private static final String PRIVACY_POLICY_URL =
        "https://busydoingnothing.co.uk/pocketrl/privacy-policy.html";

    private static final int DIALOG_ABOUT = 0;

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
    public boolean onCreateOptionsMenu(Menu menu)
    {
        MenuInflater inflater = getMenuInflater();
        inflater.inflate(R.menu.search_menu, menu);

        return true;
    }

    @Override
    public boolean onOptionsItemSelected(MenuItem item)
    {
        if (item.getItemId() == R.id.menu_about) {
            showDialog(DIALOG_ABOUT);
            return true;
        }

        return super.onOptionsItemSelected(item);
    }

    private void linkifyAboutMessage(SpannableStringBuilder string)
    {
        int pos = string.toString().indexOf("@VERSION@");

        if (pos != -1) {
            String packageVersion;

            try {
                PackageManager manager = getPackageManager();
                String packageName = getPackageName();
                PackageInfo packageInfo =
                    manager.getPackageInfo(packageName, 0);

                packageVersion = packageInfo.versionName;
            } catch (PackageManager.NameNotFoundException e) {
                packageVersion = "?";
            }

            string.replace(pos, pos + 9, packageVersion);
        }

        pos = string.toString ().indexOf("Click here for");

        if (pos != -1) {
            URLSpan span = new URLSpan(LICENSE_URL);
            string.setSpan(span, pos + 6, pos + 10, 0 /* flags */);
        }

        pos = string.toString ().indexOf("Read Lexicon");

        if (pos != -1) {
            URLSpan span = new URLSpan(READ_LEXICON_URL);
            string.setSpan(span, pos, pos + 12, 0 /* flags */);
        }

        pos = string.toString ().indexOf("Privacy policy");

        if (pos != -1) {
            URLSpan span = new URLSpan(PRIVACY_POLICY_URL);
            string.setSpan(span, pos, pos + 14, 0 /* flags */);
        }
    }

    @Override
    protected Dialog onCreateDialog(int id)
    {
        if (id != DIALOG_ABOUT)
            return null;

        Dialog dialog;
        Resources res = getResources();

        AlertDialog.Builder builder = new AlertDialog.Builder(this);
        SpannableStringBuilder message =
            new SpannableStringBuilder(res.getText(R.string.about_message));

        linkifyAboutMessage(message);

        LayoutInflater layoutInflater = getLayoutInflater();
        TextView tv =
            (TextView) layoutInflater.inflate(R.layout.about_view,
                                              null);
        tv.setText(message);
        tv.setMovementMethod(LinkMovementMethod.getInstance());

        builder
            .setView(tv)
            .setCancelable(true)
            .setNegativeButton(R.string.close,
                               new DialogInterface.OnClickListener() {
                                   @Override
                                   public void onClick (DialogInterface dialog,
                                                        int whichButton)
                                   {
                                   }
                               });
        dialog = builder.create();

        return dialog;
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
