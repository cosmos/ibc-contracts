#!/usr/bin/env python3
"""Exercise Docker orchestration with a fake CLI; no engine or downloads required."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class DockerBuildTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "project with spaces"
        (self.root / "scripts").mkdir(parents=True)
        (self.root / "deploy/build").mkdir(parents=True)
        shutil.copy(ROOT / "scripts/docker-build.sh", self.root / "scripts")
        self.bin = Path(self.temporary.name) / "bin"
        self.bin.mkdir()
        self.log = Path(self.temporary.name) / "docker.jsonl"
        docker = self.bin / "docker"
        docker.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
a = sys.argv[1:]
with open(os.environ["MOCK_LOG"], "a") as f:
    f.write(json.dumps(a) + "\\n")
if "scripts/docker-build-entrypoint.sh" in a:
    sys.exit(int(os.environ.get("MOCK_BUILD_STATUS", "0")))
''')
        docker.chmod(0o755)
        git = self.bin / "git"
        git.write_text('''#!/usr/bin/env python3
import json, os, sys
with open(os.environ["MOCK_LOG"], "a") as f:
    f.write(json.dumps(["git", *sys.argv[1:]]) + "\\n")
sys.exit(int(os.environ.get("MOCK_GIT_STATUS", "0")))
''')
        git.chmod(0o755)
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
                        MOCK_LOG=str(self.log))
        for name in ("DOCKER_HOST", "BUILD_IMAGE", "CARGO_BUILD_JOBS", "MOCK_GIT_STATUS"):
            self.env.pop(name, None)

    def run_build(self, action, **extra):
        result = subprocess.run(["bash", str(self.root / "scripts/docker-build.sh"), action],
                                env=dict(self.env, **extra), capture_output=True, text=True)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []
        return result, calls

    def test_same_host_path_and_persistent_outputs(self):
        result, calls = self.run_build("all")
        self.assertEqual(result.returncode, 0, result.stderr)
        run = next(call for call in calls if "scripts/docker-build-entrypoint.sh" in call)
        self.assertIn(f"type=bind,source={self.root},target=/workspace", run)
        self.assertIn("CARGO_TARGET_DIR=/workspace/target/docker", run)
        self.assertIn("CARGO_BUILD_JOBS=1", run)
        self.assertIn(f"{os.getuid()}:{os.getgid()}", run)
        self.assertFalse(any("docker.sock" in arg or "DOCKER_HOST=" in arg for arg in run))
        self.assertFalse(any("SDKROOT" in arg for arg in run))
        self.assertTrue((self.root / "artifacts").is_dir())
        self.assertEqual(sum(call[0] == "run" for call in calls), 1)

    def test_failure_is_preserved_without_followup_containers(self):
        result, calls = self.run_build("circuits", MOCK_BUILD_STATUS="37")
        self.assertEqual(result.returncode, 37)
        self.assertEqual(sum(call[0] == "run" for call in calls), 1)

    def test_contract_actions_initialize_submodule_on_host_before_build(self):
        for action in ("all", "contracts", "test"):
            with self.subTest(action=action):
                self.log.unlink(missing_ok=True)
                result, calls = self.run_build(action)
                self.assertEqual(result.returncode, 0, result.stderr)
                git_call = ["git", "-C", str(self.root), "submodule", "update",
                            "--init", "--recursive", "contracts/lib/sp1-contracts"]
                self.assertEqual([call for call in calls if call[0] == "git"], [git_call])
                build_index = next(i for i, call in enumerate(calls) if call[0] == "build")
                self.assertLess(calls.index(git_call), build_index)

    def test_non_contract_actions_do_not_initialize_submodules(self):
        for action in ("host", "proof-api", "circuits"):
            with self.subTest(action=action):
                self.log.unlink(missing_ok=True)
                result, calls = self.run_build(action)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertFalse(any(call[0] == "git" for call in calls))

    def test_submodule_failure_stops_before_docker_build(self):
        result, calls = self.run_build("contracts", MOCK_GIT_STATUS="37")
        self.assertEqual(result.returncode, 37)
        self.assertFalse(any(call[0] in ("build", "run") for call in calls))

    def test_cargo_job_override_reaches_container(self):
        result, calls = self.run_build("host", CARGO_BUILD_JOBS="2")
        self.assertEqual(result.returncode, 0, result.stderr)
        run = next(call for call in calls if "scripts/docker-build-entrypoint.sh" in call)
        self.assertIn("CARGO_BUILD_JOBS=2", run)

    def test_no_action_mounts_a_docker_socket(self):
        for action in ("host", "circuits", "contracts", "test"):
            result, calls = self.run_build(action)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(any(call[0] == "context" for call in calls))
            self.assertFalse(any("docker.sock" in arg or "DOCKER_HOST=" in arg
                                 for call in calls for arg in call))

    def test_invalid_action_does_not_call_docker(self):
        result, calls = self.run_build("invalid")
        self.assertEqual(result.returncode, 64)
        self.assertEqual(calls, [])

    def test_make_build_proof_api_compiles_binary_and_runtime_image(self):
        result = subprocess.run(["make", "--no-print-directory", "-n", "build-proof-api"],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines(), [
            "bash scripts/docker-build.sh proof-api",
            'docker build --platform linux/amd64 --tag "besu-proof-api:local" --file deploy/Dockerfile deploy',
        ])

    def test_make_build_docker_retains_complete_build(self):
        result = subprocess.run(["make", "--no-print-directory", "-n", "build-docker"],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "bash scripts/docker-build.sh all")


class SP1BuildModeTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / "scripts").mkdir()
        (self.root / "contracts").mkdir()
        for name in ("build-sp1.sh", "generate-vkeys.sh", "docker-build-entrypoint.sh"):
            shutil.copy(ROOT / "scripts" / name, self.root / "scripts")
        (self.root / "scripts/install-contract-deps.sh").write_text("#!/bin/bash\nexit 0\n")
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.log = self.root / "tools.jsonl"
        tool = self.bin / "tool"
        tool.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ["MOCK_LOG"], "a") as f:
    f.write(json.dumps([name, args, os.environ.get("RUSTUP_TOOLCHAIN")]) + "\\n")
if name == "cargo" and args[:2] == ["prove", "build"]:
    output = pathlib.Path(args[args.index("--output-directory") + 1])
    output.mkdir(parents=True, exist_ok=True)
    (output / args[args.index("--elf-name") + 1]).write_text(os.environ.get("MOCK_ELF", "fixture"))
if name == "cargo" and "generate-vkeys" in args:
    if os.environ.get("MOCK_FAIL_GENERATE"):
        sys.exit(23)
    import hashlib
    directory = pathlib.Path(args[-1])
    print(json.dumps({name: {"elfSha256": "0x" + hashlib.sha256((directory / (name + ".elf")).read_bytes()).hexdigest(),
                             "programVKey": "0x" + str(i) * 64}
                      for i, name in enumerate(["update-client", "membership", "non-membership"], 1)}))
if name == "rustup" and os.environ.get("MOCK_NO_TOOLCHAIN"):
    sys.exit(1)
if name == "cargo" and args == ["prove", "--version"]:
    print("wrong revision" if os.environ.get("MOCK_BAD_CLI") else "cargo-prove sp1 (d454975)")
''')
        tool.chmod(0o755)
        for name in ("cargo", "rustup", "git", "forge"):
            (self.bin / name).symlink_to(tool)
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
                        MOCK_LOG=str(self.log))
        self.env.pop("RUSTUP_TOOLCHAIN", None)
        # Publication runs in the Linux builder. Emulate these two Linux utilities on macOS.
        (self.bin / "flock").write_text("#!/bin/bash\nexit 0\n")
        (self.bin / "flock").chmod(0o755)
        (self.bin / "mv").write_text('''#!/usr/bin/env python3
import os, sys
if sys.argv[1] == "-Tf":
    os.replace(sys.argv[2], sys.argv[3])
else:
    os.execv("/bin/mv", ["mv", *sys.argv[1:]])
''')
        (self.bin / "mv").chmod(0o755)


    def invoke(self, script, args, **extra):
        result = subprocess.run(["bash", str(self.root / "scripts" / script), *args],
                                env=dict(self.env, **extra), capture_output=True, text=True)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []
        return result, calls

    def test_container_builds_all_guests_directly(self):
        result, calls = self.invoke("docker-build-entrypoint.sh", ["all"])
        self.assertEqual(result.returncode, 0, result.stderr)
        guests = [call for call in calls if call[0] == "cargo" and call[1][:2] == ["prove", "build"]]
        self.assertEqual(len(guests), 3)
        self.assertEqual({call[1][call[1].index("-p") + 1] for call in guests},
                         {"update-client", "membership", "non-membership"})
        for _, args, toolchain in guests:
            self.assertNotIn("--docker", args)
            self.assertIn("--locked", args)
            self.assertEqual(toolchain, "succinct")
        self.assertTrue(any(call[0] == "forge" and call[1] == ["build"] for call in calls))

    def test_failed_generation_preserves_published_artifacts(self):
        result, _ = self.invoke("docker-build-entrypoint.sh", ["circuits"])
        self.assertEqual(result.returncode, 0, result.stderr)
        published = os.readlink(self.root / "artifacts/current")
        before = (self.root / "artifacts/vkeys.json").read_bytes()
        result, _ = self.invoke("docker-build-entrypoint.sh", ["circuits"], MOCK_FAIL_GENERATE="1", MOCK_ELF="new")
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(os.readlink(self.root / "artifacts/current"), published)
        self.assertEqual((self.root / "artifacts/vkeys.json").read_bytes(), before)
        self.assertEqual((self.root / "artifacts/membership.elf").read_text(), "fixture")
        result, _ = self.invoke("docker-build-entrypoint.sh", ["circuits"], MOCK_ELF="new")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotEqual(os.readlink(self.root / "artifacts/current"), published)
        self.assertEqual((self.root / "artifacts/membership.elf").read_text(), "new")

    def test_circuit_build_uses_standalone_generator_without_other_host_tools(self):
        result, calls = self.invoke("docker-build-entrypoint.sh", ["circuits"])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(any(call[0] in ("git", "forge") for call in calls))
        cargo = [call[1] for call in calls if call[0] == "cargo"]
        self.assertFalse(any(args[0] == "build" for args in cargo))
        generator = next(args for args in cargo if args[0] == "run")
        self.assertEqual(generator[generator.index("-p") + 1], "generate-vkeys")
        self.assertIn("--locked", generator)

    def test_native_script_keeps_existing_docker_mode(self):
        result, calls = self.invoke("build-sp1.sh", [])
        self.assertEqual(result.returncode, 0, result.stderr)
        guests = [call for call in calls if call[1][:2] == ["prove", "build"]]
        self.assertEqual(len(guests), 3)
        self.assertTrue(all("--docker" in call[1] for call in guests))

    def test_missing_guest_compiler_stops_before_build(self):
        result, calls = self.invoke("build-sp1.sh", ["--local"], MOCK_NO_TOOLCHAIN="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(call[0] == "cargo" for call in calls))

    def test_wrong_cli_revision_stops_before_build(self):
        result, calls = self.invoke("build-sp1.sh", ["--local"], MOCK_BAD_CLI="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(call[1][:2] == ["prove", "build"] for call in calls))


class MountedDeploymentTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "workspace with spaces"
        for directory in ("scripts", "config", "artifacts", "target/docker/release"):
            (self.root / directory).mkdir(parents=True)
        shutil.copy(ROOT / "scripts/deploy.sh", self.root / "scripts")
        (self.root / "config/chain.json").write_text('{"name":"chain"}')
        bootstrap = self.root / "target/docker/release/bootstrap-state"
        bootstrap.touch()
        bootstrap.chmod(0o755)
        self.key = Path(self.temporary.name) / "deployer key"
        self.key.write_text("private-key-fixture")
        self.log = Path(self.temporary.name) / "docker.jsonl"
        bin_dir = Path(self.temporary.name) / "bin"
        bin_dir.mkdir()
        docker = bin_dir / "docker"
        docker.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
with open(os.environ["MOCK_LOG"], "a") as log:
    log.write(json.dumps(args) + "\\n")
if args[:2] == ["image", "inspect"]:
    print("sha256:fixture")
elif args[0] == "run":
    assert "build" not in args
    print('{"deployment":{"lightClient":"0xfixture"}}')
    sys.exit(int(os.environ.get("MOCK_DEPLOY_STATUS", "0")))
else:
    sys.exit(99)
''')
        docker.chmod(0o755)
        self.env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ["PATH"],
                        MOCK_LOG=str(self.log), BUILD_IMAGE="build-tools:fixture",
                        SOURCE_RPC_URL="http://source/?token=source-secret",
                        DESTINATION_RPC_URL="http://destination/?token=destination-secret",
                        DEPLOYER_KEY_FILE=str(self.key))

    def invoke(self, args, **extra):
        return subprocess.run(["bash", str(self.root / "scripts/deploy.sh"), *args],
                              env=dict(self.env, **extra), capture_output=True, text=True)

    def test_deployment_mounts_existing_outputs_without_packaging(self):
        result = self.invoke(["deploy", "chain"])
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual([call[0] for call in calls], ["image", "run"])
        run = calls[-1]
        self.assertIn(str(self.root) + ":/workspace", run)
        self.assertIn(str(self.root / "artifacts") + ":/workspace/artifacts:ro", run)
        self.assertEqual(run[-4:], ["build-tools:fixture", "deploy/entrypoint.sh", "deploy", "chain"])
        self.assertNotIn("source-secret", str(calls))
        self.assertNotIn("destination-secret", str(calls))
        self.assertNotIn("private-key-fixture", str(calls))
        self.assertEqual(len(list((self.root / "deployments/chain").glob("*.json"))), 1)
        for index, arg in enumerate(run):
            if arg == "--volume" and ":/run/secrets/" in run[index + 1] and "deployer-key" not in run[index + 1]:
                self.assertFalse(Path(run[index + 1].split(":/run/secrets/")[0]).exists())
        self.assertFalse((self.root / ".e2e-state").exists())

    def test_register_uses_environment_address(self):
        result = self.invoke(["register", "chain"], LIGHT_CLIENT="0xfixture")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual(calls[-1][-3:], ["register", "chain", "0xfixture"])

    def test_bootstrap_uses_environment_height(self):
        result = self.invoke(["bootstrap", "chain"], HEIGHT="123")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual(calls[-1][-3:], ["bootstrap", "chain", "123"])

    def test_explicit_height_overrides_environment(self):
        result = self.invoke(["bootstrap", "chain", "456"], HEIGHT="123")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual(calls[-1][-3:], ["bootstrap", "chain", "456"])

    def test_make_exports_optional_deployment_inputs(self):
        for name in ("Makefile", "rust-toolchain.toml"):
            shutil.copy(ROOT / name, self.root / name)
        for action, setting, expected in (("register", "LIGHT_CLIENT=0xfixture", "0xfixture"),
                                          ("bootstrap", "HEIGHT=123", "123")):
            with self.subTest(action=action):
                result = subprocess.run(["make", "deploy", f"ACTION={action}", "PROFILE=chain", setting],
                                        cwd=self.root, env=self.env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                calls = [json.loads(line) for line in self.log.read_text().splitlines()]
                self.assertEqual(calls[-1][-3:], [action, "chain", expected])

    def test_deployment_failure_is_preserved(self):
        result = self.invoke(["deploy", "chain"], MOCK_DEPLOY_STATUS="37")
        self.assertEqual(result.returncode, 37)

    def test_profile_traversal_is_rejected_before_docker(self):
        result = self.invoke(["deploy", "../chain"])
        self.assertEqual(result.returncode, 64)
        self.assertFalse(self.log.exists())


if __name__ == "__main__":
    unittest.main()
