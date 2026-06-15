package com.puce.virtual

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import java.util.concurrent.ConcurrentLinkedQueue

/**
 * PUCE Virtual Input Service for Android.
 * 
 * Uses Android Accessibility API to inject generic touch events that map to 
 * virtual controller inputs when low-level Uinput access is not available (non-root).
 * For rooted devices, PUCE uses a direct /dev/uinput binary wrapper.
 */
class VirtualInputService : AccessibilityService() {

    companion object {
        private const val TAG = "PUCE_VirtualInput"
        var instance: VirtualInputService? = null
            private set
    }

    private val actionQueue = ConcurrentLinkedQueue<Runnable>()
    private var isDispatching = false

    override fun onServiceConnected() {
        super.onServiceConnected()
        instance = this
        Log.i(TAG, "VirtualInputService connected. Ready to inject emulated events.")
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // We don't need to listen to events, only inject them.
    }

    override fun onInterrupt() {
        Log.w(TAG, "VirtualInputService interrupted.")
        actionQueue.clear()
        isDispatching = false
    }

    override fun onUnbind(intent: android.content.Intent?): Boolean {
        instance = null
        Log.i(TAG, "VirtualInputService unbound.")
        return super.onUnbind(intent)
    }

    /**
     * Called via JNI from the Rust PUCE core to inject a simulated button press or stick movement.
     */
    fun injectTouch(x: Float, y: Float, durationMs: Long) {
        val path = Path().apply {
            moveTo(x, y)
        }
        val stroke = GestureDescription.StrokeDescription(path, 0, durationMs)
        val gesture = GestureDescription.Builder().addStroke(stroke).build()

        actionQueue.add(Runnable {
            dispatchGesture(gesture, object : GestureResultCallback() {
                override fun onCompleted(gestureDescription: GestureDescription?) {
                    super.onCompleted(gestureDescription)
                    processNextAction()
                }

                override fun onCancelled(gestureDescription: GestureDescription?) {
                    super.onCancelled(gestureDescription)
                    processNextAction()
                }
            }, null)
        })

        if (!isDispatching) {
            processNextAction()
        }
    }

    private fun processNextAction() {
        val action = actionQueue.poll()
        if (action != null) {
            isDispatching = true
            action.run()
        } else {
            isDispatching = false
        }
    }
}
