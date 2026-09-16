"""Resolve an eligible release tag without checking out or publishing its contents."""

import argparse
import subprocess
import sys
import tomllib


def git(*arguments):
    return subprocess.check_output(["git", *arguments], text=True).strip()


def validate_release(tag):
    git("check-ref-format", f"refs/tags/{tag}")
    commit = git("rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}")
    subprocess.run(
        ["git", "merge-base", "--is-ancestor", commit, "refs/remotes/origin/main"],
        check=True,
    )
    package = tomllib.loads(git("show", f"{commit}:Cargo.toml"))["package"]
    if package["name"] != "agents-config":
        raise ValueError("release must contain the agents-config package")
    if tag != f"v{package['version']}":
        raise ValueError("release tag must exactly equal v<Cargo.toml version>")
    if package.get("publish") != ["crates-io"]:
        raise ValueError("release package must restrict publication to crates-io")
    return commit


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag", help="Existing v<version> tag reachable from origin/main")
    arguments = parser.parse_args()
    try:
        print(validate_release(arguments.tag))
    except (subprocess.CalledProcessError, KeyError, ValueError) as error:
        print(f"Release validation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
