import whatsappService from "../services/whatsapp.js"

export default function setupMessageRoutes(app) {
    // Send a message
    app.post("/send", async (req, res) => {
        const { jid, text } = req.body

        if (!jid || !text) {
            return res.status(400).json({
                ok: false,
                error: "Missing jid or text"
            })
        }

        try {
            await whatsappService.sendMessage(jid, text)
            res.json({ ok: true })
        } catch (err) {
            res.status(500).json({
                ok: false,
                error: err.toString()
            })
        }
    })

    // Get contacts
    app.get("/contacts", async (req, res) => {
        try {
            const contacts = await whatsappService.fetchContacts()
            res.json(contacts)
        } catch (err) {
            res.status(500).json({
                error: err.toString()
            })
        }
    })

    // Get messages for a specific contact/chat
    app.get("/messages/:jid", async (req, res) => {
        const { jid } = req.params

        try {
            const sock = whatsappService.getSocket()
            if (!sock) {
                return res.status(503).json({
                    error: "WhatsApp not connected"
                })
            }

            // Try to fetch message history
            // Note: Baileys has limited message history access
            const messages = []
            res.json(messages)
        } catch (err) {
            res.status(500).json({
                error: err.toString()
            })
        }
    })
}
