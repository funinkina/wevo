import makeWASocket, { useMultiFileAuthState, DisconnectReason, fetchLatestBaileysVersion, Browsers } from "@whiskeysockets/baileys"
import P from "pino"
import fs from "fs/promises"
import path from "path"
import qrcode from "qrcode"

class WhatsAppService {
    constructor() {
        this.sock = null
        this.eventCallback = null // Unified event callback
        this.isConnecting = false
        this.retryCount = 0
        this.maxRetries = 5
        this.currentQr = null
        this.isAuthenticated = false
    }

    onEvent(callback) {
        this.eventCallback = callback
    }

    async clearAuthAndDatabase() {
        console.log("Clearing auth folder...")
        try {
            const authPath = "./auth"
            const files = await fs.readdir(authPath)
            for (const file of files) {
                const filePath = path.join(authPath, file)
                await fs.unlink(filePath)
                console.log(`Deleted: ${filePath}`)
            }
            console.log("Auth folder cleared successfully")
        } catch (error) {
            console.error("Error clearing auth folder:", error)
        }
    }

    async initialize() {
        if (this.isConnecting) {
            console.log("Already connecting, skipping...")
            return
        }

        this.isConnecting = true
        this.currentQr = null

        try {
            const { state, saveCreds } = await useMultiFileAuthState("./auth")
            const { version, isLatest } = await fetchLatestBaileysVersion()

            console.log(`Using WA version ${version.join('.')}, isLatest: ${isLatest}`)

            this.sock = makeWASocket({
                version,
                logger: P({ level: "silent" }),
                printQRInTerminal: false,
                auth: state,
                connectTimeoutMs: 60000,
                defaultQueryTimeoutMs: 60000,
                keepAliveIntervalMs: 30000,
                markOnlineOnConnect: true,
                syncFullHistory: true,
                browser: Browsers.ubuntu("Linux"),
            })

            console.log("Socket created with syncFullHistory enabled")

            this.sock.ev.on("connection.update", async (update) => {
                const { qr, connection, lastDisconnect } = update
                if (qr) {
                    this.currentQr = qr
                }

                if (this.eventCallback) {
                    // Send QR as a data URL
                    if (update.qr) {
                        try {
                            const qrImage = await qrcode.toDataURL(update.qr)
                            this.eventCallback("connection.update", { ...update, qr: qrImage })
                        } catch (e) {
                            console.error("Error converting QR to data URL", e)
                            this.eventCallback("connection.update", update)
                        }
                    } else {
                        this.eventCallback("connection.update", update)
                    }
                }

                if (connection === "open") {
                    console.log("Baileys connected successfully")
                    this.isConnecting = false
                    this.retryCount = 0
                    this.currentQr = null
                    this.isAuthenticated = true
                }

                if (connection === "close") {
                    this.isConnecting = false
                    this.isAuthenticated = false
                    const statusCode = lastDisconnect?.error?.output?.statusCode
                    const reason = lastDisconnect?.error?.output?.payload?.error || 'Unknown'

                    console.log(`Connection closed. Status: ${statusCode}, Reason: ${reason}`)

                    if (statusCode === DisconnectReason.loggedOut) {
                        console.log("Logged out. Clearing auth and re-initializing.")
                        await this.clearAuthAndDatabase()
                        this.retryCount = 0
                        this.currentQr = null
                        setTimeout(() => this.initialize(), 1000)
                    } else {
                        const shouldReconnect = statusCode !== DisconnectReason.connectionClosed
                        if (shouldReconnect && this.retryCount < this.maxRetries) {
                            this.retryCount++
                            const delay = Math.min(3000 * this.retryCount, 15000)
                            console.log(`Attempting reconnect ${this.retryCount}/${this.maxRetries} in ${delay / 1000}s...`)
                            setTimeout(() => this.initialize(), delay)
                        } else if (this.retryCount >= this.maxRetries) {
                            console.log("Max retries reached. Please restart the server.")
                        }
                    }
                }

                if (connection === "connecting") {
                    console.log("🔌 Connecting to WhatsApp...")
                }
            })

            // Forward all relevant events to the frontend
            const eventsToForward = [
                "messages.upsert", "messages.update", "messages.delete",
                "chats.upsert", "chats.update", "chats.delete",
                "contacts.set", "contacts.upsert", "contacts.update",
                "presence.update",
                "groups.upsert", "groups.update", "group-participants.update",
                "messaging-history.set"
            ]

            for (const event of eventsToForward) {
                this.sock.ev.on(event, (payload) => {
                    // Log when event is received from Baileys
                    console.log(`\n📥 Received event from Baileys: ${event}`)

                    if (this.eventCallback) {
                        this.eventCallback(event, payload)
                    }
                })
            }

            this.sock.ev.on("creds.update", saveCreds)

        } catch (error) {
            console.error("Error initializing WhatsApp:", error)
            this.isConnecting = false

            if (this.retryCount < this.maxRetries) {
                this.retryCount++
                const delay = Math.min(3000 * this.retryCount, 15000)
                console.log(`Retrying initialization in ${delay / 1000}s...`)
                setTimeout(() => this.initialize(), delay)
            }
        }
    }

    getAuthStatus() {
        return {
            isAuthenticated: this.isAuthenticated,
            isConnecting: this.isConnecting,
            hasQr: !!this.currentQr,
        }
    }

    getCurrentQr() {
        return this.currentQr
    }

    isConnected() {
        return this.isAuthenticated
    }

    getSocket() {
        return this.sock
    }

    async getProfilePicture(jid) {
        if (!this.sock) return null
        try {
            const url = await this.sock.profilePictureUrl(jid, "image")
            return url
        } catch (error) {
            console.error(`Error fetching profile picture for ${jid}:`, error)
            return null
        }
    }

    async sendMessage(jid, content) {
        if (!this.sock) throw new Error("WhatsApp not connected")
        return await this.sock.sendMessage(jid, content)
    }
}

export default new WhatsAppService()