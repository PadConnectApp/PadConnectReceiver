
package io.github.padconnect.receiver.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.github.padconnect.receiver.SystemInfo
import io.github.padconnect.receiver.data.GamepadState
import io.github.padconnect.receiver.input.LinuxInputExecutor
import io.github.padconnect.receiver.input.XInputExecutor
import io.github.padconnect.receiver.utils.DiscoveryServer
import io.github.padconnect.receiver.utils.UdpReceiver
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlin.time.Duration.Companion.milliseconds

class ReceiverViewModel : ViewModel() {
    private val _lastState = MutableStateFlow<GamepadState?>(null)
    val lastState: StateFlow<GamepadState?> = _lastState

    var lastUiUpdate = 0L

    @Volatile private var lastReceiveTime = 0L
    @Volatile private var isReceiving = false
    private val TIMEOUT_MS = 2000L

    private val executor by lazy {
        if (SystemInfo.OS.contains("win")) {
            XInputExecutor()
        } else {
            LinuxInputExecutor()
        }
    }

    private val receiver = UdpReceiver(8082) { buttons, lx, ly, rx, ry, lt, rt ->
        lastReceiveTime = System.currentTimeMillis()
        if (!isReceiving) {
            isReceiving = true
            discovery.stop()
        }

        executor.submit(buttons, lx, ly, rx, ry, lt, rt)
        val now = System.currentTimeMillis()
        if (now - lastUiUpdate > 16) {
            lastUiUpdate = now
            onEvent(buttons, lx, ly, rx, ry, lt, rt)
        }
    }

    val discovery = DiscoveryServer(port = 8083)

    init {
        receiver.start()
        discovery.start()
        when (executor) {
            is XInputExecutor -> {
                (executor as XInputExecutor).onRumble = { large: Int, small: Int ->
                    receiver.onRumble(large, small)
                }
            }
            else -> println("${executor.javaClass.name}: Rumble is not supported yet")
        }

        discovery.onResponded = { features ->
            receiver.setEnabledFeatures(features)
        }

        startConnectionMonitor()
    }

    private fun startConnectionMonitor() {
        viewModelScope.launch {
            while (true) {
                if (isReceiving && System.currentTimeMillis() - lastReceiveTime > TIMEOUT_MS) {
                    isReceiving = false
                    println("PadConnect disconnected. Restarting discovery server.")
                    discovery.start()
                }
                delay(500.milliseconds)
            }
        }
    }

    fun onEvent(buttons: Int, lx: Short, ly: Short, rx: Short, ry: Short, lt: Byte, rt: Byte) {
        _lastState.value = GamepadState(buttons, lx, ly, rx, ry, lt, rt)
    }

    override fun onCleared() {
        receiver.stop()
        executor.shutdown()
    }
}
