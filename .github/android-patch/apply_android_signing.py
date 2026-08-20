#!/usr/bin/env python3
"""向 tauri 生成的 build.gradle.kts 注入 Android 签名配置。

tauri-cli 2.10+ 的模板 build.gradle.kts 不包含 signingConfigs，
导致 CI 上即使有 keystore.properties 也产出 unsigned APK。
本脚本注入：keystoreProperties 加载 + signingConfigs("release") + release 引用。
用法: python3 apply_android_signing.py <path/to/app/build.gradle.kts>
"""
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

# 2. buildTypes 前插入 signingConfigs
if "signingConfigs" not in content:
    signing = (
        '    signingConfigs {\n'
        '        create("release") {\n'
        '            if (keystoreProperties.containsKey("storeFile")) {\n'
        '                storeFile = file(keystoreProperties["storeFile"] as String)\n'
        '                storePassword = keystoreProperties["storePassword"] as String\n'
        '                keyAlias = keystoreProperties["keyAlias"] as String\n'
        '                keyPassword = keystoreProperties["keyPassword"] as String\n'
        '                enableV1Signing = (keystoreProperties["v1SigningEnabled"] as? String)?.toBoolean() ?: true\n'
        '                enableV2Signing = (keystoreProperties["v2SigningEnabled"] as? String)?.toBoolean() ?: true\n'
        '                enableV3Signing = (keystoreProperties["v3SigningEnabled"] as? String)?.toBoolean() ?: true\n'
        '            }\n'
        '        }\n'
        '    }\n'
        '\n'
    )
    if "    buildTypes {" in content:
        content = content.replace("    buildTypes {", signing + "    buildTypes {", 1)
        changed = True
    else:
        print("ERROR: anchor '    buildTypes {' not found")
        sys.exit(1)

# 3. release buildType 内加 signingConfig 引用
if "signingConfig = signingConfigs.getByName" not in content:
    if "        release {" in content:
        content = content.replace(
            "        release {",
            '        release {\n            signingConfig = signingConfigs.getByName("release")',
            1,
        )
        changed = True
    else:
        print("ERROR: anchor '        release {' not found")
        sys.exit(1)

with open(path, "w", encoding="utf-8") as f:
    f.write(content)

print("signingConfigs injected" if changed else "already present, no change")
