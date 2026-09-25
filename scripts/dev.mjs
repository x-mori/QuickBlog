import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { mkdir } from 'node:fs/promises';
import { createServer, connect } from 'node:net';
import { loadEnvFile } from 'node:process';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const localDir = join(root, '.local');
const dataDir = join(localDir, 'postgres');
const toolsDir = join(localDir, 'tools');
const executable = process.platform === 'win32' ? '.exe' : '';

loadEnvFile(join(root, '.env'));

let postgresProcess;
let apiProcess;
let trunkProcess;
let pgCtl;
let stopping = false;

function available(command) {
  return spawnSync(command, ['--version'], { stdio: 'ignore', windowsHide: true }).status === 0;
}

function postgresTool(name) {
  if (process.env.PG_BIN) {
    const command = join(process.env.PG_BIN, name + executable);
    if (available(command)) return command;
  }
  if (available(name)) return name;
  if (process.platform === 'win32') {
    const base = 'C:\\Program Files\\PostgreSQL';
    if (existsSync(base)) {
      const versions = readdirSync(base).sort((a, b) => Number(b) - Number(a));
      for (const version of versions) {
        const command = join(base, version, 'bin', name + executable);
        if (available(command)) return command;
      }
    }
  }
  throw new Error('PostgreSQL tool ' + name + ' was not found. Install PostgreSQL or set PG_BIN to its bin directory.');
}

function run(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: options.cwd ?? root,
      env: options.env ?? process.env,
      stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
      windowsHide: true,
    });
    let stdout = '';
    let stderr = '';
    if (options.capture) {
      child.stdout.setEncoding('utf8').on('data', chunk => { stdout += chunk; });
      child.stderr.setEncoding('utf8').on('data', chunk => { stderr += chunk; });
    }
    child.on('error', reject);
    child.on('exit', code => {
      if (code === 0 || options.allowFailure) resolve({ code, stdout, stderr });
      else reject(new Error(command + ' exited with code ' + code + (stderr ? ': ' + stderr.trim() : '')));
    });
  });
}

async function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const port = server.address().port;
      server.close(() => resolve(port));
    });
  });
}

async function portOpen(port) {
  return new Promise(resolve => {
    const socket = connect({ host: '127.0.0.1', port });
    socket.once('connect', () => { socket.destroy(); resolve(true); });
    socket.once('error', () => resolve(false));
    socket.setTimeout(500, () => { socket.destroy(); resolve(false); });
  });
}

async function waitFor(check, label, child, timeoutMs = 60000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await check()) return;
    if (child.exitCode !== null) throw new Error(label + ' stopped before it became ready.');
    await new Promise(resolve => setTimeout(resolve, 300));
  }
  throw new Error(label + ' did not become ready.');
}

async function ensureTrunk() {
  const localTrunk = join(toolsDir, 'bin', 'trunk' + executable);
  if (existsSync(localTrunk)) return localTrunk;
  if (available('trunk')) return 'trunk';
  console.log('Installing Trunk into .local/tools. This may take several minutes on the first run.');
  await mkdir(toolsDir, { recursive: true });
  await run('cargo', ['install', '--locked', '--root', toolsDir, 'trunk']);
  return localTrunk;
}

async function startDatabase() {
  const initdb = postgresTool('initdb');
  const postgres = postgresTool('postgres');
  const pgIsReady = postgresTool('pg_isready');
  const psql = postgresTool('psql');
  const createdb = postgresTool('createdb');
  pgCtl = postgresTool('pg_ctl');

  await mkdir(localDir, { recursive: true });
  if (!existsSync(join(dataDir, 'PG_VERSION'))) {
    console.log('Initializing local PostgreSQL data in .local/postgres...');
    await run(initdb, ['-D', dataDir, '-U', 'quickblog', '-A', 'trust', '--encoding=UTF8', '--no-instructions']);
  }

  const port = await freePort();
  postgresProcess = spawn(postgres, ['-D', dataDir, '-h', '127.0.0.1', '-p', String(port)], {
    cwd: root,
    stdio: 'inherit',
    windowsHide: true,
  });
  postgresProcess.on('error', error => { console.error(error); void stop(1); });
  await waitFor(async () => {
    const result = await run(pgIsReady, ['-h', '127.0.0.1', '-p', String(port), '-U', 'quickblog', '-d', 'postgres'], { capture: true, allowFailure: true });
    return result.code === 0;
  }, 'PostgreSQL', postgresProcess, 30000);

  const connection = ['-h', '127.0.0.1', '-p', String(port), '-U', 'quickblog'];
  const exists = await run(psql, [...connection, '-d', 'postgres', '-tAc',
    "SELECT 1 FROM pg_database WHERE datname = 'quickblog'"], { capture: true });
  if (exists.stdout.trim() !== '1') {
    await run(createdb, [...connection, 'quickblog']);
  }
  return 'postgres://quickblog@127.0.0.1:' + port + '/quickblog';
}

async function stop(code) {
  if (stopping) return;
  stopping = true;
  for (const child of [trunkProcess, apiProcess]) {
    if (!child?.pid) continue;
    if (process.platform === 'win32') {
      spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore', windowsHide: true });
    } else {
      child.kill('SIGTERM');
    }
  }
  if (postgresProcess && pgCtl) {
    await run(pgCtl, ['stop', '-D', dataDir, '-m', 'fast', '-w'], { capture: true, allowFailure: true });
  }
  process.exit(code);
}

process.on('SIGINT', () => { void stop(130); });
process.on('SIGTERM', () => { void stop(143); });

try {
  if (!process.env.JWT_SECRET || process.env.JWT_SECRET.length < 32) {
    throw new Error('Set JWT_SECRET to at least 32 characters in .env.');
  }
  const trunk = await ensureTrunk();
  const databaseUrl = process.env.DATABASE_URL || await startDatabase();
  const apiPort = Number(process.env.PORT || 3000);
  if (await portOpen(apiPort)) throw new Error('API port ' + apiPort + ' is already in use.');
  if (await portOpen(8080)) throw new Error('Frontend port 8080 is already in use.');

  apiProcess = spawn('cargo', ['run'], {
    cwd: root,
    env: { ...process.env, DATABASE_URL: databaseUrl },
    stdio: 'inherit',
    windowsHide: true,
  });
  apiProcess.on('error', error => { console.error(error); void stop(1); });
  apiProcess.on('exit', code => {
    if (!stopping) { console.error('API stopped with code ' + code); void stop(code || 1); }
  });
  console.log('Waiting for the Rust API...');
  await waitFor(() => portOpen(apiPort), 'Rust API', apiProcess, 120000);

  const trunkEnv = { ...process.env };
  if (trunkEnv.NO_COLOR) trunkEnv.NO_COLOR = 'true';
  else delete trunkEnv.NO_COLOR;
  trunkProcess = spawn(trunk, ['serve'], {
    cwd: join(root, 'client'),
    env: trunkEnv,
    stdio: 'inherit',
    windowsHide: true,
  });
  trunkProcess.on('error', error => { console.error(error); void stop(1); });
  trunkProcess.on('exit', code => {
    if (!stopping) { console.error('Trunk stopped with code ' + code); void stop(code || 1); }
  });
  console.log('Waiting for the frontend...');
  await waitFor(async () => {
    try { return (await fetch('http://127.0.0.1:8080/')).ok; }
    catch { return false; }
  }, 'Trunk frontend', trunkProcess, 180000);
  console.log('Open http://127.0.0.1:8080');
  if (!process.env.OPENAI_API_KEY) {
    console.log('OPENAI_API_KEY is empty. Add it to .env and restart before testing summaries.');
  }
} catch (error) {
  console.error(error.message);
  await stop(1);
}

