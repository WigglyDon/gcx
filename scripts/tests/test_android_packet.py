# SPDX-License-Identifier: MIT
"""Offline regression fixtures: no device alias, ADB or private data."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "android_packet", Path(__file__).resolve().parents[1] / "android_packet.py")
packet = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(packet)
OWN = packet.PACKAGE
RESUMED = f"mResumedActivity: ActivityRecord{{1 u0 {OWN}/.GcxActivity t1}}"
WINDOW = f"mCurrentFocus=Window{{1 u0 {OWN}/{OWN}.GcxActivity}}"
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
        elif "dumpsys window |" in joined:
            out = self.windows.encode()
        elif "uiautomator dump" in joined:
            out = XML.encode()
            if self.after_dump:
                self.windows = self.after_dump
        elif args == ["exec-out", "screencap", "-p"]:
            out = b"unrelated pixels must not survive"
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
            out = b"Display State=OFF"
        elif "dumpsys window policy" in joined:
            out = b"showing=true secure=false"
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
