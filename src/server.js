import "dotenv/config"
import { WebSocketServer } from "ws"
import express from "express"
import expressBasicAuth from "express-basic-auth"

const log = (message) => console.log(`[${new Date().toLocaleTimeString()}] ${message}`)

const app = express()
const wss = new WebSocketServer({ noServer: true })
const port = process.env.PORT

// Setup
if (process.env_REVERSE_PROXY === "TRUE") {
	app.set("trust proxy", true)
}
if (!process.env.PORT) {
	log("No port set, please set the PORT environment variable")
	process.exit(1)
}
if (!process.env.DEFAULT_PASSWORD) {
	log("No default password set, please set the DEFAULT_PASSWORD environment variable")
	process.exit(1)
}

// Log IPs (prioritize X-Forwarded-For)
app.use((req, res, next) => {
	let ip = req.ip || req.headers["x-forwarded-for"] || req.connection.remoteAddress
	log(`Request from "${ip}": "${req.method} ${req.url}"`)
	next()
})

app.use(expressBasicAuth({
	// TODO: Implement a better authentication system, not sure what yet though
    users: { "admin": process.env.DEFAULT_PASSWORD },
	challenge: true,
}))

app.use(express.static("./web"))

// This function is used to strip the "ws" and "req" properties from the clients array, so that we can send them over the network.
// Telemasters don't need them anyway.
const sanitizeClientArray = (clients) => {
	return clients.map((client) => {
		return {
			id: client.id,
			ip: client.ip,
			joined: client.joined,
			type: client.type,
			status: client.status
		}
	})
}

let ids = 0
let clients = []
let options = {
	text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum",
	speed: 0.1,
	font: "Arial",
	fontSize: 48,
	fontColor: "white",
	backgroundColor: "black",

	mirrored: false,
	reversed: false,
	
	playing: false,
	percent: 0
}

wss.on("connection", (ws, req) => {
	let ip = req.ip || req.headers["x-forwarded-for"] || req.connection.remoteAddress
	let joined = new Date()
	let type = undefined

	// Validate
	if (ws.protocol !== "teleprompter" && ws.protocol !== "telemaster") {
		ws.close(1002, "Invalid protocol")
		log(`Disconnected ${ip} for: "invalid protocol (${ws.protocol})"`)
		return
	}

	// Setup

	type = ws.protocol

	log(`Connected ${ip} as a ${type}`)
	clients.push({
		id: ids++,
		ws: ws,
		req: req,
		ip: ip,
		joined: joined,
		type: type,
	})

	// Tell all telemasters about the new client
	clients.filter((client) => client.type === "telemaster").forEach((client) => {
		client.ws.send(JSON.stringify({
			type: "replaceClients",
			clients: sanitizeClientArray(clients)
		}))
	})

	// Send options
	ws.send(JSON.stringify({
		type: "replaceOptions",
		options: options
	}))

	ws.on("error", console.error)

	ws.on("close", (code, reason) => {
		log(`Disconnected ${ip} Code: ${code}, Reason: "${reason}"`)
		clients = clients.filter((client) => client.ws !== ws)

		// Tell all telemasters about the leaving client
		clients.filter((client) => client.type === "telemaster").forEach((client) => {
			client.ws.send(JSON.stringify({
				type: "replaceClients",
				clients: sanitizeClientArray(clients)
			}))
		})
	})

	// Events
	
	ws.on("message", (data) => {
		try { JSON.parse(data) } catch (err) {
			log(`Received invalid JSON from ${ip}: "${data}"`)
			return
		}

		let json = JSON.parse(data)

		log(`Message from ${ip}: "${data}"`)

		if (json.type === "updateOption") {
			options[json.key] = json.value
			clients.forEach((client) => {
				// Broadcast to everyone but the sender

				if (client.ws !== ws) {
					client.ws.send(JSON.stringify({
						type: "updateOption",
						key: json.key,
						value: json.value
					}))
				}
			})
		} else if (json.type === "updatePercent") {
			options.percent = json.value
			clients.forEach((client) => {
				// Broadcast to all telemasters

				if (client.type === "telemaster") {
					client.ws.send(JSON.stringify({
						type: "updateOption",
						key: "percent",
						value: json.value
					}))
				}
			})
		}
	})
})

const server = app.listen(port, () => {
	console.log(`Ignite teleprompter server listening on port ${port}`)
})

// Support handling websockets through Express, so we don't have to listen on a separate port
server.on("upgrade", (req, socket, head) => {
	wss.handleUpgrade(req, socket, head, (ws) => {
		wss.emit("connection", ws, req)
	})
})