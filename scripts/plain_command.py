import re
import subprocess
import sys


def plain(text):
    text = re.sub("[\U0001f000-\U0001faff\u2600-\u27bf\ufe0f]", "", text)
    if re.fullmatch(r"\s*(?:[=-]\s*){3,}", text):
        return ""
    return text.encode("ascii", "backslashreplace").decode("ascii")


def main():
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        raise SystemExit("A command is required.")
    with subprocess.Popen(
        command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT
    ) as process:
        for raw in process.stdout:
            sys.stdout.buffer.write(
                plain(raw.decode("utf-8", "backslashreplace")).encode("ascii")
            )
            sys.stdout.buffer.flush()
        raise SystemExit(process.wait())


if __name__ == "__main__":
    main()
