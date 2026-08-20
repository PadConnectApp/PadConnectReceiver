
package io.github.padconnect.receiver.utils

import java.net.InetSocketAddress
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.channels.DatagramChannel

class UdpReceiver(
    val port: Int,
    private val onEvent: (Int, Short, Short, Short, Short, Byte, Byte) -> Unit
) {
    val channel: DatagramChannel = DatagramChannel.open()

    @Volatile
    private var isLatencyFeatureEnabled = false
    @Volatile
    private var isRumbleFeatureEnabled = false

    @Volatile
    var currentSender: InetSocketAddress? = null

    fun start() {
        Thread {
            channel.configureBlocking(true)
            channel.bind(InetSocketAddress(port))

            val bb = ByteBuffer.allocateDirect(21).order(ByteOrder.LITTLE_ENDIAN)

            while (channel.isOpen) {
                bb.clear()
                val sender = channel.receive(bb) as? InetSocketAddress ?: continue
                bb.flip()

                val type = bb.get().toInt()

                if (type == 0) {
                    onEvent(
                        bb.short.toInt(), // buttons
                        bb.short,         // lx
                        bb.short,         // ly
                        bb.short,         // rx
                        bb.short,         // ry
                        bb.get(),         // lt
                        bb.get()          // rt
                    )

                    if (sender != currentSender) {
                        currentSender = sender
                    }

                    if (isLatencyFeatureEnabled) sendLatency(bb.long)
                }
            }
        }.apply {
            name = "udp-io"
            priority = Thread.MAX_PRIORITY
            start()
            println("Started $this")
        }
    }

    private val rumbleBuffer = ByteBuffer.allocateDirect(3)

    fun onRumble(large: Int, small: Int) {
        val targetAddress = currentSender
        if (targetAddress == null || !isRumbleFeatureEnabled) return

        synchronized(rumbleBuffer) {
            rumbleBuffer.clear()

            rumbleBuffer.put(1.toByte())
            rumbleBuffer.put(large.toByte())
            rumbleBuffer.put(small.toByte())

            rumbleBuffer.flip()

            channel.send(rumbleBuffer, targetAddress)
        }
    }

    private val responseBuffer = ByteBuffer.allocateDirect(17).order(ByteOrder.LITTLE_ENDIAN)

    fun sendLatency(sentTime: Long) {
        val targetAddress = currentSender ?: return
        responseBuffer.clear()

        responseBuffer.put(2)
        responseBuffer.putLong(sentTime)
        responseBuffer.putLong(System.nanoTime())

        responseBuffer.flip()

        channel.send(responseBuffer, targetAddress)
    }

    fun setEnabledFeatures(features: Int) {
        isRumbleFeatureEnabled = (features and FEATURE_RUMBLE) != 0
        isLatencyFeatureEnabled = (features and FEATURE_LATENCY) != 0
    }

    fun stop() {
        channel.close()
    }
}
