#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""The existing gcx-local JSON test helper, with scoped UI/deadline guards.

Run only inside an admitted android-device transaction. This does not grant a
window, select a device, change shared tooling, or implement the gcx application.
"""
import json
import os
from pathlib import Path
import re
import select
import subprocess
import struct
import sys
import time
import xml.etree.ElementTree as ET

PACKAGE = "com.machinerhythm.gcx"
ACTIVITY = PACKAGE + "/.GcxActivity"


class Refused(RuntimeError):
    pass


class Budget:
    def __init__(self, now, outer_end, helper_seconds=1020, cleanup_seconds=300,
                 return_seconds=60):
        self.hard_end = min(now + helper_seconds, outer_end)
        self.cleanup_at = self.hard_end - cleanup_seconds
        self.commands_end = self.hard_end - return_seconds
        self.forced_cleanup = False
        if self.cleanup_at <= now:
            raise Refused("insufficient admitted cleanup reserve")

    def cleanup(self, now):
        return self.forced_cleanup or now >= self.cleanup_at

    def admit(self, now, cleanup, seconds):
        if now >= self.commands_end:
            raise Refused("command window ended; return reserve only")
        if not cleanup and (self.cleanup(now) or now + seconds > self.cleanup_at):
            self.forced_cleanup = True
            raise Refused("CLEANUP_REQUIRED; no new tests or owner waiting")

    def owner_wait(self, now, seconds):
        if self.cleanup(now):
            raise Refused("CLEANUP_REQUIRED; owner feedback cannot extend the window")
        return max(0, min(15, seconds, self.cleanup_at - now))


def component(text):
    matches = re.findall(r"\b([\w.]+)/([\w.$]+)", text)
    normalized = {(p, p + a if a.startswith(".") else a) for p, a in matches}
    if len(normalized) != 1:
        raise Refused("missing or ambiguous current component")
    return normalized.pop()


def focused_pair(resumed, windows):
    # Only the current top-focused display is relevant. A full window dump can
    # contain historical ANR focus records before the live display section.
    top = set(re.findall(r"mTopFocusedDisplayId=(\d+)", windows))
    sections = re.split(r"Display: mDisplayId=(\d+)[^\n]*", windows)
    ids = sections[1::2]
    # Android13's `window displays` omits the global top-display field. A
    # single live display is unambiguous; several require its explicit ID.
    if not top and len(ids) == 1:
        top.add(ids[0])
    if len(top) != 1:
        raise Refused("current top-focused display missing or ambiguous")
    display = top.pop()
    current = [sections[i + 1] for i in range(1, len(sections), 2)
               if sections[i] == display]
    if len(current) != 1:
        raise Refused("current focused display section missing or ambiguous")
    focus = re.findall(r"mCurrentFocus=([^\n]+)", current[0])
    if len(focus) != 1:
        raise Refused("focused-window observation missing or ambiguous")
    activities = re.split(r"Display #(\d+)[^\n]*", resumed)
    active = [activities[i + 1] for i in range(1, len(activities), 2)
              if activities[i] == display]
    if len(active) != 1:
        raise Refused("resumed Activity display missing or ambiguous")
    activity = component(active[0])
    window = component(focus[0])
    if window != activity:
        raise Refused("resumed Activity and focused window do not match")
    frames = tuple(re.findall(r"DisplayFrames w=(\d+) h=(\d+) r=(\d+)", current[0]))
    return activity, (display, focus[0].strip(), frames)


def own_focus(resumed, windows):
    activity, window = focused_pair(resumed, windows)
    if activity != (PACKAGE, PACKAGE + ".GcxActivity"):
        raise Refused("expected gcx Activity AND focused window do not match")
    return activity, window


def button_center(xml, text):
    tree = ET.fromstring(xml)
    roots = list(tree)
    if not roots:
        raise Refused("no current UI root")
    def bounds(node):
        numbers = re.findall(r"\d+", node.attrib.get("bounds", ""))
        if len(numbers) != 4:
            raise Refused("current UI bounds missing")
        return tuple(map(int, numbers))
    root = bounds(roots[0])
    matches = [n for n in tree.iter("node") if n.attrib.get("text") == text
               and n.attrib.get("package") == PACKAGE
               and n.attrib.get("class") == "android.widget.Button"
               and n.attrib.get("enabled") == "true"
               and n.attrib.get("clickable") == "true"]
    if len(matches) != 1:
        raise Refused("requested current gcx button missing or ambiguous")
    a, b, c, d = bounds(matches[0])
    a, b, c, d = max(a, root[0]), max(b, root[1]), min(c, root[2]), min(d, root[3])
    if a >= c or b >= d:
        raise Refused("button is outside the currently observed layout")
    return (a + c) // 2, (b + d) // 2


def hash_arguments(paths):
    # ADB joins shell arguments: literal validated paths avoid the former sh -c
    # quoting bug, which accidentally ran sha256sum with no input arguments.
    if not paths or any(not re.fullmatch(r"files/gcx/[A-Za-z0-9_./-]+", p)
                        or ".." in p.split("/") for p in paths):
        raise Refused("hash requires explicit literal gcx file paths")
    return ["shell", "run-as", PACKAGE, "sha256sum", *paths]


class Driver:
    def __init__(self, root, budget, invoke, clock=time.monotonic, sleep=time.sleep):
        self.root, self.budget, self.invoke = Path(root), budget, invoke
        self.clock, self.sleep = clock, sleep
        self.operation_end = budget.commands_end
        self.timeout = False
        self.actions, self.captures, self.received = 0, 0, 0
        self.mismatch = False
        self.last_capture = None
        self.receipt = {"started": time.time(), "operations": [], "handback": False,
                        "phase": "admitted", "recovery_resolved": False}

    def save(self):
        p = self.root / "receipt.private.json"
        p.write_text(json.dumps(self.receipt, indent=2))
        p.chmod(0o600)

    def call(self, args):
        if self.timeout:
            raise Refused("timed-out transport requires transaction retirement")
        timeout = min(30, self.operation_end - self.clock(),
                      self.budget.hard_end - self.clock())
        if timeout <= 0:
            raise Refused("operation deadline reached")
        try:
            result = self.invoke(args, timeout)
        except subprocess.TimeoutExpired:
            # End the helper; the existing transaction wrapper retires its own
            # foreground process group. Do not issue more commands concurrently
            # with a possibly surviving timed-out adb descendant.
            self.timeout = True
            raise
        self.received += len(result.stdout) + len(result.stderr)
        if self.received > 268435456:
            self.budget.forced_cleanup = True
            raise Refused("collected-byte ceiling reached")
        return result

    def focus(self):
        a = self.call(["shell", "dumpsys activity activities | grep -E 'Display #[0-9]+|mResumedActivity:|topResumedActivity='"])
        w = self.call(["shell", "dumpsys window displays | grep -E 'Display:|mCurrentFocus=|mTopFocusedDisplayId=|DisplayFrames'"])
        try:
            return own_focus(a.stdout.decode(), w.stdout.decode())
        except Refused:
            self.mismatch = True
            self.budget.forced_cleanup = True
            self.receipt["foreground_mismatch"] = True
            raise

    def action(self, args):
        if self.actions >= 100:
            raise Refused("UI action ceiling reached")
        self.actions += 1
        self.last_capture = None
        return self.call(args)

    def execute(self, command):
        op = command["op"]
        cleanup = bool(command.get("cleanup"))
        if op == "wait_owner":
            self.sleep(self.budget.owner_wait(self.clock(), command.get("seconds", 15)))
            return {"cleanup_required": self.budget.cleanup(self.clock())}
        seconds = max(0, min(30, command.get("timeout", 30)))
        self.budget.admit(self.clock(), cleanup, seconds)
        self.operation_end = min(self.budget.commands_end, self.clock() + seconds)
        if op == "capture":
            if self.captures >= 8:
                raise Refused("image ceiling reached")
            self.captures += 1
            before = self.focus()
            result = self.call(["exec-out", "screencap", "-p"])
            # Discard rather than save/return an image if focus changed during it.
            if self.focus() != before:
                self.mismatch = True
                self.budget.forced_cleanup = True
                raise Refused("capture discarded: focused window changed")
            if len(result.stdout) < 24 or result.stdout[:8] != b"\x89PNG\r\n\x1a\n" or result.stdout[12:16] != b"IHDR":
                raise Refused("capture did not return a PNG")
            dimensions = struct.unpack(">II", result.stdout[16:24])
            name = command["name"]
            if Path(name).name != name or not name.endswith(".png"):
                raise Refused("invalid local capture name")
            p = self.root / name
            p.write_bytes(result.stdout)
            p.chmod(0o600)
            self.last_capture = (name, self.clock(), before, dimensions)
            return {"capture": str(p), "bytes": len(result.stdout)}
        if op == "gcx_tap":
            # Automated native-button checks use an agent-inspected, just-taken
            # own image. Bind one tap to its live window AND display geometry;
            # never reuse a guessed historical layout or a batch of points.
            if command["target"] not in ("Pause", "Start", "A", "B"):
                raise Refused("unadmitted native control")
            if cleanup and command["target"] != "Pause":
                raise Refused("cleanup native input is Pause only")
            capture = self.last_capture
            if not capture or capture[0] != command["capture"] or self.clock() - capture[1] > 10:
                raise Refused("native tap requires a fresh inspected own capture")
            focus = self.focus()
            if focus != capture[2]:
                self.mismatch = True
                self.budget.forced_cleanup = True
                raise Refused("native tap aborted: live window/display changed")
            frames = focus[1][2]
            if len(frames) != 1 or tuple(map(int, frames[0][:2])) != capture[3]:
                raise Refused("native capture/display geometry is not coherent")
            bounds = command["bounds"]
            if len(bounds) != 4 or any(type(v) is not int for v in bounds):
                raise Refused("native button requires observed integer bounds")
            a, b, c, d = bounds
            width, height = capture[3]
            if not (0 <= a < c <= width and 0 <= b < d <= height):
                raise Refused("native button outside current image")
            result = self.action(["shell", "input", "tap", str((a+c)//2), str((b+d)//2)])
        elif op == "gcx_button":
            text = command["text"]
            if cleanup and text not in ("Stop game", "Stop current game"):
                raise Refused("cleanup cannot resume a test")
            before = self.focus()
            path = "/data/local/tmp/gcx-packet-ui.xml"
            result = self.call(["shell", f"uiautomator dump {path} >/dev/null && cat {path}; rm -f {path}"])
            xml = result.stdout.decode()
            try:
                x, y = button_center(xml[xml.index("<?xml"):], text)
            except (ValueError, ET.ParseError) as error:
                raise Refused("no usable current gcx UI layout") from error
            if self.focus() != before:
                self.mismatch = True
                self.budget.forced_cleanup = True
                raise Refused("layout batch aborted: focused window changed")
            result = self.action(["shell", "input", "tap", str(x), str(y)])
        elif op == "launch":
            if self.mismatch and not command.get("owner_reconfirmed"):
                raise Refused("unexpected foreground requires owner clarification")
            a = self.call(["shell", "dumpsys activity activities | grep -E 'Display #[0-9]+|mResumedActivity:|topResumedActivity='"])
            w = self.call(["shell", "dumpsys window displays | grep -E 'Display:|mCurrentFocus=|mTopFocusedDisplayId=|DisplayFrames'"])
            current, _ = focused_pair(a.stdout.decode(), w.stdout.decode())
            home = self.call(["shell", "cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.HOME"])
            if current not in (component(home.stdout.decode()), (PACKAGE, PACKAGE + ".GcxActivity")) and not command.get("owner_reconfirmed"):
                self.mismatch = True
                self.budget.forced_cleanup = True
                raise Refused("unexpected app before launch; owner clarification required")
            result = self.action(["shell", "am", "start", "--user", "0", "-a",
                                  "android.intent.action.MAIN", "-c",
                                  "android.intent.category.LAUNCHER", "-n", ACTIVITY])
        elif op == "system":
            step = command["step"]
            if step == "home":
                self.focus()  # Home departure must originate from the expected game.
                result = self.action(["shell", "input", "keyevent", "KEYCODE_HOME"])
            elif step == "wake_nonsecure":
                policy = self.call(["shell", "dumpsys window policy | grep -E 'secure=|showing='"])
                if "secure=false" not in policy.stdout.decode():
                    raise Refused("nonsecure wake authority does not match observation")
                self.action(["shell", "input", "keyevent", "KEYCODE_WAKEUP"])
                result = self.action(["shell", "wm", "dismiss-keyguard"])
            else:
                raise Refused("unadmitted system UI step")
        elif op == "hash_files":
            result = self.call(hash_arguments(command["paths"]))
        elif op == "adb":
            args = command["args"]
            joined = " ".join(args)
            if (re.search(r"\binput\b", joined) and args != ["shell", "dumpsys", "input"]
                    or re.search(r"\bam\b|\bwm\s+dismiss-keyguard", joined)
                    or "screencap" in joined or "uiautomator" in joined
                    or "sha256sum" in joined):
                raise Refused("raw UI/capture/hash batch refused; use guarded operations")
            result = self.call(args)
        else:
            raise Refused("unknown operation")
        name = command.get("save", f"operation-{len(self.receipt['operations']):03}.private.txt")
        if Path(name).name != name:
            raise Refused("invalid local output name")
        p = self.root / name
        p.write_bytes(result.stdout if command.get("binary") else
                      result.stdout + b"\nSTDERR\n" + result.stderr)
        p.chmod(0o600)
        self.receipt["operations"].append({"at": time.time(), "op": op,
                                           "returncode": result.returncode, "output": str(p)})
        self.save()
        return {"returncode": result.returncode, "saved": str(p),
                "stdout": "" if command.get("binary") else result.stdout.decode(errors="replace")[:command.get("print_limit", 1500)]}

    def finish(self):
        self.operation_end = self.budget.hard_end
        if self.timeout:
            self.receipt["pending_recovery"] = "command timed out; no further device commands"
            return
        try:
            result = self.call(["exec-out", "run-as", PACKAGE, "cat", "files/gcx/collection.json"])
            data = json.loads(result.stdout)
            entry = next((e for e in data["entries"] if e["game_id"] == data.get("selected")), None)
            clean = bool(entry and entry.get("stopped") and not entry.get("runtime_incomplete")
                         and not entry.get("restart_required"))
            self.receipt["clean_stop_metadata"] = clean
            pid = self.call(["shell", "pidof", PACKAGE]).stdout.strip()
            if pid and clean:
                self.call(["shell", "am", "force-stop", PACKAGE])
                pid = self.call(["shell", "pidof", PACKAGE]).stdout.strip()
            if pid:
                # Never replace normal Stop with killing a live profile writer.
                self.receipt["pending_recovery"] = "live runtime retained; normal Stop not verified"
                try:
                    self.focus()
                    self.action(["shell", "input", "keyevent", "KEYCODE_HOME"])
                except Refused:
                    pass  # Do not act on an unexpected app to finish cleanup.
                return
            self.receipt["clients_retired"] = True
            # Screen-off is an explicit return step, only from gcx or the observed
            # current Home. No unrelated-app screenshot or touch is used.
            display = self.call(["shell", "dumpsys display | grep -E 'Display State=|mState='"]).stdout.decode()
            if "Display State=OFF" not in display:
                home = self.call(["shell", "cmd package resolve-activity --brief -a android.intent.action.MAIN -c android.intent.category.HOME"]).stdout.decode()
                resumed = self.call(["shell", "dumpsys activity activities | grep -E 'Display #[0-9]+|mResumedActivity:|topResumedActivity='"]).stdout.decode()
                windows = self.call(["shell", "dumpsys window displays | grep -E 'Display:|mCurrentFocus=|mTopFocusedDisplayId=|DisplayFrames'"]).stdout.decode()
                current, _ = focused_pair(resumed, windows)
                if current not in (component(home), (PACKAGE, PACKAGE + ".GcxActivity")):
                    raise Refused("unexpected app during return; no screen input")
                self.action(["shell", "input", "keyevent", "KEYCODE_SLEEP"])
            # Sleep/keyguard transitions are asynchronous. Re-observe finitely,
            # within the same remaining hard deadline; never infer success from
            # the key-event command's return code.
            for attempt in range(4):
                display = self.call(["shell", "dumpsys display | grep -E 'Display State=|mState='"]).stdout.decode()
                keyguard = self.call(["shell", "dumpsys window policy | grep -E 'secure=|showing='"]).stdout.decode()
                self.receipt["display_off"] = "Display State=OFF" in display
                self.receipt["keyguard_showing"] = "showing=true" in keyguard
                if self.receipt["display_off"] and self.receipt["keyguard_showing"]:
                    break
                if attempt < 3 and self.clock() + 0.3 < self.budget.hard_end:
                    self.sleep(0.3)
            self.receipt["handback"] = self.receipt["display_off"] and self.receipt["keyguard_showing"]
            self.receipt["recovery_resolved"] = self.receipt["handback"]
            if not self.receipt["handback"]:
                self.receipt["pending_recovery"] = "physical display/keyguard return not verified"
        except (Refused, ValueError, subprocess.TimeoutExpired) as error:
            self.receipt["pending_recovery"] = str(error)


def main():
    alias, root = sys.argv[1:3]
    root = Path(root)
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    fd = int(os.environ["CODEX_ANDROID_LEASE_FD"])
    budget = Budget(time.monotonic(), float(os.environ["GCX_OUTER_DEADLINE_MONO"]))
    def invoke(args, timeout):
        return subprocess.run(["android-device", "adb", alias, "--", *args],
                              stdin=subprocess.DEVNULL, capture_output=True,
                              timeout=timeout, pass_fds=(fd,))
    driver = Driver(root, budget, invoke)
    task = os.environ["GCX_DEVICE_TASK"]
    if not re.fullmatch(r"gcx-[a-z0-9-]{1,100}", task):
        raise Refused("invalid own task record name")
    canonical = Path.home() / ".local/state/codex-android/tasks" / task / "receipt.json"
    def canonical_pending():
        canonical.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        r = dict(driver.receipt, alias=alias, returned=driver.receipt["handback"],
                 lease_released=False, task_owned_resources_pending=not driver.receipt.get("clients_retired", False),
                 return_observations_pending=not driver.receipt["handback"], source_evidence=str(root))
        canonical.write_text(json.dumps(r, indent=2)); canonical.chmod(0o600)
    driver.save(); canonical_pending()
    print("LEASE_ACQUIRED; new work stops at earliest deadline minus300s; return reserve60s", flush=True)
    cleanup_announced = False
    try:
        while time.monotonic() < budget.commands_end:
            now = time.monotonic()
            if budget.cleanup(now) and not cleanup_announced:
                driver.receipt["cleanup_entered"] = time.time()
                print("CLEANUP_REQUIRED; owner waiting ended; cleanup operations only", flush=True)
                cleanup_announced = True
            boundary = budget.commands_end if budget.cleanup(now) else budget.cleanup_at
            ready, _, _ = select.select([sys.stdin], [], [], min(15, max(0, boundary-now)))
            if not ready:
                continue
            line = sys.stdin.readline()
            if not line:
                break
            command = json.loads(line)
            if command["op"] == "finish":
                break
            try:
                result = driver.execute(command)
                print(json.dumps(result), flush=True)
            except Refused as error:
                print(json.dumps({"REFUSED": str(error), "cleanup_required": budget.cleanup(time.monotonic())}), flush=True)
            driver.save()
    finally:
        driver.finish()
        driver.receipt.update(phase="returned" if driver.receipt["handback"] else "pending-recovery",
                              finished=time.time(), actions=driver.actions,
                              captures=driver.captures, collected_bytes=driver.received)
        driver.save(); canonical_pending()
        print(json.dumps({"FINISHED": True, "handback": driver.receipt["handback"],
                          "pending_recovery": driver.receipt.get("pending_recovery")}), flush=True)


if __name__ == "__main__":
    main()
