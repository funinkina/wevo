import express from "express"
import { WebSocketServer } from "ws"
import whatsappService from "./services/whatsapp.js"
import WebSocketManager from "./websocket/manager.js"
import setupMessageRoutes from "./routes/messages.js"

const app = express()
const wss = new WebSocketServer({ port: 8787 })
app.use(express.json())

// Initialize WebSocket manager
const wsManager = new WebSocketManager(wss)

// Setup WhatsApp service to forward all events to WebSocket
whatsappService.onEvent((type, payload) => {
    wsManager.sendEvent(type, payload)
})

// Setup routes
setupMessageRoutes(app)

// Health check
app.get("/health", (req, res) => {
    res.json({
        status: "ok",
        connected: whatsappService.isConnected(),
        hasSocket: whatsappService.getSocket() !== null
    })
})

// Auth status endpoint
app.get("/auth/status", (req, res) => {
    res.json(whatsappService.getAuthStatus())
})

// Request QR code endpoint
app.post("/auth/request-qr", async (req, res) => {
    try {
        const authStatus = whatsappService.getAuthStatus()

        if (authStatus.isAuthenticated) {
            return res.json({
                success: false,
                message: "Already authenticated"
            })
        }

        if (authStatus.isConnecting) {
            // If already connecting and has QR, return it
            if (authStatus.hasQr) {
                return res.json({
                    success: true,
                    message: "Connection in progress",
                    qr: whatsappService.getCurrentQr()
                })
            }
            return res.json({
                success: true,
                message: "Connection in progress, waiting for QR..."
            })
        }

        // Start the connection process
        await whatsappService.initialize()

        res.json({
            success: true,
            message: "QR generation started. Listen to WebSocket for QR code."
        })
    } catch (error) {
        res.status(500).json({
            success: false,
            error: error.message
        })
    }
})

// Fetch contacts endpoint - DEPRECATED, use WebSocket events
app.get("/contacts", async (req, res) => {
    res.status(404).json({
        success: false,
        message: "This endpoint is deprecated. Contacts are sent via WebSocket events."
    })
})

// Fetch profile picture endpoint
app.get("/profile-picture", async (req, res) => {
    try {
        const { jid } = req.query

        if (!jid) {
            return res.status(400).json({
                success: false,
                error: "Missing jid parameter"
            })
        }

        const url = await whatsappService.getProfilePicture(jid)

        res.json({
            success: true,
            url: url
        })
    } catch (error) {
        res.status(500).json({
            success: false,
            error: error.message
        })
    }
})

// Start server
app.listen(3000, async () => {
    console.log("=".repeat(50))
    console.log("WhatsApp Backend Server Started")
    console.log("=".repeat(50))
    console.log("HTTP API:    http://localhost:3000")
    console.log("WebSocket:   ws://localhost:8787")
    console.log("Health:      http://localhost:3000/health")
    console.log("Auth Status: http://localhost:3000/auth/status")
    console.log("Request QR:  POST http://localhost:3000/auth/request-qr")
    console.log("=".repeat(50))

    // Check if we have auth credentials and auto-initialize
    const fs = await import('fs')
    const credsPath = './auth/creds.json'

    if (fs.existsSync(credsPath)) {
        console.log("Found existing credentials, auto-initializing WhatsApp connection...")
        console.log("=".repeat(50))
        try {
            await whatsappService.initialize()
        } catch (error) {
            console.error("Error during auto-initialization:", error.message)
            console.log("Waiting for manual QR code request...")
        }
    } else {
        console.log("No credentials found. Waiting for frontend to request QR code...")
        console.log("=".repeat(50))
    }
})

console.log("Server ready.")
