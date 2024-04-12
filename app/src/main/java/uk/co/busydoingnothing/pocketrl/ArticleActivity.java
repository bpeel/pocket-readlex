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
import android.content.SharedPreferences;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.support.v7.app.AppCompatActivity;
import android.util.Log;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.LinearLayout;
import android.widget.RelativeLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.ZoomControls;
import java.io.IOException;
import java.util.Locale;

public class ArticleActivity extends AppCompatActivity
    implements SharedPreferences.OnSharedPreferenceChangeListener
{
    public static final String EXTRA_ARTICLE_NUMBER =
        "uk.co.busydoingnothing.pocketrl.ArticleNumber";
    public static final String POCKETRL_PREFERENCES =
        "PocketrlPreferences";
    public static final String PREF_FONT_SIZE =
        "fontSize";

    public static final String TAG = "pocketrlarticle";

    private ScrollView scrollView;
    private View articleView;
    private int articleNumber;

    private ZoomControls zoomControls;
    private RelativeLayout layout;

    // There are 10 font sizes ranging from 0 to 9. The actual font size
    // used is calculated from a logarithmic scale and set in density
    // independent pixels.
    private static final int N_FONT_SIZES = 10;
    private static final float FONT_SIZE_ROOT = 1.2f;

    private int fontSize = N_FONT_SIZES / 2;
    private float baseTextSize;

    private static final int MSG_HIDE_ZOOM_CONTROLS = 4;

    private static final int N_ARTICLES_PER_FILE = 128;

    private Handler handler;

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

    private CharSequence readString(BinaryReader in)
        throws IOException
    {
        StringBuilder stringBuf = new StringBuilder();
        readStringIntoBuffer(in, stringBuf);
        return stringBuf;
    }

    private CharSequence readTranslation(BinaryReader in)
        throws IOException
    {
        StringBuilder stringBuf = new StringBuilder();

        readStringIntoBuffer(in, stringBuf);
        stringBuf.append(" → ");
        readStringIntoBuffer(in, stringBuf);

        return stringBuf;
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

        while (in.getPosition() - articleStart < articleLength) {
            View entry = layoutInflater.inflate(R.layout.article_entry,
                                                layout,
                                                false /* attachToRoot */);

            CharSequence translation = readTranslation(in);
            TextView tv = (TextView) entry.findViewById(R.id.entry_translation);
            tv.setText(translation);

            baseTextSize = tv.getTextSize();

            CharSequence type = readPartOfSpeech(in);
            tv = (TextView) entry.findViewById(R.id.entry_type);
            tv.setText(type);

            CharSequence ipa = readString(in);
            tv = (TextView) entry.findViewById(R.id.entry_ipa);
            tv.setText(ipa);

            CharSequence var = readVariant(in);
            tv = (TextView) entry.findViewById(R.id.entry_var);
            tv.setText(var);

            layout.addView(entry);
        }

        return layout;
    }

    private void updateZoomability()
    {
        if (zoomControls != null) {
            zoomControls.setIsZoomInEnabled(fontSize < N_FONT_SIZES - 1);
            zoomControls.setIsZoomOutEnabled(fontSize > 0);
        }
    }

    private void setFontSize(int fontSize)
    {
        if (fontSize < 0)
            fontSize = 0;
        else if (fontSize >= N_FONT_SIZES)
            fontSize = N_FONT_SIZES - 1;

        if (fontSize != this.fontSize) {
            // There’s no point in updating the font size if a reload is
            // queued because it will just get set back to the default
            // when it is finally reloaded.
            if (!reloadQueued) {
                float fontSizeScale =
                    (float) Math.pow(FONT_SIZE_ROOT,
                                     fontSize - N_FONT_SIZES / 2);

                // FIXME
            }

            this.fontSize = fontSize;

            updateZoomability();
        }
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

        // The font size will have been reset to the default so we need to
        // update it.
        int oldFontSize = this.fontSize;
        this.fontSize = N_FONT_SIZES / 2;
        setFontSize(oldFontSize);
    }

    @Override
    public void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);

        setContentView(R.layout.article);

        scrollView = (ScrollView) findViewById(R.id.article_scroll_view);
        layout = (RelativeLayout) findViewById(R.id.article_layout);

        reloadQueued = true;

        SharedPreferences prefs =
            getSharedPreferences(POCKETRL_PREFERENCES,
                                 Activity.MODE_PRIVATE);

        setFontSize(prefs.getInt(PREF_FONT_SIZE, fontSize));

        prefs.registerOnSharedPreferenceChangeListener(this);
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

    @Override
    public void onDestroy()
    {
        SharedPreferences prefs =
            getSharedPreferences(POCKETRL_PREFERENCES,
                                 Activity.MODE_PRIVATE);

        prefs.unregisterOnSharedPreferenceChangeListener(this);

        super.onDestroy();
    }

    private void zoom(int direction)
    {
        int fontSize = this.fontSize + direction;

        if (fontSize >= N_FONT_SIZES)
            fontSize = N_FONT_SIZES - 1;
        else if (fontSize < 0)
            fontSize = 0;

        SharedPreferences prefs =
            getSharedPreferences(POCKETRL_PREFERENCES,
                                 Activity.MODE_PRIVATE);
        SharedPreferences.Editor editor = prefs.edit();
        editor.putInt(PREF_FONT_SIZE, fontSize);
        editor.commit();

        setHideZoom();
        updateZoomability();
    }

    private void setHideZoom()
    {
        handler.removeMessages(MSG_HIDE_ZOOM_CONTROLS);
        handler.sendEmptyMessageDelayed(MSG_HIDE_ZOOM_CONTROLS, 10000);
    }

    private void showZoomController()
    {
        if (zoomControls == null) {
            final int wrap = RelativeLayout.LayoutParams.WRAP_CONTENT;
            RelativeLayout.LayoutParams lp =
                new RelativeLayout.LayoutParams(wrap, wrap);
            final float scale = getResources().getDisplayMetrics().density;

            lp.addRule(RelativeLayout.CENTER_HORIZONTAL);
            lp.addRule(RelativeLayout.ALIGN_PARENT_BOTTOM);
            lp.bottomMargin = (int) (10.0f * scale + 0.5f);

            zoomControls = new ZoomControls(this);
            zoomControls.setVisibility(View.GONE);
            layout.addView(zoomControls, lp);

            zoomControls.setOnZoomInClickListener(new View.OnClickListener() {
                    @Override
                    public void onClick(View v)
                    {
                        zoom(+1);
                    }
                });
            zoomControls.setOnZoomOutClickListener(new View.OnClickListener() {
                    @Override
                    public void onClick(View v)
                    {
                        zoom(-1);
                    }
                });

            handler = new Handler() {
                    @Override
                    public void handleMessage(Message msg)
                    {
                        switch (msg.what)
                            {
                            case MSG_HIDE_ZOOM_CONTROLS:
                                if (zoomControls != null)
                                    zoomControls.hide();
                                break;
                            }
                    }
                };

            updateZoomability();
        }

        if (zoomControls.getVisibility() != View.VISIBLE) {
            zoomControls.show();
            setHideZoom();
        }
    }

    @Override
    public void onSharedPreferenceChanged(SharedPreferences prefs,
                                          String key)
    {
        if (key.equals(PREF_FONT_SIZE)) {
            setFontSize(prefs.getInt(PREF_FONT_SIZE, fontSize));
        }
    }
}
