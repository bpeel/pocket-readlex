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

import android.app.Activity;
import android.content.Intent;
import android.content.res.AssetManager;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.util.Log;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import java.io.IOException;
import java.util.Locale;

public class ArticleActivity extends AppCompatActivity
{
    public static final String EXTRA_ARTICLE_NUMBER =
        "uk.co.busydoingnothing.pocketrl.ArticleNumber";

    public static final String TAG = "pocketrlarticle";

    private ScrollView scrollView;
    private View articleView;
    private int articleNumber;

    private static final int N_ARTICLES_PER_FILE = 128;

    private boolean reloadQueued;

    private void skipArticles(BinaryReader in,
                              int numArticles)
        throws IOException
    {
        for (int i = 0; i < numArticles; i++) {
            int articleLength = in.readShort();

            in.skip(articleLength);
        }
    }

    private void readStringIntoBuffer(BinaryReader in,
                                      StringBuilder stringBuf)
        throws IOException
    {
        int length = in.readByte() & 0xff;
        long startPosition = in.getPosition();
        byte[] characterBuf = new byte[6];

        while (in.getPosition() - startPosition < length) {
            characterBuf[0] = (byte) in.readByte();
            in.readAll(characterBuf, 1, Utf8.getLength(characterBuf[0]) - 1);
            int ch = Utf8.getCharacter(characterBuf, 0);
            stringBuf.appendCodePoint(ch);
        }
    }

    private CharSequence readPartOfSpeech(BinaryReader in)
        throws IOException
    {
        int nParts = in.readByte() & 0xff;

        if (nParts == 1) {
            return PartOfSpeech.name(in.readByte() & 0xff);
        } else {
            StringBuilder stringBuf = new StringBuilder();

            for (int i = 0; i < nParts; i++) {
                if (i > 0)
                    stringBuf.append(" + ");

                stringBuf.append(PartOfSpeech.name(in.readByte() & 0xff));
            }

            return stringBuf;
        }
    }

    private CharSequence readIpa(BinaryReader in)
        throws IOException
    {
        StringBuilder stringBuf = new StringBuilder("/");

        readStringIntoBuffer(in, stringBuf);

        stringBuf.append("/");

        return stringBuf;
    }

    private String readVariant(BinaryReader in)
        throws IOException
    {
        return Variant.name(in.readByte() & 0xff);
    }

    private LinearLayout loadArticle(int article)
        throws IOException
    {
        AssetManager assetManager = getAssets();
        String filename = String.format(Locale.US,
                                        "articles/article-%04x.bin",
                                        article & ~(N_ARTICLES_PER_FILE - 1));
        BinaryReader in = new BinaryReader(assetManager.open(filename));

        skipArticles(in, article & (N_ARTICLES_PER_FILE - 1));

        int articleLength = in.readShort();
        long articleStart = in.getPosition();

        LinearLayout layout = new LinearLayout(this);
        layout.setOrientation(LinearLayout.VERTICAL);

        LayoutInflater layoutInflater = getLayoutInflater();

        int entryNum = 0;

        while (in.getPosition() - articleStart < articleLength) {
            View entry = layoutInflater.inflate(R.layout.article_entry,
                                                layout,
                                                false /* attachToRoot */);

            StringBuilder stringBuf = new StringBuilder();
            readStringIntoBuffer(in, stringBuf);

            if (entryNum == 0)
                setTitle(stringBuf.toString());

            stringBuf.append(" → ");
            readStringIntoBuffer(in, stringBuf);
            TextView tv = (TextView) entry.findViewById(R.id.entry_translation);
            tv.setText(stringBuf);

            CharSequence type = readPartOfSpeech(in);
            tv = (TextView) entry.findViewById(R.id.entry_type);
            tv.setText(type);

            CharSequence ipa = readIpa(in);
            tv = (TextView) entry.findViewById(R.id.entry_ipa);
            tv.setText(ipa);

            CharSequence var = readVariant(in);
            tv = (TextView) entry.findViewById(R.id.entry_var);
            tv.setText(var);

            layout.addView(entry);

            entryNum++;
        }

        return layout;
    }

    private void loadIntendedArticle()
    {
        Intent intent = getIntent();

        if (intent != null) {
            int article = intent.getIntExtra(EXTRA_ARTICLE_NUMBER, -1);

            if (article != -1) {
                try {
                    this.articleNumber = article;
                    if (articleView != null)
                        scrollView.removeView(articleView);
                    articleView = loadArticle(article);
                    scrollView.addView(articleView);
                }
                catch (IOException e) {
                    Log.wtf("Error while loading an asset", e);
                }
            }
        }
    }

    @Override
    public void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);

        setContentView(R.layout.article);

        scrollView = (ScrollView) findViewById(R.id.article_scroll_view);

        reloadQueued = true;
    }

    @Override
    public void onStart()
    {
        super.onStart();

        if (reloadQueued) {
            reloadQueued = false;
            loadIntendedArticle();
        }
    }
}
