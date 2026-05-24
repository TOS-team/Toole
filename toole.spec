# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['src/ui/app.py'],
    pathex=[],
    binaries=[],
    datas=[('/home/gerard/Documents/Burkina-Open-Place/Toole/assets/logo.png', 'assets'), ('/home/gerard/Documents/Burkina-Open-Place/Toole/assets/logo.ico', 'assets'), ('/home/gerard/Documents/Burkina-Open-Place/Toole/build/libtoole_bridge.so', '.')],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='toole',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
