# SPDX-License-Identifier: MIT
"""Offline regression fixtures: no device alias, ADB or private data."""
import importlib.util
from pathlib import Path
import subprocess
import struct
import hashlib
import io
import json
import tarfile
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "android_packet", Path(__file__).resolve().parents[1] / "android_packet.py")
packet = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(packet)
OWN = packet.PACKAGE
RESUMED = f"Display #0 (activities):\nmResumedActivity: ActivityRecord{{1 u0 {OWN}/.GcxActivity t1}}"
WINDOW = f"Display: mDisplayId=0 rootTasks=1\nmCurrentFocus=Window{{1 u0 {OWN}/{OWN}.GcxActivity}}\nDisplayFrames w=800 h=400 r=0\nmTopFocusedDisplayId=0"
OTHER = "com.example.other/.OtherActivity"
VIEWPORT = "DisplayViewport{type=INTERNAL, valid=true, isActive=true, displayId=0, uniqueId='local:123', orientation=0}"
XML = f'''<?xml version="1.0"?><hierarchy><node bounds="[0,0][800,400]">
<node text="Resume" package="{OWN}" class="android.widget.Button"
enabled="true" clickable="true" bounds="[40,80][300,160]"/>
<node text="Stop game" package="{OWN}" class="android.widget.Button"
enabled="true" clickable="true" bounds="[400,200][700,280]"/>
</node></hierarchy>'''


class Fixture:
    def __init__(self, root):
        self.now = 100
        self.calls = []
        self.timeouts = []
        self.resumed, self.windows = RESUMED, WINDOW
        self.after_capture = None
        self.viewports = VIEWPORT
        self.after_viewports = None
        self.after_dump = None
        self.fail = False
        self.live = True
        self.clean = False
        self.display_states = ["OFF"]
        self.keyguard_states = ["true"]
        self.retirement = None
        self.budget = packet.Budget(self.now, 1300)
        self.driver = packet.Driver(root, self.budget, self.invoke,
                                    lambda: self.now, self.sleep)

    def sleep(self, seconds):
        self.now += seconds

    def invoke(self, args, timeout):
        self.calls.append(args)
        self.timeouts.append(timeout)
        if self.fail:
            raise subprocess.TimeoutExpired(args, timeout)
        joined = " ".join(args)
        out = b""
        if args == ["shell", packet.FOCUS_QUERY]:
            out = (self.windows + "\n" + self.resumed).encode()
        elif "dumpsys activity" in joined:
            out = self.resumed.encode()
        elif "dumpsys window displays |" in joined:
            out = self.windows.encode()
        elif "uiautomator dump" in joined:
            out = XML.encode()
            if self.after_dump:
                self.windows = self.after_dump
        elif args[:4] == ["shell", "-T", "screencap", "-d"]:
            out = b"\x89PNG\r\n\x1a\n" + b"\0\0\0\rIHDR" + struct.pack(">II", 800, 400)
            if self.after_capture:
                self.windows = self.after_capture
            if self.after_viewports is not None:
                self.viewports = self.after_viewports
        elif "DisplayViewport{" in joined:
            out = self.viewports.encode()
        elif "collection.json" in joined and "sha256sum" not in args:
            if self.retirement:
                return subprocess.CompletedProcess(args, 0, self.retirement["collection"], b"")
            value = "true" if self.clean else "false"
            out = ('{"selected":"fixture","entries":[{"game_id":"fixture",'
                   f'"stopped":{value},"runtime_incomplete":false,'
                   '"restart_required":false}]}').encode()
        elif "pidof" in joined:
            out = b"1000" if self.live else b""
        elif "force-stop" in joined:
            self.live = False
        elif "resolve-activity" in joined:
            out = b"com.example.home/.HomeActivity"
        elif "sha256sum" in args and self.retirement:
            out = "".join(f"{self.retirement['hashes'][p]}  {p}\n" for p in args[4:]).encode()
        elif "/proc/1000/stat" in args and self.retirement:
            out = ("1000 (gcx) S " + "0 " * 18 + str(self.retirement["ticks"])).encode()
        elif "logcat" in joined and self.retirement:
            out = self.retirement["logs"].encode()
        elif "dumpsys display" in joined:
            state = self.display_states.pop(0) if len(self.display_states) > 1 else self.display_states[0]
            out = ("Display State=" + state).encode()
        elif "dumpsys window policy" in joined:
            state = self.keyguard_states.pop(0) if len(self.keyguard_states) > 1 else self.keyguard_states[0]
            out = ("showing=" + state + " secure=false").encode()
        return subprocess.CompletedProcess(args, 0, out, b"")


class CaptureBudgetTests(unittest.TestCase):
    def test_existing_packets_keep_eight_capture_default(self):
        with tempfile.TemporaryDirectory() as root:
            fixture = Fixture(root)
            self.assertEqual(fixture.driver.capture_limit, 8)
            self.assertEqual(fixture.driver.receipt['capture_limit'], 8)
            fixture.driver.captures = 8
            with self.assertRaises(packet.Refused):
                fixture.driver.execute(dict(op='capture', name='extra.png', capture_limit=10))
            self.assertEqual(fixture.calls, [])

    def test_task_admitted_ten_capture_ceiling_is_enforced(self):
        with tempfile.TemporaryDirectory() as root:
            fixture = Fixture(root)
            driver = packet.Driver(root, fixture.budget, fixture.invoke,
                                   lambda: fixture.now, fixture.sleep, capture_limit=10)
            driver.captures = 9
            self.assertEqual(driver.receipt['capture_limit'], 10)
            driver.execute(dict(op='capture', name='tenth.png'))
            self.assertEqual(driver.captures, 10)
            before = len(fixture.calls)
            with self.assertRaises(packet.Refused):
                driver.execute(dict(op='capture', name='eleventh.png'))
            self.assertEqual(len(fixture.calls), before)

    def test_ceiling_above_stock_lab_max_refuses_before_transport(self):
        for limit in [0, 11, 10., True]:
            with self.subTest(limit=limit), self.assertRaises(packet.Refused):
                packet.Driver('/unused', None, None, capture_limit=limit)


class InspectedRegionTests(unittest.TestCase):
    def setUp(self):
        self.size = (128, 96)
        self.rgb = bytes([72, 104, 156]) * (128 * 96)
        self.bounds = (32, 24, 96, 88)

    def matches(self, **changes):
        args = dict(reference_size=self.size, reference_rgb=self.rgb,
                    current_size=self.size, current_rgb=self.rgb, bounds=self.bounds)
        args.update(changes)
        return packet.matching_inspected_region(**args)

    def test_exact_control_pixels_match(self):
        self.assertTrue(self.matches())

    def test_one_changed_control_pixel_refuses(self):
        changed = bytearray(self.rgb)
        changed[(40 * 128 + 50) * 3] ^= 1
        self.assertFalse(self.matches(current_rgb=changed))

    def test_background_outside_control_may_change(self):
        changed = bytearray(self.rgb)
        changed[0] ^= 1
        self.assertTrue(self.matches(current_rgb=changed))

    def test_changed_frame_size_refuses(self):
        self.assertFalse(self.matches(current_size=(96, 128)))

    def test_malformed_frame_sizes_refuse(self):
        for size in [None, (), (128,), (128., 96), (0, 96), (8193, 96)]:
            with self.subTest(size=size):
                self.assertFalse(self.matches(current_size=size))

    def test_malformed_rgb_plane_refuses(self):
        self.assertFalse(self.matches(current_rgb=self.rgb[:-1]))
        self.assertFalse(self.matches(reference_rgb=b''))
        self.assertFalse(self.matches(current_rgb=None))
        self.assertFalse(self.matches(reference_rgb=''))

    def test_invalid_or_tiny_bounds_refuse(self):
        for bounds in [None, (-1, 0, 64, 64), (32, 24, 129, 88), (32, 24, 63, 88),
                       (32, 24, 96, 24), (32., 24, 96, 88), (32, 24, 96)]:
            with self.subTest(bounds=bounds):
                self.assertFalse(self.matches(bounds=bounds))


class PacketTests(unittest.TestCase):
    def test_optional_lifecycle_limit_retains_fresh_state_and_presentation_proof(self):
        trace = ('native.pause.end playing\ngeometry present.ready buffer=2280x1080\n'
                 'lifecycle.optional_limit optional records omitted; native acknowledgment and Stop slots reserved\n')
        self.assertTrue(packet.playing_presentation_ready(trace))

    def test_missing_native_acknowledgments_refuse_even_with_a_fresh_frame(self):
        trace = ('native.pause.end playing\n'
                 'lifecycle.ack_limit trace cap: native state acknowledgments unavailable\n'
                 'geometry present.ready buffer=2280x1080\n')
        self.assertFalse(packet.playing_presentation_ready(trace))

    def test_play_readiness_rejects_historical_frame_after_surface_or_measurement_loss(self):
        first = "surface.init\ngeometry present.ready buffer=2280x1080\nnative.pause.end playing\n"
        self.assertTrue(packet.playing_presentation_ready(first))
        for invalid in ("surface.init", "surface.terminate", "geometry measurement=false presented=false", "geometry measurement=true presented=false", "geometry present.unavailable buffer=2280x1080"):
            stale = first + invalid + "\nnative.pause.end playing\n"
            self.assertFalse(packet.playing_presentation_ready(stale))
            self.assertTrue(packet.playing_presentation_ready(stale + "geometry present.ready buffer=2280x1080\n"))
        self.assertFalse(packet.playing_presentation_ready(first + "native.pause.end paused\n"))
        self.assertFalse(packet.playing_presentation_ready(first + "presentation trace cap reached\n"))

    def test_every_fold_display_must_be_off_and_policy_mstate_is_not_physical_state(self):
        self.assertFalse(packet.physical_displays_off("Display State=ON\nDisplay State=OFF\nmState=OFF"))
        self.assertTrue(packet.physical_displays_off("Display State=OFF\nDisplay State=OFF\nmState=AUTO_BRIGHTNESS_ENABLED"))
        self.assertFalse(packet.physical_displays_off("mState=OFF"))
        self.assertFalse(packet.physical_displays_off("Display State=DOZE\nDisplay State=OFF"))
        self.assertFalse(packet.physical_displays_off("Display State=ON_SUSPEND\nDisplay State=OFF"))

    def test_capture_resolves_only_active_local_panel_for_focused_display(self):
        self.f.viewports += "\n" + VIEWPORT.replace("isActive=true", "isActive=false").replace("local:123", "local:456")
        self.f.viewports += "\n" + VIEWPORT.replace("displayId=0", "displayId=3").replace("local:123", "local:789")
        self.f.driver.execute({"op": "capture", "name": "fold.png"})
        self.assertIn(["shell", "-T", "screencap", "-d", "123", "-p"], self.f.calls)
        self.assertTrue((Path(self.temp.name) / "fold.png").exists())

    def test_capture_refuses_missing_inactive_remote_or_ambiguous_panels_before_pixels(self):
        for views in ("", VIEWPORT.replace("isActive=true", "isActive=false"),
                      VIEWPORT.replace("valid=true", "valid=false"),
                      VIEWPORT.replace("local:123", "virtual:123"),
                      VIEWPORT.replace("displayId=0", "displayId=3"),
                      VIEWPORT + VIEWPORT.replace("local:123", "local:456")):
            with self.subTest(views=views):
                self.f.viewports = views
                with self.assertRaises(packet.Refused):
                    self.f.driver.execute({"op": "capture", "name": "refused.png"})
                self.assertFalse(any("screencap" in c for c in self.f.calls))

    def test_capture_panel_change_discards_pixels_and_enters_cleanup(self):
        for after in ("", VIEWPORT.replace("local:123", "local:456")):
            with self.subTest(after=after):
                self.f.viewports = VIEWPORT
                self.f.after_viewports = after
                with self.assertRaises(packet.Refused):
                    self.f.driver.execute({"op": "capture", "name": "discard.png"})
                self.assertTrue(self.f.budget.forced_cleanup)
                self.assertFalse(list(Path(self.temp.name).glob("*.png")))
                self.f.budget.forced_cleanup = False

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.f = Fixture(self.temp.name)

    def prepare_retirement(self):
        root = self.f.driver.root
        payload = b"original complete native payload fixture"
        checkpoint = hashlib.sha256(payload).hexdigest()
        game, profile = "a" * 32, "b" * 32
        base = f"files/gcx/profiles/{profile}/"
        collection = json.dumps({"schema_version": 1, "selected": game, "entries": [{
            "game_id": game, "profile_id": profile, "checkpoint": checkpoint,
            "paused": True, "runtime_incomplete": True, "stopped": False,
            "restart_required": False}]}).encode()
        files = {"files/gcx/collection.json": collection, base + "save-generation.json": b"generation",
                 base + "GC/SRAM.raw": b"console", base + "GC/MemoryCardA.USA.raw": b"ordinary save",
                 base + f"checkpoints/{checkpoint}.gcx": payload}
        archive = root / "complete.tar"
        with tarfile.open(archive, "w") as tar:
            for name, data in files.items():
                member = tarfile.TarInfo(name); member.size = len(data)
                tar.addfile(member, io.BytesIO(data))
        (root / "verified-generation.private.json").write_text(json.dumps({
            "checkpoint": checkpoint, "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
            "archive_bytes": archive.stat().st_size,
            "pause_intent": True,
            "verification": "PASS read-only checkpoint inspection: captured pause intent=true"}))
        (root / "trusted-binding.private.json").write_text(json.dumps({"profile_id": profile, "game_id": game}))
        self.f.retirement = {"collection": collection,
                             "hashes": {p: hashlib.sha256(data).hexdigest() for p, data in files.items()},
                             "ticks": 777, "logs": "native.pause.end paused\nCheckpoint complete: durable\n"}
        self.f.resumed = RESUMED.replace(OWN + "/.GcxActivity", "com.example.home/.HomeActivity")
        self.f.windows = WINDOW.replace(OWN + "/" + OWN + ".GcxActivity", "com.example.home/.HomeActivity")
        return {"op": "retire_checkpoint", "archive": "complete.tar", "pid": 1000, "start_ticks": 777}

    def test_checkpoint_retirement_requires_verified_backup_background_and_current_process(self):
        command = self.prepare_retirement()
        self.f.driver.execute(command)
        self.assertFalse(self.f.live)
        self.assertTrue(self.f.driver.receipt["checkpoint_process_retired"])
        self.assertFalse(self.f.driver.receipt["retirement_is_normal_stop"])
        self.assertEqual(sum("force-stop" in c for c in self.f.calls), 1)

    def test_retirement_refuses_newer_saves_reused_pid_playing_or_foreground_runtime(self):
        for case in ("card", "pid", "playing", "foreground"):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as root:
                self.f = Fixture(root); command = self.prepare_retirement()
                if case == "card":
                    path = next(p for p in self.f.retirement["hashes"] if p.endswith("USA.raw"))
                    self.f.retirement["hashes"][path] = "0" * 64
                elif case == "pid":
                    self.f.retirement["ticks"] = 888
                elif case == "playing":
                    self.f.retirement["logs"] += "native.pause.end playing\n"
                else:
                    self.f.resumed, self.f.windows = RESUMED, WINDOW
                with self.assertRaises(packet.Refused):
                    self.f.driver.execute(command)
                self.assertTrue(self.f.live)
                self.assertFalse(any("force-stop" in c for c in self.f.calls))
                self.assertTrue(self.f.budget.forced_cleanup)

    def test_changed_backup_or_cleanup_flag_cannot_retire_live_runtime(self):
        command = self.prepare_retirement()
        command["cleanup"] = True
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(command)
        command.pop("cleanup")
        with (self.f.driver.root / "complete.tar").open("ab") as file:
            file.write(b"changed")
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(command)
        self.assertEqual(self.f.calls, [])

    def test_bounded_quiescent_recovery_keeps_all_proofs_and_is_never_joined_stop(self):
        command = self.prepare_retirement()
        command.update(op="recover_checkpoint", cleanup=True)
        self.f.budget.forced_cleanup = True
        self.f.driver.execute(command)
        self.assertFalse(self.f.live)
        self.assertTrue(self.f.driver.receipt["quiescent_recovery"])
        self.assertFalse(self.f.driver.receipt["retirement_is_normal_stop"])
        proof = self.f.driver.receipt["retirement_validation"]
        self.assertEqual((proof["pid"], proof["start_ticks"]), ("1000", "777"))
        self.assertIn("native.pause.end paused", Path(proof["quiescence"]).read_text())
        self.assertEqual(proof["actual_file_sha256"], self.f.retirement["hashes"])

    def test_failed_quiescence_is_retained_without_retirement_then_retry_is_distinct(self):
        command = self.prepare_retirement()
        self.f.retirement["logs"] += "native.pause.end playing\n"
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(command)
        proof = self.f.driver.receipt["retirement_observations"][0]
        self.assertIn("native.pause.end playing", Path(proof["quiescence"]).read_text())
        self.assertNotIn("retirement_validation", self.f.driver.receipt)
        self.assertFalse(any("force-stop" in c for c in self.f.calls))
        self.f.retirement["logs"] += "native.pause.end paused\nCheckpoint complete: durable\n"
        command.update(op="recover_checkpoint", cleanup=True)
        self.f.driver.execute(command)
        latest = self.f.driver.receipt["retirement_validation"]
        self.assertNotEqual(latest["quiescence"], proof["quiescence"])
        self.assertEqual(len(self.f.driver.receipt["retirement_observations"]), 2)

    def test_automated_cutoff_finalizes_instead_of_waiting_through_cleanup(self):
        budget = self.f.budget
        self.assertEqual(budget.input_boundary(budget.cleanup_at - 1, True), budget.cleanup_at)
        self.assertEqual(budget.input_boundary(budget.cleanup_at, True), budget.cleanup_at)
        self.assertEqual(budget.input_boundary(budget.cleanup_at + 1, True), budget.cleanup_at + 1)
        self.assertEqual(budget.input_boundary(budget.cleanup_at, False), budget.commands_end)
        budget.forced_cleanup = True
        self.assertEqual(budget.input_boundary(101, True), 101)

    def test_pre_restore_recovery_requires_full_bootstrap_trace_and_refuses_any_restore_or_play(self):
        bootstrap = "activity.init\nruntime.boot.begin\nboot.checkpoint_validate.end ok\nboot.native_paused.end ok\nruntime.boot.end ready_for_restore_or_play\nnative.pause.end paused\n"
        for suffix in ("", "restore.validate.begin\n", "restore.load.end ok_paused\n", "native.pause.end playing\n", "runtime.boot.begin\n"):
            with self.subTest(suffix=suffix), tempfile.TemporaryDirectory() as root:
                self.f=Fixture(root);command=self.prepare_retirement();command.update(op="recover_pending_restore",cleanup=True)
                self.f.retirement["logs"]=bootstrap+suffix
                if suffix:
                    with self.assertRaises(packet.Refused):self.f.driver.execute(command)
                    self.assertTrue(self.f.live)
                else:
                    self.f.driver.execute(command);self.assertFalse(self.f.live)
                    self.assertTrue(self.f.driver.receipt["pending_restore_recovery"])
                    self.assertFalse(self.f.driver.receipt["retirement_is_normal_stop"])

    def test_cleanup_at_locked_screen_off_needs_no_resumed_activity_or_wake(self):
        command=self.prepare_retirement();command.update(op="recover_checkpoint",cleanup=True)
        self.f.windows="Display: mDisplayId=0\nmCurrentFocus=Window{1 u0 NotificationShade}"
        self.f.resumed="Display #0 (activities):"
        self.f.driver.execute(command)
        self.assertFalse(self.f.live)
        self.assertEqual(self.f.driver.receipt["retirement_background"],"physical_display_off_and_keyguard_showing")
        self.assertFalse(any("input" in c or "wake" in c for c in self.f.calls))

    def test_earliest_deadline_and_real_cleanup_reserve(self):
        budget = packet.Budget(100, 800)
        self.assertEqual((budget.hard_end, budget.cleanup_at, budget.commands_end),
                         (800, 500, 740))
        self.assertEqual(budget.owner_wait(498, 60), 2)
        with self.assertRaises(packet.Refused):
            budget.owner_wait(500, 1)
        with self.assertRaises(packet.Refused):
            budget.admit(480, False, 30)
        budget.admit(500, True, 30)
        with self.assertRaises(packet.Refused):
            budget.admit(740, True, 1)
        with self.assertRaises(packet.Refused):
            packet.Budget(100, 399)

    def test_wait_crosses_to_cleanup_but_never_waits_after_it(self):
        self.f.now = self.f.budget.cleanup_at - 2
        answer = self.f.driver.execute({"op": "wait_owner", "seconds": 60})
        self.assertTrue(answer["cleanup_required"])
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "wait_owner"})
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "capture", "name": "late.png"})
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "gcx_button", "text": "Resume", "cleanup": True})
        self.assertEqual(self.f.calls, [])

    def test_requested_short_timeout_is_also_the_actual_operation_deadline(self):
        self.f.now = self.f.budget.cleanup_at - 2
        self.f.driver.execute({"op": "hash_files", "paths": ["files/gcx/card.raw"],
                               "timeout": 1})
        self.assertEqual(self.f.driver.operation_end, self.f.budget.cleanup_at - 1)
        self.assertEqual(self.f.timeouts, [1])

    def test_live_display_section_excludes_historical_and_other_display_focus(self):
        windows = ("historical ANR mCurrentFocus=Window{old u0 " + OTHER + "}\n" +
                   "Display: mDisplayId=4 rootTasks=1\nmCurrentFocus=Window{other u0 " + OTHER + "}\n" + WINDOW)
        activities = "Display #4 (activities):\ntopResumedActivity=" + OTHER + "\n" + RESUMED
        self.assertEqual(packet.own_focus(activities, windows)[0], (OWN, OWN + ".GcxActivity"))
        with self.assertRaises(packet.Refused):
            packet.own_focus(activities, windows.replace("mTopFocusedDisplayId=0", "mTopFocusedDisplayId=4"))

    def test_android13_single_live_display_needs_no_global_top_field(self):
        windows = WINDOW.replace("mTopFocusedDisplayId=0", "")
        self.assertEqual(packet.own_focus(RESUMED, windows)[0], (OWN, OWN + ".GcxActivity"))
        with self.assertRaises(packet.Refused):
            packet.own_focus(RESUMED, windows + "\nDisplay: mDisplayId=4 rootTasks=1\n")

    def test_resumed_and_focused_must_both_be_own_activity(self):
        for resumed, windows in ((RESUMED, f"mCurrentFocus=Window{{1 u0 {OTHER}}}"),
                                 (f"mResumedActivity: {OTHER}", WINDOW)):
            with self.assertRaises(packet.Refused):
                packet.own_focus(resumed, windows)
        # Android can report the same resumed component twice, in two formats.
        self.assertEqual(packet.own_focus(RESUMED + "\ntopResumedActivity=" +
                                         OWN + "/" + OWN + ".GcxActivity", WINDOW)[0],
                         (OWN, OWN + ".GcxActivity"))

    def test_mismatch_blocks_capture_and_touch_before_mutation(self):
        self.f.windows = f"mCurrentFocus=Window{{1 u0 {OTHER}}}"
        for command in ({"op": "capture", "name": "wrong.png"},
                        {"op": "gcx_button", "text": "Resume", "cleanup": True}):
            with self.assertRaises(packet.Refused):
                self.f.driver.execute(command)
        self.assertTrue(self.f.driver.mismatch)
        self.assertFalse(any("screencap" in c or "input" in c for c in self.f.calls))
        self.assertFalse(list(Path(self.temp.name).glob("*.png")))

    def test_focus_change_during_capture_discards_pixels(self):
        self.f.after_capture = f"mCurrentFocus=Window{{1 u0 {OTHER}}}"
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "capture", "name": "discard.png"})
        self.assertFalse(list(Path(self.temp.name).glob("*.png")))

    def test_native_tap_is_single_use_and_bound_to_current_inspected_image(self):
        self.f.driver.execute({"op": "capture", "name": "native.png"})
        tap = {"op": "gcx_tap", "capture": "native.png", "target": "Pause",
               "bounds": [30, 300, 90, 390]}
        self.f.driver.execute(tap)
        self.assertEqual(self.f.calls[-1], ["shell", "input", "tap", "60", "345"])
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(tap)

    def test_focus_metadata_is_batched_without_dropping_either_observation(self):
        self.f.driver.focus()
        self.assertEqual(self.f.calls, [["shell", packet.FOCUS_QUERY]])
        self.f.resumed = RESUMED.replace(OWN + "/.GcxActivity", OTHER)
        with self.assertRaises(packet.Refused):
            self.f.driver.focus()
        self.assertTrue(self.f.driver.mismatch)

    def test_held_test_stimulus_is_one_bounded_stationary_contact(self):
        self.f.driver.execute({"op": "capture", "name": "native.png"})
        self.f.driver.execute({"op": "gcx_tap", "capture": "native.png", "target": "Start",
                               "bounds": [30, 300, 90, 390], "press_ms": 120})
        self.assertEqual(self.f.calls[-1], ["shell", "input", "swipe", "60", "345", "60", "345", "120"])
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "gcx_tap", "capture": "native.png", "target": "Start",
                                   "bounds": [30, 300, 90, 390], "press_ms": 120})

    def test_held_test_stimulus_refuses_invalid_duration_before_input(self):
        self.f.driver.execute({"op": "capture", "name": "native.png"})
        for invalid in (-1, 251, True, "120", 0.5):
            with self.assertRaises(packet.Refused):
                self.f.driver.execute({"op": "gcx_tap", "capture": "native.png", "target": "Start",
                                       "bounds": [30, 300, 90, 390], "press_ms": invalid})
        self.assertFalse(any("input" in c for c in self.f.calls))

    def test_pause_cannot_hold_contact_across_opening_a_dialog_window(self):
        self.f.driver.execute({"op": "capture", "name": "native.png"})
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "gcx_tap", "capture": "native.png", "target": "Pause",
                                   "bounds": [30, 300, 90, 390], "press_ms": 120})
        self.assertFalse(any("input" in c for c in self.f.calls))

    def test_native_tap_refuses_stale_or_rotated_capture(self):
        self.f.driver.execute({"op": "capture", "name": "native.png"})
        tap = {"op": "gcx_tap", "capture": "native.png", "target": "Pause",
               "bounds": [30, 300, 90, 390]}
        self.f.now += 31
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(tap)
        self.f.now -= 31
        self.f.windows = WINDOW.replace("w=800 h=400 r=0", "w=400 h=800 r=1")
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(tap)
        self.assertFalse(any("input" in c for c in self.f.calls))

    def test_native_image_age_is_rechecked_at_dispatch_after_slow_focus_queries(self):
        self.f.driver.execute({"op": "capture", "name": "native.png"})
        self.f.now += 10
        invoke = self.f.driver.invoke
        def slow_focus(args, timeout):
            result = invoke(args, timeout)
            if "dumpsys window displays" in " ".join(args):
                self.f.now += 25
            return result
        self.f.driver.invoke = slow_focus
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "gcx_tap", "capture": "native.png", "target": "Pause",
                                   "bounds": [30, 300, 90, 390]})
        self.assertFalse(any("input" in c for c in self.f.calls))

    def test_each_button_uses_fresh_observed_bounds_and_focus(self):
        self.f.driver.execute({"op": "gcx_button", "text": "Resume"})
        self.f.driver.execute({"op": "gcx_button", "text": "Stop game", "cleanup": True})
        self.assertEqual([c for c in self.f.calls if "tap" in c],
                         [["shell", "input", "tap", "170", "120"],
                          ["shell", "input", "tap", "550", "240"]])
        self.assertEqual(sum("dumpsys activity" in " ".join(c) for c in self.f.calls), 4)

    def test_scroll_view_clips_button_even_when_button_is_inside_the_screen(self):
        xml = XML.replace('<node text="Stop game"',
                          '<node bounds="[300,10][750,190]"><node text="Stop game"')
        xml = xml.replace('bounds="[400,200][700,280]"/>',
                          'bounds="[400,200][700,280]"/></node>')
        with self.assertRaises(packet.Refused):
            packet.button_center(xml, "Stop game")
        visible = xml.replace('[300,10][750,190]', '[300,10][750,250]')
        self.assertEqual(packet.button_center(visible, "Stop game"), (550, 225))

    def test_negative_offscreen_bounds_cannot_become_positive_tap_coordinates(self):
        xml = XML.replace('[400,200][700,280]', '[400,-280][700,-200]')
        with self.assertRaises(packet.Refused):
            packet.button_center(xml, "Stop game")

    def test_focus_change_after_layout_aborts_the_input_batch(self):
        self.f.after_dump = f"mCurrentFocus=Window{{1 u0 {OTHER}}}"
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "gcx_button", "text": "Resume"})
        self.assertFalse(any("input" in c for c in self.f.calls))
        with self.assertRaises(packet.Refused):
            packet.button_center(XML, "missing stale button")

    def test_same_package_new_window_invalidates_the_observed_button_layout(self):
        self.f.after_dump = WINDOW.replace("Window{1", "Window{2")
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "gcx_button", "text": "Resume"})
        self.assertFalse(any("input" in c for c in self.f.calls))

    def test_focused_app_without_a_current_focused_window_is_not_admission(self):
        self.f.windows = f"mFocusedApp=ActivityRecord{{1 u0 {OWN}/.GcxActivity t1}}"
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "capture", "name": "missing-focus.png"})
        self.assertFalse(any("screencap" in c for c in self.f.calls))
        self.assertFalse(list(Path(self.temp.name).glob("*.png")))
        proof = self.f.driver.receipt["focus_refusals"]
        self.assertEqual(len(proof), 1)
        self.assertIn("mFocusedApp=", Path(proof[0]).read_text())

    def test_raw_batches_and_option_variants_cannot_bypass_guards(self):
        for args in (["shell", "input", "--display", "0", "tap", "1", "2"],
                     ["shell", "input tap 1 2; input keyevent HOME"],
                     ["shell", "sh", "-c", "input tap 1 2"],
                     ["exec-out", "screencap", "-p"],
                     ["shell", "am", "start", "-n", OTHER],
                     ["shell", "uiautomator", "dump"],
                     ["shell", "run-as", OWN, "sh", "-c", "sha256sum"]):
            with self.assertRaises(packet.Refused):
                self.f.driver.execute({"op": "adb", "args": args})
        self.assertEqual(self.f.calls, [])

    def test_unexpected_app_cannot_be_displaced_by_ordinary_launch(self):
        self.f.resumed = "mResumedActivity: " + OTHER
        self.f.windows = f"mCurrentFocus=Window{{1 u0 {OTHER}}}"
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "launch"})
        self.assertFalse(any("am" in c for c in self.f.calls))

    def test_authorized_initial_own_entry_cannot_override_later_mismatch(self):
        self.f.resumed = RESUMED.replace(OWN + "/.GcxActivity", OTHER)
        self.f.windows = WINDOW.replace(OWN + "/" + OWN + ".GcxActivity", OTHER)
        self.f.driver.execute({"op": "launch", "initial_entry": True})
        self.assertEqual(sum("am" in c for c in self.f.calls), 1)
        self.assertIn("-W", next(c for c in self.f.calls if "am" in c))
        with self.assertRaises(packet.Refused):
            self.f.driver.execute({"op": "launch", "initial_entry": True})
        self.assertEqual(sum("am" in c for c in self.f.calls), 1)

    def test_hashing_always_has_literal_file_arguments(self):
        paths = ["files/gcx/profiles/fixture/cards/MemoryCardA.USA.raw"]
        args = packet.hash_arguments(paths)
        self.assertEqual(args, ["shell", "run-as", OWN, "sha256sum", *paths])
        for invalid in ([], ["files/gcx/*.raw"], ["files/gcx/a b"],
                        ["files/gcx/../private"], ["files/gcx/$(echo x)"],
                        ["/files/gcx/card"]):
            with self.assertRaises(packet.Refused):
                packet.hash_arguments(invalid)
        self.f.driver.execute({"op": "hash_files", "paths": paths})
        self.assertEqual(self.f.calls, [args])

    def test_unclean_live_writer_is_retained_and_foreign_ui_untouched(self):
        self.f.windows = f"mCurrentFocus=Window{{1 u0 {OTHER}}}"
        self.f.driver.finish()
        self.assertFalse(self.f.driver.receipt["handback"])
        self.assertIn("pending_recovery", self.f.driver.receipt)
        self.assertFalse(any("force-stop" in c or "input" in c for c in self.f.calls))

    def test_timeout_stops_all_further_device_commands(self):
        self.f.fail = True
        with self.assertRaises(subprocess.TimeoutExpired):
            self.f.driver.execute({"op": "hash_files", "paths": ["files/gcx/card.raw"]})
        count = len(self.f.calls)
        self.f.driver.finish()
        self.assertEqual(len(self.f.calls), count)
        self.assertFalse(self.f.driver.receipt["handback"])
        with self.assertRaises(packet.Refused):
            self.f.driver.call(["shell", "pidof", OWN])
        self.assertEqual(len(self.f.calls), count)

    def test_asynchronous_sleep_transition_requires_fresh_return_observations(self):
        self.f.clean = True
        self.f.display_states = ["ON", "ON", "OFF"]
        self.f.keyguard_states = ["false", "true"]
        self.f.driver.finish()
        self.assertTrue(self.f.driver.receipt["handback"])
        self.assertAlmostEqual(self.f.now, 100.3)
        self.assertEqual(sum("KEYCODE_SLEEP" in c for c in self.f.calls), 1)

    def test_unsettled_display_return_has_finite_attempts_and_pending_reason(self):
        self.f.clean = True
        self.f.display_states = ["ON"]
        self.f.keyguard_states = ["false"]
        self.f.driver.finish()
        self.assertFalse(self.f.driver.receipt["handback"])
        self.assertIn("pending_recovery", self.f.driver.receipt)
        self.assertEqual(sum("dumpsys display" in " ".join(c) for c in self.f.calls), 5)
        self.assertAlmostEqual(self.f.now, 100.9)

    def test_clean_metadata_then_client_retirement_and_return(self):
        self.f.clean = True
        self.f.driver.finish()
        self.assertTrue(self.f.driver.receipt["clean_stop_metadata"])
        self.assertTrue(self.f.driver.receipt["clients_retired"])
        self.assertTrue(self.f.driver.receipt["handback"])
        self.assertTrue(self.f.driver.receipt["recovery_resolved"])
        self.assertEqual(sum("force-stop" in c for c in self.f.calls), 1)

    def test_verified_new_install_can_close_empty_library_without_claiming_stop(self):
        self.f.driver.receipt["verified_new_install"] = True
        invoke = self.f.driver.invoke
        def new_install(args, timeout):
            if "cat" in args and "files/gcx/collection.json" in args:
                self.assertEqual(args, ["shell", "run-as", OWN, "cat", "files/gcx/collection.json"])
                return subprocess.CompletedProcess(args, 1, b"", b"cat: No such file or directory")
            return invoke(args, timeout)
        self.f.driver.invoke = new_install
        self.f.driver.finish()
        self.assertTrue(self.f.driver.receipt["handback"])
        self.assertTrue(self.f.driver.receipt["new_install_without_game"])
        self.assertFalse(self.f.driver.receipt["clean_stop_metadata"])

    def test_missing_file_message_in_success_stdout_is_not_empty_install_proof(self):
        self.f.driver.receipt["verified_new_install"] = True
        invoke = self.f.driver.invoke
        def merged_error(args, timeout):
            if "cat" in args and "files/gcx/collection.json" in args:
                return subprocess.CompletedProcess(args, 0, b"cat: No such file or directory", b"")
            return invoke(args, timeout)
        self.f.driver.invoke = merged_error
        self.f.driver.finish()
        self.assertFalse(self.f.driver.receipt["handback"])
        self.assertTrue(self.f.live)
        self.assertFalse(any("force-stop" in c for c in self.f.calls))

    def test_missing_collection_without_new_install_proof_cannot_retire_client(self):
        invoke = self.f.driver.invoke
        def missing(args, timeout):
            if "cat" in args and "files/gcx/collection.json" in args:
                return subprocess.CompletedProcess(args, 1, b"", b"cat: No such file or directory")
            return invoke(args, timeout)
        self.f.driver.invoke = missing
        self.f.driver.finish()
        self.assertFalse(self.f.driver.receipt["handback"])
        self.assertTrue(self.f.live)
        self.assertFalse(any("force-stop" in c for c in self.f.calls))


if __name__ == "__main__":
    unittest.main()
