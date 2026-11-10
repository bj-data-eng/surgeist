import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

const corpus = new URL('../', import.meta.url);
const crate = new URL('../../../', corpus);
const reportFile = new URL('flex_composition_script_custody_report.json', corpus);
const reportBytes = readFileSync(reportFile);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
assert.equal(hash(reportBytes), 'cd167f9ecd34d142423885080df4f839ccf94b81c5ba80eb70ea868770547024');

const controlFile = new URL('captured/flex-composition-differences/gutter-control.json', corpus);
const controlBytes = readFileSync(controlFile);
assert.equal(hash(controlBytes), 'edcff878101c01973ca991b95a8f7ada4dce5859f9074c63fc1afa91581a71b7');

const report = JSON.parse(reportBytes);
const source = new URL('html/flex/fri07_flex_composition.html', corpus);
const browser = new URL(report.metadata.browser_executable, crate);
const manifestBytes = readFileSync(new URL('corpus.toml', corpus));
assert.equal(hash(readFileSync(browser)), report.metadata.browser_executable_sha256);
for (const input of ['html/flex/fri07_flex_composition.html', 'scripts/gentest/test_base_style.css', 'scripts/gentest/test_helper.js']) {
    assert.equal(hash(readFileSync(new URL(input, corpus))), report.metadata.inputs[input]);
}
assert.equal(hash(manifestBytes), report.metadata.inputs['browser-owner.toml']);
const flags = [...manifestBytes.toString().match(/\narguments = \[([\s\S]*?)\]/)[1].matchAll(/"([^"]+)"/g)].map(match => '--' + match[1]);
const directory = mkdtempSync(resolve(tmpdir(), 'surgeist-flex-gutter-'));
const profile = directory + '/profile';
const lifetime = new AbortController();
const signal = AbortSignal.any([lifetime.signal, AbortSignal.timeout(25_000)]);
const interrupt = () => lifetime.abort(new Error('gutter diagnostic interrupted'));
process.once('SIGTERM', interrupt);
process.once('SIGINT', interrupt);

let browserProcess;
let exited;
let socket;
try {
    browserProcess = spawn(fileURLToPath(browser), [
        ...flags, '--remote-debugging-port=0', '--user-data-dir=' + profile, 'about:blank',
    ], { stdio: ['ignore', 'ignore', 'inherit'] });
    exited = new Promise(resolve => {
        browserProcess.once('exit', (code, exitSignal) => {
            resolve();
            lifetime.abort(new Error(`browser exited with ${code ?? exitSignal}`));
        });
        browserProcess.once('error', error => {
            resolve();
            lifetime.abort(error);
        });
    });
    let port;
    for (let i = 0; i < 100 && !port; ++i) {
        try { port = Number(readFileSync(profile + '/DevToolsActivePort', 'utf8').split('\n')[0]); } catch {}
        if (!port) await sleep(100, undefined, { signal });
    }
    assert.ok(port, 'browser must start a DevTools endpoint');
    const targets = await fetch('http://127.0.0.1:' + port + '/json/list', { signal }).then(response => response.json());
    signal.throwIfAborted();
    socket = new WebSocket(targets.find(target => target.type === 'page').webSocketDebuggerUrl);
    const failSocket = () => lifetime.abort(new Error('browser debugging connection closed'));
    socket.addEventListener('error', failSocket, { once: true });
    socket.addEventListener('close', failSocket, { once: true });
    await new Promise((resolve, reject) => {
        const failOpen = () => reject(signal.reason);
        signal.addEventListener('abort', failOpen, { once: true });
        socket.addEventListener('open', () => {
            signal.removeEventListener('abort', failOpen);
            resolve();
        }, { once: true });
    });
    let next = 0;
    const pending = new Map();
    signal.addEventListener('abort', () => {
        for (const continuation of pending.values()) continuation.reject(signal.reason);
        pending.clear();
    }, { once: true });
    socket.addEventListener('message', ({data}) => {
        let message;
        try {
            message = JSON.parse(data);
        } catch (error) {
            lifetime.abort(error);
            return;
        }
        const continuation = pending.get(message.id);
        if (continuation) {
            pending.delete(message.id);
            if (message.error) continuation.reject(message.error);
            else continuation.resolve(message.result);
        }
    });
    const call = (method, params = {}) => new Promise((resolve, reject) => {
        signal.throwIfAborted();
        const id = ++next;
        pending.set(id, {resolve,reject});
        socket.send(JSON.stringify({id,method,params}));
    });
    await call('Emulation.setDeviceMetricsOverride', {width:800,height:600,deviceScaleFactor:1,mobile:false});
    const navigation = await call('Page.navigate',{url:source.href});
    assert.equal(navigation.errorText,undefined);
    for (let i = 0; i < 100; ++i) {
        const page = await call('Runtime.evaluate',{expression:"location.href + '|' + document.readyState",returnByValue:true});
        if (page.result.value === source.href + '|complete') break;
        assert.ok(i < 99,'pinned source did not load');
        await sleep(100, undefined, { signal });
    }
    const evaluate = async () => {
        const root = document.getElementById('test-root');
        const [_, image, collapsed, scroller] = root.children;
        const child = scroller.firstElementChild;
        await image.decode();
        await document.fonts.ready;
        const body = document.body.className;
        const initialStyle = collapsed.getAttribute('style');
        const rows = [];
        for (const box of ['border-box','content-box']) {
            for (const direction of ['ltr','rtl']) {
                document.body.className = box + ' ' + direction;
                for (const visibility of ['collapse','visible']) {
                    collapsed.style.visibility = visibility;
                    const style = getComputedStyle(scroller);
                    const childStyle = getComputedStyle(child);
                    const outer = scroller.getBoundingClientRect(), inner = child.getBoundingClientRect();
                    rows.push({box,direction,visibility,computedCollapse:getComputedStyle(collapsed).visibility,
                        scroller: {width:outer.width,height:outer.height,clientLeft:scroller.clientLeft,
                            clientTop:scroller.clientTop,clientWidth:scroller.clientWidth,clientHeight:scroller.clientHeight,
                            scrollLeft:scroller.scrollLeft,scrollTop:scroller.scrollTop,
                            scrollWidth:scroller.scrollWidth,scrollHeight:scroller.scrollHeight,
                            writingMode:style.writingMode,direction:style.direction,borderTop:style.borderTopWidth,
                            overflowX:style.overflowX,overflowY:style.overflowY,alignItems:style.alignItems},
                        child:{x:inner.x-outer.x,y:inner.y-outer.y,width:inner.width,height:inner.height,
                            writingMode:childStyle.writingMode,direction:childStyle.direction,alignSelf:childStyle.alignSelf},
                    });
                }
            }
        }
        document.body.className = body;
        collapsed.setAttribute('style',initialStyle);
        return rows;
    };
    const payload = await call('Runtime.evaluate',{expression:'(' + evaluate.toString() + ')()',awaitPromise:true,returnByValue:true});
    assert.equal(payload.exceptionDetails, undefined);
    const rows = payload.result.value;
    assert.equal(rows.length,8);
    for (const row of rows) {
        const {scroller:port, child} = row;
        assert.deepEqual([port.width,port.height,port.clientLeft,port.clientTop,port.clientWidth,port.clientHeight],[70,60,0,0,70,45]);
        assert.deepEqual([port.scrollLeft,port.scrollTop,port.scrollWidth,port.scrollHeight],[0,0,120,45]);
        assert.deepEqual([child.x,child.y,child.width,child.height],[-50,0,120,45]);
        assert.equal(port.writingMode,'sideways-rl');
        assert.equal(port.direction,row.direction);
        assert.equal(row.computedCollapse,row.visibility);
    }
    const raw = JSON.stringify(rows, null, 2) + '\n';
    assert.equal(raw, controlBytes.toString(), 'browser gutter control changed');
    process.stdout.write(raw);
} finally {
    lifetime.abort();
    let stopped = !browserProcess;
    try {
        if (socket?.readyState === WebSocket.OPEN) socket.close();
        if (browserProcess) {
            const waitForExit = () => Promise.race([
                exited.then(() => true),
                sleep(1500, false, { ref: false }),
            ]);
            browserProcess.kill('SIGTERM');
            stopped = await waitForExit();
            if (!stopped) {
                browserProcess.kill('SIGKILL');
                stopped = await waitForExit();
            }
            assert.ok(stopped, 'browser did not stop after termination');
        }
    } finally {
        process.removeListener('SIGTERM', interrupt);
        process.removeListener('SIGINT', interrupt);
        if (stopped) rmSync(directory, { recursive: true });
    }
}
