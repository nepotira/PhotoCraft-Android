package ai.storyteller.photocraft;

import android.app.Activity;
import android.content.ClipData;
import android.net.Uri;
import android.util.Log;
import android.view.DragEvent;
import android.view.PointerIcon;
import android.view.View;
import android.view.WindowInsets;
import java.util.ArrayList;
import java.util.List;

/**
 * DropHelper provides robust Drag & Drop support (Google Drive & local Downloads),
 * mouse pointer icon updating on the Android UI thread, and window caption
 * bar inset detection for desktop environments like Googlebook OS and Samsung DeX.
 */
public class DropHelper {
    private static final String TAG = "PhotoCraft";
    private static final List<String> PENDING_DROPS = new ArrayList<String>();
    private static Object sLastPermissions = null;

    public static native void onDrop(String uri);

    public static synchronized void queueDrop(String uri) {
        if (uri != null && !uri.isEmpty()) {
            Log.i(TAG, "Queueing dropped URI in Java: " + uri);
            PENDING_DROPS.add(uri);
        }
    }

    public static synchronized String pollDrop() {
        if (PENDING_DROPS.isEmpty()) {
            return null;
        }
        return PENDING_DROPS.remove(0);
    }

    public static void attach(final Activity activity) {
        if (activity == null) return;
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    View.OnDragListener dragListener = new View.OnDragListener() {
                        @Override
                        public boolean onDrag(View v, DragEvent event) {
                            int action = event.getAction();
                            if (action == DragEvent.ACTION_DRAG_STARTED) {
                                return true;
                            } else if (action == DragEvent.ACTION_DRAG_ENTERED || action == DragEvent.ACTION_DRAG_LOCATION) {
                                return true;
                            } else if (action == DragEvent.ACTION_DROP) {
                                Log.i(TAG, "ACTION_DROP received in DropHelper");
                                try {
                                    sLastPermissions = activity.requestDragAndDropPermissions(event);
                                } catch (Throwable t) {
                                    Log.w(TAG, "requestDragAndDropPermissions exception: " + t.getMessage());
                                }

                                ClipData clip = event.getClipData();
                                if (clip != null) {
                                    for (int i = 0; i < clip.getItemCount(); i++) {
                                        ClipData.Item item = clip.getItemAt(i);
                                        if (item != null) {
                                            Uri uri = item.getUri();
                                            if (uri != null) {
                                                String uriStr = uri.toString();
                                                queueDrop(uriStr);
                                                try {
                                                    onDrop(uriStr);
                                                } catch (Throwable ignored) {}
                                            } else {
                                                CharSequence text = item.getText();
                                                if (text != null) {
                                                    String s = text.toString().trim();
                                                    if (s.startsWith("content://") || s.startsWith("file://") || s.startsWith("/")) {
                                                        queueDrop(s);
                                                        try {
                                                            onDrop(s);
                                                        } catch (Throwable ignored) {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                return true;
                            } else if (action == DragEvent.ACTION_DRAG_ENDED) {
                                return true;
                            }
                            return true;
                        }
                    };

                    View decor = activity.getWindow().getDecorView();
                    if (decor != null) {
                        decor.setOnDragListener(dragListener);
                    }
                    View content = activity.findViewById(android.R.id.content);
                    if (content != null) {
                        content.setOnDragListener(dragListener);
                    }
                    Log.i(TAG, "DropHelper attached successfully to decorView and contentView");
                } catch (Throwable t) {
                    Log.e(TAG, "Failed to attach drag listener: " + t.getMessage());
                }
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
                    View decor = activity.getWindow().getDecorView();
                    if (decor != null) {
                        decor.setPointerIcon(icon);
                    }
                    View content = activity.findViewById(android.R.id.content);
                    if (content != null) {
                        content.setPointerIcon(icon);
                    }
                } catch (Throwable ignored) {}
            }
        });
    }

    public static int getCaptionTopInset(Activity activity) {
        if (activity == null) return 0;
        try {
            View decor = activity.getWindow().getDecorView();
            if (decor != null) {
                WindowInsets insets = decor.getRootWindowInsets();
                if (insets != null) {
                    // WindowInsets.Type.captionBar() = 4 | WindowInsets.Type.statusBars() = 1 => 5
                    android.graphics.Insets ins = insets.getInsets(5);
                    return ins.top;
                }
            }
        } catch (Throwable ignored) {}
        return 0;
    }
}
