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

import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;
import android.os.HandlerThread;
import android.os.Looper;
import android.os.Message;
import android.os.SystemClock;
import android.support.v7.app.AppCompatActivity;
import android.text.Editable;
import android.text.TextWatcher;
import android.text.method.ScrollingMovementMethod;
import android.util.Log;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.widget.TextView;
import java.io.IOException;

public class TransliterationActivity extends AppCompatActivity
    implements TextWatcher
{
    private HandlerThread workerThread;
    private Handler workerHandler;
    private long lastTransliterationTime;
    private Handler uiHandler;
    private Object transliterationToken = new String("transliterationToken");
    private boolean transliterationQueued = false;
    private Trie trie;

    @Override
    public void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.transliteration);

        try {
            trie = TrieCache.getTrie(this);
        }
        catch (IOException e) {
            Log.wtf("Error while loading an asset", e);
        }

        TextView tv = (TextView) findViewById(R.id.transliteration_source);
        tv.addTextChangedListener(this);

        uiHandler = new Handler(Looper.getMainLooper());
        transliterationToken = new String("transliterationToken");

        workerThread = new HandlerThread("transliteration");
        workerThread.start();
        Looper workerLooper = workerThread.getLooper();
        workerHandler = new Handler(workerLooper) {
                @Override public void handleMessage(Message msg)
                {
                    handleWorkerMessage(msg);
                }
            };
    }

    @Override
    public void onDestroy()
    {
        // Cancel any queued transliterations
        uiHandler.removeCallbacksAndMessages(transliterationToken);
        transliterationQueued = false;

        workerThread.quit();

        try {
            workerThread.join();
        } catch (InterruptedException e) {
        }

        workerThread = null;
        workerHandler = null;

        super.onDestroy();
    }

    @Override
    public void onStart()
    {
        super.onStart();

        TextView tv = (TextView) findViewById(R.id.transliteration_source);
        tv.requestFocus();
    }

    @Override
    public boolean onCreateOptionsMenu(Menu menu)
    {
        MenuInflater inflater = getMenuInflater();
        inflater.inflate(R.menu.transliteration_menu, menu);

        return true;
    }

    @Override
    public boolean onOptionsItemSelected(MenuItem item)
    {
        int itemId = item.getItemId();

        if (itemId == R.id.menu_search) {
            Intent intent = new Intent(this, SearchActivity.class);
            intent.setFlags(Intent.FLAG_ACTIVITY_REORDER_TO_FRONT);
            startActivity(intent);
            return true;
        }

        return super.onOptionsItemSelected(item);
    }

    @Override
    public void afterTextChanged(Editable s)
    {
        if (transliterationQueued)
            return;

        long now = SystemClock.uptimeMillis();
        // Delay the transliteration until at least one second since
        // the last transliteration timey
        long delay = lastTransliterationTime + 1000 - now;

        if (delay < 0)
            delay = 0;

        lastTransliterationTime = now + delay;
        transliterationQueued = true;

        uiHandler.postAtTime(new Runnable() {
                public void run()
                {
                    transliterationQueued = false;
                    int id = R.id.transliteration_source;
                    TextView tv = (TextView) findViewById(id);
                    String text = tv.getText().toString();
                    Message msg = workerHandler.obtainMessage(0, // what
                                                              text);
                    workerHandler.sendMessage(msg);
                }
            },
            transliterationToken,
            lastTransliterationTime);
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

    private CharSequence transliterate(String source)
    {
        StringBuilder buf = new StringBuilder();

        Compiledb.transliterate(trie.getData(), source, buf);

        return buf;
    }

    private void handleWorkerMessage(Message msg)
    {
        if (msg.obj != null && msg.obj instanceof String) {
            CharSequence transliteration = transliterate((String) msg.obj);

            runOnUiThread(new Runnable() {
                    public void run()
                    {
                        int id = R.id.transliteration_dest;
                        TextView tv = (TextView) findViewById(id);
                        tv.setText(transliteration);
                    }
                });
        }
    }
}
