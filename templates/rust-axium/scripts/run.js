import {buildBackend, buildFrontend, cleanup} from "./app.js";
import {execSync} from "node:child_process";

run()

function run() {
    buildFrontend();
    buildBackend();
    execSync(`${process.cwd()}/dist/stacked.exe`, {cwd: `${process.cwd()}/dist`, stdio: "inherit"});
}