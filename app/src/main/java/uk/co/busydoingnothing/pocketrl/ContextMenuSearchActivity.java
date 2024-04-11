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

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;

// This is intended to be invoked by the PROCESS_TEXT intent action.

public class ContextMenuSearchActivity extends AppCompatActivity
{
    private void handleSearchIntent()
    {
        Intent intent = getIntent();

        if (intent == null)
            return;

        String action = intent.getAction();

        if (action == null || !action.equals(Intent.ACTION_PROCESS_TEXT))
            return;

        String searchString = intent.getStringExtra(Intent.EXTRA_PROCESS_TEXT);

        if (searchString == null)
            return;

        intent = new Intent(this, SearchActivity.class);
        intent.putExtra(SearchActivity.EXTRA_SEARCH_TERM, searchString);
        startActivity(intent);
    }

    @Override
    public void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);

        handleSearchIntent();

        // Finish this activity to get it out of the call stack.
        finish();
    }
}
