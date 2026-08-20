#!/usr/bin/env python3
"""向 tauri 生成的 build.gradle.kts 注入 Android 签名配置。

tauri-cli 2.10+ 的模板 build.gradle.kts 不包含 signingConfigs，
导致即使有 keystore.properties 也产出 unsigned APK。
本脚本注入：keystoreProperties 加载 + signingConfigs("release") + release 引用。
用正则匹配任意缩进，兼容不同 tauri-cli 版本生成的模板。
用法: python3 apply_android_signing.py <path/to/app/build.gradle.kts>
"""
import re
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    content = f.read()

changed = False

# 1. 顶部插入 import + keystoreProperties 加载
if "keystoreProperties" not in content:
    header = (
        'import java.util.Properties\n'
        '\n'
        'val keystoreProperties = Properties()\n'
        'val keystorePropertiesFile = file("keystore.properties")\n'
        'if (keystorePropertiesFile.exists()) {\n'
        '    keystorePropertiesFile.inputStream().use { keystoreProperties.load(it) }\n'
        '}\n'
        '\n'
    )
    content = header + content
    changed = True

# 2. buildTypes 前插入 signingConfigs（保留 buildTypes 的缩进）
if "signingConfigs" not in content:
    m = re.search(r"^(\s*)buildTypes\s*\{", content, re.MULTILINE)
    if not m:
        print("ERROR: anchor 'buildTypes {' not found")
        sys.exit(1)
    ind = m.group(1)
    signing = (
        ind + "signingConfigs {\n"
        + ind + "    create(\"release\") {\n"
        + ind + "        if (keystoreProperties.containsKey(\"storeFile\")) {\n"
        + ind + "            storeFile = file(keystoreProperties[\"storeFile\"] as String)\n"
        + ind + "            storePassword = keystoreProperties[\"storePassword\"] as String\n"
        + ind + "            keyAlias = keystoreProperties[\"keyAlias\"] as String\n"
        + ind + "            keyPassword = keystoreProperties[\"keyPassword\"] as String\n"
        + ind + "            enableV1Signing = (keystoreProperties[\"v1SigningEnabled\"] as? String)?.toBoolean() ?: true\n"
        + ind + "            enableV2Signing = (keystoreProperties[\"v2SigningEnabled\"] as? String)?.toBoolean() ?: true\n"
        + ind + "            enableV3Signing = (keystoreProperties[\"v3SigningEnabled\"] as? String)?.toBoolean() ?: true\n"
        + ind + "        }\n"
        + ind + "    }\n"
        + ind + "}\n"
        + "\n"
    )
    content = content[: m.start()] + signing + content[m.start():]
    changed = True

# 3. release buildType 内加 signingConfig 引用
if "signingConfig = signingConfigs.getByName" not in content:
    m = re.search(r"^(\s*)(?:release|getByName\(\"release\"\))\s*\{", content, re.MULTILINE)
    if not m:
        print("ERROR: anchor 'release {' not found")
        sys.exit(1)
    ind = m.group(1)
    insert = "\n" + ind + "    signingConfig = signingConfigs.getByName(\"release\")"
    content = content[: m.end()] + insert + content[m.end():]
    changed = True

with open(path, "w", encoding="utf-8") as f:
    f.write(content)

print("signingConfigs injected" if changed else "already present, no change")
