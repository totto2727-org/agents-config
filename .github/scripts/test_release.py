"""Exercise the publishing guard through its CLI against real temporary Git repos."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("validate-release.py").resolve()


class ReleaseValidationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        self.git("init", "--initial-branch=main")
        self.git("config", "user.name", "Release guard tests")
        self.git("config", "user.email", "release-tests@example.invalid")
        self.git("config", "core.hooksPath", str(self.root / "no-hooks"))
        self.write_manifest()
        self.git("add", "Cargo.toml")
        self.git("commit", "-m", "Initial package")
        self.commit = self.git("rev-parse", "HEAD").strip()
        self.git("update-ref", "refs/remotes/origin/main", self.commit)
        self.git("tag", "v0.1.0")

    def git(self, *arguments):
        return subprocess.check_output(
            ["git", *arguments], cwd=self.root, env=self.env, text=True, stderr=subprocess.PIPE
        )

    def write_manifest(self, name="agents-config", publish='["crates-io"]'):
        (self.root / "Cargo.toml").write_text(
            f'[package]\nname = "{name}"\nversion = "0.1.0"\npublish = {publish}\n'
        )

    def validate(self, tag):
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--", tag],
            cwd=self.root, env=self.env, capture_output=True, text=True, timeout=10,
        )

    def test_accepts_main_release_and_returns_immutable_commit(self):
        result = self.validate("v0.1.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), self.commit)

    def test_accepts_annotated_release_tag(self):
        self.git("tag", "--force", "--annotate", "v0.1.0", "--message", "Release")
        result = self.validate("v0.1.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), self.commit)

    def test_rejects_tag_version_mismatch(self):
        self.git("tag", "v0.2.0")
        result = self.validate("v0.2.0")
        self.assertEqual(result.returncode, 1)
        self.assertIn("tag must exactly equal", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_rejects_commit_not_on_main(self):
        self.git("checkout", "-b", "unreviewed")
        self.git("commit", "--allow-empty", "-m", "Unreviewed change")
        self.git("tag", "--force", "v0.1.0")
        result = self.validate("v0.1.0")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout, "")

    def test_rejects_nonexistent_tag(self):
        result = self.validate("v9.9.9")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout, "")

    def test_rejects_wrong_package(self):
        self.write_manifest(name="unrelated")
        self.git("commit", "-am", "Wrong package")
        self.git("update-ref", "refs/remotes/origin/main", "HEAD")
        self.git("tag", "--force", "v0.1.0")
        result = self.validate("v0.1.0")
        self.assertEqual(result.returncode, 1)
        self.assertIn("must contain the agents-config", result.stderr)

    def test_rejects_unrestricted_registry(self):
        self.write_manifest(publish="true")
        self.git("commit", "-am", "Unrestricted registry")
        self.git("update-ref", "refs/remotes/origin/main", "HEAD")
        self.git("tag", "--force", "v0.1.0")
        result = self.validate("v0.1.0")
        self.assertEqual(result.returncode, 1)
        self.assertIn("restrict publication", result.stderr)

    def test_rejects_invalid_ref_without_executing_tag_contents(self):
        result = self.validate("v0.1.0; touch injected")
        self.assertEqual(result.returncode, 1)
        self.assertFalse((self.root / "injected").exists())
        self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
