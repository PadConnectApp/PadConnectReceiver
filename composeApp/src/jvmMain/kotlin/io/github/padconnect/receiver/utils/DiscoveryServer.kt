package io.github.padconnect.receiver.utils

import io.github.padconnect.receiver.dialogs.AlertDialogQueue
import io.github.padconnect.receiver.dialogs.AppDialog
import java.net.DatagramPacket
import java.net.DatagramSocket

const val MIN_SUPPORTED_VERSION = 2
const val FEATURE_RUMBLE = 1 shl 0
const val FEATURE_LATENCY = 1 shl 1

class DiscoveryServer(
    private val port: Int
) {

    private var socket: DatagramSocket? = null

    @Volatile private var isRunning = false

    private val SERVER_VERSION = 2
    private val SERVER_FEATURES = FEATURE_RUMBLE or FEATURE_LATENCY

    var onResponded: ((features: Int) -> Unit)? = null

    fun start() {
        if (isRunning) return
        isRunning = true

        socket = DatagramSocket(port)

        Thread {
            println("Discovery server listening on $port")
            val buffer = ByteArray(256)

            while (isRunning) {
                try {
                    val packet = DatagramPacket(buffer, buffer.size)
                    socket?.receive(packet)

                    val msg = String(packet.data, 0, packet.length)

                    if (msg.startsWith("PADCONNECT_DISCOVER")) {
                        val parts = msg.split(":")

                        val clientVersion = parts.getOrNull(1)?.toIntOrNull() ?: 1
                        val clientFeatures = parts.getOrNull(2)?.toIntOrNull() ?: 1

                        if (clientVersion < MIN_SUPPORTED_VERSION) {
                            AlertDialogQueue.show(
                                AppDialog.Message(
                                    title = "App Update Required",
                                    message = "Your PadConnect app is outdated.\n\nPlease update the app to connect to this receiver."
                                )
                            )
                        }

                        val agreedVersion = minOf(clientVersion, SERVER_VERSION)
                        val agreedFeatures = clientFeatures and SERVER_FEATURES

                        val response =
                            "PADCONNECT_HERE:8082:$agreedVersion:$agreedFeatures".toByteArray()

                        val responsePacket = DatagramPacket(
                            response,
                            response.size,
                            packet.address,
                            packet.port
                        )

                        socket?.send(responsePacket)
                        onResponded?.invoke(agreedFeatures)
                        println(
                            "Responded to ${packet.address.hostAddress} " +
                                    "v=$agreedVersion features=$agreedFeatures"
                        )
                    }
                } catch (e: Exception) {
                    if (isRunning) {
                        e.printStackTrace()
                    }
                }
            }
            println("Discovery server thread stopped.")
        }.apply {
            name = "discovery-server"
            start()
            println("Started $this")
        }
    }

    fun stop() {
        if (!isRunning) return
        isRunning = false
        socket?.close()
        socket = null
        println("Discovery server stopped on $port")
    }
}
