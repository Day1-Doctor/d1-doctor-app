import { spawn, ChildProcess } from 'child_process';
import { resolve } from 'path';

let tauriDriver: ChildProcess;

export const config = {
  specs: ['./e2e/**/*.e2e.ts'],
  maxInstances: 1,
  capabilities: [
    {
      'tauri:options': {
        application: resolve(
          __dirname,
          'src-tauri/target/release/bundle/macos/Day1 Doctor.app/Contents/MacOS/d1-doctor-desktop'
        ),
      },
    } as any,
  ],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  onPrepare: () => {
    tauriDriver = spawn('tauri-driver', [], {
      stdio: [null, process.stdout, process.stderr],
    });
  },
  onComplete: () => {
    tauriDriver.kill();
  },

  hostname: '127.0.0.1',
  port: 4444,
};
