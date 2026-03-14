from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

IGNORED_NAMES = {".DS_Store", "Thumbs.db"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Package one OpenClaw skill directory, or every skill under a root directory, "
            "into versioned zip archives."
        )
    )
    parser.add_argument(
        "source",
        help=(
            "Path to a skill directory that contains SKILL.md, or a root directory that "
            "contains multiple skill directories when --all is used"
        ),
    )
    parser.add_argument("output_dir", help="Directory where the zip archives will be written")
    parser.add_argument(
        "--all",
        action="store_true",
        help="Package every immediate child directory that contains SKILL.md",
    )
    parser.add_argument(
        "--version",
        help="Override the version embedded in the output filename. Defaults to the workspace version.",
    )
    return parser.parse_args()


def workspace_version(repo_root: Path) -> str:
    cargo_toml = repo_root / "Cargo.toml"
    data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    return data["workspace"]["package"]["version"]


def iter_skill_files(skill_dir: Path):
    for path in sorted(skill_dir.rglob("*")):
        if not path.is_file():
            continue
        if path.name in IGNORED_NAMES:
            continue
        yield path


def resolve_skill_dirs(source: Path, package_all: bool) -> list[Path]:
    if package_all:
        if not source.is_dir():
            raise ValueError(f"skills root does not exist: {source}")

        skill_dirs = [
            path
            for path in sorted(source.iterdir())
            if path.is_dir() and (path / "SKILL.md").is_file()
        ]
        if not skill_dirs:
            raise ValueError(f"no skill directories with SKILL.md found under: {source}")
        return skill_dirs

    if not source.is_dir():
        raise ValueError(f"skill directory does not exist: {source}")
    if not (source / "SKILL.md").is_file():
        raise ValueError(f"missing SKILL.md in skill directory: {source}")
    return [source]


def package_skill(skill_dir: Path, output_dir: Path, version: str) -> Path:
    archive_path = output_dir / f"{skill_dir.name}-{version}.zip"
    with ZipFile(archive_path, "w", compression=ZIP_DEFLATED) as archive:
        for path in iter_skill_files(skill_dir):
            relative_path = path.relative_to(skill_dir.parent)
            archive.write(path, arcname=relative_path.as_posix())
    return archive_path


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    source = Path(args.source).resolve()
    output_dir = Path(args.output_dir).resolve()

    try:
        skill_dirs = resolve_skill_dirs(source, args.all)
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1

    version = args.version or workspace_version(repo_root)
    output_dir.mkdir(parents=True, exist_ok=True)

    archive_paths = [package_skill(skill_dir, output_dir, version) for skill_dir in skill_dirs]
    for archive_path in archive_paths:
        print(archive_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
