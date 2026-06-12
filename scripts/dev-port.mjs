import net from 'node:net'

const DEFAULT_PORT = 5173
const MAX_PORT = 5273

function canListen(host, port) {
  return new Promise((resolve) => {
    const server = net.createServer()

    server.once('error', () => {
      resolve(false)
    })

    server.once('listening', () => {
      server.close(() => resolve(true))
    })

    server.listen(port, host)
  })
}

export async function findAvailablePort({
  host = '0.0.0.0',
  startPort = DEFAULT_PORT,
  endPort = MAX_PORT
} = {}) {
  for (let port = startPort; port <= endPort; port += 1) {
    if (await canListen(host, port)) {
      return port
    }
  }

  throw new Error(`No available port found between ${startPort} and ${endPort}.`)
}

export { DEFAULT_PORT }
