import argparse
import subprocess
from pathlib import Path


def crop_output(in_text):
    started = False
    ended = False
    output = []
    for line in in_text.splitlines():
        if "Called auto-stubbed function" in line or "Called uninstatied mock" in line:
            started = True

        if "Caused by:" in line:
            ended = True

        if started and not ended:
            output.append(line)

    return "\n".join(output)


def check_stderr(
    args,
    expected_file,
    update=False,
):
    result = subprocess.run(
        args,
        capture_output=True,
        text=True,
    )

    stderr = crop_output(result.stderr)
    expected_file = Path(expected_file)

    if update:
        expected_file.write_text(stderr)
        return True

    expected = expected_file.read_text()
    if stderr != expected:
        print(f"Failed {args[-1]}\n{stderr}")
    return stderr == expected


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="runner",
        description="Cesty panic test runner",
    )
    parser.add_argument("-u", "--update", action="store_true")
    args = parser.parse_args()
    tests = [
        "test_autostubbed",
        "test_mocked_simple",
        "test_mocked_nested_func",
        "test_mocked_nested_closure",
    ]

    for t in tests:
        print(f"-------- {t} ---------")
        check_stderr(["cargo", "test", t], f"./test_outputs/{t}", update=args.update)
        print(f"---------------------------------")
