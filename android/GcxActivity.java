// SPDX-License-Identifier: MIT
// Mechanical framework trampoline only. Rust owns document/result policy.
package com.machinerhythm.gcx;

@SuppressWarnings("deprecation")
public final class GcxActivity extends android.app.NativeActivity {
    // NativeActivity's native-code loader does not register JNI methods with
    // this Java class loader. Associate the same library before callbacks.
    static { System.loadLibrary("gcx_android"); }
    private native void documentResult(int request, int result, android.content.Intent data);
    private native void uiAction(int action, long menu, long runtime);
    private native void beforeDestroy();

    public static final class Action implements android.view.View.OnClickListener {
        private final int action;
        private final long menu;
        private final long runtime;
        private final GcxActivity activity;
        public Action(GcxActivity activity, int action, long menu, long runtime) {
            this.activity = activity; this.action = action; this.menu = menu; this.runtime = runtime;
        }
        @Override public void onClick(android.view.View view) { activity.uiAction(action, menu, runtime); }
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
