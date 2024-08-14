import {Client} from 'basic-ftp'
import {lstatSync, readdirSync, readFileSync, writeFileSync} from 'fs'

console.log("Uploading files")
const client = new Client()
client.ftp.verbose = true

const includePaths = [
    "dist:/",
    // "api:/api",
    // "nginx.conf:/nginx.conf",
]

try {
    await client.access({
        host: "",
        user: "",
        password: ""
    })

    const jsFiles = readdirSync("dist", {recursive: true}).filter(i => i.endsWith(".js"))

    for (const file of jsFiles) {
        const path = `dist/${file}`
        const data = readFileSync(path, "utf8")
        const newData = data.replace(/http:\/\/example.local\//g, "/")
        writeFileSync(path, newData, "utf8")
    }

    for (const include of includePaths) {
        const [path, remotePath] = include.split(":")
        const isDirectory = lstatSync(path).isDirectory();
        if (isDirectory) {
            try {
                if (remotePath !== "/")
                    await client.removeDir(remotePath)
            } catch (err) {
                console.error(err)
            }
            try {
                await client.uploadFromDir(path, remotePath)
            } catch (err) {
                console.error(err)
            }
        } else {
            try {
                await client.remove(remotePath)
            } catch (err) {
                console.error(err)
            }
            try {
                await client.uploadFrom(path, remotePath)
            } catch (err) {
                console.error(err)
            }
        }
    }

} catch (err) {
    console.log(err)
}
client.close()