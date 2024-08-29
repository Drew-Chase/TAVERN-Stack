import {buildBackend, buildFrontend, cleanup, deploy, incrementVersion} from "./stacked.js";

publish();

async function publish() {
    console.log("Publishing...");
    cleanup();
    incrementVersion();
    buildFrontend();
    buildBackend();
    await deploy();
    cleanup();
}