# PlatformIO pre-script: load .env (and fall back to OS env) into CFG_* macros.

Import("env")  # noqa: N816 — SCons API

import os


def load_dotenv(path):
    out = {}
    try:
        with open(path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if "=" not in line:
                    continue
                key, _, val = line.partition("=")
                key = key.strip()
                val = val.strip()
                if len(val) >= 2 and val[0] == val[-1] and val[0] in "\"'":
                    val = val[1:-1]
                out[key] = val
    except OSError:
        pass
    return out


def resolve(dotenv, key):
    v = dotenv.get(key)
    if v is not None and v != "":
        return v
    return os.environ.get(key, "")


def cpp_string_macro(name, raw):
    esc = raw.replace("\\", "\\\\").replace('"', '\\"')
    return (name, f'\\"{esc}\\"')


def main():
    project_dir = env["PROJECT_DIR"]
    dotenv_path = os.path.join(project_dir, ".env")
    d = load_dotenv(dotenv_path)

    mapping = (
        ("WIFI_SSID", "CFG_WIFI_SSID"),
        ("WIFI_PASS", "CFG_WIFI_PASS"),
        ("POWER_CHECKER_URL", "CFG_POWER_CHECKER_URL"),
        ("FTP_USER", "CFG_FTP_USER"),
        ("FTP_PASS", "CFG_FTP_PASS"),
    )

    defines = []
    empty_keys = []
    for env_key, macro_name in mapping:
        val = resolve(d, env_key)
        if not val:
            empty_keys.append(env_key)
        defines.append(cpp_string_macro(macro_name, val))

    if empty_keys:
        print(
            "load_credentials: 次の変数が空です。.env か export を確認してください: "
            + ", ".join(empty_keys)
        )

    env.Append(CPPDEFINES=list(defines))


main()
