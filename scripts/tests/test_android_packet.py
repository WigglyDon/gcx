# SPDX-License-Identifier: MIT
"""Offline regression fixtures: no device alias, ADB or private data."""
import importlib.util
from pathlib import Path
import subprocess
import struct
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
        self.after_dump = None
        self.fail = False
        self.live = True
        self.clean = False
        self.display_states = ["OFF"]
        self.keyguard_states = ["true"]
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
        if "dumpsys activity" in joined:
            out = self.resumed.encode()
        elif "dumpsys window displays |" in joined:
            out = self.windows.encode()
        elif "uiautomator dump" in joined:
            out = XML.encode()
            if self.after_dump:
                self.windows = self.after_dump
        elif args == ["exec-out", "screencap", "-p"]:
            out = b"\x89PNG\r\n\x1a\n" + b"\0\0\0\rIHDR" + struct.pack(">II", 800, 400)
            if self.after_capture:
                self.windows = self.after_capture
        elif "collection.json" in joined:
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
        elif "dumpsys display" in joined:
            state = self.display_states.pop(0) if len(self.display_states) > 1 else self.display_states[0]
            out = ("Display State=" + state).encode()
        elif "dumpsys window policy" in joined:
            state = self.keyguard_states.pop(0) if len(self.keyguard_states) > 1 else self.keyguard_states[0]
            out = ("showing=" + state + " secure=false").encode()
        return subprocess.CompletedProcess(args, 0, out, b"")


class PacketTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.f = Fixture(self.temp.name)

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

    def test_native_tap_refuses_stale_or_rotated_capture(self):
        self.f.driver.execute({"op": "capture", "name": "native.png"})
        tap = {"op": "gcx_tap", "capture": "native.png", "target": "Pause",
               "bounds": [30, 300, 90, 390]}
        self.f.now += 11
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(tap)
        self.f.now -= 11
        self.f.windows = WINDOW.replace("w=800 h=400 r=0", "w=400 h=800 r=1")
        with self.assertRaises(packet.Refused):
            self.f.driver.execute(tap)
        self.assertFalse(any("input" in c for c in self.f.calls))

    def test_each_button_uses_fresh_observed_bounds_and_focus(self):
        self.f.driver.execute({"op": "gcx_button", "text": "Resume"})
        self.f.driver.execute({"op": "gcx_button", "text": "Stop game", "cleanup": True})
        self.assertEqual([c for c in self.f.calls if "tap" in c],
                         [["shell", "input", "tap", "170", "120"],
                          ["shell", "input", "tap", "550", "240"]])
        self.assertEqual(sum("dumpsys activity" in " ".join(c) for c in self.f.calls), 4)

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


if __name__ == "__main__":
    unittest.main()
