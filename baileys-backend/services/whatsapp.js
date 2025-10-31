import makeWASocket, { useMultiFileAuthState, DisconnectReason, fetchLatestBaileysVersion, Browsers } from "@whiskeysockets/baileys"
import P from "pino"
import fs from "fs/promises"
import path from "path"

class WhatsAppService {
    constructor() {
        this.sock = null
        this.chatsCache = new Map() // Manual cache for chats
        this.qrCallback = null
        this.connectionCallback = null
        this.messageCallback = null
        this.contactsCallback = null
        this.isConnecting = false
        this.retryCount = 0
        this.maxRetries = 5
        this.currentQr = null
        this.isAuthenticated = false
    }

    async clearAuthAndDatabase() {
        console.log("Clearing auth folder and database...")

        try {
            // Clear auth folder
            const authPath = "./auth"
            const files = await fs.readdir(authPath)

            for (const file of files) {
                const filePath = path.join(authPath, file)
                await fs.unlink(filePath)
                console.log(`Deleted: ${filePath}`)
            }

            // Clear database
            const dbPath = "./db/client.db"
            try {
                await fs.unlink(dbPath)
                console.log(`Deleted: ${dbPath}`)
            } catch (err) {
                if (err.code !== 'ENOENT') {
                    console.warn(`Could not delete database: ${err.message}`)
                }
            }

            console.log("Auth and database cleared successfully")
        } catch (error) {
            console.error("Error clearing auth and database:", error)
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
                // Add connection options to prevent frequent disconnects
                connectTimeoutMs: 60000,
                defaultQueryTimeoutMs: 60000,
                keepAliveIntervalMs: 30000,
                markOnlineOnConnect: true,
                // Enable full history sync to get all past conversations
                syncFullHistory: true,
                // Emulate desktop browser to receive full chat history
                browser: Browsers.ubuntu("Linux"),
            })

            console.log("Socket created with syncFullHistory enabled")

            this.sock.ev.on("connection.update", async ({ qr, connection, lastDisconnect }) => {
                if (qr) {
                    console.log("QR code generated")
                    this.currentQr = qr
                    if (this.qrCallback) {
                        this.qrCallback(qr)
                    }
                }

                if (connection === "open") {
                    console.log("Baileys connected successfully")
                    this.isConnecting = false
                    this.retryCount = 0
                    this.currentQr = null
                    this.isAuthenticated = true

                    if (this.connectionCallback) {
                        this.connectionCallback()
                    }

                    // Wait for history sync events to populate cache, then send contacts
                    console.log("Waiting for history sync events...")

                    // Also try to send any cached contacts after a delay
                    setTimeout(() => {
                        console.log("Checking for cached contacts...")
                        if (this.chatsCache.size > 0) {
                            console.log(`Found ${this.chatsCache.size} chats in cache, sending to frontend`)
                            const contacts = this.getContactsFromCache()
                            if (contacts.length > 0 && this.contactsCallback) {
                                this.contactsCallback(contacts)
                            }
                        } else {
                            console.log("No chats in cache yet, waiting for chats.set event...")
                        }
                    }, 2000)

                    // Retry after longer delay if still no contacts
                    setTimeout(() => {
                        if (this.chatsCache.size > 0) {
                            const contacts = this.getContactsFromCache()
                            if (contacts.length > 0 && this.contactsCallback) {
                                console.log(`Retry: Sending ${contacts.length} contacts`)
                                this.contactsCallback(contacts)
                            }
                        }
                    }, 5000)
                }

                if (connection === "close") {
                    this.isConnecting = false
                    this.isAuthenticated = false
                    const statusCode = lastDisconnect?.error?.output?.statusCode
                    const reason = lastDisconnect?.error?.output?.payload?.error || 'Unknown'

                    console.log(`Connection closed. Status: ${statusCode}, Reason: ${reason}`)

                    // Check if it's a 401 Unauthorized error
                    if (statusCode === 401) {
                        console.log("Unauthorized (401) - Clearing auth and database...")
                        await this.clearAuthAndDatabase()
                        console.log("Please scan QR code again.")
                        this.retryCount = 0
                        this.currentQr = null
                        // Reinitialize to generate new QR
                        setTimeout(() => this.initialize(), 1000)
                        return
                    }

                    const shouldReconnect = statusCode !== DisconnectReason.loggedOut

                    if (statusCode === DisconnectReason.loggedOut) {
                        console.log("Logged out. Please scan QR code again.")
                        this.retryCount = 0
                        this.currentQr = null
                    } else if (shouldReconnect && this.retryCount < this.maxRetries) {
                        this.retryCount++
                        const delay = Math.min(3000 * this.retryCount, 15000)
                        console.log(`Attempting reconnect ${this.retryCount}/${this.maxRetries} in ${delay / 1000}s...`)

                        setTimeout(() => this.initialize(), delay)
                    } else if (this.retryCount >= this.maxRetries) {
                        console.log("Max retries reached. Please restart the server.")
                        this.retryCount = 0
                    }
                }

                if (connection === "connecting") {
                    console.log("🔌 Connecting to WhatsApp...")
                }
            })

            this.sock.ev.on("messages.upsert", (m) => {
                const msg = m.messages[0]
                if (msg && this.messageCallback) {
                    console.log("Message received from:", msg.key.remoteJid)
                    this.messageCallback(msg)
                }
            })

            this.sock.ev.on("creds.update", saveCreds)

            // Listen for chat history events - this is the key event for syncFullHistory
            this.sock.ev.on("chats.set", ({ chats }) => {
                console.log(`[chats.set] Received ${chats.length} chats from history sync`)
                console.log(`[chats.set] Sample data:`, JSON.stringify(chats.slice(0, 2), null, 2))

                // Cache all chats
                chats.forEach(chat => {
                    if (chat.id) {
                        this.chatsCache.set(chat.id, chat)
                    }
                })
                console.log(`Cached ${this.chatsCache.size} chats`)

                // Process and send contacts immediately
                this.processChats(chats)
            })

            this.sock.ev.on("chats.upsert", (chats) => {
                console.log(`[chats.upsert] Received ${chats.length} new/updated chats`)
                console.log(`[chats.upsert] Sample data:`, JSON.stringify(chats.slice(0, 2), null, 2))

                // Update cache
                chats.forEach(chat => {
                    if (chat.id) {
                        this.chatsCache.set(chat.id, chat)
                    }
                })

                this.processChats(chats)
            })

            this.sock.ev.on("chats.update", (updates) => {
                console.log(`[chats.update] Chat updates received: ${updates.length}`)
                console.log(`[chats.update] Sample data:`, JSON.stringify(updates.slice(0, 2), null, 2))

                // Update cache with these chats
                updates.forEach(chat => {
                    if (chat.id) {
                        // Merge with existing or add new
                        const existing = this.chatsCache.get(chat.id)
                        this.chatsCache.set(chat.id, { ...existing, ...chat })
                    }
                })

                console.log(`Cache now has ${this.chatsCache.size} chats`)

                // If we have enough chats cached, send them to frontend
                if (this.chatsCache.size > 0) {
                    console.log(`📤 Sending cached contacts to frontend...`)
                    const contacts = this.getContactsFromCache()
                    if (contacts.length > 0 && this.contactsCallback) {
                        this.contactsCallback(contacts)
                    }
                }
            })

            // Listen for contacts events
            this.sock.ev.on("contacts.set", ({ contacts }) => {
                console.log(`[contacts.set] Received ${contacts.length} contacts`)
                console.log(`[contacts.set] Sample data:`, JSON.stringify(contacts.slice(0, 2), null, 2))
                this.processContactsSet(contacts)
            })

            this.sock.ev.on("contacts.upsert", (contacts) => {
                console.log(`[contacts.upsert] Received ${contacts.length} contacts`)
                console.log(`[contacts.upsert] Sample data:`, JSON.stringify(contacts.slice(0, 2), null, 2))
            })

            this.sock.ev.on("contacts.update", (updates) => {
                console.log(`[contacts.update] Contact updates received: ${updates.length}`)
            })

            // Listen for messaging history sync
            this.sock.ev.on("messaging-history.set", ({ chats, contacts, messages, isLatest }) => {
                console.log(`[messaging-history.set] History sync event`)
                console.log(`Chats: ${chats.length}, Contacts: ${contacts.length}, Messages: ${messages.length}, IsLatest: ${isLatest}`)

                if (chats.length > 0) {
                    console.log(`Processing ${chats.length} chats from history...`)
                    this.processChats(chats)
                }

                if (contacts.length > 0) {
                    console.log(`Processing ${contacts.length} contacts from history...`)
                    this.processContactsSet(contacts)
                }
            })

            // Handle connection errors
            this.sock.ev.on("connection.error", (error) => {
                console.error("Connection error:", error)
                this.isConnecting = false
            })

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

    getContactsFromCache() {
        try {
            const chats = Array.from(this.chatsCache.values())

            if (chats.length === 0) {
                return []
            }

            const contactList = chats
                .filter(chat => chat.id && !chat.id.includes("@broadcast") && chat.id !== "status@broadcast")
                .map(chat => {
                    const isGroup = chat.id.includes("@g.us")
                    return {
                        jid: chat.id,
                        name: chat.name || chat.notify || chat.verifiedName || chat.id.split('@')[0],
                        unreadCount: chat.unreadCount || 0,
                        conversationTimestamp: chat.conversationTimestamp || 0,
                        isGroup: isGroup,
                        archived: chat.archived || false,
                        pinned: chat.pinned || 0,
                        muteEndTime: chat.muteEndTime || 0,
                    }
                })
                .sort((a, b) => b.conversationTimestamp - a.conversationTimestamp)

            console.log(`[getContactsFromCache] Returning ${contactList.length} contacts`)
            return contactList
        } catch (err) {
            console.error("Error getting contacts from cache:", err)
            return []
        }
    }

    processChats(chats) {
        try {
            const contactList = chats
                .filter(chat => {
                    // Filter out broadcasts and include both individual chats and groups
                    return chat.id && !chat.id.includes("@broadcast")
                })
                .map(chat => {
                    const isGroup = chat.id.includes("@g.us")
                    return {
                        jid: chat.id,
                        name: chat.name || chat.notify || chat.id.split('@')[0],
                        unreadCount: chat.unreadCount || 0,
                        conversationTimestamp: chat.conversationTimestamp || 0,
                        isGroup: isGroup,
                        // Additional metadata
                        archived: chat.archived || false,
                        pinned: chat.pinned || 0,
                        muteEndTime: chat.muteEndTime || 0,
                    }
                })
                .sort((a, b) => b.conversationTimestamp - a.conversationTimestamp) // Sort by most recent

            console.log(`[processChats] Processed ${contactList.length} chats`)
            console.log(`   Groups: ${contactList.filter(c => c.isGroup).length}, Individual: ${contactList.filter(c => !c.isGroup).length}`)

            if (contactList.length > 0) {
                console.log(`   Sample contacts:`, JSON.stringify(contactList.slice(0, 3).map(c => ({
                    jid: c.jid,
                    name: c.name,
                    isGroup: c.isGroup
                })), null, 2))
            }

            if (this.contactsCallback && contactList.length > 0) {
                console.log(`Sending ${contactList.length} contacts via callback`)
                this.contactsCallback(contactList)
            } else if (contactList.length === 0) {
                console.log(`No contacts to send`)
            }

            return contactList
        } catch (err) {
            console.error("Error processing chats:", err)
            return []
        }
    }

    processContactsSet(contacts) {
        try {
            console.log(`[processContactsSet] Processing ${contacts.length} contacts`)

            // Map contacts to our format
            const contactList = contacts
                .filter(contact => contact.id && !contact.id.includes("@broadcast"))
                .map(contact => ({
                    jid: contact.id,
                    name: contact.name || contact.notify || contact.verifiedName || contact.id.split('@')[0],
                    unreadCount: 0, // contacts.set doesn't include unread count
                    conversationTimestamp: 0,
                    isGroup: contact.id.includes("@g.us"),
                    archived: false,
                    pinned: 0,
                    muteEndTime: 0,
                }))

            console.log(`[processContactsSet] Processed ${contactList.length} contacts`)

            if (contactList.length > 0) {
                console.log(`   Sample contacts:`, JSON.stringify(contactList.slice(0, 3).map(c => ({
                    jid: c.jid,
                    name: c.name
                })), null, 2))

                if (this.contactsCallback) {
                    console.log(`Sending ${contactList.length} contacts via callback`)
                    this.contactsCallback(contactList)
                }
            }

            return contactList
        } catch (err) {
            console.error("Error processing contacts:", err)
            return []
        }
    }

    async fetchContacts() {
        try {
            console.log("Fetching contacts from cache...")

            if (!this.sock) {
                console.log("Socket not available")
                return []
            }

            // Get all chats from cache
            const chats = Array.from(this.chatsCache.values())

            if (chats.length === 0) {
                console.log("No chats found in cache yet. History sync may still be in progress.")
                return []
            }

            const contactList = chats
                .filter(chat => chat.id && !chat.id.includes("@broadcast") && chat.id !== "status@broadcast")
                .map(chat => {
                    const isGroup = chat.id.includes("@g.us")
                    return {
                        jid: chat.id,
                        name: chat.name || chat.notify || chat.verifiedName || chat.id.split('@')[0],
                        unreadCount: chat.unreadCount || 0,
                        conversationTimestamp: chat.conversationTimestamp || 0,
                        isGroup: isGroup,
                        archived: chat.archived || false,
                        pinned: chat.pinned || 0,
                        muteEndTime: chat.muteEndTime || 0,
                    }
                })
                .sort((a, b) => b.conversationTimestamp - a.conversationTimestamp)

            console.log(`Fetched ${contactList.length} contacts/chats from cache`)

            if (this.contactsCallback) {
                this.contactsCallback(contactList)
            }

            return contactList
        } catch (err) {
            console.error("Error fetching contacts:", err)
            return []
        }
    }

    async sendMessage(jid, text) {
        if (!this.sock) {
            throw new Error("WhatsApp not connected")
        }

        console.log(`Sending message to ${jid}`)
        const result = await this.sock.sendMessage(jid, { text })
        console.log("Message sent successfully")
        return result
    }

    async getProfilePicture(jid) {
        if (!this.sock) {
            throw new Error("WhatsApp not connected")
        }

        try {
            console.log(`Fetching profile picture for ${jid}`)
            // For high res picture, use 'image' as second parameter
            const ppUrl = await this.sock.profilePictureUrl(jid, 'image')
            console.log(`Profile picture URL fetched: ${ppUrl}`)
            return ppUrl
        } catch (error) {
            // If profile picture doesn't exist, Baileys throws an error
            console.log(`No profile picture found for ${jid}`)
            return null
        }
    }

    onQr(callback) {
        this.qrCallback = callback
    }

    onConnection(callback) {
        this.connectionCallback = callback
    }

    onMessage(callback) {
        this.messageCallback = callback
    }

    onContacts(callback) {
        this.contactsCallback = callback
    }

    getSocket() {
        return this.sock
    }

    isConnected() {
        return this.sock !== null && !this.isConnecting
    }

    getCurrentQr() {
        return this.currentQr
    }

    getAuthStatus() {
        return {
            isAuthenticated: this.isAuthenticated,
            isConnecting: this.isConnecting,
            hasQr: this.currentQr !== null
        }
    }
}

export default new WhatsAppService()
