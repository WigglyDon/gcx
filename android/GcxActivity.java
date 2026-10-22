// SPDX-License-Identifier: MIT
// Mechanical framework trampoline only. Rust owns document/result policy.
package com.machinerhythm.gcx;

@SuppressWarnings("deprecation")
public final class GcxActivity extends android.app.NativeActivity {
    private native void documentResult(int request, int result, android.content.Intent data);
    private native void uiAction(int action);
    private native void beforeDestroy();

    public static final class Action implements android.view.View.OnClickListener {
        private final int action;
        private final GcxActivity activity;
        public Action(GcxActivity activity, int action) { this.activity = activity; this.action = action; }
        @Override public void onClick(android.view.View view) { activity.uiAction(action); }
    }

    @Override
    protected void onActivityResult(int request, int result, android.content.Intent data) {
        super.onActivityResult(request, result, data);
        documentResult(request, result, data);
    }

    @Override
    protected void onDestroy() {
        beforeDestroy();
        super.onDestroy();
    }
}
