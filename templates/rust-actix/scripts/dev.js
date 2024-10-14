import {spawn} from "node:child_process";

let first_data = false;

// Spawn the process
let apiServerProcess = spawn('npm', ['run', 'watch-api'],
    {
        shell: true
    });

// Capture the standard output stream
apiServerProcess.stdout.on('data', (data) => {
    if (data && !first_data) first_data = true;
    process.stdout.write(data);
    apiServerProcess.stdout.on('data', (data) => {
        process.stdout.write(data);
        debug(data.toString());
    });
});

while(!first_data) {
    console.log('waiting for first data');
    // add a delay to allow the process to start
    await new Promise(resolve => setTimeout(resolve, 1000));
}

// Capture the standard error stream
apiServerProcess.stderr.on('data', (data) => {
    process.stderr.write(data);
    debug(data.toString());
});

// Handle process exit
apiServerProcess.on('close', (code) => {
    if (code !== 0) {
        console.error(`API server process exited with code ${code}`);
        process.exit(1);
    }
});

// Start the Vite server
let viteServerProcess = spawn('vite', ['.'],
    {
        shell: true
    });

// Handle process exit
viteServerProcess.on('close', (code) => {
    if (code !== 0) {
        console.error(`Vite server process exited with code ${code}`);
        process.exit(1);
    }
});

function debug(data) {
    const logData = data.toString();
    const logPattern = /\[(.*?)\s(\w+)\s+(.*?)\] (.*)/;
    const match = logData.match(logPattern);

    if (match) {
        const [, timestamp, logLevel, source, message] = match;
        let style = 'color: blue; font-weight: bold;';

        switch (logLevel) {
            case 'TRACE':
                style = 'color: gray;';
                break;
            case 'DEBUG':
                style = 'color: green;';
                break;
            case 'INFO':
                style = 'color: blue;';
                break;
            case 'WARN':
                style = 'color: orange;';
                break;
            case 'ERROR':
                style = 'color: red;';
                break;
        }

        console.log(`%c[${timestamp} ${logLevel} ${source}]%c ${message}`, style, 'color: white;');
    }
}