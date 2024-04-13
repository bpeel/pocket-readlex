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

import android.util.AttributeSet;
import android.widget.TextView;
import android.content.Context;
import android.widget.LinearLayout;

public class VariantView extends LinearLayout
{
    private CharSequence latin;
    private CharSequence shavian;
    private CharSequence ipa;

    public class ContextMenuInfo
        implements android.view.ContextMenu.ContextMenuInfo
    {
        public CharSequence latin;
        public CharSequence shavian;
        public CharSequence ipa;
    }

    private ContextMenuInfo contextMenuInfo;

    public VariantView(Context context)
    {
        super(context);
    }

    public VariantView(Context context, AttributeSet attrs)
    {
        super(context, attrs);
    }

    public VariantView(Context context, AttributeSet attrs, int defStyle)
    {
        super(context, attrs, defStyle);
    }

    public void setSpellings(CharSequence latin,
                             CharSequence shavian,
                             CharSequence ipa)
    {
        this.latin = latin;
        this.shavian = shavian;
        this.ipa = ipa;
        contextMenuInfo = null;
    }

    public ContextMenuInfo getContextMenuInfo()
    {
        if (contextMenuInfo == null) {
            contextMenuInfo = new ContextMenuInfo();
            contextMenuInfo.latin = this.latin;
            contextMenuInfo.shavian = this.shavian;
            contextMenuInfo.ipa = this.ipa;
        }

        return contextMenuInfo;
    }
}
