package ai.storyteller.photocraft;

import android.app.Activity;
import android.content.ClipData;
import android.net.Uri;
import android.view.DragEvent;
import android.view.PointerIcon;
import android.view.View;
import android.view.WindowInsets;

/**
 * DropHelper provides native Drag & Drop support (Google Drive & Downloads),
 * mouse pointer icon updating on the Android UI thread, and window caption
 * bar inset detection for desktop environments like Googlebook OS and Samsung DeX.
 */
public class DropHelper {
    public static native void onDrop(String uri);

    public static void attach(final Activity activity) {
        if (activity == null) return;
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    View decor = activity.getWindow().getDecorView();
                    decor.setOnDragListener(new View.OnDragListener() {
                        @Override
                        public boolean onDrag(View v, DragEvent event) {
                            int action = event.getAction();
                            if (action == DragEvent.ACTION_DRAG_STARTED) {
                                return true;
                            } else if (action == DragEvent.ACTION_DROP) {
                                try {
                                    activity.requestDragAndDropPermissions(event);
                                } catch (Throwable ignored) {}

                                ClipData clip = event.getClipData();
                                if (clip != null) {
                                    for (int i = 0; i < clip.getItemCount(); i++) {
                                        ClipData.Item item = clip.getItemAt(i);
                                        if (item != null) {
                                            Uri uri = item.getUri();
                                            if (uri != null) {
                                                try {
                                                    onDrop(uri.toString());
                                                } catch (Throwable ignored) {}
                                            }
                                        }
                                    }
                                }
                                return true;
                            }
                            return true;
                        }
                    });
                } catch (Throwable ignored) {}
            }
        });
    }

    public static void setPointerIcon(final Activity activity, final int type) {
        if (activity == null) return;
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    PointerIcon icon = PointerIcon.getSystemIcon(activity, type);
                    activity.getWindow().getDecorView().setPointerIcon(icon);
                } catch (Throwable ignored) {}
            }
        });
    }

    public static int getCaptionTopInset(Activity activity) {
        if (activity == null) return 0;
        try {
            View decor = activity.getWindow().getDecorView();
            WindowInsets insets = decor.getRootWindowInsets();
            if (insets != null) {
                // WindowInsets.Type.captionBar() = 4 | WindowInsets.Type.statusBars() = 1 => 5
                android.graphics.Insets ins = insets.getInsets(5);
                return ins.top;
            }
        } catch (Throwable ignored) {}
        return 0;
    }
}
