"""Check workflow routing and failure propagation without builds or chain access."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class WorkflowTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / "project with spaces"
        (self.root / "scripts/tests").mkdir(parents=True)
        (self.root / "contracts").mkdir()
        (self.root / "scripts/tests/example.sh").touch()
        for name in ("test-workflow.sh", "release-workflow.sh", "deploy.sh", "coverage.sh"):
            shutil.copy(ROOT / "scripts" / name, self.root / "scripts" / name)
        for name in ("Makefile", "rust-toolchain.toml"):
            shutil.copy(ROOT / name, self.root / name)
        self.bin = Path(temporary.name) / "bin"
        self.bin.mkdir()
        self.log = Path(temporary.name) / "calls.jsonl"
        for name in ("bash", "cargo", "forge", "cast", "git", "test-python"):
            tool = self.bin / name
            tool.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
with open(os.environ["WORKFLOW_LOG"], "a") as output:
    output.write(json.dumps([name, *sys.argv[1:]]) + "\\n")
if name == os.environ.get("FAIL_TOOL"):
    sys.exit(37)
if name == "git": print("0" * 40)
if name == "forge" and sys.argv[1:] == ["--version"]: print("forge 1.8.1")
if name == "cast": print("0x" + "a" * 64)
''')
            tool.chmod(0o755)
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
                        WORKFLOW_LOG=str(self.log), PYTHON=str(self.bin / "test-python"), SUMMARY="0")
        artifact = self.root / "contracts/out/BesuQBFTZKLightClient.sol/BesuQBFTZKLightClient.json"
        artifact.parent.mkdir(parents=True)
        artifact.write_text(json.dumps({"abi": [], "bytecode": {"object": "0x6001"},
                                        "deployedBytecode": {"object": "0x6002"}}))
        (self.root / "artifacts").mkdir()
        keys = {}
        for index, program in enumerate(("update-client", "membership", "non-membership"), 1):
            data = program.encode()
            (self.root / "artifacts" / f"{program}.elf").write_bytes(data)
            keys[program] = {"elfSha256": "0x" + hashlib.sha256(data).hexdigest(),
                             "programVKey": "0x" + str(index) * 64}
        (self.root / "artifacts/vkeys.json").write_text(json.dumps(keys))
        # Nested fixture Make invocations must not inherit the caller's ACTION.
        for name in ("MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES", "ACTION", "PROOFS_OUTPUT", "RELEASE_MANIFEST", "IBC_CONFIG", "LIGHT_CLIENT", "HEIGHT"):
            self.env.pop(name, None)

    def invoke(self, workflow, *args, **env):
        result = subprocess.run(["/bin/bash", str(self.root / "scripts" / workflow), *args],
                                env=dict(self.env, **env), capture_output=True, text=True)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []
        return result, calls

    def test_default_tests_include_all_layers(self):
        result, calls = self.invoke("test-workflow.sh")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call[1:] for call in calls if call[0] == "bash"], [
            ["scripts/tests/example.sh"], ["scripts/coverage.sh", "all"],
            ["scripts/docker-build.sh", "circuits"],
            ["scripts/tests/tools/generate-golden-proofs.sh", "artifacts/proofs"],
            ["e2e/scripts/forge.sh", "test"],
        ])
        self.assertTrue(any("sp1_execute" in call for call in calls))
        self.assertTrue(any("SP1VerifierIntegrationTest" in call for call in calls))
        self.assertTrue(any("clippy" in call for call in calls))

    def test_focused_rust_does_not_start_other_layers(self):
        result, calls = self.invoke("test-workflow.sh", "rust")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 2)
        self.assertTrue(all(call[:2] == ["cargo", "test"] for call in calls))

    def test_failed_test_stops_all_work(self):
        result, calls = self.invoke("test-workflow.sh", FAIL_TOOL="cargo")
        self.assertEqual(result.returncode, 37)
        self.assertEqual(len(calls), 1)

    def test_default_release_orders_checks_before_manifest_and_metadata(self):
        result, calls = self.invoke("release-workflow.sh")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([Path(call[1]).name for call in calls if call[0] == "bash"], [
            "check-reproducible-build.sh", "verify-ibc-storage-layout.sh",
        ])
        self.assertTrue((self.root / "release/manifest.json").exists())
        self.assertIn("release metadata validation passed", result.stdout)

    def test_failed_release_stops_before_manifest(self):
        result, calls = self.invoke("release-workflow.sh", FAIL_TOOL="bash")
        self.assertEqual(result.returncode, 37)
        self.assertEqual(len(calls), 1)

    def test_release_manifest_honors_output_override(self):
        result, calls = self.invoke("release-workflow.sh", "manifest", RELEASE_MANIFEST="release/custom.json")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.root / "release/custom.json").exists())
        self.assertFalse((self.root / "release/manifest.json").exists())

    def test_invalid_actions_do_not_run_tools(self):
        for workflow in ("test-workflow.sh", "release-workflow.sh", "deploy.sh", "coverage.sh"):
            with self.subTest(workflow=workflow):
                result, calls = self.invoke(workflow, "typo")
                self.assertEqual(result.returncode, 64)
                self.assertEqual(calls, [])

    def test_register_requires_address_before_running_deploy(self):
        result, calls = self.invoke("deploy.sh", "register", "chain")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("LIGHT_CLIENT", result.stderr)
        self.assertEqual(calls, [])

    def test_make_defaults_and_action_override(self):
        # Use the real Bash for Make's shell, but mock the called workflow.
        for target, extra, expected in (("test", [], "all"), ("release", [], "all"),
                                        ("test", ["ACTION=contracts"], "contracts"),
                                        ("deploy", ["ACTION=preflight", "PROFILE=chain"], "preflight")):
            with self.subTest(target=target, extra=extra):
                self.log.unlink(missing_ok=True)
                result = subprocess.run(["make", "SHELL=/bin/bash", target, *extra], cwd=self.root,
                                        env=self.env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                call = json.loads(self.log.read_text().splitlines()[0])
                expected_call = (["bash", "scripts/deploy.sh", expected, "chain"] if target == "deploy"
                                 else ["bash", f"scripts/{target}-workflow.sh", expected])
                self.assertEqual(call, expected_call)


if __name__ == "__main__":
    unittest.main()
